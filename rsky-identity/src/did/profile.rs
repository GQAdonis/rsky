use crate::did::capability::{DidCapability, DidError, DidResolution};

/// ATProto DID Profile validator.
///
/// Applies constraints from the AT Protocol specification that go beyond bare DID Core validity.
/// These constraints exist to ensure interoperability within the ATProto federation network.
pub struct AtprotoAccountDidProfile;

impl AtprotoAccountDidProfile {
    /// Validate a `DidResolution` against the requirements for `capability`.
    pub fn validate(
        resolution: &DidResolution,
        capability: &DidCapability,
    ) -> Result<(), DidError> {
        let method = &resolution.metadata.method;

        // Check that the DID method is permitted for this capability tier.
        if !capability
            .permitted_methods()
            .contains(&method.as_str())
        {
            return Err(DidError::MethodNotPermitted(
                method.clone(),
                capability.clone(),
            ));
        }

        // AccountIdentity requires stricter ATProto-specific checks.
        if matches!(capability, DidCapability::AccountIdentity) {
            Self::validate_account_identity(resolution)?;
        }

        Ok(())
    }

    fn validate_account_identity(resolution: &DidResolution) -> Result<(), DidError> {
        let doc = &resolution.document;

        // Must resolve within 500 ms (soft constraint — enforced at resolve time via timeout).
        // Hard cap: resolution must have returned a duration value at all.
        if resolution.metadata.duration_ms == 0 && !resolution.metadata.cached {
            return Err(DidError::ProfileConstraintViolated(
                "resolution duration unknown (not cached and duration is 0)".into(),
            ));
        }

        // Must have at least one service of type "AtprotoPersonalDataServer".
        let has_pds_service = doc.service.as_ref().map_or(false, |services| {
            services
                .iter()
                .any(|s| s.r#type == "AtprotoPersonalDataServer")
        });
        if !has_pds_service {
            return Err(DidError::ProfileConstraintViolated(
                "document must include a service of type AtprotoPersonalDataServer".into(),
            ));
        }

        // Must have at least one verificationMethod.
        if doc
            .verification_method
            .as_ref()
            .map_or(true, |v| v.is_empty())
        {
            return Err(DidError::ProfileConstraintViolated(
                "document must include at least one verificationMethod".into(),
            ));
        }

        // At least one verificationMethod must have a publicKeyMultibase or publicKeyJwk.
        let has_key = doc.verification_method.as_ref().map_or(false, |vms| {
            vms.iter()
                .any(|vm| vm.public_key_multibase.is_some() || vm.public_key_jwk.is_some())
        });
        if !has_key {
            return Err(DidError::ProfileConstraintViolated(
                "at least one verificationMethod must carry a public key".into(),
            ));
        }

        // Must have an alsoKnownAs entry with a valid handle URI (at:// or https://).
        let has_aka = doc.also_known_as.as_ref().map_or(false, |akas| {
            akas.iter().any(|a| a.starts_with("at://") || a.starts_with("https://"))
        });
        if !has_aka {
            return Err(DidError::ProfileConstraintViolated(
                "document must have an alsoKnownAs entry for the account handle".into(),
            ));
        }

        Ok(())
    }
}
