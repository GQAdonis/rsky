use crate::common::decode_uri_component;
use crate::did::capability::{DidError, DidResolution, ResolutionMetadata};
use crate::did::resolver_trait::DidResolver;
use crate::errors::Error;
use crate::types::DidCache;
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::time::{Duration, Instant};
use url::Url;

pub const DOC_PATH: &str = "/.well-known/did.json";

#[derive(Clone, Debug)]
pub struct DidWebResolver {
    pub timeout: Duration,
    pub cache: Option<DidCache>,
}

impl DidWebResolver {
    pub fn new(timeout: Duration, cache: Option<DidCache>) -> Self {
        Self { timeout, cache }
    }

    pub async fn resolve_no_check(&self, did: String) -> Result<Option<Value>> {
        let parsed_id: String = did.split(":").collect::<Vec<&str>>()[2..].join(":");
        let parts = parsed_id
            .split(":")
            .into_iter()
            .map(|part| decode_uri_component(part))
            .collect::<Result<Vec<String>>>()?;
        let path: String;
        if parts.len() < 1 {
            bail!(Error::PoorlyFormattedDidError(did))
        } else if parts.len() == 1 {
            path = parts[0].clone() + DOC_PATH;
        } else {
            bail!(Error::UnsupportedDidWebPathError(did))
        }

        let mut url = Url::parse(&format!("https://{path}"))?;

        if url.host_str() == Some("localhost") {
            let _ = url.set_scheme("http");
        }

        let client = reqwest::Client::new();
        let response = client
            .get(url.to_string())
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
impl DidResolver for DidWebResolver {
    fn method(&self) -> &str {
        "web"
    }

    async fn resolve(&self, did: &str) -> Result<DidResolution, DidError> {
        let start = Instant::now();
        let raw = self
            .resolve_no_check(did.to_string())
            .await
            .map_err(|e| DidError::NetworkError(e.to_string()))?;

        let val = raw.ok_or_else(|| DidError::NotFound(did.to_string()))?;
        let document =
            serde_json::from_value(val).map_err(|e| DidError::MalformedDocument(e.to_string()))?;

        Ok(DidResolution {
            did: did.to_string(),
            document,
            metadata: ResolutionMetadata {
                duration_ms: start.elapsed().as_millis() as u64,
                method: "web".to_string(),
                cached: false,
                http_status: None,
            },
        })
    }
}
