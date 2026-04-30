use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppViewError {
    #[error("InvalidRequest: {0}")]
    InvalidRequest(String),

    #[error("AuthRequired")]
    AuthRequired,

    #[error("AuthMissing")]
    AuthMissing,

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Forbidden")]
    Forbidden,

    #[error("NotFound")]
    NotFound,

    #[error("RateLimited")]
    RateLimited,

    #[error("UpstreamError: {0}")]
    UpstreamError(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Identity error: {0}")]
    Identity(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct XrpcError {
    error: &'static str,
    message: String,
}

impl IntoResponse for AppViewError {
    fn into_response(self) -> Response {
        let (status, error, message) = match &self {
            AppViewError::InvalidRequest(msg) => {
                (StatusCode::BAD_REQUEST, "InvalidRequest", msg.clone())
            }
            AppViewError::AuthRequired => (
                StatusCode::UNAUTHORIZED,
                "AuthRequired",
                "Authentication required".into(),
            ),
            AppViewError::AuthMissing => (
                StatusCode::UNAUTHORIZED,
                "AuthMissing",
                "Missing authorization header".into(),
            ),
            AppViewError::Auth(msg) => (
                StatusCode::UNAUTHORIZED,
                "AuthenticationRequired",
                msg.clone(),
            ),
            AppViewError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden", "Forbidden".into()),
            AppViewError::NotFound => (StatusCode::NOT_FOUND, "NotFound", "Not found".into()),
            AppViewError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "RateLimited",
                "Rate limit exceeded".into(),
            ),
            AppViewError::UpstreamError(msg) => {
                (StatusCode::BAD_GATEWAY, "UpstreamError", msg.clone())
            }
            AppViewError::Database(msg) => {
                tracing::error!("database error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "InternalServerError",
                    "Internal server error".into(),
                )
            }
            AppViewError::Storage(msg) => {
                tracing::error!("storage error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "InternalServerError",
                    "Internal server error".into(),
                )
            }
            AppViewError::Identity(msg) => {
                tracing::error!("identity error: {msg}");
                (
                    StatusCode::BAD_GATEWAY,
                    "UpstreamError",
                    format!("Failed to resolve identity: {}", msg),
                )
            }
            AppViewError::Internal(msg) => {
                tracing::error!("internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "InternalServerError",
                    "Internal server error".into(),
                )
            }
        };

        (status, Json(XrpcError { error, message })).into_response()
    }
}

impl From<sqlx::Error> for AppViewError {
    fn from(e: sqlx::Error) -> Self {
        AppViewError::Database(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppViewError>;
