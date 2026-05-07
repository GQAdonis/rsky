use std::io::BufRead;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::time::{Duration, Instant};

use bytes::{Buf, Bytes};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use hashbrown::HashSet;
use lru::LruCache;
use reqwest::Client;
use serde::Deserialize;
use serde_json::value::RawValue;
use thiserror::Error;
use tokio::time::timeout;

use rsky_identity::types::DidDocument;

use crate::PgPool;
use crate::config::{CAPACITY_CACHE, DO_PLC_EXPORT, PLC_EXPORT_INTERVAL};
use crate::validator::event::{DidEndpoint, DidKey};

/// Hot-path interface used by the validator. Returns owned values so the resolver isn't
/// borrowed across the rest of the validation pipeline. Implemented by the production
/// `Resolver` and by test fakes.
pub trait IdentityResolver: Send {
    fn expire(&mut self, did: &str, time: DateTime<Utc>);
    fn resolve_owned(
        &mut self, did: &str,
    ) -> Result<Option<(Option<String>, DidKey)>, ResolverError>;
    fn request_direct(&mut self, did: &str);
    fn poll(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Vec<String>, ResolverError>> + Send;
}

const POLL_TIMEOUT: Duration = Duration::from_micros(10);
const REQ_TIMEOUT: Duration = Duration::from_secs(30);
const TCP_KEEPALIVE: Duration = Duration::from_secs(300);

const PLC_URL: &str = "https://plc.directory";
const PLC_EXPORT: &str = "export?count=1000&after";
const DOC_PATH: &str = ".well-known/did.json";

type RequestFuture = Pin<Box<dyn Future<Output = (Query, reqwest::Result<Bytes>)> + Send>>;

#[derive(Debug)]
enum Query {
    Did(String),
    Export(String),
}

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("size error")]
    SizeError,
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
}

pub struct Resolver {
    cache: LruCache<String, (DidEndpoint, DidKey)>,
    /// Only present when DO_PLC_EXPORT is true.
    /// Tuple: (pool, last_export_instant, after_cursor)
    plc_db: Option<(PgPool, Instant, Option<String>)>,
    client: Client,
    inflight: HashSet<String>,
    futures: FuturesUnordered<RequestFuture>,
}

impl Resolver {
    pub fn new(pool: PgPool) -> Result<Self, ResolverError> {
        #[expect(clippy::unwrap_used)]
        let cache = LruCache::new(NonZeroUsize::new(CAPACITY_CACHE).unwrap());

        let plc_db = if *DO_PLC_EXPORT {
            let now = Instant::now();
            let last = now.checked_sub(PLC_EXPORT_INTERVAL).unwrap_or(now);
            // Determine the most recent created_at already stored so we can resume.
            let after = tokio::task::block_in_place(|| {
                let handle = tokio::runtime::Handle::try_current().map_err(|e| {
                    sqlx::Error::Protocol(format!("no tokio runtime in Resolver::new: {e}"))
                })?;
                handle.block_on(async {
                    use sqlx::Row as _;
                    let row = sqlx::query(
                        "SELECT created_at FROM plc_operations ORDER BY created_at DESC LIMIT 1",
                    )
                    .fetch_optional(&pool)
                    .await?;
                    Ok::<_, sqlx::Error>(row.and_then(|r| {
                        r.try_get::<DateTime<Utc>, _>("created_at").map(|ts| ts.to_rfc3339()).ok()
                    }))
                })
            })?;
            Some((pool, last, after))
        } else {
            None
        };

        let client = Client::builder()
            .user_agent("rsky-relay")
            .timeout(REQ_TIMEOUT)
            .tcp_keepalive(Some(TCP_KEEPALIVE))
            .https_only(true)
            .build()?;
        let inflight = HashSet::new();
        let futures = FuturesUnordered::new();
        Ok(Self { cache, plc_db, client, inflight, futures })
    }

    pub fn expire(&mut self, did: &str, time: DateTime<Utc>) {
        let stale =
            self.plc_db.as_ref().and_then(|(_, _, after)| after.as_deref()).map_or(true, |after| {
                DateTime::parse_from_rfc3339(after).map_or(true, |after| after < time)
            });
        if stale {
            tracing::trace!("expiring did");
            self.cache.pop(did);
            self.request(did);
        }
    }

    pub fn resolve(&mut self, did: &str) -> Result<Option<(Option<&str>, &DidKey)>, ResolverError> {
        // the identity might have expired, so check inflight dids first
        if self.inflight.contains(did) {
            return Ok(None);
        }
        // if let Some(_) = self.cache.get(did) doesn't work because of NLL
        if self.cache.get(did).is_some() {
            return Ok(self.cache.peek_mru().map(|(_, v)| (v.0.as_ref().map(AsRef::as_ref), &v.1)));
        }
        self.request(did);
        Ok(None)
    }

    /// Query the PLC keys table for a cached key/endpoint pair.
    ///
    /// Returns `true` and populates the LRU cache when a row is found.
    pub async fn query_db(&mut self, did: &str) -> Result<bool, ResolverError> {
        let Some((pool, _, _)) = self.plc_db.as_ref() else {
            return Ok(false);
        };
        use sqlx::Row as _;
        let did_owned = did.to_owned();
        let row = sqlx::query(
            "SELECT pds_endpoint AS endpoint, pds_key AS key FROM plc_keys WHERE did = $1",
        )
        .bind(&did_owned)
        .fetch_optional(pool)
        .await?;

        if let Some(r) = row {
            let endpoint: Option<String> = r.try_get("endpoint").unwrap_or(None);
            let key: Option<String> = r.try_get("key").unwrap_or(None);
            if let Some(pair) = parse_key_endpoint(endpoint.as_deref(), key.as_deref()) {
                self.cache.put(did.to_owned(), pair);
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn request(&mut self, did: &str) {
        self.request_inner(did, false);
    }

    /// Force an individual DID lookup from plc.directory, bypassing the export stream.
    /// Used when a hostname mismatch suggests the user may have migrated.
    pub fn request_direct(&mut self, did: &str) {
        self.request_inner(did, true);
    }

    fn request_inner(&mut self, did: &str, force_direct: bool) {
        self.inflight.insert(did.to_owned());
        if let Some(plc) = did.strip_prefix("did:plc:") {
            // Use export stream when available; fall back to direct lookup otherwise.
            let plc = if *DO_PLC_EXPORT && !force_direct { None } else { Some(plc) };
            self.send_req(None, plc);
        } else if let Some(web) = did.strip_prefix("did:web:") {
            let Ok(web) = urlencoding::decode(web) else {
                tracing::debug!(%did, "invalid did");
                return;
            };
            self.send_req(Some(&web), None);
        } else {
            tracing::debug!(%did, "invalid did");
            self.inflight.remove(did);
        }
    }

    fn send_req(&mut self, web: Option<&str>, plc: Option<&str>) {
        let (req, query) = if let Some(web) = web {
            tracing::trace!("fetching did");
            (self.client.get(format!("https://{web}/{DOC_PATH}")), Query::Did(web.to_owned()))
        } else if let Some(plc) = plc {
            tracing::trace!("fetching did");
            (self.client.get(format!("{PLC_URL}/did:plc:{plc}")), Query::Did(plc.to_owned()))
        } else if let Some((_, last, after_slot)) = self.plc_db.as_mut() {
            let Some(after) = after_slot.take() else { return };
            tracing::trace!(%after, "fetching after");
            *last = Instant::now();
            (self.client.get(format!("{PLC_URL}/{PLC_EXPORT}={after}")), Query::Export(after))
        } else {
            return;
        };
        self.futures.push(Box::pin(async move {
            match req.send().await {
                Ok(req) => match req.bytes().await {
                    Ok(bytes) => (query, Ok(bytes)),
                    Err(err) => (query, Err(err)),
                },
                Err(err) => (query, Err(err)),
            }
        }));
    }

    pub async fn poll_inner(&mut self) -> Result<Vec<String>, ResolverError> {
        if let Ok(Some((query, res))) = timeout(POLL_TIMEOUT, self.futures.next()).await {
            match res {
                Ok(bytes) => match query {
                    Query::Did(query) => {
                        if let Some((did, (pds, key))) = parse_did_doc(&bytes) {
                            if query != did[8..] {
                                tracing::warn!(%query, found = %&did[8..], "did query mismatch");
                                return Ok(Vec::new());
                            }
                            self.inflight.remove(&did);
                            self.cache.put(did.clone(), (pds, key));
                            return Ok(vec![did]);
                        }
                    }
                    Query::Export(after) => {
                        let Some((pool, _, after_slot)) = self.plc_db.as_mut() else {
                            return Ok(Vec::new());
                        };
                        *after_slot = Some(after);
                        let mut dids = Vec::new();
                        let mut count = 0;

                        // Collect all PLC documents from the response.
                        let mut docs: Vec<OwnedPlcDocument> = Vec::new();
                        for line in bytes.reader().lines() {
                            count += 1;
                            if let Some(doc) = parse_plc_doc_owned(&line.unwrap_or_default()) {
                                docs.push(doc);
                            }
                        }

                        // Bulk-insert in an explicit transaction.
                        let pool_ref = pool.clone();
                        let mut tx = pool_ref.begin().await?;
                        for doc in &docs {
                            let created_at = doc
                                .created_at
                                .parse::<DateTime<Utc>>()
                                .unwrap_or(DateTime::UNIX_EPOCH);
                            sqlx::query(
                                "INSERT INTO plc_operations \
                                 (cid, did, created_at, nullified, operation) \
                                 VALUES ($1, $2, $3, $4, $5) \
                                 ON CONFLICT (cid) DO NOTHING",
                            )
                            .bind(&doc.cid)
                            .bind(&doc.did)
                            .bind(created_at)
                            .bind(doc.nullified)
                            .bind(doc.operation.as_bytes())
                            .execute(&mut *tx)
                            .await?;
                            *after_slot = Some(doc.created_at.clone());
                            if self.inflight.remove(&doc.did) {
                                dids.push(doc.did.clone());
                            }
                        }
                        tx.commit().await?;

                        if count == 1000 {
                            self.send_req(None, None);
                        } else {
                            // no more plc operations, drain inflight dids
                            dids.extend(
                                self.inflight.extract_if(|did| did.starts_with("did:plc:")),
                            );
                        }
                        return Ok(dids);
                    }
                },
                Err(err) => {
                    tracing::debug!(%err, "fetch error");
                    // Restore the after cursor on export failure so exports can be retried
                    if let Query::Export(after) = query {
                        if let Some((_, _, after_slot)) = self.plc_db.as_mut() {
                            *after_slot = Some(after);
                        }
                    }
                }
            }
        } else if let Some((_, last, _)) = self.plc_db.as_ref() {
            if last.elapsed() > PLC_EXPORT_INTERVAL {
                self.send_req(None, None);
            }
        }
        Ok(Vec::new())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlcDocument<'a> {
    did: String,
    #[serde(borrow)]
    operation: &'a RawValue,
    cid: String,
    nullified: bool,
    created_at: String,
}

/// Owned variant of `PlcDocument` used after the line-parsing phase.
struct OwnedPlcDocument {
    did: String,
    operation: String,
    cid: String,
    nullified: bool,
    created_at: String,
}

impl IdentityResolver for Resolver {
    #[inline]
    fn expire(&mut self, did: &str, time: DateTime<Utc>) {
        Self::expire(self, did, time);
    }

    #[inline]
    fn resolve_owned(
        &mut self, did: &str,
    ) -> Result<Option<(Option<String>, DidKey)>, ResolverError> {
        match self.resolve(did)? {
            Some((pds, key)) => Ok(Some((pds.map(str::to_owned), *key))),
            None => Ok(None),
        }
    }

    #[inline]
    fn request_direct(&mut self, did: &str) {
        Self::request_direct(self, did);
    }

    #[inline]
    fn poll(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Vec<String>, ResolverError>> + Send {
        self.poll_inner()
    }
}

fn parse_plc_doc_owned(input: &str) -> Option<OwnedPlcDocument> {
    match serde_json::from_slice::<PlcDocument<'_>>(input.as_bytes()) {
        Ok(doc) => Some(OwnedPlcDocument {
            did: doc.did,
            cid: doc.cid,
            nullified: doc.nullified,
            created_at: doc.created_at,
            operation: doc.operation.get().to_owned(),
        }),
        Err(err) => {
            tracing::debug!(%input, %err, "parse error");
            None
        }
    }
}

fn parse_did_doc(input: &Bytes) -> Option<(String, (DidEndpoint, DidKey))> {
    match serde_json::from_slice::<DidDocument>(input) {
        Ok(doc) => {
            let endpoint =
                if cfg!(feature = "labeler") { "#atproto_labeler" } else { "#atproto_pds" };
            let key = if cfg!(feature = "labeler") { "#atproto_label" } else { "#atproto" };
            let endpoint = doc
                .service
                .as_ref()
                .and_then(|services| services.iter().find(|service| service.id.ends_with(endpoint)))
                .and_then(|service| service.service_endpoint.as_str());
            let key = doc
                .verification_method
                .as_ref()
                .and_then(|methods| methods.iter().find(|method| method.id.ends_with(key)))
                .and_then(|method| method.public_key_multibase.as_deref());
            Some((doc.id, parse_key_endpoint(endpoint, key)?))
        }
        Err(err) => {
            tracing::debug!(?input, %err, "parse error");
            None
        }
    }
}

fn parse_key_endpoint(endpoint: Option<&str>, key: Option<&str>) -> Option<(DidEndpoint, DidKey)> {
    // key can be null for legacy doc formats
    if let Some(key) = key {
        match multibase::decode(key.trim_start_matches("did:key:")) {
            Ok((_, vec)) => match vec.try_into() {
                Ok(key) => {
                    // endpoint can be null for legacy doc formats
                    let pds = endpoint.and_then(|endpoint| {
                        Some(endpoint.strip_prefix("https://")?.trim_end_matches('/').into())
                    });
                    return Some((pds, key));
                }
                Err(_) => {
                    tracing::debug!(%key, "invalid key length");
                }
            },
            Err(err) => {
                tracing::debug!(%key, %err, "invalid key");
            }
        }
    }
    None
}

#[cfg(test)]
pub(crate) type ResolveResult = Result<Option<(Option<String>, DidKey)>, ResolverError>;
#[cfg(test)]
pub(crate) type PollResult = Result<Vec<String>, ResolverError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    /// Minimal scriptable `IdentityResolver` fake for unit tests. The validator hot path
    /// only exercises `resolve_owned` + `request_direct` + `expire` + `poll`.
    pub struct FakeResolver {
        pub script: VecDeque<ResolveResult>,
        pub direct_requests: Vec<String>,
        pub expirations: Vec<(String, DateTime<Utc>)>,
        pub polls: VecDeque<PollResult>,
    }

    impl FakeResolver {
        pub fn new() -> Self {
            Self {
                script: VecDeque::new(),
                direct_requests: Vec::new(),
                expirations: Vec::new(),
                polls: VecDeque::new(),
            }
        }
    }

    impl IdentityResolver for FakeResolver {
        fn expire(&mut self, did: &str, time: DateTime<Utc>) {
            self.expirations.push((did.to_owned(), time));
        }

        fn resolve_owned(&mut self, _did: &str) -> ResolveResult {
            self.script.pop_front().unwrap_or(Ok(None))
        }

        fn request_direct(&mut self, did: &str) {
            self.direct_requests.push(did.to_owned());
        }

        fn poll(&mut self) -> impl std::future::Future<Output = PollResult> + Send {
            let next = self.polls.pop_front().unwrap_or_else(|| Ok(Vec::new()));
            std::future::ready(next)
        }
    }

    #[test]
    fn fake_resolver_resolve_owned_returns_scripted_value() {
        let mut fake = FakeResolver::new();
        fake.script.push_back(Ok(Some((Some("pds.example".to_owned()), [7u8; 35]))));
        fake.script.push_back(Ok(None));
        let r1 = fake.resolve_owned("did:plc:a").unwrap();
        let r2 = fake.resolve_owned("did:plc:b").unwrap();
        assert_eq!(r1, Some((Some("pds.example".to_owned()), [7u8; 35])));
        assert_eq!(r2, None);
    }

    #[test]
    fn fake_resolver_request_direct_records_did() {
        let mut fake = FakeResolver::new();
        fake.request_direct("did:plc:a");
        fake.request_direct("did:plc:b");
        assert_eq!(fake.direct_requests, vec!["did:plc:a".to_owned(), "did:plc:b".to_owned()]);
    }

    #[test]
    fn fake_resolver_expire_records_did_and_time() {
        let mut fake = FakeResolver::new();
        let t = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z").unwrap().with_timezone(&Utc);
        fake.expire("did:plc:a", t);
        assert_eq!(fake.expirations, vec![("did:plc:a".to_owned(), t)]);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn fake_resolver_poll_returns_scripted_value() {
        let mut fake = FakeResolver::new();
        fake.polls.push_back(Ok(vec!["did:plc:a".to_owned()]));
        fake.polls.push_back(Ok(Vec::new()));
        assert_eq!(fake.poll().await.unwrap(), vec!["did:plc:a".to_owned()]);
        assert_eq!(fake.poll().await.unwrap(), Vec::<String>::new());
    }

    #[test]
    fn parse_key_endpoint_with_null_key_returns_none() {
        assert!(parse_key_endpoint(None, None).is_none());
        assert!(parse_key_endpoint(Some("https://pds.example"), None).is_none());
    }

    #[test]
    fn parse_key_endpoint_strips_https_prefix_and_trailing_slash() {
        let valid_key = "did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme";
        let pair = parse_key_endpoint(Some("https://pds.example.com/"), Some(valid_key));
        match pair {
            Some((Some(pds), _key)) => assert_eq!(pds.as_ref(), "pds.example.com"),
            other => panic!("expected Some endpoint, got {other:?}"),
        }
    }
}
