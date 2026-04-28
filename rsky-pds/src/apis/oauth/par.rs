use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ParRequest {
    pub response_type: Option<String>,
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParResponse {
    pub request_uri: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct OAuthErrorResponse {
    pub error: String,
    pub error_description: String,
}

/// Pushed Authorization Request endpoint (RFC 9126).
/// Full PAR + PKCE + DPoP implementation is pending; returns a structured error
/// rather than panicking so that OAuth clients receive a valid error response.
#[rocket::post("/oauth/par", data = "<_body>")]
pub fn oauth_par(_body: rocket::Data<'_>) -> (Status, Json<OAuthErrorResponse>) {
    (
        Status::NotImplemented,
        Json(OAuthErrorResponse {
            error: "server_error".to_string(),
            error_description: "OAuth PAR is not yet implemented on this PDS".to_string(),
        }),
    )
}
