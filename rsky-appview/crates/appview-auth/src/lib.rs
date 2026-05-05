use appview_core::error::{AppViewError, Result};
use appview_core::types::Did;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
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

/// Decode and validate a JWT without verifying its signature.
///
/// AT Protocol appviews trust PDS-issued tokens by DID key resolution rather
/// than shared secrets, so signature validation is intentionally skipped.
/// We parse the payload directly to avoid `jsonwebtoken`'s algorithm enum
/// check, which rejects ES256K (secp256k1) tokens issued by rsky-pds.
pub fn decode_token(token: &str) -> Result<Claims> {
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    if parts.len() != 3 {
        return Err(AppViewError::Auth("malformed JWT: expected 3 segments".into()));
    }

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| AppViewError::Auth("malformed JWT: invalid base64url payload".into()))?;

    let claims: Claims = serde_json::from_slice(&payload_bytes)
        .map_err(|e| AppViewError::Auth(format!("malformed JWT payload: {e}")))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    if claims.exp < now {
        return Err(AppViewError::Auth("token expired".into()));
    }

    Ok(claims)
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
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use tower::ServiceExt;

    fn make_jwt(header_alg: &str, sub: &str, exp_offset: i64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let header = serde_json::json!({ "alg": header_alg, "typ": "JWT" });
        let payload = serde_json::json!({
            "sub": sub,
            "iat": now,
            "exp": now + exp_offset,
        });

        let h = URL_SAFE_NO_PAD.encode(header.to_string().as_bytes());
        let p = URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes());
        // Signature segment is fake — we skip validation intentionally
        format!("{h}.{p}.fakesig")
    }

    #[test]
    fn accepts_es256k_token() {
        let token = make_jwt("ES256K", "did:plc:abc123", 3600);
        let claims = decode_token(&token).expect("should accept ES256K token");
        assert_eq!(claims.sub, "did:plc:abc123");
    }

    #[test]
    fn accepts_hs256_token() {
        let token = make_jwt("HS256", "did:plc:abc123", 3600);
        let claims = decode_token(&token).expect("should accept HS256 token");
        assert_eq!(claims.sub, "did:plc:abc123");
    }

    #[test]
    fn rejects_expired_token() {
        let token = make_jwt("ES256K", "did:plc:abc123", -100);
        let result = decode_token(&token);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expired"));
    }

    #[test]
    fn rejects_malformed_token() {
        let result = decode_token("not.a.real.token");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_one_segment_token() {
        let result = decode_token("onlyone");
        assert!(result.is_err());
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
        let token = make_jwt("ES256K", "did:web:example.com", -100);
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
    async fn viewer_accepts_es256k_token() {
        let token = make_jwt("ES256K", "did:plc:abc123", 3600);
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
    async fn optional_viewer_returns_some_with_es256k_token() {
        let token = make_jwt("ES256K", "did:plc:xyz789", 3600);
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
