use crate::{AppState, AppViewError};
use appview_core::error::Result;
use appview_db as db;
use appview_lexicon::bsky::actor::ProfileViewBasic;
use appview_lexicon::bsky::feed::GeneratorView;
use appview_lexicon::bsky::unspecced::GetPopularFeedGeneratorsOutput;
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetPopularFeedGeneratorsParams {
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
}

pub async fn get_popular_feed_generators(
    State(state): State<AppState>,
    Query(params): Query<GetPopularFeedGeneratorsParams>,
) -> Result<Json<GetPopularFeedGeneratorsOutput>> {
    let limit = params.limit.unwrap_or(10).min(100);
    let rows =
        db::generator::get_popular_feed_generators(&state.db, limit, params.cursor.as_deref())
            .await?;

    let mut feeds = Vec::with_capacity(rows.len());
    for row in rows {
        let creator_profile = db::actor::get_profile(&state.db, &row.creator).await?;
        let creator = creator_profile
            .map(|a| ProfileViewBasic {
                r#type: Some("app.bsky.actor.defs#profileViewBasic".into()),
                did: a.did.clone(),
                handle: a.handle.unwrap_or_else(|| a.did.clone()),
                display_name: a.display_name,
                avatar: a.avatar_cid,
                associated: None,
                viewer: None,
                labels: None,
                created_at: None,
            })
            .unwrap_or_else(|| ProfileViewBasic {
                r#type: Some("app.bsky.actor.defs#profileViewBasic".into()),
                did: row.creator.clone(),
                handle: row.creator.clone(),
                display_name: None,
                avatar: None,
                associated: None,
                viewer: None,
                labels: None,
                created_at: None,
            });

        feeds.push(GeneratorView {
            r#type: Some("app.bsky.feed.defs#generatorView".into()),
            uri: row.uri,
            cid: row.cid,
            did: row.feed_did,
            creator,
            display_name: row.display_name,
            description: row.description,
            description_facets: None,
            avatar: row.avatar_cid,
            like_count: Some(row.like_count.unwrap_or(0)),
            accepts_interactions: None,
            labels: None,
            viewer: None,
            indexed_at: row.indexed_at,
        });
    }

    let cursor = feeds.last().map(|f| f.uri.clone());
    Ok(Json(GetPopularFeedGeneratorsOutput { feeds, cursor }))
}
