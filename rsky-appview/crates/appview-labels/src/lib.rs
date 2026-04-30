use appview_core::error::{AppViewError, Result};
use appview_db::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Label definitions from com.atproto.label.defs
#[derive(Debug, Clone)]
pub struct Label {
    pub src: String,
    pub uri: String,
    pub cid: Option<String>,
    pub val: String,
    pub neg: bool,
    pub cts: String,
}

/// In-memory label cache for fast lookup
#[derive(Clone)]
pub struct LabelStore {
    pool: PgPool,
    cache: Arc<RwLock<HashMap<String, Vec<Label>>>>,
}

impl LabelStore {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get labels for a URI from cache or DB
    pub async fn get_labels(&self, uri: &str) -> Result<Vec<Label>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(labels) = cache.get(uri) {
                return Ok(labels.clone());
            }
        }

        // Load from DB
        let labels = self.load_labels_from_db(uri).await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(uri.to_string(), labels.clone());
        }

        Ok(labels)
    }

    /// Store a new label
    pub async fn store_label(&self, label: Label) -> Result<()> {
        // Insert into DB
        sqlx::query(
            r#"
            INSERT INTO label (src, uri, cid, val, neg, cts)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (src, uri, cid, val) DO UPDATE
            SET neg = EXCLUDED.neg, cts = EXCLUDED.cts
            "#,
        )
        .bind(&label.src)
        .bind(&label.uri)
        .bind(&label.cid)
        .bind(&label.val)
        .bind(label.neg)
        .bind(&label.cts)
        .execute(&self.pool)
        .await
        .map_err(|e| AppViewError::Storage(format!("failed to store label: {e}")))?;

        // Invalidate cache for this URI
        {
            let mut cache = self.cache.write().await;
            cache.remove(&label.uri);
        }

        debug!("stored label {} for {}", label.val, label.uri);
        Ok(())
    }

    /// Load labels from database
    async fn load_labels_from_db(&self, uri: &str) -> Result<Vec<Label>> {
        let rows = sqlx::query_as::<_, LabelRow>(
            r#"
            SELECT src, uri, cid, val, neg, cts
            FROM label
            WHERE uri = $1 AND neg = false
            ORDER BY cts DESC
            "#,
        )
        .bind(uri)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppViewError::Storage(format!("failed to load labels: {e}")))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Clear cache for a URI
    pub async fn invalidate(&self, uri: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(uri);
    }
}

#[derive(sqlx::FromRow)]
struct LabelRow {
    src: String,
    uri: String,
    cid: Option<String>,
    val: String,
    neg: bool,
    cts: String,
}

impl From<LabelRow> for Label {
    fn from(row: LabelRow) -> Self {
        Label {
            src: row.src,
            uri: row.uri,
            cid: row.cid,
            val: row.val,
            neg: row.neg,
            cts: row.cts,
        }
    }
}

/// Filter labels based on viewer preferences
pub fn filter_labels(labels: &[Label], viewer_prefs: Option<&ViewerPrefs>) -> Vec<Label> {
    if let Some(prefs) = viewer_prefs {
        labels
            .iter()
            .filter(|l| !prefs.hidden_labels.contains(&l.val))
            .cloned()
            .collect()
    } else {
        labels.to_vec()
    }
}

/// Viewer label preferences
#[derive(Debug, Clone, Default)]
pub struct ViewerPrefs {
    pub hidden_labels: Vec<String>,
}
