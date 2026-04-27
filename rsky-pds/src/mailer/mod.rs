pub mod moderation;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;
use std::env;

#[derive(Serialize)]
struct ResendEmail<'a> {
    from: &'a str,
    to: Vec<&'a str>,
    subject: &'a str,
    html: &'a str,
}

async fn send_via_resend(from: &str, to: &str, subject: &str, html: &str) -> Result<()> {
    let api_key = env::var("RESEND_API_KEY").context("RESEND_API_KEY must be set")?;
    let body = ResendEmail {
        from,
        to: vec![to],
        subject,
        html,
    };
    let resp = Client::new()
        .post("https://api.resend.com/emails")
        .bearer_auth(&api_key)
        .json(&body)
        .send()
        .await
        .context("failed to send email via Resend")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Resend API error {status}: {body}");
    }
    Ok(())
}

fn sender() -> String {
    let name = env::var("PDS_EMAIL_FROM_NAME").unwrap_or_else(|_| "rsky PDS".to_string());
    let addr = env::var("PDS_EMAIL_FROM_ADDRESS")
        .unwrap_or_else(|_| "noreply@pds.know-me.tools".to_string());
    format!("{name} <{addr}>")
}

pub struct MailOpts {
    pub to: String,
    pub subject: String,
    pub template: String,
    pub template_vars: HashMap<String, String>,
}

fn render_template(template: &str, vars: &HashMap<String, String>) -> String {
    match template {
        "reset password" => {
            let token = vars.get("token").map(|s| s.as_str()).unwrap_or("");
            let identifier = vars.get("identifier").map(|s| s.as_str()).unwrap_or("");
            format!(
                r#"<p>A password reset was requested for <strong>{identifier}</strong>.</p>
<p>Your reset token is: <strong>{token}</strong></p>
<p>If you did not request this, you can ignore this email.</p>"#
            )
        }
        "delete account" => {
            let token = vars.get("token").map(|s| s.as_str()).unwrap_or("");
            format!(
                r#"<p>An account deletion was requested.</p>
<p>Your confirmation token is: <strong>{token}</strong></p>
<p>If you did not request this, you can ignore this email.</p>"#
            )
        }
        "confirm email" => {
            let token = vars.get("token").map(|s| s.as_str()).unwrap_or("");
            format!(
                r#"<p>Please confirm your email address.</p>
<p>Your confirmation token is: <strong>{token}</strong></p>"#
            )
        }
        "email update" => {
            let token = vars.get("token").map(|s| s.as_str()).unwrap_or("");
            format!(
                r#"<p>An email address update was requested.</p>
<p>Your confirmation token is: <strong>{token}</strong></p>
<p>If you did not request this, you can ignore this email.</p>"#
            )
        }
        "plc operation" => {
            let token = vars.get("token").map(|s| s.as_str()).unwrap_or("");
            format!(
                r#"<p>A PLC update operation was requested for your account.</p>
<p>Your authorization token is: <strong>{token}</strong></p>
<p>If you did not request this, contact support immediately.</p>"#
            )
        }
        other => format!("<p>Notification: {other}</p>"),
    }
}

pub async fn send_template(opts: MailOpts) -> Result<()> {
    let html = render_template(&opts.template, &opts.template_vars);
    let from = sender();
    send_via_resend(&from, &opts.to, &opts.subject, &html).await
}

#[derive(Clone, Debug, PartialEq, Serialize, serde_derive::Deserialize)]
pub struct IdentifierAndTokenParams {
    pub identifier: String,
    pub token: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, serde_derive::Deserialize)]
pub struct TokenParam {
    pub token: String,
}

pub async fn send_reset_password(to: String, params: IdentifierAndTokenParams) -> Result<()> {
    let mut template_vars = HashMap::new();
    template_vars.insert("identifier".to_string(), params.identifier);
    template_vars.insert("token".to_string(), params.token);
    send_template(MailOpts {
        to,
        subject: "Password Reset Requested".to_string(),
        template: "reset password".to_string(),
        template_vars,
    })
    .await
}

pub async fn send_account_delete(to: String, params: TokenParam) -> Result<()> {
    let mut template_vars = HashMap::new();
    template_vars.insert("token".to_string(), params.token);
    send_template(MailOpts {
        to,
        subject: "Account Deletion Requested".to_string(),
        template: "delete account".to_string(),
        template_vars,
    })
    .await
}

pub async fn send_confirm_email(to: String, params: TokenParam) -> Result<()> {
    let mut template_vars = HashMap::new();
    template_vars.insert("token".to_string(), params.token);
    send_template(MailOpts {
        to,
        subject: "Email Confirmation".to_string(),
        template: "confirm email".to_string(),
        template_vars,
    })
    .await
}

pub async fn send_update_email(to: String, params: TokenParam) -> Result<()> {
    let mut template_vars = HashMap::new();
    template_vars.insert("token".to_string(), params.token);
    send_template(MailOpts {
        to,
        subject: "Email Update Requested".to_string(),
        template: "email update".to_string(),
        template_vars,
    })
    .await
}

pub async fn send_plc_operation(to: String, params: TokenParam) -> Result<()> {
    let mut template_vars = HashMap::new();
    template_vars.insert("token".to_string(), params.token);
    send_template(MailOpts {
        to,
        subject: "PLC Update Operation Requested".to_string(),
        template: "plc operation".to_string(),
        template_vars,
    })
    .await
}
