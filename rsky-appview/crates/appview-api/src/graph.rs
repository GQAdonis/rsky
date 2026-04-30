use crate::{AppState, AppViewError};
use appview_auth::Viewer;
use appview_core::error::Result;
use appview_db as db;
use appview_lexicon::bsky::actor::{ProfileView, ProfileViewBasic, ProfileViewDetailed};
use appview_lexicon::bsky::graph::{
    GetBlocksOutput, GetFollowersOutput, GetFollowsOutput, GetKnownFollowersOutput, GetListOutput,
    GetListsOutput, GetMutesOutput, ListItemView, ListView,
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

fn actor_to_profile_basic(row: &db::models::ActorRow) -> ProfileViewBasic {
    ProfileViewBasic {
        r#type: Some("app.bsky.actor.defs#profileViewBasic".into()),
        did: row.did.clone(),
        handle: row.handle.clone().unwrap_or_else(|| row.did.clone()),
        display_name: row.display_name.clone(),
        avatar: row.avatar_cid.clone(),
        associated: None,
        viewer: None,
        labels: None,
        created_at: None,
    }
}

fn actor_to_profile_view(row: &db::models::ActorRow) -> ProfileView {
    ProfileView {
        r#type: Some("app.bsky.actor.defs#profileView".into()),
        did: row.did.clone(),
        handle: row.handle.clone().unwrap_or_else(|| row.did.clone()),
        display_name: row.display_name.clone(),
        description: row.description.clone(),
        avatar: row.avatar_cid.clone(),
        associated: None,
        indexed_at: row.indexed_at.clone(),
        created_at: None,
        viewer: None,
        labels: None,
    }
}

fn actor_to_profile_detailed(row: &db::models::ActorRow) -> ProfileViewDetailed {
    ProfileViewDetailed {
        r#type: Some("app.bsky.actor.defs#profileViewDetailed".into()),
        did: row.did.clone(),
        handle: row.handle.clone().unwrap_or_else(|| row.did.clone()),
        display_name: row.display_name.clone(),
        description: row.description.clone(),
        avatar: row.avatar_cid.clone(),
        banner: row.banner_cid.clone(),
        followers_count: row.followers_count,
        follows_count: row.follows_count,
        posts_count: row.posts_count,
        associated: None,
        joined_via_starter_pack: None,
        indexed_at: row.indexed_at.clone(),
        created_at: None,
        viewer: None,
        labels: None,
    }
}

fn list_view_from_row(row: db::models::ListRow) -> ListView {
    ListView {
        r#type: Some("app.bsky.graph.defs#listView".into()),
        uri: row.uri,
        cid: row.cid,
        name: row.name,
        purpose: Some(row.purpose),
        avatar: row.avatar_cid,
        description: row.description,
        description_facets: None,
        list_item_count: None,
        creator: ProfileViewBasic {
            r#type: Some("app.bsky.actor.defs#profileViewBasic".into()),
            did: row.creator_did.clone(),
            handle: row.creator_did,
            display_name: None,
            avatar: None,
            associated: None,
            viewer: None,
            labels: None,
            created_at: None,
        },
        indexed_at: row.indexed_at,
        viewer: None,
        labels: None,
    }
}

#[derive(Deserialize)]
pub struct GetFollowsParams {
    actor: String,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn get_follows(
    State(state): State<AppState>,
    Query(params): Query<GetFollowsParams>,
) -> Result<Json<GetFollowsOutput>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let subject_row = db::actor::get_profile(&state.db, &params.actor)
        .await?
        .ok_or(AppViewError::NotFound)?;
    let subject = actor_to_profile_basic(&subject_row);
    let rows =
        db::graph::get_follows(&state.db, &params.actor, limit, params.cursor.as_deref()).await?;
    let cursor = rows.last().and_then(|r| r.indexed_at.clone());
    let follows: Vec<ProfileView> = rows.iter().map(actor_to_profile_view).collect();
    Ok(Json(GetFollowsOutput {
        subject,
        follows,
        cursor,
    }))
}

#[derive(Deserialize)]
pub struct GetFollowersParams {
    actor: String,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn get_followers(
    State(state): State<AppState>,
    Query(params): Query<GetFollowersParams>,
) -> Result<Json<GetFollowersOutput>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let subject_row = db::actor::get_profile(&state.db, &params.actor)
        .await?
        .ok_or(AppViewError::NotFound)?;
    let subject = actor_to_profile_basic(&subject_row);
    let rows =
        db::graph::get_followers(&state.db, &params.actor, limit, params.cursor.as_deref()).await?;
    let cursor = rows.last().and_then(|r| r.indexed_at.clone());
    let followers: Vec<ProfileView> = rows.iter().map(actor_to_profile_view).collect();
    Ok(Json(GetFollowersOutput {
        subject,
        followers,
        cursor,
    }))
}

#[derive(Deserialize)]
pub struct GetKnownFollowersParams {
    actor: String,
    viewer: String,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn get_known_followers(
    State(state): State<AppState>,
    Query(params): Query<GetKnownFollowersParams>,
) -> Result<Json<GetKnownFollowersOutput>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let subject_row = db::actor::get_profile(&state.db, &params.actor)
        .await?
        .ok_or(AppViewError::NotFound)?;
    let subject = actor_to_profile_basic(&subject_row);
    let rows = db::graph::get_known_followers(
        &state.db,
        &params.actor,
        &params.viewer,
        limit,
        params.cursor.as_deref(),
    )
    .await?;
    let cursor = rows.last().and_then(|r| r.indexed_at.clone());
    let followers: Vec<ProfileView> = rows.iter().map(actor_to_profile_view).collect();
    Ok(Json(GetKnownFollowersOutput {
        subject,
        followers,
        cursor,
    }))
}

#[derive(Deserialize)]
pub struct GetBlocksParams {
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn get_blocks(
    State(state): State<AppState>,
    viewer: Viewer,
    Query(params): Query<GetBlocksParams>,
) -> Result<Json<GetBlocksOutput>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let rows = db::graph::get_blocks(
        &state.db,
        viewer.did.as_str(),
        limit,
        params.cursor.as_deref(),
    )
    .await?;
    let cursor = rows.last().and_then(|r| r.indexed_at.clone());
    let blocks: Vec<ProfileViewDetailed> = rows.iter().map(actor_to_profile_detailed).collect();
    Ok(Json(GetBlocksOutput { blocks, cursor }))
}

#[derive(Deserialize)]
pub struct GetMutesParams {
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn get_mutes(
    State(state): State<AppState>,
    viewer: Viewer,
    Query(params): Query<GetMutesParams>,
) -> Result<Json<GetMutesOutput>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let rows = db::graph::get_mutes(
        &state.db,
        viewer.did.as_str(),
        limit,
        params.cursor.as_deref(),
    )
    .await?;
    let cursor = rows.last().and_then(|r| r.indexed_at.clone());
    let mutes: Vec<ProfileView> = rows.iter().map(actor_to_profile_view).collect();
    Ok(Json(GetMutesOutput { mutes, cursor }))
}

#[derive(Deserialize)]
pub struct GetListParams {
    list: String,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn get_list(
    State(state): State<AppState>,
    Query(params): Query<GetListParams>,
) -> Result<Json<GetListOutput>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let list_row = db::graph::get_list(&state.db, &params.list).await?;
    let list = list_row
        .map(list_view_from_row)
        .ok_or(AppViewError::NotFound)?;
    let items_rows =
        db::graph::get_list_items(&state.db, &params.list, limit, params.cursor.as_deref()).await?;
    let cursor = items_rows.last().map(|r| r.indexed_at.clone());
    let items: Vec<ListItemView> = items_rows
        .iter()
        .map(|row| ListItemView {
            uri: row.uri.clone(),
            subject: ProfileViewBasic {
                r#type: Some("app.bsky.actor.defs#profileViewBasic".into()),
                did: row.subject_did.clone(),
                handle: row
                    .subject_handle
                    .clone()
                    .unwrap_or_else(|| row.subject_did.clone()),
                display_name: row.subject_display_name.clone(),
                avatar: row.subject_avatar_cid.clone(),
                associated: None,
                viewer: None,
                labels: None,
                created_at: None,
            },
        })
        .collect();
    Ok(Json(GetListOutput {
        list,
        items,
        cursor,
    }))
}

#[derive(Deserialize)]
pub struct GetListsParams {
    actor: String,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn get_lists(
    State(state): State<AppState>,
    Query(params): Query<GetListsParams>,
) -> Result<Json<GetListsOutput>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let rows =
        db::graph::get_lists(&state.db, &params.actor, limit, params.cursor.as_deref()).await?;
    let cursor = rows.last().map(|r| r.indexed_at.clone());
    let lists: Vec<ListView> = rows.into_iter().map(list_view_from_row).collect();
    Ok(Json(GetListsOutput { lists, cursor }))
}
