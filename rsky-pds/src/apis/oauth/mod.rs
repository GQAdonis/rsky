//! AT Protocol OAuth 2.0 provider endpoints.
//!
//! Implements:
//!   GET  /.well-known/oauth-authorization-server   — server metadata (RFC 8414)
//!   GET  /oauth/jwks.json                           — JWKS (RFC 7517)
//!   POST /oauth/par                                 — Pushed Authorization Requests (RFC 9126)
//!   GET  /oauth/authorize                           — Authorization endpoint
//!   POST /oauth/token                               — Token endpoint (RFC 6749)
//!   POST /oauth/revoke                              — Token revocation (RFC 7009)
//!   POST /oauth/introspect                          — Token introspection (RFC 7662)
//!
//! Current implementation state: endpoints are wired and return correct-shaped
//! responses. Full PKCE/DPoP/PAR flows are not yet implemented — callers receive
//! an `unsupported_grant_type` / `server_error` error response rather than a panic.

pub mod authorize;
pub mod introspect;
pub mod jwks;
pub mod metadata;
pub mod par;
pub mod revoke;
pub mod token;
