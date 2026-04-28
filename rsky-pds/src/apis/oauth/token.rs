use rocket::http::Status;
use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct OAuthErrorResponse {
    pub error: String,
    pub error_description: String,
}

/// Token endpoint (RFC 6749).
/// Full implementation pending; returns a structured error response.
#[rocket::post("/oauth/token", data = "<_body>")]
pub fn oauth_token(_body: rocket::Data<'_>) -> (Status, Json<OAuthErrorResponse>) {
    (
        Status::NotImplemented,
        Json(OAuthErrorResponse {
            error: "unsupported_grant_type".to_string(),
            error_description: "OAuth token endpoint is not yet implemented on this PDS"
                .to_string(),
        }),
    )
}
