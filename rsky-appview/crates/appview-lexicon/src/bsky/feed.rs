use serde::{Deserialize, Serialize};

use super::actor::ProfileViewBasic;

/// The viewer's relationship to a specific post.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostViewerState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub repost: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_disabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_muted: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_disabled: Option<bool>,
}

/// A fully hydrated post.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostView {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    pub uri: String,
    pub cid: String,
    pub author: ProfileViewBasic,

    /// The raw lexicon record value.
    pub record: serde_json::Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_count: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub repost_count: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_count: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_count: Option<i64>,

    pub indexed_at: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<PostViewerState>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub threadgate: Option<serde_json::Value>,
}

/// Reason for a repost appearing in a feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasonRepost {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    pub by: ProfileViewBasic,
    pub indexed_at: String,
}

/// Reply context — parent and root.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplyRef {
    pub root: serde_json::Value,
    pub parent: serde_json::Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grandparent_author: Option<ProfileViewBasic>,
}

/// A single item in a feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedViewPost {
    pub post: PostView,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<ReasonRepost>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<ReplyRef>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_context: Option<String>,
}

/// A single like actor + timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeView {
    pub indexed_at: String,
    pub actor: ProfileViewBasic,
}

/// Feed generator descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorView {
    #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    pub uri: String,
    pub cid: String,
    pub did: String,
    pub creator: ProfileViewBasic,
    pub display_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_facets: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_count: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepts_interactions: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer: Option<serde_json::Value>,

    pub indexed_at: String,
}

// --- Response envelopes ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineOutput {
    pub feed: Vec<FeedViewPost>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorFeedOutput {
    pub feed: Vec<FeedViewPost>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostThreadOutput {
    /// Thread union — can be a ThreadViewPost, NotFoundPost, or BlockedPost.
    pub thread: serde_json::Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub threadgate: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLikesOutput {
    pub uri: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,

    pub likes: Vec<LikeView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRepostedByOutput {
    pub uri: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,

    pub reposted_by: Vec<ProfileViewBasic>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetQuotesOutput {
    pub uri: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,

    pub posts: Vec<PostView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorLikesOutput {
    pub feed: Vec<FeedViewPost>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFeedOutput {
    pub feed: Vec<FeedViewPost>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFeedGeneratorOutput {
    pub view: GeneratorView,
    pub is_online: bool,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFeedGeneratorsOutput {
    pub feeds: Vec<GeneratorView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPostsOutput {
    pub posts: Vec<PostView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hits_total: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetListFeedOutput {
    pub feed: Vec<FeedViewPost>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}
