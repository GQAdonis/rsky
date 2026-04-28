use crate::apis::ApiError;

#[rocket::post("/xrpc/com.atproto.server.reserveSigningKey")]
pub async fn reserve_signing_key() -> Result<(), ApiError> {
    Err(ApiError::NotImplemented)
}
