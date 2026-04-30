use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub room_name: String,
    pub host_did: String,
    pub sdp_offer: String,
    pub sdp_answer: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub session_type: SessionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    Whip,
    Whep,
}

const SESSION_TIMEOUT_SECS: i64 = 3600;

pub struct SessionStore {
    sessions: RwLock<HashMap<String, Session>>,
}

impl SessionStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            sessions: RwLock::new(HashMap::new()),
        })
    }

    pub async fn create(
        &self,
        host_did: &str,
        room_name: &str,
        session_type: SessionType,
        sdp_offer: &str,
    ) -> Session {
        let now = Utc::now();
        let session = Session {
            id: Uuid::new_v4().to_string(),
            room_name: room_name.to_string(),
            host_did: host_did.to_string(),
            sdp_offer: sdp_offer.to_string(),
            sdp_answer: None,
            created_at: now,
            last_activity: now,
            session_type,
        };

        self.sessions
            .write()
            .await
            .insert(session.id.clone(), session.clone());

        info!(
            "created {} session {} for room {}",
            match session_type {
                SessionType::Whip => "WHIP",
                SessionType::Whep => "WHEP",
            },
            session.id,
            room_name
        );

        session
    }

    pub async fn set_answer(&self, session_id: &str, sdp_answer: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.sdp_answer = Some(sdp_answer.to_string());
            session.last_activity = Utc::now();
            debug!("set SDP answer for session {}", session_id);
            true
        } else {
            false
        }
    }

    pub async fn get(&self, session_id: &str) -> Option<Session> {
        self.sessions.read().await.get(session_id).cloned()
    }

    pub async fn remove(&self, session_id: &str) -> Option<Session> {
        let removed = self.sessions.write().await.remove(session_id);
        if removed.is_some() {
            debug!("removed session {}", session_id);
        }
        removed
    }

    pub async fn cleanup_expired(&self) -> usize {
        let now = Utc::now();
        let mut sessions = self.sessions.write().await;
        let before = sessions.len();

        sessions.retain(|_, session| {
            let elapsed = (now - session.last_activity).num_seconds();
            elapsed < SESSION_TIMEOUT_SECS
        });

        let removed = before - sessions.len();
        if removed > 0 {
            info!("cleaned up {} expired WebRTC sessions", removed);
        }
        removed
    }
}
