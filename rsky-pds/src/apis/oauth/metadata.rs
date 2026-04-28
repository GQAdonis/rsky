use rocket::serde::json::Json;
use serde::Serialize;
use std::env;

#[derive(Debug, Serialize)]
pub struct OAuthServerMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    pub registration_endpoint: Option<String>,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub response_modes_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub pushed_authorization_request_endpoint: String,
    pub require_pushed_authorization_requests: bool,
    pub dpop_signing_alg_values_supported: Vec<String>,
    pub token_endpoint_auth_signing_alg_values_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub revocation_endpoint: String,
    pub introspection_endpoint: String,
    /// DID methods this PDS accepts for agent/service authentication.
    ///
    /// Advertised so federation peers know which DID types to use when
    /// constructing A2A service JWTs destined for this server.
    pub did_methods_supported: Vec<String>,
}

#[rocket::get("/.well-known/oauth-authorization-server")]
pub fn oauth_server_metadata() -> Json<OAuthServerMetadata> {
    let hostname = env::var("PDS_HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
    let base = format!("https://{hostname}");

    Json(OAuthServerMetadata {
        issuer: base.clone(),
        authorization_endpoint: format!("{base}/oauth/authorize"),
        token_endpoint: format!("{base}/oauth/token"),
        jwks_uri: format!("{base}/oauth/jwks.json"),
        registration_endpoint: None,
        scopes_supported: vec![
            "atproto".to_string(),
            "transition:generic".to_string(),
            "transition:chat.bsky".to_string(),
        ],
        response_types_supported: vec!["code".to_string()],
        response_modes_supported: vec!["query".to_string(), "fragment".to_string()],
        grant_types_supported: vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ],
        code_challenge_methods_supported: vec!["S256".to_string()],
        token_endpoint_auth_methods_supported: vec![
            "none".to_string(),
            "private_key_jwt".to_string(),
        ],
        pushed_authorization_request_endpoint: format!("{base}/oauth/par"),
        require_pushed_authorization_requests: true,
        dpop_signing_alg_values_supported: vec!["ES256".to_string(), "ES256K".to_string()],
        token_endpoint_auth_signing_alg_values_supported: vec![
            "ES256".to_string(),
            "ES256K".to_string(),
        ],
        subject_types_supported: vec!["public".to_string()],
        revocation_endpoint: format!("{base}/oauth/revoke"),
        introspection_endpoint: format!("{base}/oauth/introspect"),
        did_methods_supported: vec![
            "did:plc".to_string(),
            "did:web".to_string(),
            "did:key".to_string(),
            "did:peer".to_string(),
        ],
    })
}
