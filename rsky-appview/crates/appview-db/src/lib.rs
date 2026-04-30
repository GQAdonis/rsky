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
