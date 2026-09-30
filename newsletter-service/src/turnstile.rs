//! Cloudflare Turnstile verification, ported from `turnstile.ts`.
//!
//! This is the bot gate that runs BEFORE any confirmation email is sent, so a
//! flood of fake signups can't run up the Cloudflare email bill. Any network or
//! parse error resolves to "not verified" (fail closed — the safe direction for a
//! gate that guards spend).

use crate::config::Config;

const VERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

pub async fn verify_turnstile(
    http: &reqwest::Client,
    config: &Config,
    token_response: &str,
    remote_ip: Option<&str>,
) -> bool {
    if token_response.is_empty() {
        return false;
    }

    let mut form = vec![
        ("secret", config.turnstile_secret.as_str()),
        ("response", token_response),
    ];
    if let Some(ip) = remote_ip {
        form.push(("remoteip", ip));
    }

    match http.post(VERIFY_URL).form(&form).send().await {
        Ok(res) => match res.json::<serde_json::Value>().await {
            Ok(data) => data.get("success").and_then(|v| v.as_bool()) == Some(true),
            Err(e) => {
                tracing::warn!(error = %e, "turnstile response parse failed");
                false
            }
        },
        Err(e) => {
            tracing::warn!(error = %e, "turnstile request failed");
            false
        }
    }
}
