use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use std::{io, thread};

use exponential_backoff::{Backoff, IntoIter as BackoffIter};
use hashbrown::{HashMap, HashSet};
use magnetic::Consumer;
use magnetic::buffer::dynamic::DynamicBufferP2;
use thiserror::Error;
use tokio::runtime::Handle;

use crate::BAN_REFRESH_NEEDED;
use crate::PgPool;
use crate::SHUTDOWN;
use crate::config::{BAN_REFRESH_INTERVAL, CAPACITY_STATUS};
use crate::crawler::RequestCrawl;
use crate::crawler::types::{Command, CommandSender, RequestCrawlReceiver, Status, StatusReceiver};
use crate::crawler::worker::{Worker, WorkerError};
use crate::types::{Cursor, MessageSender};

const SLEEP: Duration = Duration::from_millis(10);

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("spawn error: {0}")]
    Spawn(#[from] io::Error),
    #[error("worker error: {0}")]
    Worker(#[from] WorkerError),
    #[error("rtrb error: {0}")]
    Push(#[from] Box<rtrb::PushError<Command>>),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("join error")]
    Join,
}

impl From<rtrb::PushError<Command>> for ManagerError {
    fn from(value: rtrb::PushError<Command>) -> Self {
        Box::new(value).into()
    }
}

#[derive(Debug)]
struct WorkerHandle {
    pub command_tx: CommandSender,
    pub thread_handle: thread::JoinHandle<Result<(), WorkerError>>,
}

pub struct Manager {
    workers: Box<[WorkerHandle]>,
    next_id: usize,
    hosts: HashMap<String, [BackoffIter; 2]>,
    retries: BTreeMap<Instant, (usize, String)>,
    banned: HashSet<String>,
    last_ban_check: Instant,
    pool: PgPool,
    /// Handle to the main Tokio runtime used to execute async DB queries from
    /// this synchronous OS thread. Using a Handle (not a new Runtime) ensures
    /// the pool's I/O driver and background tasks run on the same runtime that
    /// created the PgPool, preventing connection-acquisition timeouts that occur
    /// when a pool is used from a different runtime's block_on.
    rt: Handle,
    request_crawl_rx: RequestCrawlReceiver,
    status_rx: StatusReceiver,
}

impl Manager {
    pub fn new(
        n_workers: usize, message_tx: &MessageSender, request_crawl_rx: RequestCrawlReceiver,
        pool: PgPool, rt: Handle,
    ) -> Result<Self, ManagerError> {
        #[expect(clippy::unwrap_used)]
        let (status_tx, status_rx) =
            magnetic::mpsc::mpsc_queue(DynamicBufferP2::new(CAPACITY_STATUS).unwrap());
        let workers = (0..n_workers)
            .map(|worker_id| -> Result<_, ManagerError> {
                let message_tx = message_tx.clone();
                let status_tx = status_tx.clone();
                let (command_tx, command_rx) = rtrb::RingBuffer::new(CAPACITY_STATUS);
                let thread_handle =
                    thread::Builder::new().name(format!("rsky-crawl-{worker_id}")).spawn(
                        move || Worker::new(worker_id, message_tx, command_rx, status_tx)?.run(),
                    )?;
                Ok(WorkerHandle { command_tx, thread_handle })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let banned = HashSet::new();
        let now = Instant::now();
        let last_ban_check = now.checked_sub(BAN_REFRESH_INTERVAL).unwrap_or(now);
        Ok(Self {
            workers: workers.into_boxed_slice(),
            next_id: 0,
            hosts: HashMap::new(),
            retries: BTreeMap::new(),
            banned,
            last_ban_check,
            pool,
            rt,
            request_crawl_rx,
            status_rx,
        })
    }

    pub fn run(mut self) -> Result<(), ManagerError> {
        while self.update()? {
            thread::sleep(SLEEP);
        }
        tracing::info!("shutting down crawler");
        self.shutdown()
    }

    pub fn shutdown(self) -> Result<(), ManagerError> {
        SHUTDOWN.store(true, Ordering::Relaxed);
        for (id, worker) in self.workers.into_iter().enumerate() {
            if let Err(err) = worker.thread_handle.join().map_err(|_| ManagerError::Join)? {
                tracing::warn!(%id, %err, "crawler worker error");
            }
        }
        Ok(())
    }

    fn update(&mut self) -> Result<bool, ManagerError> {
        if SHUTDOWN.load(Ordering::Relaxed) {
            return Ok(false);
        }

        // Refresh the ban list when either:
        // (a) the PgListener task set BAN_REFRESH_NEEDED (instant NOTIFY-based path), or
        // (b) the fallback polling interval has elapsed (handles listener restarts / missed NOTIFYs).
        let notify_triggered = BAN_REFRESH_NEEDED.swap(false, Ordering::Relaxed);
        let interval_elapsed = self.last_ban_check.elapsed() > BAN_REFRESH_INTERVAL;
        if notify_triggered || interval_elapsed {
            if let Err(err) = self.refresh_bans() {
                tracing::warn!(%err, "unable to refresh banned hosts");
            }
            self.last_ban_check = Instant::now();
        }

        if let Some(entry) = self.retries.first_entry() {
            if *entry.key() < Instant::now() {
                let (id, hostname) = entry.remove();
                if self.banned.contains(&hostname) {
                    tracing::debug!(%hostname, "skipping retry for banned host");
                } else {
                    let prev = self.next_id;
                    self.next_id = id;
                    self.handle_connect(RequestCrawl { hostname, cursor: None })?;
                    self.next_id = prev;
                }
            }
        }

        if let Ok(status) = self.status_rx.try_pop() {
            self.handle_status(status);
        }

        if let Ok(request_crawl) = self.request_crawl_rx.pop() {
            if self.banned.contains(&request_crawl.hostname) {
                tracing::debug!(host = %request_crawl.hostname, "ignoring requestCrawl for banned host");
            } else if !self.hosts.contains_key(&request_crawl.hostname) {
                self.handle_connect(request_crawl)?;
            }
        }

        Ok(true)
    }

    fn handle_status(&mut self, status: Status) {
        match status {
            Status::Disconnected { worker_id: id, hostname, connected } => {
                if self.banned.contains(&hostname) {
                    tracing::debug!(%hostname, "ignoring disconnect for banned host");
                    return;
                }
                let Some(backoffs) = self.hosts.get_mut(&hostname) else {
                    tracing::debug!(%hostname, "ignoring disconnect for unknown host");
                    return;
                };
                #[expect(clippy::unwrap_used)]
                let backoff = backoffs.get_mut(usize::from(connected)).unwrap();
                let Some(Some(delay)) = backoff.next() else { unreachable!() };
                let next = Instant::now() + delay;
                assert!(self.retries.insert(next, (id, hostname)).is_none());
            }
        }
    }

    fn handle_connect(&mut self, mut request_crawl: RequestCrawl) -> Result<(), ManagerError> {
        self.hosts.entry(request_crawl.hostname.clone()).or_insert_with(|| {
            let backoff_connect =
                Backoff::new(u32::MAX, Duration::from_secs(60), Duration::from_secs(60 * 60 * 6));
            let backoff_reconnect =
                Backoff::new(u32::MAX, Duration::from_secs(1), Duration::from_secs(60 * 60));
            [backoff_connect.iter(), backoff_reconnect.iter()]
        });
        if request_crawl.cursor.is_none() {
            request_crawl.cursor = self.get_cursor(&request_crawl.hostname)?;
        }
        self.workers[self.next_id].command_tx.push(Command::Connect(request_crawl))?;
        self.next_id = (self.next_id + 1) % self.workers.len();
        thread::sleep(SLEEP);
        Ok(())
    }

    fn get_cursor(&self, host: &str) -> Result<Option<Cursor>, ManagerError> {
        let pool = self.pool.clone();
        let host = host.to_owned();
        self.rt
            .block_on(async move {
                let row = sqlx::query("SELECT cursor FROM hosts WHERE host = $1")
                    .bind(&host)
                    .fetch_optional(&pool)
                    .await?;
                Ok::<_, sqlx::Error>(row.map(|r| {
                    use sqlx::Row as _;
                    #[expect(clippy::cast_sign_loss)]
                    let cursor: i64 = r.try_get("cursor").unwrap_or(0);
                    Cursor::from(cursor as u64)
                }))
            })
            .map_err(ManagerError::Database)
    }

    fn refresh_bans(&mut self) -> Result<(), ManagerError> {
        let pool = self.pool.clone();
        let new_bans: HashSet<String> = self
            .rt
            .block_on(async move {
                use sqlx::Row as _;
                let rows = sqlx::query("SELECT host FROM banned_hosts").fetch_all(&pool).await?;
                Ok::<_, sqlx::Error>(
                    rows.into_iter().filter_map(|r| r.try_get::<String, _>("host").ok()).collect(),
                )
            })
            .map_err(ManagerError::Database)?;

        for host in &new_bans {
            if !self.banned.contains(host.as_str()) {
                tracing::warn!(%host, "host banned, sending disconnect");
                for worker in &mut *self.workers {
                    if let Err(err) = worker.command_tx.push(Command::Disconnect(host.clone())) {
                        tracing::warn!(%host, %err, "unable to send disconnect to worker");
                    }
                }
                self.hosts.remove(host.as_str());
                self.retries.retain(|_, (_, h)| h != host);
            }
        }

        self.banned = new_bans;
        Ok(())
    }
}
