use crate::did::capability::{DidCapability, DidError, DidResolution, ResolutionMetadata};
use crate::did::resolver_trait::DidResolver;
use crate::types::{DidDocument, VerificationMethod};
use async_trait::async_trait;
use rsky_crypto::did::parse_did_key;
use std::time::Instant;

/// Resolver for `did:key` — derives the DID document entirely from the encoded public key.
///
/// No network calls are made. The document is synthesised from the multibase key material
/// following the did:key spec (https://w3c-ccg.github.io/did-method-key/).
///
/// ## Capability constraints
///
/// `did:key` is **never** permitted as `AccountIdentity` or `OrgIdentity` — it is
/// ephemeral by design and cannot be looked up or updated. Permitted tiers:
/// - `AgentIdentity`  — A2A / UAR agent flows
/// - `SessionIdentity` — ephemeral per-session identity
/// - `CredentialIssuer` — not typical, but not disallowed structurally
pub struct DidKeyResolver;

impl DidKeyResolver {
    pub fn new() -> Self {
        Self
    }

    fn synthesise_document(did: &str) -> Result<DidDocument, DidError> {
        // Verify the key parses before constructing the document.
        parse_did_key(&did.to_string())
            .map_err(|e| DidError::MalformedDocument(format!("did:key parse error: {e}")))?;

        let vm_id = format!("{did}#{}", did.trim_start_matches("did:key:"));
        let vm = VerificationMethod {
            id: vm_id.clone(),
            r#type: "Multikey".to_string(),
            controller: did.to_string(),
            public_key_multibase: Some(did.trim_start_matches("did:key:").to_string()),
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
            authentication: Some(vec![
                crate::types::VerificationMethodRef::Reference(vm_id.clone()),
            ]),
            assertion_method: Some(vec![
                crate::types::VerificationMethodRef::Reference(vm_id.clone()),
            ]),
            capability_invocation: Some(vec![
                crate::types::VerificationMethodRef::Reference(vm_id.clone()),
            ]),
            capability_delegation: Some(vec![
                crate::types::VerificationMethodRef::Reference(vm_id),
            ]),
            service: None,
        })
    }
}

impl Default for DidKeyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DidResolver for DidKeyResolver {
    fn method(&self) -> &str {
        "key"
    }

    async fn resolve(&self, did: &str) -> Result<DidResolution, DidError> {
        let start = Instant::now();
        let document = Self::synthesise_document(did)?;
        Ok(DidResolution {
            did: did.to_string(),
            document,
            metadata: ResolutionMetadata {
                duration_ms: start.elapsed().as_millis() as u64,
                method: "key".to_string(),
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
            DidCapability::AccountIdentity | DidCapability::OrgIdentity => {
                Err(DidError::MethodNotPermitted("key".to_string(), capability.clone()))
            }
            _ => Ok(()),
        }
    }
}
