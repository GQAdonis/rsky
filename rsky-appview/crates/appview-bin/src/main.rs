use appview_api::{AppStateInner, create_router};
use appview_core::error::AppViewError;
use appview_db::create_pool;
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

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/appview".to_string());
    let db = create_pool(&database_url, 32)
        .await
        .map_err(|e| AppViewError::Database(e.to_string()))?;

    info!("Database pool created");

    let handle: PrometheusHandle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder");

    let queue = Arc::new(IndexQueue::new(Some(std::path::PathBuf::from(
        "/tmp/appview-queue",
    )))?);
    let state = Arc::new(AppStateInner::new(&database_url).await?);

    // Firehose consumer
    let firehose_queue = queue.clone();
    tokio::spawn(async move {
        let consumer = FirehoseConsumer::new(
            vec!["wss://bsky.network/xrpc/com.atproto.sync.subscribeRepos".to_string()],
            firehose_queue,
        );
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
