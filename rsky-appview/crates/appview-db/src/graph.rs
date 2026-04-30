use crate::models::*;
use appview_core::error::{AppViewError, Result};
use sqlx::PgPool;

pub async fn get_followers(
    db: &PgPool,
    actor_did: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<ActorRow>> {
    let rows = sqlx::query_as::<_, ActorRow>(
        r#"
        SELECT
            a.did, a.handle,
            pr."displayName" AS display_name, pr.description,
            pr."avatarCid" AS avatar_cid, pr."bannerCid" AS banner_cid,
            a."indexedAt" AS indexed_at,
            COALESCE(pag."followersCount", 0) AS followers_count,
            COALESCE(pag."followsCount", 0) AS follows_count,
            COALESCE(pag."postsCount", 0) AS posts_count
        FROM follow f
        JOIN actor a ON a.did = f.creator
        LEFT JOIN profile pr ON pr.creator = a.did
        LEFT JOIN profile_agg pag ON pag.did = a.did
        WHERE f."subjectDid" = $1
          AND ($3::text IS NULL OR f."createdAt" < $3)
        ORDER BY f."createdAt" DESC
        LIMIT $2
        "#,
    )
    .bind(actor_did)
    .bind(limit)
    .bind(cursor)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn get_follows(
    db: &PgPool,
    actor_did: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<ActorRow>> {
    let rows = sqlx::query_as::<_, ActorRow>(
        r#"
        SELECT
            a.did, a.handle,
            pr."displayName" AS display_name, pr.description,
            pr."avatarCid" AS avatar_cid, pr."bannerCid" AS banner_cid,
            a."indexedAt" AS indexed_at,
            COALESCE(pag."followersCount", 0) AS followers_count,
            COALESCE(pag."followsCount", 0) AS follows_count,
            COALESCE(pag."postsCount", 0) AS posts_count
        FROM follow f
        JOIN actor a ON a.did = f."subjectDid"
        LEFT JOIN profile pr ON pr.creator = a.did
        LEFT JOIN profile_agg pag ON pag.did = a.did
        WHERE f.creator = $1
          AND ($3::text IS NULL OR f."createdAt" < $3)
        ORDER BY f."createdAt" DESC
        LIMIT $2
        "#,
    )
    .bind(actor_did)
    .bind(limit)
    .bind(cursor)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn get_known_followers(
    db: &PgPool,
    actor_did: &str,
    viewer_did: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<ActorRow>> {
    let rows = sqlx::query_as::<_, ActorRow>(
        r#"
        SELECT
            a.did, a.handle,
            pr."displayName" AS display_name, pr.description,
            pr."avatarCid" AS avatar_cid, pr."bannerCid" AS banner_cid,
            a."indexedAt" AS indexed_at,
            COALESCE(pag."followersCount", 0) AS followers_count,
            COALESCE(pag."followsCount", 0) AS follows_count,
            COALESCE(pag."postsCount", 0) AS posts_count
        FROM follow f
        JOIN actor a ON a.did = f.creator
        LEFT JOIN profile pr ON pr.creator = a.did
        LEFT JOIN profile_agg pag ON pag.did = a.did
        WHERE f."subjectDid" = $1
          AND EXISTS (SELECT 1 FROM follow vf WHERE vf.creator = $2 AND vf."subjectDid" = f.creator)
          AND ($4::text IS NULL OR f."createdAt" < $4)
        ORDER BY f."createdAt" DESC
        LIMIT $3
        "#,
    )
    .bind(actor_did)
    .bind(viewer_did)
    .bind(limit)
    .bind(cursor)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn get_mutes(
    db: &PgPool,
    viewer_did: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<ActorRow>> {
    let rows = sqlx::query_as::<_, ActorRow>(
        r#"
        SELECT
            a.did, a.handle,
            pr."displayName" AS display_name, pr.description,
            pr."avatarCid" AS avatar_cid, pr."bannerCid" AS banner_cid,
            a."indexedAt" AS indexed_at,
            COALESCE(pag."followersCount", 0) AS followers_count,
            COALESCE(pag."followsCount", 0) AS follows_count,
            COALESCE(pag."postsCount", 0) AS posts_count
        FROM actor_mute b
        JOIN actor a ON a.did = b."subjectDid"
        LEFT JOIN profile pr ON pr.creator = a.did
        LEFT JOIN profile_agg pag ON pag.did = a.did
        WHERE b.creator = $1
          AND ($3::text IS NULL OR b."createdAt" < $3)
        ORDER BY b."createdAt" DESC
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

pub async fn get_blocks(
    db: &PgPool,
    viewer_did: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<ActorRow>> {
    let rows = sqlx::query_as::<_, ActorRow>(
        r#"
        SELECT
            a.did, a.handle,
            pr."displayName" AS display_name, pr.description,
            pr."avatarCid" AS avatar_cid, pr."bannerCid" AS banner_cid,
            a."indexedAt" AS indexed_at,
            COALESCE(pag."followersCount", 0) AS followers_count,
            COALESCE(pag."followsCount", 0) AS follows_count,
            COALESCE(pag."postsCount", 0) AS posts_count
        FROM actor_block b
        JOIN actor a ON a.did = b."subjectDid"
        LEFT JOIN profile pr ON pr.creator = a.did
        LEFT JOIN profile_agg pag ON pag.did = a.did
        WHERE b.creator = $1
          AND ($3::text IS NULL OR b."createdAt" < $3)
        ORDER BY b."createdAt" DESC
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

use crate::models::{ListItemRow, ListRow};

pub async fn get_lists(
    db: &PgPool,
    actor_did: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<ListRow>> {
    let rows = sqlx::query_as::<_, ListRow>(
        r#"
        SELECT
            l.uri, l.cid, l.creator_did, l.name, l.purpose,
            l.description, l.avatar_cid, l.indexed_at
        FROM list l
        WHERE l.creator_did = $1
          AND ($3::text IS NULL OR l.indexed_at < $3)
        ORDER BY l.indexed_at DESC
        LIMIT $2
        "#,
    )
    .bind(actor_did)
    .bind(limit)
    .bind(cursor)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn get_list(db: &PgPool, uri: &str) -> Result<Option<ListRow>> {
    let row = sqlx::query_as::<_, ListRow>(
        r#"
        SELECT
            l.uri, l.cid, l.creator_did, l.name, l.purpose,
            l.description, l.avatar_cid, l.indexed_at
        FROM list l
        WHERE l.uri = $1
        "#,
    )
    .bind(uri)
    .fetch_optional(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(row)
}

pub async fn get_list_items(
    db: &PgPool,
    list_uri: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<ListItemRow>> {
    let rows = sqlx::query_as::<_, ListItemRow>(
        r#"
        SELECT
            li.uri,
            a.did AS subject_did, a.handle AS subject_handle,
            pr."displayName" AS subject_display_name, pr."avatarCid" AS subject_avatar_cid,
            li.indexed_at
        FROM list_item li
        JOIN actor a ON a.did = li."subjectDid"
        LEFT JOIN profile pr ON pr.creator = a.did
        WHERE li.list_uri = $1
          AND ($3::text IS NULL OR li.indexed_at < $3)
        ORDER BY li.indexed_at DESC
        LIMIT $2
        "#,
    )
    .bind(list_uri)
    .bind(limit)
    .bind(cursor)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn get_list_blocks(
    db: &PgPool,
    viewer_did: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<ActorRow>> {
    let rows = sqlx::query_as::<_, ActorRow>(
        r#"
        SELECT
            a.did, a.handle,
            pr."displayName" AS display_name, pr.description,
            pr."avatarCid" AS avatar_cid, pr."bannerCid" AS banner_cid,
            a."indexedAt" AS indexed_at,
            COALESCE(pag."followersCount", 0) AS followers_count,
            COALESCE(pag."followsCount", 0) AS follows_count,
            COALESCE(pag."postsCount", 0) AS posts_count
        FROM list_block lb
        JOIN list l ON l.uri = lb.list_uri
        JOIN actor a ON a.did = lb."subjectDid"
        LEFT JOIN profile pr ON pr.creator = a.did
        LEFT JOIN profile_agg pag ON pag.did = a.did
        WHERE l.creator_did = $1
          AND ($3::text IS NULL OR lb.indexed_at < $3)
        ORDER BY lb.indexed_at DESC
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

pub async fn get_list_mutes(
    db: &PgPool,
    viewer_did: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<ActorRow>> {
    let rows = sqlx::query_as::<_, ActorRow>(
        r#"
        SELECT
            a.did, a.handle,
            pr."displayName" AS display_name, pr.description,
            pr."avatarCid" AS avatar_cid, pr."bannerCid" AS banner_cid,
            a."indexedAt" AS indexed_at,
            COALESCE(pag."followersCount", 0) AS followers_count,
            COALESCE(pag."followsCount", 0) AS follows_count,
            COALESCE(pag."postsCount", 0) AS posts_count
        FROM list_mute lm
        JOIN list l ON l.uri = lm.list_uri
        JOIN actor a ON a.did = l.creator_did
        LEFT JOIN profile pr ON pr.creator = a.did
        LEFT JOIN profile_agg pag ON pag.did = a.did
        WHERE lm.creator = $1
          AND ($3::text IS NULL OR lm.indexed_at < $3)
        ORDER BY lm.indexed_at DESC
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
