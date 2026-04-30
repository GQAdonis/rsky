use appview_core::error::{AppViewError, Result};
use fjall::{Config, Keyspace, PartitionCreateOptions, PartitionHandle};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Block size for Fjall partitions (64KB)
const BLOCK_SIZE: u32 = 64 * 1024;
/// Memtable size (32MB)
const MEMTABLE_SIZE: u32 = 32 * 1024 * 1024;
/// Cache size (512MB)
const CACHE_SIZE: u64 = 512 * 1024 * 1024;
/// Write buffer size (256MB)
const WRITE_BUFFER_SIZE: u64 = 256 * 1024 * 1024;
/// Fsync interval (1000ms)
const FSYNC_MS: Option<u16> = Some(1000);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexJob {
    pub repo: String,
    pub commit_cid: String,
    pub rev: String,
    pub operation: IndexOperation,
    #[serde(default)]
    pub record: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexOperation {
    Create { uri: String, cid: String },
    Update { uri: String, cid: String },
    Delete { uri: String },
}

pub struct IndexQueue {
    #[allow(dead_code)]
    db: Arc<Keyspace>,
    live_queue: PartitionHandle,
    backfill_queue: PartitionHandle,
    cursors: PartitionHandle,
}

impl IndexQueue {
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| "appview_queue".into());

        let db = Config::new(&path)
            .cache_size(CACHE_SIZE)
            .max_write_buffer_size(WRITE_BUFFER_SIZE)
            .fsync_ms(FSYNC_MS)
            .open()
            .map_err(|e| AppViewError::Storage(format!("failed to open queue: {e}")))?;

        let db = Arc::new(db);

        let live_queue = db
            .open_partition(
                "live_queue",
                PartitionCreateOptions::default()
                    .max_memtable_size(MEMTABLE_SIZE)
                    .block_size(BLOCK_SIZE),
            )
            .map_err(|e| AppViewError::Storage(format!("failed to open live_queue: {e}")))?;

        let backfill_queue = db
            .open_partition(
                "backfill_queue",
                PartitionCreateOptions::default()
                    .max_memtable_size(MEMTABLE_SIZE)
                    .block_size(BLOCK_SIZE),
            )
            .map_err(|e| AppViewError::Storage(format!("failed to open backfill_queue: {e}")))?;

        let cursors = db
            .open_partition(
                "cursors",
                PartitionCreateOptions::default()
                    .max_memtable_size(MEMTABLE_SIZE)
                    .block_size(BLOCK_SIZE),
            )
            .map_err(|e| AppViewError::Storage(format!("failed to open cursors: {e}")))?;

        Ok(Self {
            db,
            live_queue,
            backfill_queue,
            cursors,
        })
    }

    pub fn enqueue_live(&self, job: &IndexJob) -> Result<()> {
        let key = format!(
            "{}:{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
            job.repo
        );
        let value = bincode::serialize(job)
            .map_err(|e| AppViewError::Storage(format!("failed to serialize job: {e}")))?;
        self.live_queue
            .insert(key.as_bytes(), value)
            .map_err(|e| AppViewError::Storage(format!("failed to enqueue live job: {e}")))?;
        Ok(())
    }

    pub fn dequeue_live(&self) -> Result<Option<(Vec<u8>, IndexJob)>> {
        let mut iter = self.live_queue.iter();
        if let Some(entry) = iter.next() {
            let (key, value) = entry
                .map_err(|e| AppViewError::Storage(format!("failed to read live queue: {e}")))?;
            let job: IndexJob = bincode::deserialize(&value)
                .map_err(|e| AppViewError::Storage(format!("failed to deserialize job: {e}")))?;
            let key_vec = key.to_vec();
            self.live_queue
                .remove(key)
                .map_err(|e| AppViewError::Storage(format!("failed to remove job: {e}")))?;
            Ok(Some((key_vec, job)))
        } else {
            Ok(None)
        }
    }

    pub fn enqueue_backfill(&self, job: &IndexJob) -> Result<()> {
        let key = format!(
            "{}:{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
            job.repo
        );
        let value = bincode::serialize(job)
            .map_err(|e| AppViewError::Storage(format!("failed to serialize job: {e}")))?;
        self.backfill_queue
            .insert(key.as_bytes(), value)
            .map_err(|e| AppViewError::Storage(format!("failed to enqueue backfill job: {e}")))?;
        Ok(())
    }

    pub fn dequeue_backfill(&self) -> Result<Option<(Vec<u8>, IndexJob)>> {
        let mut iter = self.backfill_queue.iter();
        if let Some(entry) = iter.next() {
            let (key, value) = entry.map_err(|e| {
                AppViewError::Storage(format!("failed to read backfill queue: {e}"))
            })?;
            let job: IndexJob = bincode::deserialize(&value)
                .map_err(|e| AppViewError::Storage(format!("failed to deserialize job: {e}")))?;
            let key_vec = key.to_vec();
            self.backfill_queue
                .remove(key)
                .map_err(|e| AppViewError::Storage(format!("failed to remove job: {e}")))?;
            Ok(Some((key_vec, job)))
        } else {
            Ok(None)
        }
    }

    pub fn save_cursor(&self, stream: &str, cursor: i64) -> Result<()> {
        self.cursors
            .insert(stream.as_bytes(), cursor.to_le_bytes())
            .map_err(|e| AppViewError::Storage(format!("failed to save cursor: {e}")))?;
        Ok(())
    }

    pub fn load_cursor(&self, stream: &str) -> Result<Option<i64>> {
        match self
            .cursors
            .get(stream.as_bytes())
            .map_err(|e| AppViewError::Storage(format!("failed to load cursor: {e}")))?
        {
            Some(bytes) => {
                let arr: [u8; 8] = bytes
                    .as_ref()
                    .try_into()
                    .map_err(|_| AppViewError::Storage("invalid cursor format".to_string()))?;
                Ok(Some(i64::from_le_bytes(arr)))
            }
            None => Ok(None),
        }
    }

    pub fn delete_cursor(&self, stream: &str) -> Result<()> {
        self.cursors
            .remove(stream.as_bytes())
            .map_err(|e| AppViewError::Storage(format!("failed to delete cursor: {e}")))?;
        Ok(())
    }
}
