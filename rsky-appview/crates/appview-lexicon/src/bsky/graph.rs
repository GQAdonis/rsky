use serde::{Deserialize, Serialize};

use super::actor::{ProfileView, ProfileViewBasic, ProfileViewDetailed};

/// A list record in ATProto.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListView {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    pub uri: String,
    pub cid: String,
    pub creator: ProfileViewBasic,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_facets: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_item_count: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<serde_json::Value>,

    pub indexed_at: String,
}

/// A single item in a list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItemView {
    pub uri: String,
    pub subject: ProfileViewBasic,
}

// --- Response envelopes ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFollowersOutput {
    pub subject: ProfileViewBasic,
    pub followers: Vec<ProfileView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFollowsOutput {
    pub subject: ProfileViewBasic,
    pub follows: Vec<ProfileView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetKnownFollowersOutput {
    pub subject: ProfileViewBasic,
    pub followers: Vec<ProfileView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetListOutput {
    pub list: ListView,
    pub items: Vec<ListItemView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetListsOutput {
    pub lists: Vec<ListView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMutesOutput {
    pub mutes: Vec<ProfileView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBlocksOutput {
    pub blocks: Vec<ProfileViewDetailed>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}
