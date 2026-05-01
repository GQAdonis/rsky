use crate::APP_USER_AGENT;
use anyhow::Result;
use futures::stream::{self, StreamExt};
use rsky_common::time::MINUTE;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

const NOTIFY_THRESHOLD: i32 = 20 * MINUTE; // 20 minutes;

#[derive(Debug, Clone)]
pub struct Crawlers {
    pub hostname: String,
    pub crawlers: Vec<String>,
    pub last_notified: usize,
    /// Per-relay in-flight coalesce guard: tracks which relay URLs currently
    /// have an outbound requestCrawl in progress.  Concurrent callers skip
    /// any relay that is already being notified, preventing duplicate requests.
    pub in_flight: Arc<Mutex<HashSet<String>>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CrawlerRequest {
    pub hostname: String,
}

impl Crawlers {
    pub fn new(hostname: String, crawlers: Vec<String>) -> Self {
        Crawlers {
            hostname,
            crawlers,
            last_notified: 0,
            in_flight: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn notify_of_update(&mut self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("timestamp in micros since UNIX epoch")
            .as_micros() as usize;
        if now - self.last_notified < NOTIFY_THRESHOLD as usize {
            return Ok(());
        }

        // Determine which relays are not already being notified in another
        // concurrent call and mark them as in-flight.
        let services_to_notify: Vec<String> = {
            let mut guard = self.in_flight.lock().unwrap();
            self.crawlers
                .iter()
                .filter(|s| guard.insert((*s).clone()))
                .cloned()
                .collect()
        };

        if services_to_notify.is_empty() {
            return Ok(());
        }

        let in_flight = self.in_flight.clone();
        let _ = stream::iter(services_to_notify.clone())
            .then(|service: String| async move {
                let client = reqwest::Client::builder()
                    .user_agent(APP_USER_AGENT)
                    .build()?;
                let record = CrawlerRequest {
                    hostname: service.clone(),
                };
                Ok::<reqwest::Response, anyhow::Error>(
                    client
                        .post(format!("{}/xrpc/com.atproto.sync.requestCrawl", service))
                        .json(&record)
                        .send()
                        .await?,
                )
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        // Release the in-flight markers for the relays we just notified.
        {
            let mut guard = in_flight.lock().unwrap();
            for s in &services_to_notify {
                guard.remove(s);
            }
        }

        self.last_notified = now;
        Ok(())
    }
}
