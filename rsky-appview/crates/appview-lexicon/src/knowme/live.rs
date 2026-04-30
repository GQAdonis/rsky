use serde::{Deserialize, Serialize};

use crate::bsky::actor::ProfileViewBasic;

/// Gate specification used when creating a live session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GateSpec {
    pub gate_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_usd_cents: Option<i64>,
}

/// `tools.know-me.live.session` — a live streaming session record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveSession {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Filled by the AppView after `createSession` completes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whip_endpoint: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_uri: Option<String>,

    pub created_at: String,
}

/// `tools.know-me.live.createSession` — input body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionInput {
    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate: Option<GateSpec>,
}

/// `tools.know-me.live.createSession` — response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionOutput {
    pub session_uri: String,
    pub whip_endpoint: String,
    pub stream_key: String,
}

/// Hydrated live session view returned by `getSession`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveSessionView {
    pub uri: String,
    pub cid: String,
    pub author: ProfileViewBasic,
    pub record: LiveSession,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer_count: Option<i64>,

    pub is_live: bool,
}

/// Token variants for viewers joining a stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ViewerToken {
    Whep { endpoint: String },
    LiveKit { server_url: String, token: String },
}

/// `tools.know-me.live.getSession` — response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSessionOutput {
    pub session: LiveSessionView,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer_token: Option<ViewerToken>,
}

/// `tools.know-me.live.getTokenLiveKit` — input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTokenLiveKitInput {
    pub session_uri: String,
}

/// `tools.know-me.live.getTokenLiveKit` — response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTokenLiveKitOutput {
    pub server_url: String,
    pub token: String,
}
