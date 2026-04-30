use appview_auth::Viewer;
use appview_webrtc::SessionType;
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SdpOfferRequest {
    pub room_name: String,
    pub sdp_offer: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SdpAnswerResponse {
    pub session_id: String,
    pub sdp_answer: String,
    pub room_name: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl ErrorResponse {
    fn invalid_request(msg: impl Into<String>) -> (StatusCode, Json<Self>) {
        (
            StatusCode::BAD_REQUEST,
            Json(Self {
                error: "InvalidRequest".into(),
                message: msg.into(),
            }),
        )
    }

    fn internal(msg: impl Into<String>) -> (StatusCode, Json<Self>) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Self {
                error: "InternalServerError".into(),
                message: msg.into(),
            }),
        )
    }
}

fn validate_offer(req: &SdpOfferRequest) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if req.room_name.is_empty() {
        return Err(ErrorResponse::invalid_request("roomName is required"));
    }
    if req.room_name.len() > 128 {
        return Err(ErrorResponse::invalid_request(
            "roomName exceeds 128 characters",
        ));
    }
    if req.sdp_offer.is_empty() {
        return Err(ErrorResponse::invalid_request("sdpOffer is required"));
    }
    if !req.sdp_offer.starts_with("v=0") {
        return Err(ErrorResponse::invalid_request(
            "invalid SDP offer: must start with v=0",
        ));
    }
    Ok(())
}

pub async fn whip(
    State(state): State<AppState>,
    viewer: Viewer,
    Json(body): Json<SdpOfferRequest>,
) -> Result<(StatusCode, Json<SdpAnswerResponse>), (StatusCode, Json<ErrorResponse>)> {
    validate_offer(&body)?;

    let session = state
        .webrtc_sessions
        .create(
            viewer.did.as_str(),
            &body.room_name,
            SessionType::Whip,
            &body.sdp_offer,
        )
        .await;

    let sdp_answer = build_proxy_answer(&session.id, "sendrecv");

    if !state
        .webrtc_sessions
        .set_answer(&session.id, &sdp_answer)
        .await
    {
        return Err(ErrorResponse::internal(
            "failed to store SDP answer for session",
        ));
    }

    Ok((
        StatusCode::OK,
        Json(SdpAnswerResponse {
            session_id: session.id,
            sdp_answer,
            room_name: body.room_name,
        }),
    ))
}

pub async fn whep(
    State(state): State<AppState>,
    viewer: Viewer,
    Json(body): Json<SdpOfferRequest>,
) -> Result<(StatusCode, Json<SdpAnswerResponse>), (StatusCode, Json<ErrorResponse>)> {
    validate_offer(&body)?;

    let session = state
        .webrtc_sessions
        .create(
            viewer.did.as_str(),
            &body.room_name,
            SessionType::Whep,
            &body.sdp_offer,
        )
        .await;

    let sdp_answer = build_proxy_answer(&session.id, "recvonly");

    if !state
        .webrtc_sessions
        .set_answer(&session.id, &sdp_answer)
        .await
    {
        return Err(ErrorResponse::internal(
            "failed to store SDP answer for session",
        ));
    }

    Ok((
        StatusCode::OK,
        Json(SdpAnswerResponse {
            session_id: session.id,
            sdp_answer,
            room_name: body.room_name,
        }),
    ))
}

fn build_proxy_answer(session_id: &str, direction: &str) -> String {
    format!(
        "v=0\r\n\
         o=- {session_id} 1 IN IP4 0.0.0.0\r\n\
         s=-\r\n\
         t=0 0\r\n\
         a=group:BUNDLE 0\r\n\
         m=video 9 UDP/TLS/RTP/SAVPF 96\r\n\
         c=IN IP4 0.0.0.0\r\n\
         a=setup:active\r\n\
         a=mid:0\r\n\
         a={direction}\r\n"
    )
}
