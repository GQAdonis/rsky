use crate::{AppState, AppViewError};
use appview_auth::Viewer;
use appview_core::error::Result;
use appview_db as db;
use appview_lexicon::bsky::actor::ProfileViewBasic;
use appview_lexicon::bsky::notification::{
    GetUnreadCountOutput, ListNotificationsOutput, Notification,
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListNotificationsParams {
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default)]
    seen_at: Option<String>,
}

pub async fn list_notifications(
    State(state): State<AppState>,
    viewer: Viewer,
    Query(params): Query<ListNotificationsParams>,
) -> Result<Json<ListNotificationsOutput>> {
    let limit = params.limit.unwrap_or(25).min(100);

    let rows = db::notification::list_notifications(
        &state.db,
        viewer.did.as_str(),
        limit,
        params.cursor.as_deref(),
    )
    .await?;

    let mut notifications = Vec::with_capacity(rows.len());
    for row in rows {
        let author_profile = db::actor::get_profile(&state.db, &row.author).await?;
        let author = author_profile
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
                did: row.author.clone(),
                handle: row.author.clone(),
                display_name: None,
                avatar: None,
                associated: None,
                viewer: None,
                labels: None,
                created_at: None,
            });

        notifications.push(Notification {
            r#type: Some(format!(
                "app.bsky.notification.{}#{}",
                row.reason, row.reason
            )),
            uri: row.record_uri,
            cid: row.record_cid,
            author,
            reason: row.reason,
            reason_subject: row.reason_subject,
            record: serde_json::json!({}),
            is_read: row.is_read.unwrap_or(false),
            indexed_at: row.sort_at,
            labels: None,
        });
    }

    let cursor = notifications.last().map(|n| n.indexed_at.clone());
    Ok(Json(ListNotificationsOutput {
        notifications,
        cursor,
        seen_at: params.seen_at,
        priority: None,
    }))
}

#[derive(Deserialize)]
pub struct GetUnreadCountParams {
    #[serde(default)]
    seen_at: Option<String>,
}

pub async fn get_unread_count(
    State(state): State<AppState>,
    viewer: Viewer,
    Query(_params): Query<GetUnreadCountParams>,
) -> Result<Json<GetUnreadCountOutput>> {
    let count =
        db::notification::count_unread_notifications(&state.db, viewer.did.as_str()).await?;
    Ok(Json(GetUnreadCountOutput { count }))
}
