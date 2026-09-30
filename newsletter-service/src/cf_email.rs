//! Cloudflare Email Service client, ported from `cf-email.ts`.
//!
//! Hardening notes: the HTTP client has explicit connect + total timeouts so a
//! hung Cloudflare endpoint can never wedge a request handler or a send loop.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};

use crate::config::Config;

const CF_API_BASE: &str = "https://api.cloudflare.com";

#[derive(Clone)]
pub struct EmailClient {
    http: reqwest::Client,
    config: Arc<Config>,
    /// Always `CF_API_BASE` outside tests, which point it at a local fake.
    api_base: String,
}

pub struct SendArgs<'a> {
    pub to: &'a str,
    pub subject: &'a str,
    pub html: &'a str,
    pub text: &'a str,
    pub headers: &'a [(String, String)],
}

pub struct SendResult {
    pub ok: bool,
    pub status: u16,
    pub body: Value,
}

#[derive(Debug, Clone)]
pub struct Suppression {
    pub email: String,
    pub reason: String,
}

impl EmailClient {
    pub fn new(config: Arc<Config>) -> anyhow::Result<EmailClient> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .user_agent("newsletter-service")
            .build()?;
        Ok(EmailClient {
            http,
            config,
            api_base: CF_API_BASE.to_string(),
        })
    }

    #[cfg(test)]
    pub fn with_api_base(mut self, base: &str) -> EmailClient {
        self.api_base = base.to_string();
        self
    }

    fn send_endpoint(&self) -> String {
        format!(
            "{}/client/v4/accounts/{}/email/sending/send",
            self.api_base, self.config.cf_account_id
        )
    }

    /// Send one email. In dev mode (no CF creds) the email and its links are
    /// logged instead of sent, so the whole flow works with no credentials.
    pub async fn send_email(&self, args: SendArgs<'_>) -> SendResult {
        if self.config.email_dev_mode() {
            tracing::info!(target: "dev_email", to = %args.to, subject = %args.subject, "dev email (not sent)");
            for link in extract_links(args.text) {
                tracing::info!(target: "dev_email", link = %link, "dev email link");
            }
            return SendResult {
                ok: true,
                status: 200,
                body: json!({ "dev": true }),
            };
        }

        let mut payload = json!({
            "to": args.to,
            "from": self.config.sender_field(),
            "subject": args.subject,
            "html": args.html,
            "text": args.text,
        });
        if !args.headers.is_empty() {
            let map: serde_json::Map<String, Value> = args
                .headers
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect();
            payload["headers"] = Value::Object(map);
        }

        let resp = self
            .http
            .post(self.send_endpoint())
            .bearer_auth(&self.config.cf_email_token)
            .json(&payload)
            .send()
            .await;

        match resp {
            Ok(res) => {
                let status = res.status().as_u16();
                let ok = res.status().is_success();
                // Prefer JSON; fall back to text wrapped as a JSON string; else null.
                let body = match res.text().await {
                    Ok(text) => serde_json::from_str(&text).unwrap_or(Value::String(text)),
                    Err(_) => Value::Null,
                };
                SendResult { ok, status, body }
            }
            Err(e) => {
                tracing::error!(error = %e, "cloudflare send request failed");
                SendResult {
                    ok: false,
                    status: 0,
                    body: Value::Null,
                }
            }
        }
    }

    /// Send with a small backoff retry on HTTP 429 (CF ramping quota).
    pub async fn send_email_with_retry(&self, args: SendArgs<'_>) -> SendResult {
        let mut attempt: u32 = 0;
        loop {
            let res = self.send_email(SendArgs { ..args_ref(&args) }).await;
            if res.status != 429 {
                return res;
            }
            attempt += 1;
            if attempt >= 3 {
                return res;
            }
            tokio::time::sleep(Duration::from_millis(500 * 2u64.pow(attempt))).await;
        }
    }

    /// List Cloudflare's Email Sending suppression entries, paging through all
    /// results. Returns [] in dev mode. Stops on the first failed page.
    pub async fn list_suppressions(&self) -> Vec<Suppression> {
        if self.config.email_dev_mode() {
            return Vec::new();
        }
        let per_page = 100usize;
        let mut out: Vec<Suppression> = Vec::new();

        for page in 1..=1000usize {
            let url = format!(
                "{}/client/v4/accounts/{}/email/sending/suppressions?page={}&per_page={}",
                self.api_base, self.config.cf_account_id, page, per_page
            );
            let resp = self
                .http
                .get(&url)
                .bearer_auth(&self.config.cf_email_token)
                .send()
                .await;

            let res = match resp {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(error = %e, "suppression list request failed");
                    break;
                }
            };
            if !res.status().is_success() {
                let status = res.status();
                let body = res.text().await.unwrap_or_default();
                tracing::error!(%status, body = %body, "suppression list failed");
                break;
            }

            let data: Value = match res.json().await {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(error = %e, "suppression list parse failed");
                    break;
                }
            };

            let rows = data
                .get("result")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let row_len = rows.len();
            for r in rows {
                if let Some(email) = r.get("email").and_then(Value::as_str) {
                    out.push(Suppression {
                        email: email.to_string(),
                        reason: r
                            .get("reason")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    });
                }
            }

            let total = data
                .get("result_info")
                .and_then(|i| i.get("total_count"))
                .and_then(Value::as_u64)
                .map(|t| t as usize);
            // Stop once the page wasn't full, or we've collected the reported total.
            if row_len < per_page || total.map(|t| out.len() >= t).unwrap_or(false) {
                break;
            }
        }
        out
    }
}

// Small helper so `send_email_with_retry` can re-borrow the same args each loop.
fn args_ref<'a>(a: &'a SendArgs<'a>) -> SendArgs<'a> {
    SendArgs {
        to: a.to,
        subject: a.subject,
        html: a.html,
        text: a.text,
        headers: a.headers,
    }
}

/// Recipients Cloudflare reported as permanently bounced in a send response. A
/// send can return HTTP 200 while still rejecting some addresses under
/// `result.permanent_bounces`.
pub fn permanent_bounces(res: &SendResult) -> Vec<String> {
    res.body
        .get("result")
        .and_then(|r| r.get("permanent_bounces"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_links(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|w| w.starts_with("http://") || w.starts_with("https://"))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(body: Value) -> SendResult {
        SendResult {
            ok: true,
            status: 200,
            body,
        }
    }

    #[test]
    fn reads_permanent_bounces_from_a_2xx_response() {
        let res = result(json!({
            "success": true,
            "result": { "delivered": ["a@x.co"], "permanent_bounces": ["b@x.co", 7] }
        }));
        assert_eq!(permanent_bounces(&res), ["b@x.co"]);
    }

    #[test]
    fn no_bounces_when_the_field_is_absent_or_odd() {
        for body in [
            json!({ "result": { "delivered": ["a@x.co"] } }),
            json!({ "result": { "permanent_bounces": "b@x.co" } }),
            json!("plain text error"),
            Value::Null,
        ] {
            assert!(permanent_bounces(&result(body)).is_empty());
        }
    }
}
