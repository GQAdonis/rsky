use appview_core::error::{AppViewError, Result};
use appview_db::PgPool;
use appview_queue::{IndexJob, IndexOperation, IndexQueue};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{Duration, interval};
use tracing::{debug, error, info, warn};

const MAX_CONCURRENT_JOBS: usize = 100;
const POLL_INTERVAL_MS: u64 = 100;

pub struct Indexer {
    queue: Arc<IndexQueue>,
    pool: PgPool,
    semaphore: Arc<Semaphore>,
}

impl Indexer {
    pub fn new(queue: Arc<IndexQueue>, pool: PgPool) -> Self {
        Self {
            queue,
            pool,
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_JOBS)),
        }
    }

    pub async fn run_live(&self) -> Result<()> {
        info!("starting live indexer");
        let mut ticker = interval(Duration::from_millis(POLL_INTERVAL_MS));

        loop {
            ticker.tick().await;

            match self.queue.dequeue_live()? {
                Some((_key, job)) => {
                    let permit = self.semaphore.clone().acquire_owned().await.map_err(|e| {
                        AppViewError::Internal(format!("failed to acquire semaphore: {e}"))
                    })?;

                    let pool = self.pool.clone();
                    tokio::spawn(async move {
                        if let Err(e) = process_job(&job, &pool).await {
                            error!("failed to process job: {e}");
                        }
                        drop(permit);
                    });
                }
                None => {}
            }
        }
    }

    pub async fn run_backfill(&self) -> Result<()> {
        info!("starting backfill indexer");
        let mut ticker = interval(Duration::from_millis(POLL_INTERVAL_MS));

        loop {
            ticker.tick().await;

            match self.queue.dequeue_backfill()? {
                Some((_key, job)) => {
                    let permit = self.semaphore.clone().acquire_owned().await.map_err(|e| {
                        AppViewError::Internal(format!("failed to acquire semaphore: {e}"))
                    })?;

                    let pool = self.pool.clone();
                    tokio::spawn(async move {
                        if let Err(e) = process_job(&job, &pool).await {
                            error!("failed to process backfill job: {e}");
                        }
                        drop(permit);
                    });
                }
                None => {}
            }
        }
    }
}

async fn process_job(job: &IndexJob, pool: &PgPool) -> Result<()> {
    debug!(
        "processing {} for {} at {}",
        match &job.operation {
            IndexOperation::Create { .. } => "create",
            IndexOperation::Update { .. } => "update",
            IndexOperation::Delete { .. } => "delete",
        },
        job.repo,
        job.rev
    );

    match &job.operation {
        IndexOperation::Create { uri, cid } => {
            index_record(uri, cid, &job.repo, &job.record, pool).await?;
        }
        IndexOperation::Update { uri, cid } => {
            index_record(uri, cid, &job.repo, &job.record, pool).await?;
        }
        IndexOperation::Delete { uri } => {
            delete_record(uri, &job.repo, pool).await?;
        }
    }

    Ok(())
}

async fn index_record(
    uri: &str,
    cid: &str,
    repo: &str,
    record: &Option<serde_json::Value>,
    pool: &PgPool,
) -> Result<()> {
    let parts: Vec<&str> = uri
        .strip_prefix("at://")
        .unwrap_or(uri)
        .split('/')
        .collect();
    if parts.len() < 3 {
        warn!("invalid AT URI format: {}", uri);
        return Ok(());
    }

    let collection = parts[1];
    let rkey = parts[2];

    match collection {
        "app.bsky.feed.post" => {
            index_post(repo, rkey, cid, record, pool).await?;
        }
        "app.bsky.feed.like" => {
            index_like(repo, rkey, uri, record, pool).await?;
        }
        "app.bsky.feed.repost" => {
            index_repost(repo, rkey, uri, record, pool).await?;
        }
        "app.bsky.graph.follow" => {
            index_follow(repo, rkey, uri, record, pool).await?;
        }
        "app.bsky.graph.block" => {
            index_block(repo, uri, record, pool).await?;
        }
        "app.bsky.actor.profile" => {
            index_profile(repo, record, pool).await?;
        }
        _ => {
            debug!("skipping unknown collection: {}", collection);
        }
    }

    Ok(())
}

async fn delete_record(uri: &str, repo: &str, pool: &PgPool) -> Result<()> {
    let parts: Vec<&str> = uri
        .strip_prefix("at://")
        .unwrap_or(uri)
        .split('/')
        .collect();
    if parts.len() < 3 {
        warn!("invalid AT URI format: {}", uri);
        return Ok(());
    }

    let collection = parts[1];

    match collection {
        "app.bsky.feed.post" => {
            delete_post(uri, repo, pool).await?;
        }
        "app.bsky.feed.like" => {
            delete_like(uri, pool).await?;
        }
        "app.bsky.feed.repost" => {
            delete_repost(uri, pool).await?;
        }
        "app.bsky.graph.follow" => {
            delete_follow(uri, pool).await?;
        }
        "app.bsky.graph.block" => {
            delete_block(uri, pool).await?;
        }
        _ => {
            debug!("skipping deletion of unknown collection: {}", collection);
        }
    }

    Ok(())
}

fn now_iso() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

async fn ensure_actor(repo: &str, pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"INSERT INTO actor (did, "indexedAt")
           VALUES ($1, $2)
           ON CONFLICT (did) DO UPDATE SET "indexedAt" = EXCLUDED."indexedAt""#,
    )
    .bind(repo)
    .bind(now_iso())
    .execute(pool)
    .await?;
    Ok(())
}

async fn index_post(
    repo: &str,
    rkey: &str,
    cid: &str,
    record: &Option<serde_json::Value>,
    pool: &PgPool,
) -> Result<()> {
    let Some(rec) = record else {
        debug!("skipping post with no record data: {repo}/{rkey}");
        return Ok(());
    };

    let text = rec
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let created_at = rec
        .get("createdAt")
        .and_then(|v| v.as_str())
        .unwrap_or(&now_iso())
        .to_string();
    let reply_root = rec
        .get("reply")
        .and_then(|r| r.get("root"))
        .and_then(|r| r.get("uri"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let reply_parent = rec
        .get("reply")
        .and_then(|r| r.get("parent"))
        .and_then(|r| r.get("uri"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let indexed_at = now_iso();
    let uri = format!("at://{}/app.bsky.feed.post/{rkey}", repo);

    ensure_actor(repo, pool).await?;

    sqlx::query(
        r#"INSERT INTO post (uri, cid, creator, text, "replyRoot", "replyParent", "createdAt", "indexedAt")
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           ON CONFLICT (uri) DO UPDATE SET cid = EXCLUDED.cid, text = EXCLUDED.text,
             "replyRoot" = EXCLUDED."replyRoot", "replyParent" = EXCLUDED."replyParent",
             "indexedAt" = EXCLUDED."indexedAt""#,
    )
    .bind(&uri)
    .bind(cid)
    .bind(repo)
    .bind(&text)
    .bind(&reply_root)
    .bind(&reply_parent)
    .bind(&created_at)
    .bind(&indexed_at)
    .execute(pool)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;

    sqlx::query(
        r#"INSERT INTO feed_item ("postUri", "originatorDid", "sortAt")
           VALUES ($1, $2, $3)
           ON CONFLICT DO NOTHING"#,
    )
    .bind(&uri)
    .bind(repo)
    .bind(&indexed_at)
    .execute(pool)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;

    if let Some(ref parent_uri) = reply_parent {
        let parent_creator = extract_did_from_at_uri(parent_uri);
        if parent_creator != repo {
            sqlx::query(
                r#"INSERT INTO notification (did, author, "recordUri", "recordCid", reason, "reasonSubject", "isRead", "sortAt")
                   VALUES ($1, $2, $3, $4, 'reply', $5, false, $6)
                   ON CONFLICT DO NOTHING"#,
            )
            .bind(parent_creator)
            .bind(repo)
            .bind(&uri)
            .bind(cid)
            .bind(parent_uri)
            .bind(&indexed_at)
            .execute(pool)
            .await
            .map_err(|e| AppViewError::Database(e.to_string()))?;
        }
    }

    Ok(())
}

async fn index_like(
    repo: &str,
    _rkey: &str,
    uri: &str,
    record: &Option<serde_json::Value>,
    pool: &PgPool,
) -> Result<()> {
    let Some(rec) = record else {
        debug!("skipping like with no record data: {uri}");
        return Ok(());
    };

    let subject_uri = rec
        .get("subject")
        .and_then(|s| s.get("uri"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let subject_cid = rec
        .get("subject")
        .and_then(|s| s.get("cid"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let created_at = rec
        .get("createdAt")
        .and_then(|v| v.as_str())
        .unwrap_or(&now_iso())
        .to_string();
    let indexed_at = now_iso();

    if subject_uri.is_empty() {
        debug!("skipping like with empty subject: {uri}");
        return Ok(());
    }

    sqlx::query(
        r#"INSERT INTO like (uri, creator, subject, "subjectCid", "createdAt", "indexedAt")
           VALUES ($1, $2, $3, $4, $5, $6)
           ON CONFLICT (uri) DO UPDATE SET subject = EXCLUDED.subject,
             "subjectCid" = EXCLUDED."subjectCid", "indexedAt" = EXCLUDED."indexedAt""#,
    )
    .bind(uri)
    .bind(repo)
    .bind(&subject_uri)
    .bind(&subject_cid)
    .bind(&created_at)
    .bind(&indexed_at)
    .execute(pool)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;

    let post_creator = extract_did_from_at_uri(&subject_uri);
    if post_creator != repo {
        sqlx::query(
            r#"INSERT INTO notification (did, author, "recordUri", "recordCid", reason, "reasonSubject", "isRead", "sortAt")
               VALUES ($1, $2, $3, $4, 'like', $5, false, $6)
               ON CONFLICT DO NOTHING"#,
        )
        .bind(post_creator)
        .bind(repo)
        .bind(uri)
        .bind(&subject_cid)
        .bind(&subject_uri)
        .bind(&indexed_at)
        .execute(pool)
        .await
        .map_err(|e| AppViewError::Database(e.to_string()))?;
    }

    Ok(())
}

async fn index_repost(
    repo: &str,
    _rkey: &str,
    uri: &str,
    record: &Option<serde_json::Value>,
    pool: &PgPool,
) -> Result<()> {
    let Some(rec) = record else {
        debug!("skipping repost with no record data: {uri}");
        return Ok(());
    };

    let subject_uri = rec
        .get("subject")
        .and_then(|s| s.get("uri"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let created_at = rec
        .get("createdAt")
        .and_then(|v| v.as_str())
        .unwrap_or(&now_iso())
        .to_string();
    let indexed_at = now_iso();

    if subject_uri.is_empty() {
        debug!("skipping repost with empty subject: {uri}");
        return Ok(());
    }

    sqlx::query(
        r#"INSERT INTO repost (uri, creator, subject, "createdAt", "indexedAt")
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (uri) DO UPDATE SET subject = EXCLUDED.subject, "indexedAt" = EXCLUDED."indexedAt""#,
    )
    .bind(uri)
    .bind(repo)
    .bind(&subject_uri)
    .bind(&created_at)
    .bind(&indexed_at)
    .execute(pool)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;

    let post_creator = extract_did_from_at_uri(&subject_uri);
    if post_creator != repo {
        sqlx::query(
            r#"INSERT INTO notification (did, author, "recordUri", "recordCid", reason, "reasonSubject", "isRead", "sortAt")
               VALUES ($1, $2, $3, '', 'repost', $4, false, $5)
               ON CONFLICT DO NOTHING"#,
        )
        .bind(post_creator)
        .bind(repo)
        .bind(uri)
        .bind(&subject_uri)
        .bind(&indexed_at)
        .execute(pool)
        .await
        .map_err(|e| AppViewError::Database(e.to_string()))?;
    }

    Ok(())
}

async fn index_follow(
    repo: &str,
    _rkey: &str,
    uri: &str,
    record: &Option<serde_json::Value>,
    pool: &PgPool,
) -> Result<()> {
    let Some(rec) = record else {
        debug!("skipping follow with no record data: {uri}");
        return Ok(());
    };

    let subject_did = rec
        .get("subject")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let created_at = rec
        .get("createdAt")
        .and_then(|v| v.as_str())
        .unwrap_or(&now_iso())
        .to_string();
    let indexed_at = now_iso();

    if subject_did.is_empty() {
        debug!("skipping follow with empty subject: {uri}");
        return Ok(());
    }

    sqlx::query(
        r#"INSERT INTO follow (uri, creator, "subjectDid", "createdAt", "indexedAt")
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (uri) DO UPDATE SET "subjectDid" = EXCLUDED."subjectDid", "indexedAt" = EXCLUDED."indexedAt""#,
    )
    .bind(uri)
    .bind(repo)
    .bind(&subject_did)
    .bind(&created_at)
    .bind(&indexed_at)
    .execute(pool)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;

    sqlx::query(
        r#"INSERT INTO notification (did, author, "recordUri", "recordCid", reason, "reasonSubject", "isRead", "sortAt")
           VALUES ($1, $2, $3, '', 'follow', '', false, $4)
           ON CONFLICT DO NOTHING"#,
    )
    .bind(&subject_did)
    .bind(repo)
    .bind(uri)
    .bind(&indexed_at)
    .execute(pool)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;

    Ok(())
}

async fn index_block(
    repo: &str,
    uri: &str,
    record: &Option<serde_json::Value>,
    pool: &PgPool,
) -> Result<()> {
    let Some(rec) = record else {
        debug!("skipping block with no record data: {uri}");
        return Ok(());
    };

    let subject_did = rec
        .get("subject")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let created_at = rec
        .get("createdAt")
        .and_then(|v| v.as_str())
        .unwrap_or(&now_iso())
        .to_string();
    let indexed_at = now_iso();

    if subject_did.is_empty() {
        debug!("skipping block with empty subject: {uri}");
        return Ok(());
    }

    sqlx::query(
        r#"INSERT INTO actor_block (creator, "subjectDid", "createdAt", "indexedAt")
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (creator, "subjectDid") DO UPDATE SET "indexedAt" = EXCLUDED."indexedAt""#,
    )
    .bind(repo)
    .bind(&subject_did)
    .bind(&created_at)
    .bind(&indexed_at)
    .execute(pool)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;

    Ok(())
}

async fn index_profile(
    repo: &str,
    record: &Option<serde_json::Value>,
    pool: &PgPool,
) -> Result<()> {
    ensure_actor(repo, pool).await?;

    let Some(rec) = record else {
        debug!("skipping profile with no record data: {repo}");
        return Ok(());
    };

    let display_name = rec
        .get("displayName")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let description = rec
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let avatar_cid = extract_blob_cid(rec, "avatar");
    let banner_cid = extract_blob_cid(rec, "banner");
    let indexed_at = now_iso();

    sqlx::query(
        r#"INSERT INTO profile (creator, "displayName", description, "avatarCid", "bannerCid", "indexedAt")
           VALUES ($1, $2, $3, $4, $5, $6)
           ON CONFLICT (creator) DO UPDATE SET "displayName" = EXCLUDED."displayName",
             description = EXCLUDED.description, "avatarCid" = EXCLUDED."avatarCid",
             "bannerCid" = EXCLUDED."bannerCid", "indexedAt" = EXCLUDED."indexedAt""#,
    )
    .bind(repo)
    .bind(&display_name)
    .bind(&description)
    .bind(&avatar_cid)
    .bind(&banner_cid)
    .bind(&indexed_at)
    .execute(pool)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;

    Ok(())
}

async fn delete_post(uri: &str, repo: &str, pool: &PgPool) -> Result<()> {
    sqlx::query("DELETE FROM feed_item WHERE \"postUri\" = $1")
        .bind(uri)
        .execute(pool)
        .await
        .map_err(|e| AppViewError::Database(e.to_string()))?;

    sqlx::query("DELETE FROM notification WHERE \"recordUri\" = $1 OR \"reasonSubject\" = $1")
        .bind(uri)
        .execute(pool)
        .await
        .map_err(|e| AppViewError::Database(e.to_string()))?;

    sqlx::query("DELETE FROM post WHERE uri = $1 AND creator = $2")
        .bind(uri)
        .bind(repo)
        .execute(pool)
        .await
        .map_err(|e| AppViewError::Database(e.to_string()))?;

    Ok(())
}

async fn delete_like(uri: &str, pool: &PgPool) -> Result<()> {
    sqlx::query("DELETE FROM like WHERE uri = $1")
        .bind(uri)
        .execute(pool)
        .await
        .map_err(|e| AppViewError::Database(e.to_string()))?;

    Ok(())
}

async fn delete_repost(uri: &str, pool: &PgPool) -> Result<()> {
    sqlx::query("DELETE FROM repost WHERE uri = $1")
        .bind(uri)
        .execute(pool)
        .await
        .map_err(|e| AppViewError::Database(e.to_string()))?;

    Ok(())
}

async fn delete_follow(uri: &str, pool: &PgPool) -> Result<()> {
    sqlx::query("DELETE FROM follow WHERE uri = $1")
        .bind(uri)
        .execute(pool)
        .await
        .map_err(|e| AppViewError::Database(e.to_string()))?;

    Ok(())
}

async fn delete_block(uri: &str, pool: &PgPool) -> Result<()> {
    let parts: Vec<&str> = uri
        .strip_prefix("at://")
        .unwrap_or(uri)
        .split('/')
        .collect();
    let repo = parts.first().unwrap_or(&"");
    let rkey = parts.get(2).unwrap_or(&"");

    sqlx::query(r#"DELETE FROM actor_block WHERE creator = $1 AND "subjectDid" = $2"#)
        .bind(repo)
        .bind(rkey)
        .execute(pool)
        .await
        .map_err(|e| AppViewError::Database(e.to_string()))?;

    Ok(())
}

fn extract_did_from_at_uri(uri: &str) -> &str {
    uri.strip_prefix("at://")
        .unwrap_or(uri)
        .split('/')
        .next()
        .unwrap_or("")
}

fn extract_blob_cid(rec: &serde_json::Value, field: &str) -> Option<String> {
    rec.get(field)
        .and_then(|v| v.get("ref"))
        .and_then(|v| v.get("$link"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            rec.get(field)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}
