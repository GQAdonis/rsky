use appview_core::error::{AppViewError, Result};
use livekit_api::access_token::{AccessToken, VideoGrants};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// LiveKit server configuration
#[derive(Debug, Clone)]
pub struct LiveKitConfig {
    pub api_key: String,
    pub api_secret: String,
    pub server_url: String,
}

impl LiveKitConfig {
    pub fn new(
        api_key: impl Into<String>,
        api_secret: impl Into<String>,
        server_url: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
            server_url: server_url.into(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("LIVEKIT_API_KEY")
            .map_err(|_| AppViewError::Internal("LIVEKIT_API_KEY not set".into()))?;
        let api_secret = std::env::var("LIVEKIT_API_SECRET")
            .map_err(|_| AppViewError::Internal("LIVEKIT_API_SECRET not set".into()))?;
        let server_url = std::env::var("LIVEKIT_URL")
            .map_err(|_| AppViewError::Internal("LIVEKIT_URL not set".into()))?;
        Ok(Self::new(api_key, api_secret, server_url))
    }
}

/// Token grants for a participant joining a room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantGrants {
    /// Room the token grants access to
    pub room_name: String,
    /// Participant identity (DID)
    pub identity: String,
    /// Friendly display name
    pub name: Option<String>,
    /// Whether participant can publish audio/video
    pub can_publish: bool,
    /// Whether participant can subscribe to others
    pub can_subscribe: bool,
    /// Whether participant can publish data channel messages
    pub can_publish_data: bool,
    /// Token TTL in seconds (default 3600)
    pub ttl_seconds: u64,
}

impl Default for ParticipantGrants {
    fn default() -> Self {
        Self {
            room_name: String::new(),
            identity: String::new(),
            name: None,
            can_publish: true,
            can_subscribe: true,
            can_publish_data: true,
            ttl_seconds: 3600,
        }
    }
}

/// Minted JWT response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintedToken {
    pub token: String,
    pub server_url: String,
    pub room_name: String,
    pub identity: String,
    pub expires_at: i64,
}

/// Mints LiveKit access tokens for participants
pub struct TokenMinter {
    config: LiveKitConfig,
}

impl TokenMinter {
    pub fn new(config: LiveKitConfig) -> Self {
        Self { config }
    }

    /// Mint a token for a participant with given grants
    pub fn mint(&self, grants: ParticipantGrants) -> Result<MintedToken> {
        let video_grants = VideoGrants {
            room: grants.room_name.clone(),
            room_join: true,
            can_publish: grants.can_publish,
            can_subscribe: grants.can_subscribe,
            can_publish_data: grants.can_publish_data,
            ..Default::default()
        };

        let ttl = Duration::from_secs(grants.ttl_seconds);

        let token = AccessToken::with_api_key(&self.config.api_key, &self.config.api_secret)
            .with_identity(&grants.identity)
            .with_name(grants.name.as_deref().unwrap_or(&grants.identity))
            .with_ttl(ttl)
            .with_grants(video_grants)
            .to_jwt()
            .map_err(|e| AppViewError::Internal(format!("failed to mint livekit token: {e}")))?;

        let expires_at = chrono::Utc::now().timestamp() + grants.ttl_seconds as i64;

        Ok(MintedToken {
            token,
            server_url: self.config.server_url.clone(),
            room_name: grants.room_name,
            identity: grants.identity,
            expires_at,
        })
    }

    /// Mint a host token (can publish + manage room)
    pub fn mint_host(
        &self,
        room_name: &str,
        did: &str,
        display_name: Option<&str>,
    ) -> Result<MintedToken> {
        self.mint(ParticipantGrants {
            room_name: room_name.to_string(),
            identity: did.to_string(),
            name: display_name.map(|s| s.to_string()),
            can_publish: true,
            can_subscribe: true,
            can_publish_data: true,
            ttl_seconds: 7200,
        })
    }

    /// Mint a viewer token (subscribe only, no publish)
    pub fn mint_viewer(
        &self,
        room_name: &str,
        did: &str,
        display_name: Option<&str>,
    ) -> Result<MintedToken> {
        self.mint(ParticipantGrants {
            room_name: room_name.to_string(),
            identity: did.to_string(),
            name: display_name.map(|s| s.to_string()),
            can_publish: false,
            can_subscribe: true,
            can_publish_data: false,
            ttl_seconds: 3600,
        })
    }

    /// Mint an AI agent token (can publish data, but not audio/video by default)
    pub fn mint_agent(&self, room_name: &str, agent_did: &str) -> Result<MintedToken> {
        self.mint(ParticipantGrants {
            room_name: room_name.to_string(),
            identity: agent_did.to_string(),
            name: Some(format!("agent:{}", &agent_did[..agent_did.len().min(16)])),
            can_publish: true,
            can_subscribe: true,
            can_publish_data: true,
            ttl_seconds: 86400,
        })
    }
}
