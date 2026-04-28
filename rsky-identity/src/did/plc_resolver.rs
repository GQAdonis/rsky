use crate::common::encode_uri_component;
use crate::did::capability::{DidError, DidResolution, ResolutionMetadata};
use crate::did::resolver_trait::DidResolver;
use crate::types::DidCache;
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct DidPlcResolver {
    pub plc_url: String,
    pub timeout: Duration,
    pub cache: Option<DidCache>,
}

impl DidPlcResolver {
    pub fn new(plc_url: String, timeout: Duration, cache: Option<DidCache>) -> Self {
        Self {
            plc_url,
            timeout,
            cache,
        }
    }

    pub async fn resolve_no_check(&self, did: String) -> Result<Option<Value>> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{0}/{1}", self.plc_url, encode_uri_component(&did)))
            .timeout(self.timeout)
            .header("Connection", "Keep-Alive")
            .header("Keep-Alive", "timeout=5, max=1000")
            .send()
            .await?;
        let res = &response;
        match res.error_for_status_ref() {
            Ok(_) => Ok(Some(response.json::<Value>().await?)),
            Err(error) if error.status() == Some(reqwest::StatusCode::NOT_FOUND) => Ok(None),
            Err(error) => bail!(error.to_string()),
        }
    }
}

#[async_trait]
impl DidResolver for DidPlcResolver {
    fn method(&self) -> &str {
        "plc"
    }

    async fn resolve(&self, did: &str) -> Result<DidResolution, DidError> {
        let start = Instant::now();
        let raw = self
            .resolve_no_check(did.to_string())
            .await
            .map_err(|e| DidError::NetworkError(e.to_string()))?;

        let val = raw.ok_or_else(|| DidError::NotFound(did.to_string()))?;
        let document = serde_json::from_value(val)
            .map_err(|e| DidError::MalformedDocument(e.to_string()))?;

        Ok(DidResolution {
            did: did.to_string(),
            document,
            metadata: ResolutionMetadata {
                duration_ms: start.elapsed().as_millis() as u64,
                method: "plc".to_string(),
                cached: false,
                http_status: None,
            },
        })
    }
}
