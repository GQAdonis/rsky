use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct JwksResponse {
    pub keys: Vec<serde_json::Value>,
}

/// Returns the public JWKS for this PDS. Currently returns an empty key set;
/// full implementation will export the PDS signing key in JWK format.
#[rocket::get("/oauth/jwks.json")]
pub fn oauth_jwks() -> Json<JwksResponse> {
    Json(JwksResponse { keys: vec![] })
}
