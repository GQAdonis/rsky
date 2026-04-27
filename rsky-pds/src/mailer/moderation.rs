use anyhow::{Context, Result};
use reqwest::Client;
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct ResendEmail<'a> {
    from: &'a str,
    to: Vec<&'a str>,
    subject: &'a str,
    html: &'a str,
}

pub struct HtmlMailOpts {
    pub to: String,
    pub subject: String,
    pub html: String,
}

pub struct ModerationMailer {}

impl ModerationMailer {
    pub async fn send_html(opts: HtmlMailOpts) -> Result<()> {
        let api_key = env::var("RESEND_API_KEY").context("RESEND_API_KEY must be set")?;
        let name = env::var("PDS_MODERATION_EMAIL_FROM_NAME")
            .unwrap_or_else(|_| "rsky Moderation".to_string());
        let addr = env::var("PDS_MODERATION_EMAIL_FROM_ADDRESS")
            .unwrap_or_else(|_| "moderation@pds.know-me.tools".to_string());
        let from = format!("{name} <{addr}>");

        let body = ResendEmail {
            from: &from,
            to: vec![opts.to.as_str()],
            subject: &opts.subject,
            html: &opts.html,
        };

        let resp = Client::new()
            .post("https://api.resend.com/emails")
            .bearer_auth(&api_key)
            .json(&body)
            .send()
            .await
            .context("failed to send moderation email via Resend")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Resend API error {status}: {body}");
        }
        Ok(())
    }
}
