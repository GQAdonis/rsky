use crate::types::DidDocument;
use thiserror::Error;

/// Capability tier that a DID is being validated for.
///
/// Only `AccountIdentity` and `OrgIdentity` are valid for PDS account creation.
/// `AgentIdentity` is permitted for A2A/UAR agent DIDs but not as PDS account identity.
/// `SessionIdentity` and `CredentialIssuer` are reserved for future use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DidCapability {
    /// Full AT Protocol account identity: must be did:plc or did:web.
    AccountIdentity,
    /// Organizational identity for app-view services: did:web only.
    OrgIdentity,
    /// Agent identity for A2A/UAR flows: did:key or did:plc.
    AgentIdentity,
    /// Ephemeral session identity: did:key or did:peer.
    SessionIdentity,
    /// Verifiable credential issuer: did:web or did:plc.
    CredentialIssuer,
}

impl DidCapability {
    /// Returns which DID methods are permitted for this capability tier.
    pub fn permitted_methods(&self) -> &[&str] {
        match self {
            Self::AccountIdentity => &["plc", "web"],
            Self::OrgIdentity => &["web"],
            Self::AgentIdentity => &["key", "plc"],
            Self::SessionIdentity => &["key", "peer"],
            Self::CredentialIssuer => &["web", "plc"],
        }
    }
}

/// Typed error for DID resolution and profile validation.
#[derive(Debug, Error)]
pub enum DidError {
    #[error("DID method '{0}' is not permitted for capability {1:?}")]
    MethodNotPermitted(String, DidCapability),

    #[error("DID not found: {0}")]
    NotFound(String),

    #[error("DID resolution timed out after {0}ms: {1}")]
    Timeout(u64, String),

    #[error("DID document is malformed: {0}")]
    MalformedDocument(String),

    #[error("ATProto profile constraint violated: {0}")]
    ProfileConstraintViolated(String),

    #[error("Unsupported DID method: {0}")]
    UnsupportedMethod(String),

    #[error("DID resolution network error: {0}")]
    NetworkError(String),
}

/// Metadata about a DID resolution attempt.
#[derive(Debug, Clone)]
pub struct ResolutionMetadata {
    /// Wall-clock milliseconds elapsed during resolution.
    pub duration_ms: u64,
    /// The DID method that handled resolution.
    pub method: String,
    /// Whether the result was served from cache.
    pub cached: bool,
    /// HTTP status code if the resolver made a network request.
    pub http_status: Option<u16>,
}

/// Full result of a DID resolution, including the document and metadata.
#[derive(Debug, Clone)]
pub struct DidResolution {
    pub did: String,
    pub document: DidDocument,
    pub metadata: ResolutionMetadata,
}
