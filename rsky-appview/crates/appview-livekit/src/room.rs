use appview_core::error::{AppViewError, Result};
use livekit_api::services::room::{CreateRoomOptions, RoomClient};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::token::LiveKitConfig;

/// Room metadata attached to every LiveKit room we create
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMeta {
    /// ATProto DID of the host
    pub host_did: String,
    /// AT URI of the tools.know-me.live.session record
    pub session_uri: String,
    /// Whether the session is gated (paid)
    pub gated: bool,
}

/// Options for creating a new room
#[derive(Debug, Clone)]
pub struct CreateRoomParams {
    pub room_name: String,
    pub host_did: String,
    pub session_uri: String,
    pub gated: bool,
    /// Max participants (0 = unlimited)
    pub max_participants: u32,
    /// Empty room timeout in seconds
    pub empty_timeout_secs: u32,
}

impl Default for CreateRoomParams {
    fn default() -> Self {
        Self {
            room_name: String::new(),
            host_did: String::new(),
            session_uri: String::new(),
            gated: false,
            max_participants: 0,
            empty_timeout_secs: 300,
        }
    }
}

/// Room info returned to callers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub name: String,
    pub sid: String,
    pub num_participants: u32,
    pub max_participants: u32,
    pub creation_time: i64,
    pub metadata: Option<String>,
}

/// Wrapper around the LiveKit RoomService API
pub struct RoomService {
    client: RoomClient,
}

impl RoomService {
    pub fn new(config: &LiveKitConfig) -> Result<Self> {
        let client =
            RoomClient::with_api_key(&config.server_url, &config.api_key, &config.api_secret);
        Ok(Self { client })
    }

    /// Create a new room for a live session
    pub async fn create_room(&self, params: CreateRoomParams) -> Result<RoomInfo> {
        let meta = RoomMeta {
            host_did: params.host_did.clone(),
            session_uri: params.session_uri.clone(),
            gated: params.gated,
        };
        let meta_json = serde_json::to_string(&meta).map_err(|e| {
            AppViewError::Internal(format!("failed to serialize room metadata: {e}"))
        })?;

        let options = CreateRoomOptions {
            max_participants: params.max_participants,
            empty_timeout: params.empty_timeout_secs,
            metadata: meta_json,
            ..Default::default()
        };

        let room = self
            .client
            .create_room(&params.room_name, options)
            .await
            .map_err(|e| AppViewError::Internal(format!("failed to create livekit room: {e}")))?;

        info!(
            "created livekit room {} for host {}",
            params.room_name, params.host_did
        );

        Ok(RoomInfo {
            name: room.name,
            sid: room.sid,
            num_participants: room.num_participants,
            max_participants: room.max_participants,
            creation_time: room.creation_time,
            metadata: Some(room.metadata),
        })
    }

    /// Delete a room (called when session ends)
    pub async fn delete_room(&self, room_name: &str) -> Result<()> {
        self.client
            .delete_room(room_name)
            .await
            .map_err(|e| AppViewError::Internal(format!("failed to delete livekit room: {e}")))?;

        info!("deleted livekit room {}", room_name);
        Ok(())
    }

    /// List active participants in a room
    pub async fn list_participants(&self, room_name: &str) -> Result<Vec<String>> {
        let participants = self
            .client
            .list_participants(room_name)
            .await
            .map_err(|e| AppViewError::Internal(format!("failed to list participants: {e}")))?;

        let identities = participants.into_iter().map(|p| p.identity).collect();
        debug!("listed participants for room {}", room_name);
        Ok(identities)
    }

    /// Remove a participant from a room (moderation)
    pub async fn remove_participant(&self, room_name: &str, identity: &str) -> Result<()> {
        self.client
            .remove_participant(room_name, identity)
            .await
            .map_err(|e| AppViewError::Internal(format!("failed to remove participant: {e}")))?;

        info!("removed {} from room {}", identity, room_name);
        Ok(())
    }

    /// Mute a participant's track (moderation)
    pub async fn mute_participant(
        &self,
        room_name: &str,
        identity: &str,
        track_sid: &str,
        muted: bool,
    ) -> Result<()> {
        self.client
            .mute_published_track(room_name, identity, track_sid, muted)
            .await
            .map_err(|e| AppViewError::Internal(format!("failed to mute track: {e}")))?;

        debug!(
            "set mute={} for {} track {} in room {}",
            muted, identity, track_sid, room_name
        );
        Ok(())
    }
}
