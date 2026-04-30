use appview_auth::Viewer;
use appview_livekit::token::ParticipantGrants;
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct TokenMintRequest {
    pub room_name: String,
    pub identity: Option<String>,
    pub can_publish: Option<bool>,
    pub can_subscribe: Option<bool>,
    pub can_publish_data: Option<bool>,
    pub ttl_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMintResponse {
    pub token: String,
    pub server_url: String,
    pub room_name: String,
    pub identity: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

pub async fn token_mint(
    State(state): State<AppState>,
    viewer: Viewer,
    Json(body): Json<TokenMintRequest>,
) -> Result<(StatusCode, Json<TokenMintResponse>), (StatusCode, Json<ErrorResponse>)> {
    let minter = state.livekit_minter.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse {
            error: "LivekitNotConfigured".to_string(),
            message: "LiveKit is not configured on this server".to_string(),
        }),
    ))?;

    if let Some(gate) = state.billing_gate.as_ref() {
        let result = gate
            .check_can_host(viewer.did.as_str())
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "BillingCheckFailed".to_string(),
                        message: e.to_string(),
                    }),
                )
            })?;

        if !result.allowed {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "InsufficientTier".to_string(),
                    message: result
                        .reason
                        .unwrap_or_else(|| "hosting not allowed for your tier".to_string()),
                }),
            ));
        }
    }

    let identity = body.identity.unwrap_or_else(|| viewer.did.to_string());

    let grants = ParticipantGrants {
        room_name: body.room_name.clone(),
        identity: identity.clone(),
        name: None,
        can_publish: body.can_publish.unwrap_or(true),
        can_subscribe: body.can_subscribe.unwrap_or(true),
        can_publish_data: body.can_publish_data.unwrap_or(true),
        ttl_seconds: body.ttl_seconds.unwrap_or(3600),
    };

    let minted = minter.mint(grants).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "TokenMintFailed".to_string(),
                message: e.to_string(),
            }),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(TokenMintResponse {
            token: minted.token,
            server_url: minted.server_url,
            room_name: minted.room_name,
            identity: minted.identity,
            expires_at: minted.expires_at,
        }),
    ))
}
