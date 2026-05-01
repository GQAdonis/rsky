use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
#[serde(rename = "app.bsky.feed.threadgate")]
#[serde(rename_all = "camelCase")]
pub struct Threadgate {
    pub created_at: DateTime<Utc>,
    pub post: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<AllowRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_replies: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "$type")]
pub enum AllowRule {
    #[serde(rename = "app.bsky.feed.threadgate#mentionRule")]
    MentionRule(MentionRule),
    #[serde(rename = "app.bsky.feed.threadgate#followerRule")]
    FollowerRule(FollowerRule),
    #[serde(rename = "app.bsky.feed.threadgate#followingRule")]
    FollowingRule(FollowingRule),
    #[serde(rename = "app.bsky.feed.threadgate#listRule")]
    ListRule(ListRule),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MentionRule {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FollowerRule {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FollowingRule {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ListRule {
    pub list: String,
}
