use crate::did::capability::{DidCapability, DidError, DidResolution, ResolutionMetadata};
use crate::did::resolver_trait::DidResolver;
use crate::types::{DidDocument, VerificationMethod, VerificationMethodRef};
use async_trait::async_trait;
use std::time::Instant;

/// Resolver for `did:peer` nuance 0 — single-key ephemeral DIDs.
///
/// Only nuance 0 (`did:peer:0<multibase-encoded-key>`) is supported. Nuances 1, 2,
/// and 4 require out-of-band exchange and are rejected at resolve time.
///
/// No network calls are made. The document is derived from the encoded key bytes.
///
/// ## Capability constraints
///
/// `did:peer` is **never** permitted as `AccountIdentity` or `OrgIdentity`.
/// Permitted tiers:
/// - `SessionIdentity` — primary use case; ephemeral per-session channel identity
/// - `AgentIdentity`  — allowed for local agent channels that don't federate
pub struct DidPeerResolver;

impl DidPeerResolver {
    pub fn new() -> Self {
        Self
    }

    fn synthesise_nuance0(did: &str) -> Result<DidDocument, DidError> {
        // did:peer:0<multibase-key>
        let identifier = did
            .strip_prefix("did:peer:")
            .ok_or_else(|| DidError::MalformedDocument(format!("not a did:peer DID: {did}")))?;

        if !identifier.starts_with('0') {
            return Err(DidError::MalformedDocument(format!(
                "did:peer nuance '{}' is not supported; only nuance 0 is implemented",
                identifier.chars().next().unwrap_or('?')
            )));
        }

        // Strip nuance prefix; remainder is the multibase-encoded key.
        let multibase_key = &identifier[1..];
        if multibase_key.is_empty() {
            return Err(DidError::MalformedDocument(
                "did:peer:0 has no key material".to_string(),
            ));
        }

        let vm_id = format!("{did}#key-1");
        let vm = VerificationMethod {
            id: vm_id.clone(),
            r#type: "Multikey".to_string(),
            controller: did.to_string(),
            public_key_multibase: Some(multibase_key.to_string()),
            public_key_jwk: None,
        };

        Ok(DidDocument {
            context: Some(vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/multikey/v1".to_string(),
            ]),
            id: did.to_string(),
            also_known_as: None,
            verification_method: Some(vec![vm]),
            authentication: Some(vec![VerificationMethodRef::Reference(vm_id.clone())]),
            assertion_method: Some(vec![VerificationMethodRef::Reference(vm_id.clone())]),
            capability_invocation: Some(vec![VerificationMethodRef::Reference(vm_id.clone())]),
            capability_delegation: Some(vec![VerificationMethodRef::Reference(vm_id)]),
            service: None,
        })
    }
}

impl Default for DidPeerResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DidResolver for DidPeerResolver {
    fn method(&self) -> &str {
        "peer"
    }

    async fn resolve(&self, did: &str) -> Result<DidResolution, DidError> {
        let start = Instant::now();
        let document = Self::synthesise_nuance0(did)?;
        Ok(DidResolution {
            did: did.to_string(),
            document,
            metadata: ResolutionMetadata {
                duration_ms: start.elapsed().as_millis() as u64,
                method: "peer".to_string(),
                cached: false,
                http_status: None,
            },
        })
    }

    async fn validate_for_capability(
        &self,
        _resolution: &DidResolution,
        capability: &DidCapability,
    ) -> Result<(), DidError> {
        match capability {
            DidCapability::AccountIdentity | DidCapability::OrgIdentity => Err(
                DidError::MethodNotPermitted("peer".to_string(), capability.clone()),
            ),
            _ => Ok(()),
        }
    }
}
