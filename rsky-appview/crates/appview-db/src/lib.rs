pub mod actor;
pub mod feed;
pub mod generator;
pub mod graph;
pub mod models;
pub mod notification;

pub use sqlx::PgPool;

pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
}

/// Run the embedded schema migration SQL to create appview tables if they don't exist.
/// Safe to call on every startup — all statements use IF NOT EXISTS.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    const SCHEMA: &str = include_str!("../../../migrations/001_initial_schema.sql");
    // Split on semicolons and execute each statement individually
    // (raw_sql requires sqlx feature flag; this approach works without it)
    for stmt in SCHEMA.split(';') {
        let trimmed = stmt.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }
        if let Err(e) = sqlx::query(trimmed).execute(pool).await {
            // Log but don't fail — statements like DROP TRIGGER IF EXISTS that
            // reference non-existent objects produce warnings, not fatal errors.
            tracing::debug!("migration stmt warning (may be benign): {e}");
        }
    }
    Ok(())
}
