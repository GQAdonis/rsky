use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct IntrospectionResponse {
    pub active: bool,
}

/// Token introspection endpoint (RFC 7662).
/// Returns `active: false` for all tokens until full OAuth is implemented.
#[rocket::post("/oauth/introspect", data = "<_body>")]
pub fn oauth_introspect(_body: rocket::Data<'_>) -> Json<IntrospectionResponse> {
    Json(IntrospectionResponse { active: false })
}
