//! AT Protocol OAuth scope grammar — Rust port of `@atproto/oauth-scopes`.
//!
//! Scopes follow the format:
//!   `atproto`                 — the base ATProto access scope
//!   `transition:generic`      — legacy transition scope
//!   `transition:chat.bsky`    — legacy Bluesky chat transition scope
//!
//! Granular per-NSID scopes are represented as:
//!   `com.atproto.repo.createRecord`
//!
//! A `ScopeSet` is an ordered, deduplicated set of scopes that can be
//! serialized to/from the space-separated OAuth scope string format.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

pub mod error;
pub use error::ScopeError;

/// A single OAuth scope token.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OAuthScope(String);

impl OAuthScope {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns `true` if this scope is the base AT Protocol scope.
    pub fn is_atproto(&self) -> bool {
        self.0 == "atproto"
    }

    /// Returns `true` if this scope is a transition scope.
    pub fn is_transition(&self) -> bool {
        self.0.starts_with("transition:")
    }
}

impl FromStr for OAuthScope {
    type Err = ScopeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ScopeError::Empty);
        }
        // Scope tokens must not contain whitespace per RFC 6749 §3.3
        if s.contains(char::is_whitespace) {
            return Err(ScopeError::ContainsWhitespace(s.to_string()));
        }
        Ok(OAuthScope(s.to_string()))
    }
}

impl fmt::Display for OAuthScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A deduplicated, ordered set of OAuth scopes.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScopeSet(BTreeSet<OAuthScope>);

impl ScopeSet {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn insert(&mut self, scope: OAuthScope) -> bool {
        self.0.insert(scope)
    }

    pub fn contains(&self, scope: &OAuthScope) -> bool {
        self.0.contains(scope)
    }

    pub fn contains_str(&self, s: &str) -> bool {
        self.0.iter().any(|sc| sc.as_str() == s)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Parse a space-separated OAuth scope string.
    pub fn parse(s: &str) -> Result<Self, ScopeError> {
        let mut set = ScopeSet::new();
        for token in s.split_whitespace() {
            set.insert(OAuthScope::from_str(token)?);
        }
        Ok(set)
    }

    /// Serialize to a space-separated OAuth scope string.
    pub fn to_scope_string(&self) -> String {
        self.0
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn iter(&self) -> impl Iterator<Item = &OAuthScope> {
        self.0.iter()
    }

    /// Returns `true` if `self` is a subset of `other`.
    pub fn is_subset_of(&self, other: &ScopeSet) -> bool {
        self.0.iter().all(|s| other.0.contains(s))
    }
}

impl fmt::Display for ScopeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_scope_string())
    }
}

impl FromStr for ScopeSet {
    type Err = ScopeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Scope permission levels for AT Protocol operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    /// Read-only access.
    Read,
    /// Read-write access.
    Write,
    /// No access.
    None,
}

/// Check whether a set of granted scopes permits a specific XRPC call.
///
/// Returns `true` if the granted scopes include either `atproto` (full access)
/// or the specific NSID-level scope for the requested XRPC method.
pub fn scope_permits_xrpc(granted: &ScopeSet, nsid: &str) -> bool {
    if granted.contains_str("atproto") {
        return true;
    }
    granted.contains_str(nsid)
}

/// Well-known scope constants.
pub mod known {
    /// The base AT Protocol access scope — grants full XRPC access.
    pub const ATPROTO: &str = "atproto";
    /// Legacy transition scope for Bluesky-specific endpoints.
    pub const TRANSITION_GENERIC: &str = "transition:generic";
    /// Legacy transition scope for Bluesky chat endpoints.
    pub const TRANSITION_CHAT_BSKY: &str = "transition:chat.bsky";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_scope() {
        let sc = OAuthScope::from_str("atproto").unwrap();
        assert!(sc.is_atproto());
    }

    #[test]
    fn rejects_empty_scope() {
        assert!(OAuthScope::from_str("").is_err());
    }

    #[test]
    fn rejects_whitespace_scope() {
        assert!(OAuthScope::from_str("a b").is_err());
    }

    #[test]
    fn scope_set_deduplicates() {
        let mut set = ScopeSet::parse("atproto atproto com.atproto.repo.createRecord").unwrap();
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn scope_set_roundtrips() {
        let s = "atproto transition:generic";
        let set = ScopeSet::parse(s).unwrap();
        let back = set.to_scope_string();
        // BTreeSet orders alphabetically: atproto < transition:generic
        assert_eq!(back, "atproto transition:generic");
    }

    #[test]
    fn scope_permits_xrpc_via_atproto() {
        let granted = ScopeSet::parse("atproto").unwrap();
        assert!(scope_permits_xrpc(
            &granted,
            "com.atproto.repo.createRecord"
        ));
    }

    #[test]
    fn scope_permits_xrpc_via_nsid() {
        let granted = ScopeSet::parse("com.atproto.repo.createRecord").unwrap();
        assert!(scope_permits_xrpc(
            &granted,
            "com.atproto.repo.createRecord"
        ));
        assert!(!scope_permits_xrpc(
            &granted,
            "com.atproto.repo.deleteRecord"
        ));
    }

    #[test]
    fn subset_check() {
        let full = ScopeSet::parse("atproto transition:generic").unwrap();
        let sub = ScopeSet::parse("atproto").unwrap();
        assert!(sub.is_subset_of(&full));
        assert!(!full.is_subset_of(&sub));
    }
}
