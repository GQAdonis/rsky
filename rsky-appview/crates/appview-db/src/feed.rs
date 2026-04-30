use crate::models::*;
use appview_core::error::{AppViewError, Result};
use sqlx::PgPool;

pub async fn get_timeline(
    db: &PgPool,
    viewer_did: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<PostWithAuthorRow>> {
    let rows = sqlx::query_as::<_, PostWithAuthorRow>(
        r#"
        SELECT
            p.uri, p.cid, p.creator, p.text,
            p."replyRoot" AS reply_root, p."replyParent" AS reply_parent,
            COALESCE(pa."replyCount", 0) AS reply_count,
            COALESCE(pa."repostCount", 0) AS repost_count,
            COALESCE(pa."likeCount", 0) AS like_count,
            COALESCE(pa."quoteCount", 0) AS quote_count,
            p."createdAt" AS created_at, p."indexedAt" AS indexed_at,
            a.did AS author_did, a.handle AS author_handle,
            pr."displayName" AS author_display_name, pr.description AS author_description,
            pr."avatarCid" AS author_avatar_cid, pr."bannerCid" AS author_banner_cid,
            a."indexedAt" AS author_indexed_at,
            COALESCE(pag."followersCount", 0) AS author_followers_count,
            COALESCE(pag."followsCount", 0) AS author_follows_count,
            COALESCE(pag."postsCount", 0) AS author_posts_count
        FROM feed_item fi
        JOIN post p ON p.uri = fi."postUri"
        LEFT JOIN post_agg pa ON pa.uri = p.uri
        JOIN actor a ON a.did = p.creator
        LEFT JOIN profile pr ON pr.creator = a.did
        LEFT JOIN profile_agg pag ON pag.did = a.did
        WHERE fi."originatorDid" IN (
            SELECT f."subjectDid" FROM follow f WHERE f.creator = $1
        )
          AND ($3::text IS NULL OR fi."sortAt" < $3)
        ORDER BY fi."sortAt" DESC
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

pub async fn get_author_feed(
    db: &PgPool,
    actor_did: &str,
    filter: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<PostWithAuthorRow>> {
    let reply_filter = match filter {
        "posts_no_replies" => " AND p.\"replyParent\" IS NULL",
        "posts_with_replies" => "",
        _ => " AND p.\"replyParent\" IS NULL", // default
    };
    let query_str = format!(
        r#"
        SELECT
            p.uri, p.cid, p.creator, p.text,
            p."replyRoot" AS reply_root, p."replyParent" AS reply_parent,
            COALESCE(pa."replyCount", 0) AS reply_count,
            COALESCE(pa."repostCount", 0) AS repost_count,
            COALESCE(pa."likeCount", 0) AS like_count,
            COALESCE(pa."quoteCount", 0) AS quote_count,
            p."createdAt" AS created_at, p."indexedAt" AS indexed_at,
            a.did AS author_did, a.handle AS author_handle,
            pr."displayName" AS author_display_name, pr.description AS author_description,
            pr."avatarCid" AS author_avatar_cid, pr."bannerCid" AS author_banner_cid,
            a."indexedAt" AS author_indexed_at,
            COALESCE(pag."followersCount", 0) AS author_followers_count,
            COALESCE(pag."followsCount", 0) AS author_follows_count,
            COALESCE(pag."postsCount", 0) AS author_posts_count
        FROM post p
        LEFT JOIN post_agg pa ON pa.uri = p.uri
        JOIN actor a ON a.did = p.creator
        LEFT JOIN profile pr ON pr.creator = a.did
        LEFT JOIN profile_agg pag ON pag.did = a.did
        WHERE p.creator = $1
          {}
          AND ($3::text IS NULL OR p."createdAt" < $3)
        ORDER BY p."createdAt" DESC
        LIMIT $2
        "#,
        reply_filter
    );
    let rows = sqlx::query_as::<_, PostWithAuthorRow>(&query_str)
        .bind(actor_did)
        .bind(limit)
        .bind(cursor)
        .fetch_all(db)
        .await
        .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn get_post_thread(db: &PgPool, uri: &str, _depth: i64) -> Result<Vec<PostRow>> {
    // Simplified: just return the post itself and immediate replies
    let rows = sqlx::query_as::<_, PostRow>(
        r#"
        SELECT
            p.uri, p.cid, p.creator, p.text,
            p."replyRoot" AS reply_root, p."replyParent" AS reply_parent,
            COALESCE(pa."replyCount", 0) AS reply_count,
            COALESCE(pa."repostCount", 0) AS repost_count,
            COALESCE(pa."likeCount", 0) AS like_count,
            COALESCE(pa."quoteCount", 0) AS quote_count,
            p."createdAt" AS created_at, p."indexedAt" AS indexed_at
        FROM post p
        LEFT JOIN post_agg pa ON pa.uri = p.uri
        WHERE p.uri = $1 OR p."replyParent" = $1
        ORDER BY p."createdAt"
        "#,
    )
    .bind(uri)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn get_likes(
    db: &PgPool,
    uri: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<LikeWithActorRow>> {
    let rows = sqlx::query_as::<_, LikeWithActorRow>(
        r#"
        SELECT
            l.uri AS like_uri,
            a.did AS actor_did, a.handle AS actor_handle,
            pr."displayName" AS actor_display_name, pr.description AS actor_description,
            pr."avatarCid" AS actor_avatar_cid, pr."bannerCid" AS actor_banner_cid,
            a."indexedAt" AS actor_indexed_at,
            COALESCE(pag."followersCount", 0) AS actor_followers_count,
            COALESCE(pag."followsCount", 0) AS actor_follows_count,
            COALESCE(pag."postsCount", 0) AS actor_posts_count
        FROM "like" l
        JOIN actor a ON a.did = l.creator
        LEFT JOIN profile pr ON pr.creator = a.did
        LEFT JOIN profile_agg pag ON pag.did = a.did
        WHERE l.subject = $1
          AND ($3::text IS NULL OR l."createdAt" < $3)
        ORDER BY l."createdAt" DESC
        LIMIT $2
        "#,
    )
    .bind(uri)
    .bind(limit)
    .bind(cursor)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn get_reposted_by(
    db: &PgPool,
    uri: &str,
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
        FROM repost r
        JOIN actor a ON a.did = r.creator
        LEFT JOIN profile pr ON pr.creator = a.did
        LEFT JOIN profile_agg pag ON pag.did = a.did
        WHERE r.subject = $1
          AND ($3::text IS NULL OR r."createdAt" < $3)
        ORDER BY r."createdAt" DESC
        LIMIT $2
        "#,
    )
    .bind(uri)
    .bind(limit)
    .bind(cursor)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn search_posts(
    db: &PgPool,
    query: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<PostRow>> {
    let rows = sqlx::query_as::<_, PostRow>(
        r#"
        SELECT
            p.uri, p.cid, p.creator, p.text,
            p."replyRoot" AS reply_root, p."replyParent" AS reply_parent,
            COALESCE(pa."replyCount", 0) AS reply_count,
            COALESCE(pa."repostCount", 0) AS repost_count,
            COALESCE(pa."likeCount", 0) AS like_count,
            COALESCE(pa."quoteCount", 0) AS quote_count,
            p."createdAt" AS created_at, p."indexedAt" AS indexed_at
        FROM post p
        LEFT JOIN post_agg pa ON pa.uri = p.uri
        WHERE p.text ILIKE $1
          AND ($3::text IS NULL OR p."createdAt" < $3)
        ORDER BY p."createdAt" DESC
        LIMIT $2
        "#,
    )
    .bind(format!("%{}%", query))
    .bind(limit)
    .bind(cursor)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn get_actor_likes(
    db: &PgPool,
    actor_did: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<PostWithAuthorRow>> {
    let rows = sqlx::query_as::<_, PostWithAuthorRow>(
        r#"
        SELECT
            p.uri, p.cid, p.creator, p.text,
            p."replyRoot" AS reply_root, p."replyParent" AS reply_parent,
            COALESCE(pa."replyCount", 0) AS reply_count,
            COALESCE(pa."repostCount", 0) AS repost_count,
            COALESCE(pa."likeCount", 0) AS like_count,
            COALESCE(pa."quoteCount", 0) AS quote_count,
            p."createdAt" AS created_at, p."indexedAt" AS indexed_at,
            a.did AS author_did, a.handle AS author_handle,
            pr."displayName" AS author_display_name, pr.description AS author_description,
            pr."avatarCid" AS author_avatar_cid, pr."bannerCid" AS author_banner_cid,
            a."indexedAt" AS author_indexed_at,
            COALESCE(pag."followersCount", 0) AS author_followers_count,
            COALESCE(pag."followsCount", 0) AS author_follows_count,
            COALESCE(pag."postsCount", 0) AS author_posts_count
        FROM "like" l
        JOIN post p ON p.uri = l.subject
        LEFT JOIN post_agg pa ON pa.uri = p.uri
        JOIN actor a ON a.did = p.creator
        LEFT JOIN profile pr ON pr.creator = a.did
        LEFT JOIN profile_agg pag ON pag.did = a.did
        WHERE l.creator = $1
          AND ($3::text IS NULL OR l."createdAt" < $3)
        ORDER BY l."createdAt" DESC
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
