use appview_core::error::Result;
use appview_queue::{IndexJob, IndexOperation, IndexQueue};
use futures::SinkExt;
use futures::stream::StreamExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::interval;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

const FIREHOSE_PING_INTERVAL: Duration = Duration::from_secs(30);
const CURSOR_SAVE_INTERVAL: Duration = Duration::from_secs(10);
// Allow 3 missed pongs before declaring connection dead; small PDSes have long gaps between events.
const IDLE_TIMEOUT: Duration = Duration::from_secs(90);
const MAX_CONCURRENT_JOBS: usize = 100;

pub struct FirehoseConsumer {
    relay_hosts: Vec<String>,
    queue: Arc<IndexQueue>,
    shutdown: Arc<AtomicBool>,
}

impl FirehoseConsumer {
    pub fn new(relay_hosts: Vec<String>, queue: Arc<IndexQueue>) -> Self {
        Self {
            relay_hosts,
            queue,
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }

    pub async fn run(&self) -> Result<()> {
        let mut tasks = Vec::new();

        for host in &self.relay_hosts {
            let host_clone = host.clone();
            let queue_clone = Arc::clone(&self.queue);
            let shutdown_clone = Arc::clone(&self.shutdown);

            let task = tokio::spawn(async move {
                Self::run_connection(host_clone, queue_clone, shutdown_clone).await;
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        for task in tasks {
            if let Err(e) = task.await {
                error!("firehose task panicked: {e}");
            }
        }

        Ok(())
    }

    async fn run_connection(hostname: String, queue: Arc<IndexQueue>, shutdown: Arc<AtomicBool>) {
        let mut backoff_secs = 1u64;
        let max_backoff_secs = 300u64; // 5 minutes

        loop {
            if shutdown.load(Ordering::Relaxed) {
                info!("shutdown requested for {hostname}");
                break;
            }

            match Self::connect_and_stream(&hostname, &queue, &shutdown).await {
                ConnectionResult::Success => {
                    info!("connection completed normally for {hostname}");
                    break;
                }
                ConnectionResult::Timeout => {
                    warn!("connection timed out for {hostname}, retrying");
                    tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                    backoff_secs = (backoff_secs * 2).min(max_backoff_secs);
                }
                ConnectionResult::FutureCursor => {
                    warn!("cursor in future for {hostname}, deleting cursor to restart from 0");
                    if let Err(e) = queue.delete_cursor(&format!("firehose:{hostname}")) {
                        error!("failed to delete cursor: {e}");
                    }
                    backoff_secs = 1;
                }
            }
        }
    }

    async fn connect_and_stream(
        hostname: &str,
        queue: &Arc<IndexQueue>,
        shutdown: &Arc<AtomicBool>,
    ) -> ConnectionResult {
        let cursor_key = format!("firehose:{hostname}");

        // Load cursor from Fjall
        let cursor = match queue.load_cursor(&cursor_key) {
            Ok(c) => c,
            Err(e) => {
                error!("failed to load cursor: {e}");
                return ConnectionResult::Timeout;
            }
        };

        // Use AtomicI64 for cheap, lock-free cursor updates
        let last_seq = Arc::new(AtomicI64::new(cursor.unwrap_or(0)));

        let ws_scheme = if hostname.starts_with("ws://") || hostname.starts_with("http://") {
            "ws"
        } else {
            "wss"
        };
        let clean_hostname = hostname
            .trim_start_matches("wss://")
            .trim_start_matches("ws://")
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/');

        // If the caller already provided the full XRPC path, use it directly;
        // otherwise append the subscribeRepos path.
        let full_url = if clean_hostname.contains("/xrpc/") {
            format!("{ws_scheme}://{clean_hostname}")
        } else {
            format!("{ws_scheme}://{clean_hostname}/xrpc/com.atproto.sync.subscribeRepos")
        };

        let mut url = match url::Url::parse(&full_url) {
            Ok(u) => u,
            Err(e) => {
                return ConnectionResult::Timeout;
            }
        };

        if let Some(c) = cursor {
            if c > 0 {
                url.query_pairs_mut().append_pair("cursor", &c.to_string());
                info!("connecting to {url} resuming from cursor {c}");
            } else {
                info!("connecting to {url} starting from live stream");
            }
        } else {
            info!("connecting to {url} from beginning");
        }

        let (ws_stream, _) = match tokio_tungstenite::connect_async(url.as_str()).await {
            Ok(s) => s,
            Err(e) => {
                return ConnectionResult::Timeout;
            }
        };

        let (mut write, mut read) = ws_stream.split();

        let mut last_message_time = std::time::Instant::now();

        // Spawn ping task to keep connection alive
        let ping_task = tokio::spawn(async move {
            let mut ping_interval = interval(FIREHOSE_PING_INTERVAL);
            loop {
                ping_interval.tick().await;
                if write.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
        });

        // Spawn cursor saver task
        let cursor_saver_seq = Arc::clone(&last_seq);
        let cursor_saver_queue: Arc<IndexQueue> = Arc::clone(queue);
        let cursor_saver_key = cursor_key.clone();
        let cursor_saver_shutdown = Arc::clone(shutdown);

        let cursor_saver_task = tokio::spawn(async move {
            let mut cursor_interval = interval(CURSOR_SAVE_INTERVAL);
            let mut last_saved_seq = 0i64;
            loop {
                cursor_interval.tick().await;
                if cursor_saver_shutdown.load(Ordering::Relaxed) {
                    break;
                }
                let current_seq = cursor_saver_seq.load(Ordering::Relaxed);
                if current_seq > 0 && current_seq != last_saved_seq {
                    if let Err(e) = cursor_saver_queue.save_cursor(&cursor_saver_key, current_seq) {
                        error!("cursor saver failed: {e}");
                    } else {
                        last_saved_seq = current_seq;
                        debug!("cursor saved: {current_seq}");
                    }
                }
            }
        });

        // Semaphore to limit concurrent processing
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_JOBS));

        loop {
            if shutdown.load(Ordering::Relaxed) {
                info!("shutdown requested, closing firehose connection");
                let final_seq = last_seq.load(Ordering::Relaxed);
                if final_seq > 0 {
                    drop(queue.save_cursor(&cursor_key, final_seq));
                }
                ping_task.abort();
                cursor_saver_task.abort();
                return ConnectionResult::Success;
            }

            // Detect zombie connections
            if last_message_time.elapsed() > IDLE_TIMEOUT {
                warn!(
                    "no messages received in {}s for {hostname}, assuming connection dead",
                    IDLE_TIMEOUT.as_secs()
                );
                let final_seq = last_seq.load(Ordering::Relaxed);
                if final_seq > 0 {
                    drop(queue.save_cursor(&cursor_key, final_seq));
                }
                ping_task.abort();
                cursor_saver_task.abort();
                return ConnectionResult::Timeout;
            }

            let msg = tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(m)) => m,
                        Some(Err(e)) => {
                            let final_seq = last_seq.load(Ordering::Relaxed);
                            if final_seq > 0 {
                                drop(queue.save_cursor(&cursor_key, final_seq));
                            }
                            ping_task.abort();
                            cursor_saver_task.abort();
                            return ConnectionResult::Timeout;
                        }
                        None => break,
                    }
                }
                () = tokio::time::sleep(Duration::from_millis(100)) => continue,
            };

            // Any message (including Pong) resets the idle timer.
            last_message_time = std::time::Instant::now();

            if let Message::Binary(data) = msg {
                let queue_clone = Arc::clone(queue);
                let semaphore_clone = Arc::clone(&semaphore);
                let last_seq_clone = Arc::clone(&last_seq);

                // Acquire permit before spawning task
                let permit = match semaphore_clone.clone().acquire_owned().await {
                    Ok(permit) => permit,
                    Err(_) => {
                        error!("semaphore closed");
                        break;
                    }
                };

                tokio::spawn(async move {
                    let _permit = permit; // Hold permit for task duration

                    match Self::parse_and_enqueue(&data, &queue_clone, &last_seq_clone).await {
                        Ok(()) => {}
                        Err(e) => {
                            error!("failed to parse and enqueue message: {e}");
                        }
                    }
                });
            }
        }

        ping_task.abort();
        cursor_saver_task.abort();
        ConnectionResult::Timeout
    }

    fn extract_record(
        cid_str: &str,
        block_map: &Option<rsky_repo::block_map::BlockMap>,
        uri: &str,
    ) -> Option<String> {
        let blocks = block_map.as_ref()?;
        let cid = lexicon_cid::Cid::try_from(cid_str).ok()?;
        match rsky_repo::parse::get_and_parse_record(blocks, cid) {
            Ok(parsed) => serde_json::to_value(&parsed.record)
                .ok()
                .and_then(|v| serde_json::to_string(&v).ok()),
            Err(e) => {
                debug!("failed to parse record for {uri}: {e}");
                None
            }
        }
    }

    async fn parse_and_enqueue(
        data: &[u8],
        queue: &Arc<IndexQueue>,
        last_seq: &Arc<AtomicI64>,
    ) -> Result<()> {
        use rsky_firehose::firehose::read;
        use rsky_lexicon::com::atproto::sync::SubscribeRepos;

        let (_header, event) = match read(data) {
            Ok(Some(v)) => v,
            Ok(None) => return Ok(()), // Skip empty/unknown messages
            Err(e) => {
                // Malformed messages from the relay (e.g. negative integer in CBOR header)
                // must be skipped rather than retried — returning Err here causes the caller
                // to log the error but the connection stays alive and the relay advances to
                // the next sequence number automatically.
                warn!("skipping unparseable firehose message: {e}");
                return Ok(());
            }
        };

        // Handle different event types
        match event {
            SubscribeRepos::Commit(commit) => {
                last_seq.store(commit.seq, Ordering::Relaxed);

                if commit.ops.is_empty() {
                    return Ok(());
                }

                let block_map = if commit.blocks.is_empty() {
                    None
                } else {
                    match rsky_repo::car::read_car(commit.blocks.clone()).await {
                        Ok(output) => Some(output.blocks),
                        Err(e) => {
                            warn!("failed to parse CAR blocks for seq={}: {e}", commit.seq);
                            None
                        }
                    }
                };

                for op in commit.ops {
                    let uri = format!("at://{}/{}", commit.repo, op.path);

                    let (operation, record) = match op.action.as_str() {
                        "create" | "update" => {
                            if let Some(ref cid) = op.cid {
                                let cid_str = cid.to_string();
                                let op_type = if op.action == "create" {
                                    IndexOperation::Create {
                                        uri: uri.clone(),
                                        cid: cid_str.clone(),
                                    }
                                } else {
                                    IndexOperation::Update {
                                        uri: uri.clone(),
                                        cid: cid_str.clone(),
                                    }
                                };
                                let record = Self::extract_record(&cid_str, &block_map, &uri);
                                (op_type, record)
                            } else {
                                warn!("{} operation missing cid for {}", op.action, uri);
                                continue;
                            }
                        }
                        "delete" => (IndexOperation::Delete { uri }, None),
                        _ => {
                            warn!("unknown operation action: {}", op.action);
                            continue;
                        }
                    };

                    let job = IndexJob {
                        repo: commit.repo.clone(),
                        commit_cid: commit.commit.to_string(),
                        rev: commit.rev.clone(),
                        operation,
                        record_json: record,
                    };

                    queue.enqueue_live(&job)?;
                }
            }
            SubscribeRepos::Handle(handle) => {
                debug!("handle update: {} -> {}", handle.did, handle.handle);
                // Handle updates are processed inline, not queued
                // TODO: update actor table with new handle
            }
            // Migrate is not in current SubscribeRepos enum
            // SubscribeRepos::Migrate(migrate) => {
            //     warn!(
            //         "repo migration for {}, from {} to {}",
            //         migrate.did, migrate.migrate_to, migrate.seq
            //     );
            //     last_seq.store(migrate.seq, Ordering::Relaxed);
            //     Ok(())
            // }
            SubscribeRepos::Tombstone(tombstone) => {
                debug!("tombstone event: {}", tombstone.did);
                // Tombstone indicates account deletion
                // TODO: handle account deletion
            }
            SubscribeRepos::Identity(identity) => {
                debug!("identity event: {}", identity.did);
                // Identity events indicate DID document updates
                // TODO: refresh DID cache
            }
            SubscribeRepos::Account(account) => {
                debug!("account event: {}", account.did);
                // Account events indicate account status changes
                // TODO: handle account status changes
            } // Info is not in current SubscribeRepos enum
              // SubscribeRepos::Info(info) => {
              //     if info.name == "OutdatedCursor" {
              //         warn!("received OutdatedCursor info message");
              //         return Ok(());
              //     }
              //
              //     if let Some(msg) = info.message.as_ref() {
              //         if msg.contains("ConsumerTooSlow") {
              //             warn!("firehose consumer too slow: {msg}");
              //             return Ok(());
              //         }
              //     }
              //
              //     info!("received info message: {:?}", info);
              //     Ok(())
              // }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum ConnectionResult {
    Success,
    Timeout,
    FutureCursor,
}
