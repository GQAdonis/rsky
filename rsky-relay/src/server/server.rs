use std::fs::File;
use std::io::{self, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use httparse::{EMPTY_HEADER, Status};
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use thiserror::Error;
use url::Url;

use crate::PgPool;
use crate::SHUTDOWN;
use crate::config::{ADMIN_PASSWORD, HOSTS_INTERVAL, PORT};
#[cfg(not(feature = "labeler"))]
use crate::config::{HOSTS_MIN_ACCOUNTS, HOSTS_RELAYS};
use crate::crawler::{RequestCrawl, RequestCrawlSender};
#[cfg(not(feature = "labeler"))]
use crate::metrics;
use crate::publisher::{MaybeTlsStream, SubscribeRepos, SubscribeReposSender};
use crate::server::types::{BannedHost, ListBans};
#[cfg(not(feature = "labeler"))]
use crate::server::types::{GetHostStatus, Host, HostStatus, ListHosts};

#[cfg(not(feature = "labeler"))]
pub trait HostListFetcher {
    fn fetch_page(&self, cursor: Option<&str>) -> Result<ListHosts>;
}

#[cfg(not(feature = "labeler"))]
struct ReqwestHostListFetcher {
    client: reqwest::blocking::Client,
    base_url: String,
}

#[cfg(not(feature = "labeler"))]
impl HostListFetcher for ReqwestHostListFetcher {
    fn fetch_page(&self, cursor: Option<&str>) -> Result<ListHosts> {
        let mut params: Vec<(&str, &str)> = vec![("limit", "1000")];
        if let Some(c) = cursor {
            params.push(("cursor", c));
        }
        let url = Url::parse_with_params(&self.base_url, params)?;
        Ok(self.client.get(url).send()?.json()?)
    }
}

#[cfg(not(feature = "labeler"))]
pub fn fetch_page_with_retry<F: HostListFetcher + ?Sized>(
    fetcher: &F, cursor: Option<&str>, sleep: impl Fn(Duration),
) -> Result<ListHosts> {
    let mut delay = Duration::from_secs(1);
    let mut last: Option<color_eyre::Report> = None;
    for attempt in 0..3 {
        match fetcher.fetch_page(cursor) {
            Ok(page) => return Ok(page),
            Err(err) => {
                tracing::warn!(%err, attempt, "listHosts page fetch failed; retrying");
                last = Some(err);
                if attempt < 2 {
                    sleep(delay);
                    delay = delay.saturating_mul(2);
                }
            }
        }
    }
    Err(last.unwrap_or_else(|| eyre!("listHosts retries exhausted with no error")))
}

const SLEEP: Duration = Duration::from_millis(10);

#[cfg(not(feature = "labeler"))]
const PATH_LIST_HOSTS: &str = "/xrpc/com.atproto.sync.listHosts";

#[cfg(not(feature = "labeler"))]
const PATH_HOST_STATUS: &str = "/xrpc/com.atproto.sync.getHostStatus";

const PATH_SUBSCRIBE: &str = if cfg!(feature = "labeler") {
    "/xrpc/com.atproto.label.subscribeLabels"
} else {
    "/xrpc/com.atproto.sync.subscribeRepos"
};
const PATH_REQUEST_CRAWL: &str = if cfg!(feature = "labeler") {
    "/xrpc/com.atproto.label.requestCrawl"
} else {
    "/xrpc/com.atproto.sync.requestCrawl"
};

const PATH_ADMIN_BAN: &str = "/admin/pds/ban";
const PATH_ADMIN_UNBAN: &str = "/admin/pds/unban";
const PATH_ADMIN_LIST_BANS: &str = "/admin/pds/listBans";

const INDEX_ASCII: &str = r"
    .------..------..------..------.
    |R.--. ||S.--. ||K.--. ||Y.--. |
    | :(): || :/\: || :/\: || (\/) |
    | ()() || :\/: || :\/: || :\/: |
    | '--'R|| '--'S|| '--'K|| '--'Y|
    `------'`------'`------'`------'
    .------..------..------..------..------.
    |R.--. ||E.--. ||L.--. ||A.--. ||Y.--. |
    | :(): || (\/) || :/\: || (\/) || (\/) |
    | ()() || :\/: || (__) || :\/: || :\/: |
    | '--'R|| '--'E|| '--'L|| '--'A|| '--'Y|
    `------'`------'`------'`------'`------'

 This is an atproto relay instance running the
 'rsky-relay' codebase [https://github.com/blacksky-algorithms/rsky]

 The firehose WebSocket path is at:  /xrpc/com.atproto.sync.subscribeRepos
";

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("rustls error: {0}")]
    Rustls(#[from] rustls::Error),
    #[error("rtrb error: {0}")]
    PushError(#[from] rtrb::PushError<RequestCrawl>),
    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug)]
struct ErrorOnDropTcpStream(Option<MaybeTlsStream<TcpStream>>);

impl Drop for ErrorOnDropTcpStream {
    #[cold]
    fn drop(&mut self) {
        let Some(mut stream) = self.0.take() else {
            return;
        };
        let _err = stream.write_all(b"HTTP/1.1 400 Bad Request\n");
        let _err = stream.flush();
        let _err = stream.shutdown();
    }
}

fn write_response(stream: &mut ErrorOnDropTcpStream, status: &str, body: &str) -> Result<()> {
    let response = format!(
        "HTTP/1.1 {status}\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        body.len()
    );
    #[expect(clippy::unwrap_used)]
    let mut s = stream.0.take().unwrap();
    s.write_all(response.as_bytes())?;
    s.flush()?;
    s.shutdown()?;
    Ok(())
}

/// Execute an async sqlx future from a blocking OS thread using a dedicated runtime.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the query fails.
fn block_on_db<F, T>(rt: &tokio::runtime::Runtime, fut: F) -> Result<T, sqlx::Error>
where
    F: std::future::Future<Output = Result<T, sqlx::Error>> + Send,
{
    rt.block_on(fut)
}

pub struct Server {
    listener: TcpListener,
    tls_config: Option<Arc<ServerConfig>>,
    base_url: Url,
    buf: Vec<u8>,
    last: Instant,
    pool: PgPool,
    request_crawl_tx: RequestCrawlSender,
    subscribe_repos_tx: SubscribeReposSender,
    /// Dedicated single-threaded Tokio runtime for DB queries issued from
    /// the blocking OS thread that runs the server accept loop.
    rt: tokio::runtime::Runtime,
}

impl Server {
    pub fn new(
        ssl_configs: Option<(PathBuf, PathBuf)>, request_crawl_tx: RequestCrawlSender,
        subscribe_repos_tx: SubscribeReposSender, pool: PgPool,
    ) -> Result<Self, ServerError> {
        let tls_config = if let Some((certs, private_key)) = ssl_configs {
            let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(certs)?))
                .collect::<Result<Vec<_>, _>>()?;
            #[expect(clippy::expect_used)]
            let private_key =
                rustls_pemfile::private_key(&mut BufReader::new(&mut File::open(private_key)?))?
                    .expect("expected private key");
            let tls_config = rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, private_key)?;
            Some(Arc::new(tls_config))
        } else {
            None
        };

        let listener = TcpListener::bind(format!("0.0.0.0:{PORT}"))?;
        listener.set_nonblocking(true)?;
        let base_url = Url::parse("http://example.com")?;
        let now = Instant::now();
        let last = now.checked_sub(HOSTS_INTERVAL).unwrap_or(now);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| ServerError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        Ok(Self {
            listener,
            tls_config,
            base_url,
            buf: vec![0; 1024],
            last,
            pool,
            request_crawl_tx,
            subscribe_repos_tx,
            rt,
        })
    }

    pub fn run(mut self) -> Result<(), ServerError> {
        while self.update()? {
            thread::sleep(SLEEP);
        }
        Ok(())
    }

    fn update(&mut self) -> Result<bool, ServerError> {
        if SHUTDOWN.load(Ordering::Relaxed) {
            tracing::info!("shutting down server");
            return Ok(false);
        }

        if self.last.elapsed() > HOSTS_INTERVAL {
            if let Err(err) = self.query_hosts() {
                tracing::warn!(%err, "unable to query hosts");
            }
            self.last = Instant::now();
        }

        match self.listener.accept() {
            Ok((mut stream, addr)) => {
                tracing::trace!(%addr, "received request");
                let stream = if let Some(tls_config) = self.tls_config.clone() {
                    let mut conn = ServerConnection::new(tls_config)?;
                    if let Err(err) = conn.complete_io(&mut stream) {
                        tracing::info!(%addr, %err, "tls handshake error");
                    }
                    let stream = StreamOwned::new(conn, stream);
                    MaybeTlsStream::Rustls(stream)
                } else {
                    MaybeTlsStream::Plain(stream)
                };
                if let Err(err) = self.handle_stream(ErrorOnDropTcpStream(Some(stream)), addr) {
                    tracing::info!(%addr, %err, "invalid request");
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                return Ok(true);
            }
            Err(e) => Err(e)?,
        }

        Ok(true)
    }

    fn handle_stream(&mut self, mut stream: ErrorOnDropTcpStream, addr: SocketAddr) -> Result<()> {
        // only peek to allow tungstenite to complete the handshake
        #[expect(clippy::unwrap_used)]
        let len = stream.0.as_mut().unwrap().peek(&mut self.buf)?;
        let mut headers = [EMPTY_HEADER; 32];
        let mut parser = httparse::Request::new(&mut headers);
        // try parsing as an HTTP request
        let res = parser.parse(&self.buf)?;
        let method = parser.method.ok_or_else(|| eyre!("method missing"))?;
        let path = parser.path.ok_or_else(|| eyre!("path missing"))?;
        // Extract admin auth before the match block so parser's borrow on
        // self.buf is released by NLL before &mut self methods in match arms.
        let is_admin_authed = check_admin_auth(parser.headers);
        let url = Url::options().base_url(Some(&self.base_url)).parse(path)?;

        match (method, url.path()) {
            ("GET", "/_health") => write_response(&mut stream, "200 OK", "ok"),
            ("GET", "/") => write_response(&mut stream, "200 OK", INDEX_ASCII),
            #[cfg(not(feature = "labeler"))]
            ("GET", PATH_LIST_HOSTS) => {
                let (status, body) = match self.list_hosts(&url) {
                    Ok(hosts) => ("200 OK", serde_json::to_string(&hosts)?),
                    Err(e) => {
                        let error = serde_json::json!({
                            "error": "BadRequest",
                            "message": e.to_string(),
                        });
                        ("400 Bad Request", serde_json::to_string(&error)?)
                    }
                };
                write_response(&mut stream, status, &body)
            }
            #[cfg(not(feature = "labeler"))]
            ("GET", PATH_HOST_STATUS) => {
                let (status, body) = match self.host_status(&url) {
                    Ok(hosts) => ("200 OK", serde_json::to_string(&hosts)?),
                    Err(e) => {
                        let error = serde_json::json!({
                            "error": "BadRequest",
                            "message": e.to_string(),
                        });
                        ("400 Bad Request", serde_json::to_string(&error)?)
                    }
                };
                write_response(&mut stream, status, &body)
            }
            ("GET", PATH_SUBSCRIBE) => {
                let mut cursor = None;
                for (key, value) in url.query_pairs() {
                    if key == "cursor" {
                        cursor = u64::from_str(&value).ok();
                    }
                }
                self.subscribe_repos_tx.push(SubscribeRepos {
                    addr,
                    #[expect(clippy::unwrap_used)]
                    stream: stream.0.take().unwrap(),
                    cursor: cursor.map(Into::into),
                })?;
                Ok(())
            }
            ("POST", PATH_REQUEST_CRAWL) => {
                if let Status::Complete(offset) = res {
                    // Peek may not have captured the full body if it arrived in a later
                    // TCP segment. Read Content-Length from headers and ensure we have
                    // all body bytes before deserializing.
                    let content_length: usize = parser
                        .headers
                        .iter()
                        .find(|h| h.name.eq_ignore_ascii_case("content-length"))
                        .and_then(|h| std::str::from_utf8(h.value).ok())
                        .and_then(|s| s.trim().parse().ok())
                        .unwrap_or(0);
                    // Consume the peeked bytes so we own the socket data.
                    #[expect(clippy::unwrap_used)]
                    let tcp = stream.0.as_mut().unwrap();
                    let _ = tcp.read(&mut self.buf);
                    // If body is larger than what we have, read the remainder.
                    let body_have = len.saturating_sub(offset);
                    let body_need = content_length.saturating_sub(body_have);
                    let body: Vec<u8> = if body_need == 0 {
                        self.buf[offset..offset + body_have].to_vec()
                    } else {
                        let mut extra = vec![0u8; body_need];
                        #[expect(clippy::unwrap_used)]
                        let tcp = stream.0.as_mut().unwrap();
                        let _ = tcp.read_exact(&mut extra);
                        let mut combined = self.buf[offset..offset + body_have].to_vec();
                        combined.extend_from_slice(&extra);
                        combined
                    };
                    if let Ok(request_crawl) =
                        serde_json::from_slice::<RequestCrawl>(&body)
                    {
                        if self.is_host_banned(&request_crawl.hostname) {
                            tracing::info!(host = %request_crawl.hostname, "rejecting requestCrawl for banned host");
                            return write_response(
                                &mut stream,
                                "403 Forbidden",
                                "{\"error\":\"Forbidden\",\"message\":\"host is banned\"}",
                            );
                        }
                        self.request_crawl_tx.push(request_crawl)?;
                        return write_response(&mut stream, "200 OK", "");
                    }
                }
                write_response(
                    &mut stream,
                    "400 Bad Request",
                    "{\"error\":\"InvalidRequest\",\"message\":\"invalid or missing hostname\"}",
                )
            }
            ("POST", PATH_ADMIN_BAN | PATH_ADMIN_UNBAN) | ("GET", PATH_ADMIN_LIST_BANS) => {
                self.handle_admin(&mut stream, url.path(), &url, is_admin_authed)
            }
            _ => write_response(
                &mut stream,
                "404 Not Found",
                "{\"error\":\"NotFound\",\"message\":\"endpoint not found\"}",
            ),
        }
    }

    fn handle_admin(
        &self, stream: &mut ErrorOnDropTcpStream, path: &str, url: &Url, is_admin_authed: bool,
    ) -> Result<()> {
        if !is_admin_authed {
            return write_response(
                stream,
                "401 Unauthorized",
                "{\"error\":\"Unauthorized\",\"message\":\"invalid or missing auth\"}",
            );
        }
        match path {
            PATH_ADMIN_BAN | PATH_ADMIN_UNBAN => {
                let Some(hostname) = Self::get_query_param(url, "host") else {
                    return write_response(
                        stream,
                        "400 Bad Request",
                        "{\"error\":\"BadRequest\",\"message\":\"host parameter is required\"}",
                    );
                };
                let is_ban = path == PATH_ADMIN_BAN;
                let result =
                    if is_ban { self.ban_host(&hostname) } else { self.unban_host(&hostname) };
                let (status, body) = match result {
                    Ok(()) => {
                        let body = serde_json::json!({"host": hostname, "banned": is_ban});
                        ("200 OK", serde_json::to_string(&body)?)
                    }
                    Err(e) => {
                        let body =
                            serde_json::json!({"error": "InternalError", "message": e.to_string()});
                        ("500 Internal Server Error", serde_json::to_string(&body)?)
                    }
                };
                write_response(stream, status, &body)
            }
            PATH_ADMIN_LIST_BANS => {
                let (status, body) = match self.list_bans() {
                    Ok(bans) => ("200 OK", serde_json::to_string(&bans)?),
                    Err(e) => {
                        let body =
                            serde_json::json!({"error": "InternalError", "message": e.to_string()});
                        ("500 Internal Server Error", serde_json::to_string(&body)?)
                    }
                };
                write_response(stream, status, &body)
            }
            _ => write_response(
                stream,
                "404 Not Found",
                "{\"error\":\"NotFound\",\"message\":\"endpoint not found\"}",
            ),
        }
    }

    /// List hosts with keyset pagination using `host` (alphabetical) as the cursor.
    ///
    /// The cursor is the last hostname from the previous page. On the first call it is `None`.
    #[cfg(not(feature = "labeler"))]
    fn list_hosts(&self, url: &Url) -> Result<ListHosts> {
        let mut limit: i64 = 200;
        let mut cursor: Option<String> = None;

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "limit" => match value.parse::<i64>() {
                    Ok(l @ 1..=1000) => limit = l,
                    _ => {
                        return Err(eyre!("limit parameter invalid or out of range: {value}"));
                    }
                },
                "cursor" => cursor = Some(value.into_owned()),
                _ => (),
            }
        }

        use sqlx::Row as _;
        let pool = self.pool.clone();
        let banned_set = block_on_db(&self.rt, async move {
            let rows = sqlx::query("SELECT host FROM banned_hosts").fetch_all(&pool).await?;
            Ok::<_, sqlx::Error>(
                rows.into_iter()
                    .filter_map(|r| r.try_get::<String, _>("host").ok())
                    .collect::<hashbrown::HashSet<String>>(),
            )
        })
        .map_err(|e| eyre!("{e}"))?;

        let pool = self.pool.clone();
        let raw_rows = block_on_db(&self.rt, async move {
            if let Some(ref after) = cursor {
                sqlx::query("SELECT host, cursor FROM hosts WHERE host > $1 ORDER BY host LIMIT $2")
                    .bind(after)
                    .bind(limit)
                    .fetch_all(&pool)
                    .await
            } else {
                sqlx::query("SELECT host, cursor FROM hosts ORDER BY host LIMIT $1")
                    .bind(limit)
                    .fetch_all(&pool)
                    .await
            }
        })
        .map_err(|e| eyre!("{e}"))?;

        let next_cursor = raw_rows.last().and_then(|r| r.try_get::<String, _>("host").ok());

        let hosts = raw_rows
            .into_iter()
            .map(|row| {
                let hostname: String = row.try_get("host").unwrap_or_default();
                let seq: i64 = row.try_get("cursor").unwrap_or(0);
                let status = if banned_set.contains(&hostname) {
                    HostStatus::Banned
                } else {
                    HostStatus::Active
                };
                #[expect(clippy::cast_sign_loss)]
                Host { account_count: 0, seq: seq as u64, hostname, status }
            })
            .collect();

        Ok(ListHosts { cursor: next_cursor, hosts })
    }

    #[cfg(not(feature = "labeler"))]
    fn host_status(&self, url: &Url) -> Result<GetHostStatus> {
        let mut hostname = None;
        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "hostname" => hostname = Some(value.into_owned()),
                _ => (),
            }
        }
        let hostname = hostname.ok_or_else(|| eyre!("hostname param is required"))?;
        let is_banned = self.is_host_banned(&hostname);

        use sqlx::Row as _;
        let pool = self.pool.clone();
        let hn = hostname.clone();
        let row = block_on_db(&self.rt, async move {
            sqlx::query("SELECT cursor FROM hosts WHERE host = $1")
                .bind(&hn)
                .fetch_optional(&pool)
                .await
        })
        .map_err(|e| eyre!("{e}"))?
        .ok_or_else(|| eyre!("hostname {hostname:?} not found"))?;

        let seq: i64 = row.try_get("cursor").unwrap_or(0);
        #[expect(clippy::cast_sign_loss)]
        Ok(GetHostStatus {
            hostname,
            seq: seq as u64,
            status: if is_banned { HostStatus::Banned } else { HostStatus::Active },
        })
    }

    #[cfg(not(feature = "labeler"))]
    fn query_hosts(&mut self) -> Result<()> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("rsky-relay")
            .https_only(true)
            .build()?;
        let mut seen: hashbrown::HashSet<String> = hashbrown::HashSet::new();
        for upstream in HOSTS_RELAYS.iter() {
            let fetcher = ReqwestHostListFetcher {
                client: client.clone(),
                base_url: format!("https://{upstream}{PATH_LIST_HOSTS}"),
            };
            if let Err(err) =
                self.query_hosts_with_fetcher(&fetcher, thread::sleep, &mut seen, upstream)
            {
                tracing::warn!(%err, %upstream, "discovery upstream failed entirely");
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "labeler"))]
    fn query_hosts_with_fetcher<F: HostListFetcher + ?Sized>(
        &mut self, fetcher: &F, sleep: impl Fn(Duration) + Copy,
        seen: &mut hashbrown::HashSet<String>, upstream: &str,
    ) -> Result<()> {
        let mut cursor: Option<String> = None;
        let mut total_seen: usize = 0;
        let mut total_added: usize = 0;
        let mut had_failure = false;
        loop {
            let page = match fetch_page_with_retry(fetcher, cursor.as_deref(), sleep) {
                Ok(page) => page,
                Err(err) => {
                    tracing::warn!(%err, %upstream, "listHosts page failed after retries");
                    had_failure = true;
                    break;
                }
            };
            total_seen += page.hosts.len();
            let mut sorted = page.hosts;
            sorted.sort_unstable_by_key(|host| host.account_count);
            for host in sorted.into_iter().rev() {
                if host.account_count > HOSTS_MIN_ACCOUNTS
                    && matches!(host.status, HostStatus::Active | HostStatus::Idle)
                    && !self.is_host_banned(&host.hostname)
                    && seen.insert(host.hostname.clone())
                {
                    self.request_crawl_tx
                        .push(RequestCrawl { hostname: host.hostname, cursor: None })?;
                    total_added += 1;
                }
            }
            cursor = page.cursor;
            if cursor.is_none() {
                break;
            }
        }
        let outcome =
            if had_failure { if total_added > 0 { "partial" } else { "fail" } } else { "ok" };
        metrics::record_discovery_round(outcome);
        tracing::info!(total = %total_seen, added = %total_added, %outcome, %upstream, "host discovery refresh complete");
        Ok(())
    }

    #[cfg(feature = "labeler")]
    fn query_hosts(&mut self) -> Result<()> {
        use sqlx::Row as _;
        let pool = self.pool.clone();
        let labelers: Vec<String> = block_on_db(&self.rt, async move {
            let rows = sqlx::query(
                "SELECT DISTINCT pds_endpoint FROM plc_keys WHERE pds_endpoint IS NOT NULL",
            )
            .fetch_all(&pool)
            .await?;
            Ok::<_, sqlx::Error>(
                rows.into_iter()
                    .filter_map(|r| r.try_get::<Option<String>, _>("pds_endpoint").ok().flatten())
                    .collect(),
            )
        })
        .map_err(|e| color_eyre::eyre::eyre!("{e}"))?;

        for endpoint in labelers {
            if let Some(hostname) =
                endpoint.strip_prefix("https://").map(|x| x.trim_end_matches('/'))
            {
                self.request_crawl_tx
                    .push(RequestCrawl { hostname: hostname.to_owned(), cursor: None })?;
            }
        }
        Ok(())
    }

    fn ban_host(&self, hostname: &str) -> Result<()> {
        let pool = self.pool.clone();
        let host = hostname.to_owned();
        block_on_db(&self.rt, async move {
            sqlx::query(
                "INSERT INTO banned_hosts (host) VALUES ($1) ON CONFLICT (host) DO NOTHING",
            )
            .bind(&host)
            .execute(&pool)
            .await?;
            Ok::<_, sqlx::Error>(())
        })
        .map_err(|e| color_eyre::eyre::eyre!("{e}"))?;
        tracing::warn!(%hostname, "banned PDS host");
        Ok(())
    }

    fn unban_host(&self, hostname: &str) -> Result<()> {
        let pool = self.pool.clone();
        let host = hostname.to_owned();
        block_on_db(&self.rt, async move {
            sqlx::query("DELETE FROM banned_hosts WHERE host = $1")
                .bind(&host)
                .execute(&pool)
                .await?;
            Ok::<_, sqlx::Error>(())
        })
        .map_err(|e| color_eyre::eyre::eyre!("{e}"))?;
        tracing::warn!(%hostname, "unbanned PDS host");
        Ok(())
    }

    fn list_bans(&self) -> Result<ListBans> {
        use sqlx::Row as _;
        let pool = self.pool.clone();
        let rows = block_on_db(&self.rt, async move {
            sqlx::query("SELECT host, created_at FROM banned_hosts ORDER BY created_at")
                .fetch_all(&pool)
                .await
        })
        .map_err(|e| color_eyre::eyre::eyre!("{e}"))?;

        let banned_hosts = rows
            .into_iter()
            .map(|r| {
                let host: String = r.try_get("host").unwrap_or_default();
                let created_at: Option<DateTime<Utc>> = r.try_get("created_at").unwrap_or(None);
                BannedHost {
                    host,
                    created_at: created_at
                        .map_or_else(|| "1970-01-01T00:00:00Z".to_owned(), |ts| ts.to_rfc3339()),
                }
            })
            .collect();
        Ok(ListBans { banned_hosts })
    }

    fn is_host_banned(&self, hostname: &str) -> bool {
        let pool = self.pool.clone();
        let host = hostname.to_owned();
        block_on_db(&self.rt, async move {
            let row = sqlx::query("SELECT 1 AS exists FROM banned_hosts WHERE host = $1")
                .bind(&host)
                .fetch_optional(&pool)
                .await?;
            Ok::<_, sqlx::Error>(row.is_some())
        })
        .unwrap_or(false)
    }

    fn get_query_param(url: &Url, key: &str) -> Option<String> {
        url.query_pairs().find(|(k, _)| k == key).map(|(_, v)| v.to_string())
    }
}

fn check_admin_auth(headers: &[httparse::Header<'_>]) -> bool {
    let Some(password) = ADMIN_PASSWORD.as_ref() else {
        return false;
    };
    headers.iter().any(|h| {
        h.name.eq_ignore_ascii_case("Authorization")
            && std::str::from_utf8(h.value)
                .ok()
                .and_then(|v| v.strip_prefix("Bearer "))
                .is_some_and(|token| token == password.as_str())
    })
}

#[cfg(all(test, not(feature = "labeler")))]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct ScriptedFetcher {
        script: Vec<Result<ListHosts, &'static str>>,
        idx: Cell<usize>,
    }

    impl ScriptedFetcher {
        const fn new(script: Vec<Result<ListHosts, &'static str>>) -> Self {
            Self { script, idx: Cell::new(0) }
        }
        fn calls(&self) -> usize {
            self.idx.get()
        }
    }

    impl HostListFetcher for ScriptedFetcher {
        fn fetch_page(&self, _cursor: Option<&str>) -> Result<ListHosts> {
            let i = self.idx.get();
            self.idx.set(i + 1);
            let entry = self.script.get(i).ok_or_else(|| eyre!("script exhausted"))?;
            match entry {
                Ok(page) => Ok(ListHosts {
                    cursor: page.cursor.clone(),
                    hosts: page
                        .hosts
                        .iter()
                        .map(|h| Host {
                            account_count: h.account_count,
                            hostname: h.hostname.clone(),
                            seq: h.seq,
                            status: match h.status {
                                HostStatus::Active => HostStatus::Active,
                                HostStatus::Idle => HostStatus::Idle,
                                HostStatus::Offline => HostStatus::Offline,
                                HostStatus::Throttled => HostStatus::Throttled,
                                HostStatus::Banned => HostStatus::Banned,
                            },
                        })
                        .collect(),
                }),
                Err(msg) => Err(eyre!(*msg)),
            }
        }
    }

    fn page(cursor: Option<&str>, hosts: Vec<&str>) -> ListHosts {
        ListHosts {
            cursor: cursor.map(str::to_owned),
            hosts: hosts
                .into_iter()
                .map(|h| Host {
                    account_count: 1,
                    hostname: h.to_owned(),
                    seq: 0,
                    status: HostStatus::Active,
                })
                .collect(),
        }
    }

    #[test]
    fn fetch_page_with_retry_succeeds_on_first_try() {
        let fetcher = ScriptedFetcher::new(vec![Ok(page(None, vec!["a", "b"]))]);
        let res = fetch_page_with_retry(&fetcher, None, |_| {});
        assert!(res.is_ok());
        assert_eq!(fetcher.calls(), 1);
    }

    #[test]
    fn fetch_page_with_retry_succeeds_after_transient_failure() {
        let fetcher = ScriptedFetcher::new(vec![Err("transient"), Ok(page(None, vec!["a"]))]);
        let collector = std::cell::RefCell::new(Vec::<Duration>::new());
        let sleep_fn = |d: Duration| collector.borrow_mut().push(d);
        let res = fetch_page_with_retry(&fetcher, None, sleep_fn);
        assert!(res.is_ok());
        assert_eq!(fetcher.calls(), 2);
        assert_eq!(collector.borrow().len(), 1);
    }

    #[test]
    fn fetch_page_with_retry_returns_err_after_exhausting_attempts() {
        let fetcher = ScriptedFetcher::new(vec![Err("e1"), Err("e2"), Err("e3")]);
        let res = fetch_page_with_retry(&fetcher, None, |_| {});
        assert!(res.is_err());
        assert_eq!(fetcher.calls(), 3);
    }

    #[test]
    fn fetch_page_with_retry_doubles_backoff_between_attempts() {
        let fetcher = ScriptedFetcher::new(vec![Err("a"), Err("b"), Err("c")]);
        let collector = std::cell::RefCell::new(Vec::<Duration>::new());
        let sleep_fn = |d: Duration| collector.borrow_mut().push(d);
        drop(fetch_page_with_retry(&fetcher, None, sleep_fn));
        let sleeps = collector.borrow();
        assert_eq!(sleeps.len(), 2, "sleep between attempts only");
        assert_eq!(sleeps[0], Duration::from_secs(1));
        assert_eq!(sleeps[1], Duration::from_secs(2));
    }
}
