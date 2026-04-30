use crate::{
    error::Result,
    types::{AtUri, Cid, Did},
};

/// The action that triggered this write to the firehose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteAction {
    Create,
    Update,
    Delete,
}

/// A single indexing operation from the firehose.
pub struct IndexOp {
    pub uri: AtUri,
    pub cid: Cid,
    pub author: Did,
    pub record: serde_json::Value,
    pub indexed_at: chrono::DateTime<chrono::Utc>,
    pub action: WriteAction,
}

/// A collection-specific indexer that can index and delete records.
#[async_trait::async_trait]
pub trait RecordIndexer: Send + Sync {
    /// The NSID of the AT Protocol collection this indexer handles.
    /// E.g. `"app.bsky.feed.post"`.
    fn collection(&self) -> &'static str;

    /// Index (create or update) a record in the database.
    async fn index(&self, op: IndexOp, db: &sqlx::PgPool) -> Result<()>;

    /// Remove a record from the database by its AT URI.
    async fn delete(&self, uri: &AtUri, db: &sqlx::PgPool) -> Result<()>;
}
