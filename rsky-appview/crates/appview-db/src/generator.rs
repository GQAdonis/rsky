use crate::models::*;
use appview_core::error::{AppViewError, Result};
use sqlx::PgPool;

pub async fn get_feed_generator(db: &PgPool, feed_uri: &str) -> Result<Option<GeneratorRow>> {
    let row = sqlx::query_as::<_, GeneratorRow>(
        r#"
        SELECT
            fg.uri, fg.cid, fg.creator,
            fg."feedDid" AS feed_did,
            fg."displayName" AS display_name,
            fg.description,
            fg."avatarCid" AS avatar_cid,
            COALESCE(fga."likeCount", 0) AS like_count,
            fg."indexedAt" AS indexed_at
        FROM feed_generator fg
        LEFT JOIN feed_generator_agg fga ON fga.uri = fg.uri
        WHERE fg.uri = $1
        LIMIT 1
        "#,
    )
    .bind(feed_uri)
    .fetch_optional(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(row)
}

pub async fn get_feed_generators(db: &PgPool, feed_uris: &[String]) -> Result<Vec<GeneratorRow>> {
    if feed_uris.is_empty() {
        return Ok(vec![]);
    }
    let rows = sqlx::query_as::<_, GeneratorRow>(
        r#"
        SELECT
            fg.uri, fg.cid, fg.creator,
            fg."feedDid" AS feed_did,
            fg."displayName" AS display_name,
            fg.description,
            fg."avatarCid" AS avatar_cid,
            COALESCE(fga."likeCount", 0) AS like_count,
            fg."indexedAt" AS indexed_at
        FROM feed_generator fg
        LEFT JOIN feed_generator_agg fga ON fga.uri = fg.uri
        WHERE fg.uri = ANY($1)
        "#,
    )
    .bind(feed_uris)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn get_popular_feed_generators(
    db: &PgPool,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<GeneratorRow>> {
    let rows = sqlx::query_as::<_, GeneratorRow>(
        r#"
        SELECT
            fg.uri, fg.cid, fg.creator,
            fg."feedDid" AS feed_did,
            fg."displayName" AS display_name,
            fg.description,
            fg."avatarCid" AS avatar_cid,
            COALESCE(fga."likeCount", 0) AS like_count,
            fg."indexedAt" AS indexed_at
        FROM feed_generator fg
        LEFT JOIN feed_generator_agg fga ON fga.uri = fg.uri
        WHERE ($3::text IS NULL OR fg.uri > $3)
        ORDER BY fga."likeCount" DESC NULLS LAST, fg.uri
        LIMIT $2
        "#,
    )
    .bind(limit)
    .bind(cursor)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}

pub async fn get_feed(
    db: &PgPool,
    _feed_uri: &str,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<PostWithAuthorRow>> {
    // Simplified: return posts from feed_item where originatorDid matches the generator creator
    // In production, this would call the actual feed generator HTTP endpoint
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
        WHERE ($3::text IS NULL OR fi."sortAt" < $3)
        ORDER BY fi."sortAt" DESC
        LIMIT $2
        "#,
    )
    .bind(limit)
    .bind(cursor)
    .fetch_all(db)
    .await
    .map_err(|e| AppViewError::Database(e.to_string()))?;
    Ok(rows)
}
