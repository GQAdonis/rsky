mod actor;
mod feed;
mod graph;
mod livekit;
mod notification;
mod unspecced;
mod webrtc;

use appview_core::error::AppViewError;
use appview_identity::{DidResolver, HandleResolver, HandleResolverOpts};
use appview_livekit::{BillingGate, LiveKitConfig, TokenMinter};
use appview_webrtc::SessionStore;
use axum::{
    Router,
    routing::{get, post, put},
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

pub type AppState = Arc<AppStateInner>;

pub struct AppStateInner {
    pub db: PgPool,
    pub did_resolver: Arc<tokio::sync::Mutex<DidResolver>>,
    pub handle_resolver: Arc<tokio::sync::Mutex<HandleResolver>>,
    pub livekit_minter: Option<TokenMinter>,
    pub billing_gate: Option<BillingGate>,
    pub webrtc_sessions: Arc<SessionStore>,
}

impl AppStateInner {
    pub async fn new(database_url: &str) -> Result<Self, AppViewError> {
        let db = PgPool::connect(database_url)
            .await
            .map_err(|e| AppViewError::Storage(format!("failed to connect to db: {e}")))?;
        let did_resolver = DidResolver::new();
        let handle_resolver = HandleResolver::new(HandleResolverOpts {
            timeout: Some(std::time::Duration::from_secs(5)),
            backup_nameservers: None,
        });

        let livekit_minter = LiveKitConfig::from_env().ok().map(TokenMinter::new);
        let billing_gate = if livekit_minter.is_some() {
            Some(BillingGate::new(db.clone()))
        } else {
            None
        };

        Ok(Self {
            db,
            did_resolver: Arc::new(tokio::sync::Mutex::new(did_resolver)),
            handle_resolver: Arc::new(tokio::sync::Mutex::new(handle_resolver)),
            livekit_minter,
            billing_gate,
            webrtc_sessions: SessionStore::new(),
        })
    }
}

pub fn create_router(state: AppState) -> Router {
    info!("creating appview API router");

    Router::new()
        .route("/xrpc/_health", get(health))
        .route("/xrpc/app.bsky.actor.getProfile", get(actor::get_profile))
        .route("/xrpc/app.bsky.actor.getProfiles", get(actor::get_profiles))
        .route(
            "/xrpc/app.bsky.actor.searchActors",
            get(actor::search_actors),
        )
        .route(
            "/xrpc/app.bsky.actor.searchActorsTypeahead",
            get(actor::search_actors_typeahead),
        )
        .route(
            "/xrpc/app.bsky.actor.getSuggestions",
            get(actor::get_suggestions),
        )
        .route(
            "/xrpc/app.bsky.actor.getPreferences",
            get(actor::get_preferences),
        )
        .route(
            "/xrpc/app.bsky.actor.putPreferences",
            put(actor::put_preferences),
        )
        .route("/xrpc/app.bsky.feed.getTimeline", get(feed::get_timeline))
        .route(
            "/xrpc/app.bsky.feed.getAuthorFeed",
            get(feed::get_author_feed),
        )
        .route("/xrpc/app.bsky.feed.getFeed", get(feed::get_feed))
        .route(
            "/xrpc/app.bsky.feed.getFeedGenerator",
            get(feed::get_feed_generator),
        )
        .route(
            "/xrpc/app.bsky.feed.getFeedGenerators",
            get(feed::get_feed_generators),
        )
        .route(
            "/xrpc/app.bsky.feed.getPostThread",
            get(feed::get_post_thread),
        )
        .route("/xrpc/app.bsky.feed.getPosts", get(feed::get_posts))
        .route("/xrpc/app.bsky.feed.getLikes", get(feed::get_likes))
        .route(
            "/xrpc/app.bsky.feed.getRepostedBy",
            get(feed::get_reposted_by),
        )
        .route("/xrpc/app.bsky.feed.getListFeed", get(feed::get_list_feed))
        .route("/xrpc/app.bsky.feed.searchPosts", get(feed::search_posts))
        .route("/xrpc/app.bsky.graph.getFollows", get(graph::get_follows))
        .route(
            "/xrpc/app.bsky.graph.getFollowers",
            get(graph::get_followers),
        )
        .route("/xrpc/app.bsky.graph.getBlocks", get(graph::get_blocks))
        .route("/xrpc/app.bsky.graph.getMutes", get(graph::get_mutes))
        .route("/xrpc/app.bsky.graph.getList", get(graph::get_list))
        .route("/xrpc/app.bsky.graph.getLists", get(graph::get_lists))
        .route(
            "/xrpc/app.bsky.notification.listNotifications",
            get(notification::list_notifications),
        )
        .route(
            "/xrpc/app.bsky.notification.getUnreadCount",
            get(notification::get_unread_count),
        )
        .route(
            "/xrpc/app.bsky.unspecced.getPopularFeedGenerators",
            get(unspecced::get_popular_feed_generators),
        )
        .route(
            "/xrpc/tools.know-me.live.tokenMint",
            post(livekit::token_mint),
        )
        .route("/xrpc/tools.know-me.video.whip", post(webrtc::whip))
        .route("/xrpc/tools.know-me.video.whep", post(webrtc::whep))
        .route(
            "/xrpc/tools.knowme.feed.subscribeFeed",
            get(subscribe_feed_stub),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health() -> &'static str {
    "OK"
}

/// Stub for tools.knowme.feed.subscribeFeed WebSocket route.
/// Returns 501 Not Implemented until the full WebSocket feed is built.
async fn subscribe_feed_stub() -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::NOT_IMPLEMENTED,
        axum::Json(serde_json::json!({
            "error": "NotImplemented",
            "message": "tools.knowme.feed.subscribeFeed is not yet implemented"
        })),
    )
}
