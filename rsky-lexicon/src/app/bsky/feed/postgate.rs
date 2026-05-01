use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
#[serde(rename = "app.bsky.feed.postgate")]
#[serde(rename_all = "camelCase")]
pub struct Postgate {
    pub created_at: DateTime<Utc>,
    pub post: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detached_embedding_uris: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_rules: Option<Vec<EmbeddingRule>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum EmbeddingRule {
    #[serde(rename = "app.bsky.feed.postgate#disableRule")]
    DisableRule(DisableRule),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DisableRule {}
