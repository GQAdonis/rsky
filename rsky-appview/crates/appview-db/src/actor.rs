use crate::models::*;
use appview_core::error::{AppViewError, Result};
use sqlx::PgPool;

pub async fn get_profile(db: &PgPool, actor: &str) -> Result<Option<ActorRow>> {
    let row = sqlx::query_as::<_, ActorRow>(
        r#"
        SELECT
            a.did,
            a.handle,
            p."displayName" AS display_name,
            p.description,
            p."avatarCid" AS avatar_cid,
            p."bannerCid" AS banner_cid,
            a."indexedAt" AS indexed_at,
            COALESCE(pa."followersCount", 0) AS followers_count,
            COALESCE(pa."followsCount", 0) AS follows_count,
            COALESCE(pa."postsCount", 0) AS posts_count
        FROM actor a
        LEFT JOIN profile p ON p.creator = a.did
        LEFT JOIN profile_agg pa ON pa.did = a.did
        WHERE a.did = $1 OR a.handle = $1
        LIMIT 1
        "#,
    )
    .bind(actor)
    .fetch_optional(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(row)
}

pub async fn search_actors(
    db: &PgPool,
    term: &str,
    limit: i64,
    cursor_did: Option<&str>,
) -> Result<Vec<ActorRow>> {
    let rows = sqlx::query_as::<_, ActorRow>(
        r#"
        SELECT
            a.did,
            a.handle,
            p."displayName" AS display_name,
            p.description,
            p."avatarCid" AS avatar_cid,
            p."bannerCid" AS banner_cid,
            a."indexedAt" AS indexed_at,
            COALESCE(pa."followersCount", 0) AS followers_count,
            COALESCE(pa."followsCount", 0) AS follows_count,
            COALESCE(pa."postsCount", 0) AS posts_count
        FROM actor a
        LEFT JOIN profile p ON p.creator = a.did
        LEFT JOIN profile_agg pa ON pa.did = a.did
        WHERE (a.handle ILIKE $1 OR p."displayName" ILIKE $1)
          AND ($3::text IS NULL OR a.did > $3)
        ORDER BY a.did
        LIMIT $2
        "#,
    )
    .bind(format!("%{}%", term))
    .bind(limit)
    .bind(cursor_did)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn get_suggestions(db: &PgPool, viewer_did: &str, limit: i64) -> Result<Vec<ActorRow>> {
    let rows = sqlx::query_as::<_, ActorRow>(
        r#"
        SELECT
            a.did,
            a.handle,
            p."displayName" AS display_name,
            p.description,
            p."avatarCid" AS avatar_cid,
            p."bannerCid" AS banner_cid,
            a."indexedAt" AS indexed_at,
            COALESCE(pa."followersCount", 0) AS followers_count,
            COALESCE(pa."followsCount", 0) AS follows_count,
            COALESCE(pa."postsCount", 0) AS posts_count
        FROM actor a
        LEFT JOIN profile p ON p.creator = a.did
        LEFT JOIN profile_agg pa ON pa.did = a.did
        WHERE a.did != $1
          AND a.handle IS NOT NULL
          AND NOT EXISTS (
              SELECT 1 FROM follow f WHERE f.creator = $1 AND f."subjectDid" = a.did
          )
        ORDER BY pa."followersCount" DESC NULLS LAST
        LIMIT $2
        "#,
    )
    .bind(viewer_did)
    .bind(limit)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}
