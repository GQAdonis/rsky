use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, ScopedJoinHandle};
use std::time::Duration;

use clap::Parser;
use color_eyre::Result;
use file_rotate::compression::Compression;
use file_rotate::suffix::{AppendTimestamp, FileLimit};
use file_rotate::{ContentLimit, FileRotate, TimeFrequency};
use mimalloc::MiMalloc;
use rustls::crypto::aws_lc_rs::default_provider;
use signal_hook::consts::{SIGINT, TERM_SIGNALS};
use signal_hook::flag;
use signal_hook::iterator::SignalsInfo;
use signal_hook::iterator::exfiltrator::WithOrigin;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use sqlx::migrate::MigrateDatabase;
use sqlx::postgres::{PgListener, PgPoolOptions};

use rsky_relay::config::{
    CAPACITY_MSGS, CAPACITY_REQS, DATABASE_URL, METRICS_LISTEN, WORKERS_CRAWLERS,
    WORKERS_PUBLISHERS,
};
use rsky_relay::{
    BAN_REFRESH_NEEDED, CrawlerManager, MessageRecycle, PgPool, PublisherManager, RelayError,
    SHUTDOWN, Server, ValidatorManager, metrics,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const SLEEP: Duration = Duration::from_millis(10);

#[derive(Debug, clap::Parser)]
pub struct Args {
    #[clap(short, long, requires = "private_key")]
    certs: Option<PathBuf>,
    #[clap(short, long, requires = "certs")]
    private_key: Option<PathBuf>,
    #[cfg(not(feature = "labeler"))]
    #[clap(long)]
    no_plc_export: bool,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let file_appender = FileRotate::new(
        "rsky-relay.log",
        AppendTimestamp::default(FileLimit::MaxFiles(7)),
        ContentLimit::Time(TimeFrequency::Daily),
        Compression::OnRotate(0),
        None,
    );
    let (json_writer, _guard_json) = tracing_appender::non_blocking(file_appender);
    let (pretty_writer, _guard_pretty) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(Layer::new().json().with_ansi(false).with_writer(json_writer))
        .with(Layer::new().pretty().with_writer(pretty_writer))
        .init();
    color_eyre::install()?;

    #[expect(clippy::unwrap_used)]
    default_provider().install_default().unwrap();

    if let Some(addr_str) = METRICS_LISTEN.as_deref() {
        match addr_str.parse() {
            Ok(addr) => match metrics::install_listener(addr) {
                Ok(_) => tracing::info!(%addr_str, "metrics listener bound"),
                Err(err) => tracing::error!(%err, %addr_str, "failed to bind metrics listener"),
            },
            Err(err) => tracing::error!(%err, %addr_str, "invalid RELAY_METRICS_LISTEN"),
        }
    } else {
        // No listener configured: install an in-process recorder so describe()/counters work.
        if let Err(err) = metrics::install_recorder() {
            tracing::error!(%err, "failed to install metrics recorder");
        }
    }

    let args = Args::parse();

    // Ensure the database exists (no-op if already present) then run migrations.
    let db_url: &str = &DATABASE_URL;
    if !sqlx::Postgres::database_exists(db_url).await? {
        tracing::info!("creating relay database");
        sqlx::Postgres::create_database(db_url).await?;
    }
    let pool: PgPool = PgPoolOptions::new()
        .max_connections(10)
        .connect(db_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("database migrations complete");

    // Spawn a LISTEN task that watches the `banned_hosts_changed` channel and
    // sets BAN_REFRESH_NEEDED so the crawler manager reacts instantly instead
    // of waiting for the BAN_REFRESH_INTERVAL polling fallback.
    tokio::spawn(async move {
        loop {
            match PgListener::connect(db_url).await {
                Err(err) => {
                    tracing::warn!(%err, "ban-listener: failed to connect, retrying in 10s");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
                Ok(mut listener) => {
                    if let Err(err) = listener.listen("banned_hosts_changed").await {
                        tracing::warn!(%err, "ban-listener: failed to LISTEN, retrying in 10s");
                        tokio::time::sleep(Duration::from_secs(10)).await;
                        continue;
                    }
                    tracing::info!("ban-listener: listening on banned_hosts_changed");
                    loop {
                        match listener.recv().await {
                            Ok(notification) => {
                                tracing::debug!(
                                    payload = notification.payload(),
                                    "ban-listener: received notification"
                                );
                                BAN_REFRESH_NEEDED.store(true, Ordering::Relaxed);
                            }
                            Err(err) => {
                                tracing::warn!(%err, "ban-listener: connection lost, reconnecting");
                                break;
                            }
                        }
                    }
                }
            }
        }
    });

    let terminate_now = Arc::new(AtomicBool::new(false));
    flag::register_conditional_shutdown(SIGINT, 1, Arc::clone(&terminate_now))?;
    flag::register(SIGINT, Arc::clone(&terminate_now))?;

    let (message_tx, message_rx) =
        thingbuf::mpsc::blocking::with_recycle(CAPACITY_MSGS, MessageRecycle);
    let (request_crawl_tx, request_crawl_rx) = rtrb::RingBuffer::new(CAPACITY_REQS);
    let (subscribe_repos_tx, subscribe_repos_rx) = rtrb::RingBuffer::new(CAPACITY_REQS);
    let validator = ValidatorManager::new(message_rx, pool.clone())?;
    let server =
        Server::new(args.certs.zip(args.private_key), request_crawl_tx, subscribe_repos_tx, pool.clone())?;
    // Track validator status via AtomicBool so the main loop can detect
    // validator death without owning the JoinHandle (which stays outside the closure).
    let validator_dead = Arc::new(AtomicBool::new(false));
    let validator_dead_clone = Arc::clone(&validator_dead);
    let handle = tokio::spawn(async move {
        let result = validator.run().await;
        validator_dead_clone.store(true, Ordering::Relaxed);
        result
    });
    let crawler = CrawlerManager::new(WORKERS_CRAWLERS, &message_tx, request_crawl_rx, pool.clone())?;
    let publisher = PublisherManager::new(WORKERS_PUBLISHERS, subscribe_repos_rx)?;
    #[expect(clippy::vec_init_then_push)]
    let ret = thread::scope(move |s| {
        let mut handles = Vec::<ScopedJoinHandle<'_, Result<_, RelayError>>>::new();
        handles.push(
            thread::Builder::new()
                .name("rsky-crawl".into())
                .spawn_scoped(s, move || crawler.run().map_err(Into::into))?,
        );
        handles.push(
            thread::Builder::new()
                .name("rsky-pub".into())
                .spawn_scoped(s, move || publisher.run().map_err(Into::into))?,
        );
        handles.push(
            thread::Builder::new()
                .name("rsky-server".into())
                .spawn_scoped(s, move || server.run().map_err(Into::into))?,
        );
        #[expect(clippy::expect_used)]
        let mut signals =
            SignalsInfo::<WithOrigin>::new(TERM_SIGNALS).expect("failed to init signals");
        let mut validator_logged = false;
        'outer: loop {
            for signal_info in signals.pending() {
                if TERM_SIGNALS.contains(&signal_info.signal) {
                    break 'outer;
                }
            }
            for h in &handles {
                if h.is_finished() {
                    break 'outer;
                }
            }
            // Validator dying should NOT take down the relay.
            // Log it once and keep serving firehose to subscribers.
            if validator_dead.load(Ordering::Relaxed) && !validator_logged {
                validator_logged = true;
                tracing::error!("validator stopped unexpectedly, relay continues serving");
            }
            thread::sleep(SLEEP);
        }
        tracing::info!("shutting down");
        SHUTDOWN.store(true, Ordering::Relaxed);
        for h in handles {
            if let Ok(res) = h.join() {
                res?;
            }
        }
        Ok(())
    });
    // Clean up validator task
    if !handle.is_finished() {
        handle.abort();
    }
    match handle.await {
        Ok(Ok(())) => tracing::info!("validator stopped cleanly"),
        Ok(Err(e)) => tracing::error!("validator stopped with error: {e}"),
        Err(e) if e.is_cancelled() => tracing::info!("validator aborted on shutdown"),
        Err(e) => tracing::error!("validator task panicked: {e}"),
    }
    ret
}
