use crate::did::capability::{DidCapability, DidError, DidResolution};
use async_trait::async_trait;

/// Async DID resolver trait. Implementations handle a single DID method.
#[async_trait]
pub trait DidResolver: Send + Sync {
    /// The method string this resolver handles, e.g. `"plc"` or `"web"`.
    fn method(&self) -> &str;

    /// Resolve a DID to a `DidResolution` (document + metadata).
    ///
    /// Returns `Err(DidError::NotFound)` if the DID does not exist.
    async fn resolve(&self, did: &str) -> Result<DidResolution, DidError>;

    /// Validate that a resolved document meets the requirements for a given capability tier.
    ///
    /// The default implementation delegates to the profile validator in `profile.rs`.
    /// Method-specific resolvers may override to add extra checks.
    async fn validate_for_capability(
        &self,
        resolution: &DidResolution,
        capability: &DidCapability,
    ) -> Result<(), DidError> {
        use crate::did::profile::AtprotoAccountDidProfile;
        AtprotoAccountDidProfile::validate(resolution, capability)
    }
}
