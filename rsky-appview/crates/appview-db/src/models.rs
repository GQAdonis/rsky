#[derive(Debug, sqlx::FromRow)]
pub struct ActorRow {
    pub did: String,
    pub handle: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub avatar_cid: Option<String>,
    pub banner_cid: Option<String>,
    pub indexed_at: Option<String>,
    pub followers_count: Option<i64>,
    pub follows_count: Option<i64>,
    pub posts_count: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PostRow {
    pub uri: String,
    pub cid: String,
    pub creator: String,
    pub text: String,
    pub reply_root: Option<String>,
    pub reply_parent: Option<String>,
    pub reply_count: Option<i64>,
    pub repost_count: Option<i64>,
    pub like_count: Option<i64>,
    pub quote_count: Option<i64>,
    pub created_at: String,
    pub indexed_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PostWithAuthorRow {
    // Post fields
    pub uri: String,
    pub cid: String,
    pub creator: String,
    pub text: String,
    pub reply_root: Option<String>,
    pub reply_parent: Option<String>,
    pub reply_count: Option<i64>,
    pub repost_count: Option<i64>,
    pub like_count: Option<i64>,
    pub quote_count: Option<i64>,
    pub created_at: String,
    pub indexed_at: String,
    // Author fields
    pub author_did: String,
    pub author_handle: Option<String>,
    pub author_display_name: Option<String>,
    pub author_description: Option<String>,
    pub author_avatar_cid: Option<String>,
    pub author_banner_cid: Option<String>,
    pub author_indexed_at: Option<String>,
    pub author_followers_count: Option<i64>,
    pub author_follows_count: Option<i64>,
    pub author_posts_count: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct LikeWithActorRow {
    pub like_uri: String,
    pub actor_did: String,
    pub actor_handle: Option<String>,
    pub actor_display_name: Option<String>,
    pub actor_description: Option<String>,
    pub actor_avatar_cid: Option<String>,
    pub actor_banner_cid: Option<String>,
    pub actor_indexed_at: Option<String>,
    pub actor_followers_count: Option<i64>,
    pub actor_follows_count: Option<i64>,
    pub actor_posts_count: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RepostWithActorRow {
    pub repost_uri: String,
    pub actor_did: String,
    pub actor_handle: Option<String>,
    pub actor_display_name: Option<String>,
    pub actor_description: Option<String>,
    pub actor_avatar_cid: Option<String>,
    pub actor_banner_cid: Option<String>,
    pub actor_indexed_at: Option<String>,
    pub actor_followers_count: Option<i64>,
    pub actor_follows_count: Option<i64>,
    pub actor_posts_count: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct FollowRow {
    pub uri: String,
    pub creator: String,
    pub subject_did: String,
    pub created_at: String,
    pub indexed_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct NotificationRow {
    pub did: String,
    pub author: String,
    pub record_uri: String,
    pub record_cid: String,
    pub reason: String,
    pub reason_subject: Option<String>,
    pub is_read: Option<bool>,
    pub sort_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct GeneratorRow {
    pub uri: String,
    pub cid: String,
    pub creator: String,
    pub feed_did: String,
    pub display_name: String,
    pub description: Option<String>,
    pub avatar_cid: Option<String>,
    pub like_count: Option<i64>,
    pub indexed_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ListRow {
    pub uri: String,
    pub cid: String,
    pub creator_did: String,
    pub name: String,
    pub purpose: String,
    pub description: Option<String>,
    pub avatar_cid: Option<String>,
    pub indexed_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ListItemRow {
    pub uri: String,
    pub subject_did: String,
    pub subject_handle: Option<String>,
    pub subject_display_name: Option<String>,
    pub subject_avatar_cid: Option<String>,
    pub indexed_at: String,
}
