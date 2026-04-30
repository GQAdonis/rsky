use crate::{AppState, AppViewError};
use appview_core::error::Result;
use appview_db as db;
use appview_lexicon::bsky::actor::{
    GetProfileOutput, GetProfilesOutput, GetSuggestionsOutput, ProfileView, ProfileViewBasic,
    ProfileViewDetailed, SearchActorsOutput, SearchActorsTypeaheadOutput,
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetProfileParams {
    #[serde(rename = "actor")]
    actor: String,
}

pub async fn get_profile(
    State(state): State<AppState>,
    Query(params): Query<GetProfileParams>,
) -> Result<Json<GetProfileOutput>> {
    let row = db::actor::get_profile(&state.db, &params.actor)
        .await?
        .ok_or(AppViewError::NotFound)?;

    let handle = row.handle.unwrap_or_else(|| row.did.clone());
    let profile = ProfileViewDetailed {
        r#type: Some("app.bsky.actor.defs#profileViewDetailed".into()),
        did: row.did,
        handle,
        display_name: row.display_name,
        description: row.description,
        avatar: row.avatar_cid,
        banner: row.banner_cid,
        followers_count: row.followers_count,
        follows_count: row.follows_count,
        posts_count: row.posts_count,
        associated: None,
        joined_via_starter_pack: None,
        indexed_at: row.indexed_at,
        created_at: None,
        viewer: None,
        labels: None,
    };

    Ok(Json(GetProfileOutput { profile }))
}

#[derive(Deserialize)]
pub struct GetProfilesParams {
    actors: Vec<String>,
}

pub async fn get_profiles(
    State(state): State<AppState>,
    Query(params): Query<GetProfilesParams>,
) -> Result<Json<GetProfilesOutput>> {
    let mut profiles = Vec::with_capacity(params.actors.len());
    for actor in params.actors {
        if let Some(row) = db::actor::get_profile(&state.db, &actor).await? {
            let handle = row.handle.unwrap_or_else(|| row.did.clone());
            profiles.push(ProfileViewDetailed {
                r#type: Some("app.bsky.actor.defs#profileViewDetailed".into()),
                did: row.did,
                handle,
                display_name: row.display_name,
                description: row.description,
                avatar: row.avatar_cid,
                banner: row.banner_cid,
                followers_count: row.followers_count,
                follows_count: row.follows_count,
                posts_count: row.posts_count,
                associated: None,
                joined_via_starter_pack: None,
                indexed_at: row.indexed_at,
                created_at: None,
                viewer: None,
                labels: None,
            });
        }
    }
    Ok(Json(GetProfilesOutput { profiles }))
}

#[derive(Deserialize)]
pub struct SearchActorsParams {
    #[serde(rename = "term")]
    term: String,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn search_actors(
    State(state): State<AppState>,
    Query(params): Query<SearchActorsParams>,
) -> Result<Json<SearchActorsOutput>> {
    let limit = params.limit.unwrap_or(25).min(100);
    let rows =
        db::actor::search_actors(&state.db, &params.term, limit, params.cursor.as_deref()).await?;

    let cursor = rows.last().map(|r| r.did.clone());
    let actors: Vec<ProfileView> = rows
        .into_iter()
        .map(|row| {
            let handle = row.handle.unwrap_or_else(|| row.did.clone());
            ProfileView {
                r#type: Some("app.bsky.actor.defs#profileView".into()),
                did: row.did,
                handle,
                display_name: row.display_name,
                description: row.description,
                avatar: row.avatar_cid,
                associated: None,
                indexed_at: row.indexed_at,
                created_at: None,
                viewer: None,
                labels: None,
            }
        })
        .collect();

    Ok(Json(SearchActorsOutput { actors, cursor }))
}

#[derive(Deserialize)]
pub struct SearchActorsTypeaheadParams {
    #[serde(rename = "term")]
    term: String,
    #[serde(default)]
    limit: Option<i64>,
}

pub async fn search_actors_typeahead(
    State(state): State<AppState>,
    Query(params): Query<SearchActorsTypeaheadParams>,
) -> Result<Json<SearchActorsTypeaheadOutput>> {
    let limit = params.limit.unwrap_or(10).min(20);
    let rows = db::actor::search_actors(&state.db, &params.term, limit, None).await?;
    let actors = rows
        .into_iter()
        .map(|row| {
            let did = row.did.clone();
            ProfileViewBasic {
                r#type: Some("app.bsky.actor.defs#profileViewBasic".into()),
                did,
                handle: row.handle.unwrap_or_else(|| row.did.clone()),
                display_name: row.display_name,
                avatar: row.avatar_cid,
                associated: None,
                viewer: None,
                labels: None,
                created_at: None,
            }
        })
        .collect();
    Ok(Json(SearchActorsTypeaheadOutput { actors }))
}

#[derive(Deserialize)]
pub struct GetSuggestionsParams {
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn get_suggestions(
    State(state): State<AppState>,
    Query(_params): Query<GetSuggestionsParams>,
) -> Result<Json<GetSuggestionsOutput>> {
    let limit = _params.limit.unwrap_or(25).min(50);
    let rows = db::actor::get_suggestions(&state.db, "", limit).await?;
    let actors: Vec<ProfileView> = rows
        .into_iter()
        .map(|row| {
            let handle = row.handle.unwrap_or_else(|| row.did.clone());
            ProfileView {
                r#type: Some("app.bsky.actor.defs#profileView".into()),
                did: row.did,
                handle,
                display_name: row.display_name,
                description: row.description,
                avatar: row.avatar_cid,
                associated: None,
                indexed_at: row.indexed_at,
                created_at: None,
                viewer: None,
                labels: None,
            }
        })
        .collect();
    Ok(Json(GetSuggestionsOutput {
        actors,
        cursor: None,
    }))
}
