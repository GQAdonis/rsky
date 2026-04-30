use serde::{Deserialize, Serialize};

use crate::bsky::actor::ProfileViewBasic;

/// Aspect ratio of a video.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AspectRatio {
    pub width: u32,
    pub height: u32,
}

/// Gate controlling access to paid/subscription content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GateType {
    Free,
    Paid,
    Subscription,
}

/// `tools.know-me.video.post` — a short video record stored in the PDS repo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPost {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// Blob CID of the video file.
    pub video_cid: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_cid: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<AspectRatio>,

    pub labels: Vec<String>,
    pub created_at: String,
}

/// `tools.know-me.video.vod` — a long-form VOD with HLS/MoQ manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoVod {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// HLS/MoQ manifest URL.
    pub manifest_uri: String,

    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub duration_secs: f64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_cid: Option<String>,

    /// AT URI reference to a `video.gate` record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_uri: Option<String>,

    pub created_at: String,
}

/// `tools.know-me.video.gate` — access gate for a content record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoGate {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// AT URI of the gated content.
    pub content_uri: String,

    pub gate_type: GateType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_usd_cents: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_plan: Option<String>,

    pub created_at: String,
}

/// Hydrated view of a `VideoPost` record as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPostView {
    pub uri: String,
    pub cid: String,
    pub author: ProfileViewBasic,
    pub record: VideoPost,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_count: Option<i64>,

    pub indexed_at: String,
}

/// `tools.know-me.video.getFeed` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFeedOutput {
    pub feed: Vec<VideoPostView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}
