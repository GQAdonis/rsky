use appview_core::error::{AppViewError, Result};
use appview_core::types::Did;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    #[serde(default)]
    pub scope: String,
}

#[derive(Debug, Clone)]
pub struct Viewer {
    pub did: Did,
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct OptionalViewer(pub Option<Viewer>);

pub fn decode_token(token: &str) -> Result<Claims> {
    let mut validation = Validation::default();
    validation.insecure_disable_signature_validation();
    validation.validate_exp = true;
    validation.validate_nbf = false;

    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(&[]), &validation)
        .map_err(|e| AppViewError::Auth(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims)
}

fn extract_bearer_token(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|v| v.to_string())
}

impl<S> FromRequestParts<S> for Viewer
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let token = extract_bearer_token(parts)
            .ok_or((StatusCode::UNAUTHORIZED, "Missing bearer token".to_string()))?;

        let claims = decode_token(&token)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))?;

        let did = Did::new(&claims.sub);

        Ok(Viewer { did, token })
    }
}

impl<S> FromRequestParts<S> for OptionalViewer
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let token = match extract_bearer_token(parts) {
            Some(t) => t,
            None => return Ok(OptionalViewer(None)),
        };

        let claims = match decode_token(&token) {
            Ok(c) => c,
            Err(_) => return Ok(OptionalViewer(None)),
        };

        let did = Did::new(&claims.sub);

        Ok(OptionalViewer(Some(Viewer { did, token })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use jsonwebtoken::{EncodingKey, Header, encode};
    use tower::ServiceExt;

    fn mint_token(sub: &str, exp_seconds: i64) -> String {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: sub.to_string(),
            iat: now,
            exp: now + exp_seconds,
            scope: String::new(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(b"test"),
        )
        .expect("mint token")
    }

    fn mint_expired_token(sub: &str) -> String {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: sub.to_string(),
            iat: now - 200,
            exp: now - 100,
            scope: String::new(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(b"test"),
        )
        .expect("mint expired token")
    }

    #[tokio::test]
    async fn viewer_rejects_missing_token() {
        let app = axum::Router::new().route(
            "/test",
            axum::routing::get(|viewer: Viewer| async move { viewer.did.to_string() }),
        );
        let req = Request::builder().uri("/test").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn viewer_rejects_expired_token() {
        let token = mint_expired_token("did:web:example.com");
        let app = axum::Router::new().route(
            "/test",
            axum::routing::get(|viewer: Viewer| async move { viewer.did.to_string() }),
        );
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn viewer_accepts_valid_token() {
        let token = mint_token("did:plc:abc123", 3600);
        let app = axum::Router::new().route(
            "/test",
            axum::routing::get(|viewer: Viewer| async move { viewer.did.to_string() }),
        );
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert_eq!(body, "did:plc:abc123");
    }

    #[tokio::test]
    async fn optional_viewer_returns_none_without_token() {
        let app = axum::Router::new().route(
            "/test",
            axum::routing::get(|viewer: OptionalViewer| async move {
                match viewer.0 {
                    Some(v) => format!("some:{}", v.did),
                    None => "none".to_string(),
                }
            }),
        );
        let req = Request::builder().uri("/test").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert_eq!(body, "none");
    }

    #[tokio::test]
    async fn optional_viewer_returns_some_with_valid_token() {
        let token = mint_token("did:plc:xyz789", 3600);
        let app = axum::Router::new().route(
            "/test",
            axum::routing::get(|viewer: OptionalViewer| async move {
                match viewer.0 {
                    Some(v) => format!("some:{}", v.did),
                    None => "none".to_string(),
                }
            }),
        );
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = String::from_utf8(
            axum::body::to_bytes(resp.into_body(), 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert_eq!(body, "some:did:plc:xyz789");
    }

    #[tokio::test]
    async fn viewer_rejects_malformed_auth_header() {
        let app = axum::Router::new().route(
            "/test",
            axum::routing::get(|viewer: Viewer| async move { viewer.did.to_string() }),
        );
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", "Basic abc123")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn viewer_rejects_garbage_token() {
        let app = axum::Router::new().route(
            "/test",
            axum::routing::get(|viewer: Viewer| async move { viewer.did.to_string() }),
        );
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer not.a.real.token")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
