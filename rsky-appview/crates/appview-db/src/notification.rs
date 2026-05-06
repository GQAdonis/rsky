use crate::models::*;
use appview_core::error::{AppViewError, Result};
use sqlx::PgPool;

pub async fn list_notifications(
    db: &PgPool,
    viewer_did: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<NotificationRow>> {
    let rows = sqlx::query_as::<_, NotificationRow>(
        r#"
        SELECT
            n.did, n.author,
            n."recordUri" AS record_uri, n."recordCid" AS record_cid,
            n.reason, n."reasonSubject" AS reason_subject,
            n."isRead" AS is_read, n."sortAt" AS sort_at
        FROM notification n
        WHERE n.did = $1
          AND ($3::text IS NULL OR n."sortAt" < $3)
        ORDER BY n."sortAt" DESC
        LIMIT $2
        "#,
    )
    .bind(viewer_did)
    .bind(limit)
    .bind(cursor)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn count_unread_notifications(db: &PgPool, viewer_did: &str) -> Result<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM notification n
        WHERE n.did = $1 AND (n."isRead" = false OR n."isRead" IS NULL)
        "#,
    )
    .bind(viewer_did)
    .fetch_one(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(row.0)
}
