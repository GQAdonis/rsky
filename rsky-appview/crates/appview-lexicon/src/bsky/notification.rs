use serde::{Deserialize, Serialize};

use super::actor::ProfileViewBasic;

/// A single notification record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    pub uri: String,
    pub cid: String,
    pub author: ProfileViewBasic,

    /// One of: "like", "repost", "follow", "mention", "reply", "quote", "starterpack-joined".
    pub reason: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_subject: Option<String>,

    pub record: serde_json::Value,
    pub is_read: bool,
    pub indexed_at: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
}

// --- Response envelopes ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListNotificationsOutput {
    pub notifications: Vec<Notification>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub seen_at: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUnreadCountOutput {
    pub count: i64,
}
