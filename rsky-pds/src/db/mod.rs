use anyhow::Result;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use rocket_sync_db_pools::database;
use std::env;
use std::fmt::{Debug, Formatter};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_migrations(database_url: &str) {
    let mut conn = PgConnection::establish(database_url)
        .unwrap_or_else(|e| panic!("Failed to connect to database for migrations: {e}"));
    conn.run_pending_migrations(MIGRATIONS)
        .unwrap_or_else(|e| panic!("Failed to run database migrations: {e}"));
    tracing::info!("Database migrations applied successfully");
}

#[database("pg_db")]
pub struct DbConn(PgConnection);

impl Debug for DbConn {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[tracing::instrument(skip_all)]
pub fn establish_connection_for_sequencer() -> Result<PgConnection> {
    dotenv().ok();
    tracing::debug!("Establishing database connection for Sequencer");
    let database_url = env::var("DATABASE_URL").unwrap_or("".into());
    let result = PgConnection::establish(&database_url).map_err(|error| {
        let context = format!("Error connecting to {database_url:?}");
        anyhow::Error::new(error).context(context)
    })?;

    Ok(result)
}
