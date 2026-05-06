use serde::{Deserialize, Serialize};

/// Minimal actor view — DID, handle, displayName, avatar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileViewBasic {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    pub did: String,
    pub handle: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<ViewerState>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// Medium profile — no stat counts, has avatar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileView {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    pub did: String,
    pub handle: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_at: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<ViewerState>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
}

/// Full profile with follower/following/post counts, avatar and banner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileViewDetailed {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    pub did: String,
    pub handle: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub followers_count: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub follows_count: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub posts_count: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub joined_via_starter_pack: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_at: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<ViewerState>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,
}

/// Viewer relationship state between the authenticated user and the subject.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted_by_list: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_by: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking_by_list: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub following: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub followed_by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_followers: Option<serde_json::Value>,
}

// --- Response envelopes ---

// app.bsky.actor.getProfile returns ProfileViewDetailed fields directly (no wrapper key)
pub type GetProfileOutput = ProfileViewDetailed;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProfilesOutput {
    pub profiles: Vec<ProfileViewDetailed>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchActorsOutput {
    pub actors: Vec<ProfileView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchActorsTypeaheadOutput {
    pub actors: Vec<ProfileViewBasic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSuggestionsOutput {
    pub actors: Vec<ProfileView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPreferencesOutput {
    pub preferences: Vec<serde_json::Value>,
}
