/// Application-wide configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// PostgreSQL connection URL.
    pub database_url: String,

    /// DID of this AppView service, e.g. `did:web:appview.know-me.tools`.
    pub service_did: String,

    /// Public hostname of this service, e.g. `appview.know-me.tools`.
    pub service_hostname: String,

    /// Comma-separated list of relay WebSocket URLs to subscribe to.
    pub relay_hosts: Vec<String>,

    /// Port on which Prometheus metrics are exposed (default: 9091).
    pub metrics_port: u16,

    /// LiveKit server URL.
    pub livekit_url: String,

    /// LiveKit API key.
    pub livekit_api_key: String,

    /// LiveKit API secret.
    pub livekit_api_secret: String,

    /// Rust log filter string, e.g. `"info,appview_api=debug"`.
    pub rust_log: String,
}

impl AppConfig {
    /// Load configuration from environment variables, using defaults where appropriate.
    pub fn from_env() -> Self {
        let relay_hosts = std::env::var("RELAY_HOSTS")
            .unwrap_or_else(|_| "wss://bsky.network".to_owned())
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_owned())
            .collect();

        let metrics_port = std::env::var("METRICS_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(9091);

        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/appview".to_owned()),
            service_did: std::env::var("SERVICE_DID")
                .unwrap_or_else(|_| "did:web:appview.know-me.tools".to_owned()),
            service_hostname: std::env::var("SERVICE_HOSTNAME")
                .unwrap_or_else(|_| "appview.know-me.tools".to_owned()),
            relay_hosts,
            metrics_port,
            livekit_url: std::env::var("LIVEKIT_URL")
                .unwrap_or_else(|_| "wss://livekit.know-me.tools".to_owned()),
            livekit_api_key: std::env::var("LIVEKIT_API_KEY").unwrap_or_default(),
            livekit_api_secret: std::env::var("LIVEKIT_API_SECRET").unwrap_or_default(),
            rust_log: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned()),
        }
    }
}
