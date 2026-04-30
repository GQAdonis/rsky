use appview_core::error::{AppViewError, Result};
use appview_core::types::Did;
use moka::future::Cache;
use rsky_identity::did::did_resolver::DidResolver as RskyDidResolver;
use rsky_identity::types::DidDocument;
use std::sync::Arc;
use std::time::Duration;

pub use rsky_identity::handle::HandleResolver;
pub use rsky_identity::types::DidResolverOpts;
pub use rsky_identity::types::HandleResolverOpts;

#[derive(Clone)]
pub struct DidResolver {
    inner: Arc<tokio::sync::Mutex<RskyDidResolver>>,
    cache: Cache<String, DidDocument>,
}

impl DidResolver {
    pub fn new() -> Self {
        let inner = RskyDidResolver::new(DidResolverOpts {
            timeout: Some(Duration::from_secs(5)),
            plc_url: None, // Use default PLC directory
            did_cache: rsky_identity::types::DidCache::new(
                Some(Duration::from_secs(3600)),  // 1 hour stale TTL
                Some(Duration::from_secs(86400)), // 24 hour max TTL
            ),
        });

        Self {
            inner: Arc::new(tokio::sync::Mutex::new(inner)),
            cache: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(3600))
                .build(),
        }
    }

    pub async fn resolve(&self, did: &Did) -> Result<DidDocument> {
        let did_str = did.as_str();
        if let Some(cached) = self.cache.get(did_str).await {
            return Ok(cached);
        }

        // resolve() takes (did: String, force_refresh: Option<bool>) and returns Result<Option<DidDocument>>
        let mut resolver = self.inner.lock().await;
        let doc_opt = resolver
            .resolve(did_str.to_string(), None)
            .await
            .map_err(|e| AppViewError::Identity(format!("Failed to resolve DID: {}", e)))?;

        let doc =
            doc_opt.ok_or_else(|| AppViewError::Identity(format!("DID not found: {}", did_str)))?;

        self.cache.insert(did_str.to_string(), doc.clone()).await;
        Ok(doc)
    }

    pub async fn resolve_handle(&self, handle: &str) -> Result<Did> {
        let mut handle_resolver = HandleResolver::new(HandleResolverOpts {
            timeout: Some(Duration::from_secs(5)),
            backup_nameservers: None,
        });

        let did_str = handle_resolver
            .resolve(&handle.to_string())
            .await
            .map_err(|e| AppViewError::Identity(format!("Failed to resolve handle: {}", e)))?
            .ok_or_else(|| AppViewError::Identity(format!("Handle not found: {}", handle)))?;

        Ok(Did::new(&did_str))
    }

    pub async fn invalidate(&self, did: &Did) {
        self.cache.invalidate(did.as_str()).await;
    }
}

impl Default for DidResolver {
    fn default() -> Self {
        Self::new()
    }
}
