use crate::did::capability::{DidCapability, DidError, DidResolution, ResolutionMetadata};
use crate::did::key_resolver::DidKeyResolver;
use crate::did::peer_resolver::DidPeerResolver;
use crate::did::plc_resolver::DidPlcResolver;
use crate::did::resolver_trait::DidResolver;
use crate::did::web_resolver::DidWebResolver;
use crate::types::{DidCache, DidDocument};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Dispatches DID resolution to the resolver registered for a given DID method.
///
/// Wraps the existing per-method resolvers behind the `DidResolver` trait so that
/// new DID methods can be added without modifying an enum.
pub struct CompositeDidResolver {
    resolvers: HashMap<String, Box<dyn DidResolver>>,
    cache: Option<DidCache>,
    /// Maximum resolution timeout enforced at this layer (milliseconds).
    timeout_ms: u64,
}

impl CompositeDidResolver {
    pub fn new(timeout_ms: u64, cache: Option<DidCache>) -> Self {
        Self {
            resolvers: HashMap::new(),
            cache,
            timeout_ms,
        }
    }

    /// Register a resolver for the DID method it reports via `DidResolver::method()`.
    pub fn register(&mut self, resolver: Box<dyn DidResolver>) {
        self.resolvers.insert(resolver.method().to_string(), resolver);
    }

    /// Build a resolver with all four built-in methods pre-registered.
    ///
    /// - `did:plc` — AT Protocol native, mutable, requires PLC directory URL
    /// - `did:web` — organisational / domain-bound identity
    /// - `did:key` — ephemeral offline key; **not** permitted for AccountIdentity
    /// - `did:peer` — ephemeral peer channel; **not** permitted for AccountIdentity
    pub fn with_all_methods(
        plc_url: impl Into<String>,
        timeout_ms: u64,
        cache: Option<DidCache>,
    ) -> Self {
        let timeout = Duration::from_millis(timeout_ms);
        let mut resolver = Self::new(timeout_ms, cache);
        resolver.register(Box::new(DidPlcResolver::new(plc_url.into(), timeout, None)));
        resolver.register(Box::new(DidWebResolver::new(timeout, None)));
        resolver.register(Box::new(DidKeyResolver::new()));
        resolver.register(Box::new(DidPeerResolver::new()));
        resolver
    }

    fn parse_method(did: &str) -> Result<&str, DidError> {
        let parts: Vec<&str> = did.splitn(3, ':').collect();
        if parts.len() < 3 || parts[0] != "did" {
            return Err(DidError::MalformedDocument(format!(
                "not a valid DID: {did}"
            )));
        }
        Ok(parts[1])
    }

    /// Resolve a DID, honouring the cache if available.
    pub async fn resolve(&self, did: &str) -> Result<DidResolution, DidError> {
        let method = Self::parse_method(did)?;

        // Cache check.
        if let Some(ref cache) = self.cache {
            if let Ok(Some(hit)) = cache.check_cache(did.to_string()) {
                if !hit.expired {
                    return Ok(self.hit_to_resolution(did, method, hit.doc));
                }
            }
        }

        let resolver = self
            .resolvers
            .get(method)
            .ok_or_else(|| DidError::UnsupportedMethod(method.to_string()))?;

        let start = Instant::now();
        let resolution = tokio::time::timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            resolver.resolve(did),
        )
        .await
        .map_err(|_| DidError::Timeout(self.timeout_ms, did.to_string()))?
        .map(|mut r| {
            r.metadata.duration_ms = start.elapsed().as_millis() as u64;
            r
        })?;

        // Populate cache.
        if let Some(ref mut cache) = self.cache.clone() {
            let _ = cache
                .cache_did(did.to_string(), resolution.document.clone())
                .await;
        }

        Ok(resolution)
    }

    /// Resolve and then validate against a capability tier.
    pub async fn resolve_for_capability(
        &self,
        did: &str,
        capability: &DidCapability,
    ) -> Result<DidResolution, DidError> {
        let method = Self::parse_method(did)?;

        // Method permitted for this capability?
        if !capability.permitted_methods().contains(&method) {
            return Err(DidError::MethodNotPermitted(
                method.to_string(),
                capability.clone(),
            ));
        }

        let resolution = self.resolve(did).await?;

        let resolver = self
            .resolvers
            .get(method)
            .ok_or_else(|| DidError::UnsupportedMethod(method.to_string()))?;
        resolver
            .validate_for_capability(&resolution, capability)
            .await?;

        Ok(resolution)
    }

    fn hit_to_resolution(&self, did: &str, method: &str, doc: DidDocument) -> DidResolution {
        DidResolution {
            did: did.to_string(),
            document: doc,
            metadata: ResolutionMetadata {
                duration_ms: 0,
                method: method.to_string(),
                cached: true,
                http_status: None,
            },
        }
    }
}
