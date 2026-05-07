use appview_api::{AppStateInner, create_router};
use appview_core::error::AppViewError;
use appview_db::{create_pool, run_migrations};
use appview_firehose::FirehoseConsumer;
use appview_indexer::Indexer;
use appview_queue::IndexQueue;
use axum::body::Body;
use axum::http::{Request, Response};
use metrics::{counter, histogram};
use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_target(false).init();

    // Install the default Rustls crypto provider (aws-lc-rs) so that
    // tokio-tungstenite WSS connections work without panicking.
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok(); // ok() — ignore if already installed

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/appview".to_string());
    // Pool size: keep total connections well below pgbouncer pool_size=80.
    // Each pool is per-process; 15 connections handles all appview workloads
    // without exhausting the pgbouncer session-mode pool shared with PDS/feedgen.
    let db = create_pool(&database_url, 15)
        .await
        .map_err(|e| AppViewError::Database(e.to_string()))?;

    info!("Database pool created");

    // Run schema migrations on startup — creates tables if they don't exist.
    // The migration SQL is embedded at compile time.
    run_migrations(&db)
        .await
        .map_err(|e| AppViewError::Database(e.to_string()))?;
    info!("Schema migrations applied");

    let handle: PrometheusHandle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder");

    let queue_path =
        std::env::var("QUEUE_PATH").unwrap_or_else(|_| "/data/appview-queue".to_string());
    let queue = Arc::new(IndexQueue::new(Some(std::path::PathBuf::from(queue_path)))?);
    // Reuse the shared pool instead of creating a second pool in AppStateInner.
    let state = Arc::new(AppStateInner::new_with_pool(db.clone()).await?);

    // Firehose consumer — relay hosts from RELAY_HOSTS env var (comma-separated WSS URLs)
    // Falls back to the public Bluesky relay if not set.
    // The FirehoseConsumer normalizes the URL (strips wss:// prefix, appends XRPC path).
    let relay_hosts: Vec<String> = std::env::var("RELAY_HOSTS")
        .unwrap_or_else(|_| "wss://bsky.network".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    info!("Connecting to relay hosts: {:?}", relay_hosts);

    let firehose_queue = queue.clone();
    tokio::spawn(async move {
        let consumer = FirehoseConsumer::new(relay_hosts, firehose_queue);
        if let Err(e) = consumer.run().await {
            error!("Firehose consumer error: {}", e);
        }
    });

    // Indexer
    let indexer_db = db.clone();
    let indexer_queue = queue.clone();
    tokio::spawn(async move {
        let indexer = Indexer::new(indexer_queue, indexer_db);
        if let Err(e) = indexer.run_live().await {
            error!("Indexer error: {}", e);
        }
    });

    // Build router with metrics endpoint + middleware
    let metrics_renderer = handle.clone();
    let app = create_router(state)
        .route(
            "/metrics",
            axum::routing::get(move || {
                let h = metrics_renderer.clone();
                async move {
                    let rendered = h.render();
                    ([("content-type", "text/plain; version=0.0.4")], rendered)
                }
            }),
        )
        .layer(axum::middleware::from_fn(metrics_middleware));

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    info!("AppView API listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn metrics_middleware(req: Request<Body>, next: axum::middleware::Next) -> Response<Body> {
    let path = req.uri().path().to_owned();
    let start = std::time::Instant::now();
    counter!("appview_requests_total", "path" => path.clone()).increment(1);
    let resp = next.run(req).await;
    histogram!("appview_request_duration_seconds", "path" => path)
        .record(start.elapsed().as_secs_f64());
    resp
}
