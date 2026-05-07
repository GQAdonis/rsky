use crate::AppState;
use appview_core::error::Result;
use appview_db as db;
use appview_lexicon::bsky::actor::ProfileViewBasic;
use appview_lexicon::bsky::feed::{
    AuthorFeedOutput, FeedViewPost, GeneratorView, GetFeedGeneratorOutput, GetFeedGeneratorsOutput,
    GetLikesOutput, GetListFeedOutput, GetRepostedByOutput, LikeView, PostThreadOutput, PostView,
    SearchPostsOutput, TimelineOutput,
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

fn build_profile_basic(
    did: String,
    handle: Option<String>,
    display_name: Option<String>,
    avatar_cid: Option<String>,
) -> ProfileViewBasic {
    ProfileViewBasic {
        r#type: Some("app.bsky.actor.defs#profileViewBasic".into()),
        did: did.clone(),
        handle: handle.unwrap_or(did),
        display_name,
        avatar: avatar_cid,
        associated: None,
        viewer: None,
        labels: None,
        created_at: None,
    }
}

fn post_view_with_author(row: &db::models::PostWithAuthorRow) -> PostView {
    PostView {
        r#type: Some("app.bsky.feed.defs#postView".into()),
        uri: row.uri.clone(),
        cid: row.cid.clone(),
        author: build_profile_basic(
            row.author_did.clone(),
            row.author_handle.clone(),
            row.author_display_name.clone(),
            row.author_avatar_cid.clone(),
        ),
        record: serde_json::json!({
            "$type": "app.bsky.feed.post",
            "text": row.text,
            "createdAt": row.created_at,
        }),
        embed: None,
        reply_count: row.reply_count,
        repost_count: row.repost_count,
        like_count: row.like_count,
        quote_count: row.quote_count,
        indexed_at: row.indexed_at.clone(),
        viewer: None,
        labels: None,
        threadgate: None,
    }
}

fn post_view_plain(row: &db::models::PostRow) -> PostView {
    PostView {
        r#type: Some("app.bsky.feed.defs#postView".into()),
        uri: row.uri.clone(),
        cid: row.cid.clone(),
        author: build_profile_basic(row.creator.clone(), None, None, None),
        record: serde_json::json!({
            "$type": "app.bsky.feed.post",
            "text": row.text,
            "createdAt": row.created_at,
        }),
        embed: None,
        reply_count: row.reply_count,
        repost_count: row.repost_count,
        like_count: row.like_count,
        quote_count: row.quote_count,
        indexed_at: row.indexed_at.clone(),
        viewer: None,
        labels: None,
        threadgate: None,
    }
}

fn feed_item_with_author(row: &db::models::PostWithAuthorRow) -> FeedViewPost {
    FeedViewPost {
        post: post_view_with_author(row),
        reason: None,
        reply: None,
        feed_context: None,
    }
}

fn feed_item_plain(row: &db::models::PostRow) -> FeedViewPost {
    FeedViewPost {
        post: post_view_plain(row),
        reason: None,
        reply: None,
        feed_context: None,
    }
}

#[derive(Deserialize)]
pub struct GetTimelineParams {
    #[serde(default)]
    algorithm: Option<String>,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn get_timeline(
    State(state): State<AppState>,
    Query(params): Query<GetTimelineParams>,
) -> Result<Json<TimelineOutput>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let rows = db::feed::get_timeline(&state.db, "", limit, params.cursor.as_deref()).await?;
    let cursor = rows.last().map(|r| r.indexed_at.clone());
    let feed: Vec<FeedViewPost> = rows.iter().map(feed_item_with_author).collect();
    Ok(Json(TimelineOutput { feed, cursor }))
}

#[derive(Deserialize)]
pub struct GetAuthorFeedParams {
    actor: String,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default)]
    filter: Option<String>,
}

pub async fn get_author_feed(
    State(state): State<AppState>,
    Query(params): Query<GetAuthorFeedParams>,
) -> Result<Json<AuthorFeedOutput>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let filter = params.filter.as_deref().unwrap_or("posts_no_replies");
    // Resolve handle or DID to a concrete DID before querying posts
    // The DB feed query uses `WHERE p.creator = $1` which only matches DIDs
    let actor_did = match db::actor::get_profile(&state.db, &params.actor).await? {
        Some(row) => row.did,
        None => return Ok(Json(AuthorFeedOutput { feed: vec![], cursor: None })),
    };
    let rows = db::feed::get_author_feed(
        &state.db,
        &actor_did,
        filter,
        limit,
        params.cursor.as_deref(),
    )
    .await?;
    let cursor = rows.last().map(|r| r.created_at.clone());
    let feed: Vec<FeedViewPost> = rows.iter().map(feed_item_with_author).collect();
    Ok(Json(AuthorFeedOutput { feed, cursor }))
}

#[derive(Deserialize)]
pub struct GetFeedParams {
    #[serde(rename = "feed")]
    feed: String,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn get_feed(
    State(_state): State<AppState>,
    Query(_params): Query<GetFeedParams>,
) -> Result<Json<TimelineOutput>> {
    Ok(Json(TimelineOutput {
        feed: vec![],
        cursor: None,
    }))
}

#[derive(Deserialize)]
pub struct GetPostThreadParams {
    uri: String,
    #[serde(default)]
    depth: Option<i64>,
    #[serde(default)]
    parent_height: Option<i64>,
}

pub async fn get_post_thread(
    State(state): State<AppState>,
    Query(params): Query<GetPostThreadParams>,
) -> Result<Json<PostThreadOutput>> {
    let depth = params.depth.unwrap_or(6).min(1000);
    let rows = db::feed::get_post_thread(&state.db, &params.uri, depth).await?;
    let thread = rows
        .first()
        .map(|row| serde_json::to_value(feed_item_plain(row)).unwrap_or(serde_json::Value::Null))
        .unwrap_or(serde_json::Value::Null);
    Ok(Json(PostThreadOutput {
        thread,
        threadgate: None,
    }))
}

#[derive(Deserialize)]
pub struct GetPostsParams {
    uris: Vec<String>,
}

pub async fn get_posts(
    State(state): State<AppState>,
    Query(params): Query<GetPostsParams>,
) -> Result<Json<serde_json::Value>> {
    let mut posts = Vec::new();
    for uri in params.uris {
        let rows = db::feed::get_post_thread(&state.db, &uri, 1).await?;
        if let Some(row) = rows.first() {
            posts.push(post_view_plain(row));
        }
    }
    Ok(Json(serde_json::json!({ "posts": posts })))
}

#[derive(Deserialize)]
pub struct GetLikesParams {
    uri: String,
    #[serde(default)]
    cid: Option<String>,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn get_likes(
    State(state): State<AppState>,
    Query(params): Query<GetLikesParams>,
) -> Result<Json<GetLikesOutput>> {
    let limit = params.limit.unwrap_or(50).min(100);
    let rows = db::feed::get_likes(&state.db, &params.uri, limit, params.cursor.as_deref()).await?;
    let cursor = rows.last().map(|r| r.like_uri.clone());
    let likes: Vec<LikeView> = rows
        .into_iter()
        .map(|row| LikeView {
            indexed_at: row.actor_indexed_at.unwrap_or_default(),
            actor: build_profile_basic(
                row.actor_did,
                row.actor_handle,
                row.actor_display_name,
                row.actor_avatar_cid,
            ),
        })
        .collect();
    Ok(Json(GetLikesOutput {
        uri: params.uri,
        cid: params.cid,
        likes,
        cursor,
    }))
}

#[derive(Deserialize)]
pub struct GetRepostedByParams {
    uri: String,
    #[serde(default)]
    cid: Option<String>,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn get_reposted_by(
    State(state): State<AppState>,
    Query(params): Query<GetRepostedByParams>,
) -> Result<Json<GetRepostedByOutput>> {
    let limit = params.limit.unwrap_or(50).min(100);
    let rows =
        db::feed::get_reposted_by(&state.db, &params.uri, limit, params.cursor.as_deref()).await?;
    let cursor = rows.last().map(|r| r.did.clone());
    let reposted_by: Vec<ProfileViewBasic> = rows
        .into_iter()
        .map(|row| build_profile_basic(row.did, row.handle, row.display_name, row.avatar_cid))
        .collect();
    Ok(Json(GetRepostedByOutput {
        uri: params.uri,
        cid: params.cid,
        reposted_by,
        cursor,
    }))
}

#[derive(Deserialize)]
pub struct SearchPostsParams {
    #[serde(rename = "term")]
    term: String,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn search_posts(
    State(state): State<AppState>,
    Query(params): Query<SearchPostsParams>,
) -> Result<Json<SearchPostsOutput>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let rows =
        db::feed::search_posts(&state.db, &params.term, limit, params.cursor.as_deref()).await?;
    let cursor = rows.last().map(|r| r.indexed_at.clone());
    let posts: Vec<PostView> = rows.iter().map(post_view_plain).collect();
    Ok(Json(SearchPostsOutput {
        posts,
        hits_total: None,
        cursor,
    }))
}

#[derive(Deserialize)]
pub struct GetFeedGeneratorParams {
    #[serde(rename = "feed")]
    feed: String,
}

pub async fn get_feed_generator(
    State(_state): State<AppState>,
    Query(_params): Query<GetFeedGeneratorParams>,
) -> Result<Json<GetFeedGeneratorOutput>> {
    Ok(Json(GetFeedGeneratorOutput {
        view: GeneratorView {
            r#type: Some("app.bsky.feed.defs#generatorView".into()),
            uri: String::new(),
            cid: String::new(),
            did: String::new(),
            creator: ProfileViewBasic {
                r#type: Some("app.bsky.actor.defs#profileViewBasic".into()),
                did: String::new(),
                handle: String::new(),
                display_name: None,
                avatar: None,
                associated: None,
                viewer: None,
                labels: None,
                created_at: None,
            },
            display_name: String::new(),
            description: None,
            description_facets: None,
            avatar: None,
            like_count: None,
            accepts_interactions: None,
            labels: None,
            viewer: None,
            indexed_at: String::new(),
        },
        is_online: true,
        is_valid: true,
    }))
}

#[derive(Deserialize)]
pub struct GetFeedGeneratorsParams {
    feeds: Vec<String>,
}

pub async fn get_feed_generators(
    State(_state): State<AppState>,
    Query(_params): Query<GetFeedGeneratorsParams>,
) -> Result<Json<GetFeedGeneratorsOutput>> {
    Ok(Json(GetFeedGeneratorsOutput { feeds: vec![] }))
}

#[derive(Deserialize)]
pub struct GetListFeedParams {
    #[serde(rename = "list")]
    _list: String,
    #[serde(default)]
    _limit: Option<i64>,
    #[serde(default)]
    _cursor: Option<String>,
}

pub async fn get_list_feed(
    State(_state): State<AppState>,
    Query(_params): Query<GetListFeedParams>,
) -> Result<Json<GetListFeedOutput>> {
    Ok(Json(GetListFeedOutput {
        feed: vec![],
        cursor: None,
    }))
}
