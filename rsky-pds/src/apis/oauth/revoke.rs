use rocket::http::Status;

/// Token revocation endpoint (RFC 7009).
/// Returns 200 OK per spec (revocation always succeeds from client perspective).
#[rocket::post("/oauth/revoke", data = "<_body>")]
pub fn oauth_revoke(_body: rocket::Data<'_>) -> Status {
    // Per RFC 7009 §2.2, the server responds with HTTP 200 even for unknown tokens.
    Status::Ok
}
