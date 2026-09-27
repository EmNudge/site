//! Runtime configuration, ported from the original `config.ts`.
//!
//! Every environment variable name and default is preserved so this binary is a
//! drop-in replacement for the Node service: the same `.env` file just works.

use std::env;

/// The two subscription lists. Mirrors the `ListName` union in the TS service and
/// the `CHECK (list IN ('blog','notes'))` constraint in the schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListName {
    Blog,
    Notes,
}

impl ListName {
    pub fn as_str(self) -> &'static str {
        match self {
            ListName::Blog => "blog",
            ListName::Notes => "notes",
        }
    }

    /// Parse a value into a known list, rejecting anything else. This is the Rust
    /// equivalent of the TS `isList` type guard and is the ONLY way list strings
    /// enter the system, so an attacker can never smuggle an unknown list name
    /// into a SQL parameter.
    pub fn parse(value: &str) -> Option<ListName> {
        match value {
            "blog" => Some(ListName::Blog),
            "notes" => Some(ListName::Notes),
            _ => None,
        }
    }
}

/// Where the real client IP comes from — the key for every per-IP guard (rate
/// limit, per-IP cap, admin lockout). Both headers are only trustworthy when the
/// sole way to reach the service is through the proxy that sets them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientIpHeader {
    /// The rightmost `X-Forwarded-For` entry, as appended by a reverse proxy on the
    /// same host (Caddy; see deploy/Caddyfile).
    ForwardedFor,
    /// `CF-Connecting-IP`, which Cloudflare sets on every proxied request,
    /// overwriting anything the client sent (Cloudflare Tunnel; see the README).
    CfConnectingIp,
}

impl ClientIpHeader {
    pub fn parse(value: &str) -> Option<ClientIpHeader> {
        match value.trim().to_ascii_lowercase().as_str() {
            "x-forwarded-for" => Some(ClientIpHeader::ForwardedFor),
            "cf-connecting-ip" => Some(ClientIpHeader::CfConnectingIp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    /// Bind address. Defaults to loopback: the service is meant to be reached only
    /// through a proxy on the same host (Cloudflare Tunnel's `cloudflared`, or
    /// Caddy), which sets the header named by `client_ip_header`.
    /// Only bind to a non-loopback address behind a trusted proxy — otherwise a
    /// direct client can set that header and bypass every per-IP guard. The
    /// service refuses to start in dev mode on a non-loopback bind.
    pub bind_host: String,
    pub client_ip_header: ClientIpHeader,
    pub db_path: String,

    // Cloudflare Email Service. Optional: when unset, the service runs in
    // "log email" dev mode.
    pub cf_account_id: String,
    pub cf_email_token: String,
    pub from_email: String,
    pub from_name: String,
    pub author_name: String,

    // Turnstile (bot gate on /subscribe). Optional in dev.
    pub turnstile_secret: String,

    // Admin auth for /admin/*.
    pub admin_token: String,

    // Where the static site lives — CORS + human-facing confirm/unsub links.
    pub site_origin: String,
    // This service's own public URL — used for the one-click List-Unsubscribe header.
    pub public_base: String,

    // Abuse / cost guards.
    pub rate_per_min_per_ip: u32,
    pub global_confirm_cap_per_hour: i64,
    /// Daily ceiling on confirmation emails, so a sustained signup flood can't use
    /// up the Cloudflare sending quota that newsletter blasts need.
    pub global_confirm_cap_per_day: i64,
    /// Max confirmations waiting for a free slot once a cap is hit. Beyond this,
    /// `/subscribe` answers 429.
    pub confirm_queue_max: i64,
    /// Drop a queued confirmation that has waited this long — the reader has
    /// stopped expecting it.
    pub confirm_queue_max_age_hours: i64,
    /// Delete subscriptions still unconfirmed after this many days. 0 disables.
    pub pending_expiry_days: i64,
    /// Reject signups whose domain can't receive mail (no MX / A record). Skipped
    /// in dev mode.
    pub mx_check: bool,
    pub per_ip_confirm_cap_per_hour: u32,
    pub max_body_bytes: usize,
    pub max_send_body_bytes: usize,
    pub admin_auth_max_failures: u32,
    pub confirm_resend_cooldown_min: i64,
    pub suppression_sync_min: u64,
    pub send_throttle_ms: u64,

    /// Daily backups to Cloudflare R2; `None` (no R2_* vars) turns them off.
    pub r2: Option<R2Config>,
}

/// Where automatic backups go: an R2 bucket, reached through its S3-compatible API.
#[derive(Debug, Clone)]
pub struct R2Config {
    /// `https://<account id>.r2.cloudflarestorage.com` (tests point it at a fake).
    pub endpoint: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket: String,
    /// Key prefix inside the bucket, without slashes at either end.
    pub prefix: String,
}

impl R2Config {
    /// All four settings enable backups; none leaves them off. Anything in
    /// between is a startup error rather than silently running without backups.
    pub fn from_parts(
        account_id: &str,
        access_key_id: &str,
        secret_access_key: &str,
        bucket: &str,
        prefix: &str,
    ) -> Result<Option<R2Config>, String> {
        let required = [
            ("R2_ACCOUNT_ID", account_id),
            ("R2_ACCESS_KEY_ID", access_key_id),
            ("R2_SECRET_ACCESS_KEY", secret_access_key),
            ("R2_BUCKET", bucket),
        ];
        let missing: Vec<&str> = required
            .iter()
            .filter(|(_, v)| v.is_empty())
            .map(|(k, _)| *k)
            .collect();
        if missing.len() == required.len() {
            return Ok(None);
        }
        if !missing.is_empty() {
            return Err(format!(
                "R2 backups are partly configured — also set {}",
                missing.join(", ")
            ));
        }
        Ok(Some(R2Config {
            endpoint: format!("https://{account_id}.r2.cloudflarestorage.com"),
            access_key_id: access_key_id.to_string(),
            secret_access_key: secret_access_key.to_string(),
            bucket: bucket.to_string(),
            prefix: prefix.trim_matches('/').to_string(),
        }))
    }
}

fn required(name: &str) -> Result<String, String> {
    match env::var(name) {
        Ok(v) if !v.is_empty() => Ok(v),
        _ => Err(format!("Missing required env var: {name}")),
    }
}

fn optional(name: &str, fallback: &str) -> String {
    match env::var(name) {
        Ok(v) if !v.is_empty() => v,
        _ => fallback.to_string(),
    }
}

/// Parse an integer env var, falling back only when it is absent. An unparseable
/// value is an error: silently reverting a typo'd security limit to its default
/// would hide the misconfiguration.
fn optional_int<T: std::str::FromStr>(name: &str, fallback: T) -> Result<T, String> {
    match env::var(name) {
        Ok(raw) if !raw.is_empty() => raw
            .trim()
            .parse()
            .map_err(|_| format!("Invalid integer in env var {name}: {raw:?}")),
        _ => Ok(fallback),
    }
}

fn strip_trailing_slash(s: String) -> String {
    s.strip_suffix('/').map(str::to_string).unwrap_or(s)
}

impl Config {
    /// Load configuration from the environment (after `.env` has been read).
    /// Returns an error string for any missing required var rather than panicking.
    pub fn from_env() -> Result<Config, String> {
        Ok(Config {
            port: optional_int("PORT", 8787u16)?,
            bind_host: optional("BIND_HOST", "127.0.0.1"),
            client_ip_header: {
                let raw = optional("CLIENT_IP_HEADER", "x-forwarded-for");
                ClientIpHeader::parse(&raw).ok_or_else(|| {
                    format!(
                        "Invalid CLIENT_IP_HEADER {raw:?}: expected x-forwarded-for or cf-connecting-ip"
                    )
                })?
            },
            db_path: optional("DB_PATH", "./data/newsletter.db"),

            cf_account_id: optional("CF_ACCOUNT_ID", ""),
            cf_email_token: optional("CF_EMAIL_TOKEN", ""),
            from_email: optional("FROM_EMAIL", "noreply@localhost"),
            from_name: optional("FROM_NAME", ""),
            author_name: optional("AUTHOR_NAME", "EmNudge"),

            turnstile_secret: optional("TURNSTILE_SECRET", ""),

            admin_token: required("ADMIN_TOKEN")?,

            site_origin: strip_trailing_slash(required("SITE_ORIGIN")?),
            public_base: strip_trailing_slash(required("PUBLIC_BASE")?),

            rate_per_min_per_ip: optional_int("RATE_LIMIT_PER_MIN", 5)?,
            global_confirm_cap_per_hour: optional_int("GLOBAL_CONFIRM_CAP_PER_HOUR", 200)?,
            global_confirm_cap_per_day: optional_int("GLOBAL_CONFIRM_CAP_PER_DAY", 500)?,
            confirm_queue_max: optional_int("CONFIRM_QUEUE_MAX", 1000)?,
            confirm_queue_max_age_hours: optional_int("CONFIRM_QUEUE_MAX_AGE_HOURS", 24)?,
            pending_expiry_days: optional_int("PENDING_EXPIRY_DAYS", 7)?,
            mx_check: optional_int("MX_CHECK", 1u8)? != 0,
            per_ip_confirm_cap_per_hour: optional_int("PER_IP_CONFIRM_CAP_PER_HOUR", 20)?,
            max_body_bytes: optional_int("MAX_BODY_BYTES", 16 * 1024)?,
            max_send_body_bytes: optional_int("MAX_SEND_BODY_BYTES", 512 * 1024)?,
            admin_auth_max_failures: optional_int("ADMIN_AUTH_MAX_FAILURES", 10)?,
            confirm_resend_cooldown_min: optional_int("CONFIRM_RESEND_COOLDOWN_MIN", 15)?,
            suppression_sync_min: optional_int("SUPPRESSION_SYNC_MIN", 60)?,
            send_throttle_ms: optional_int("SEND_THROTTLE_MS", 500)?,
            r2: R2Config::from_parts(
                &optional("R2_ACCOUNT_ID", ""),
                &optional("R2_ACCESS_KEY_ID", ""),
                &optional("R2_SECRET_ACCESS_KEY", ""),
                &optional("R2_BUCKET", ""),
                &optional("R2_PREFIX", "newsletter"),
            )?,
        })
    }

    /// True when Cloudflare creds are absent — emails are logged to the console,
    /// not sent. Mirrors `emailDevMode` in the TS service.
    pub fn email_dev_mode(&self) -> bool {
        self.cf_account_id.is_empty() || self.cf_email_token.is_empty()
    }

    /// `From:` header value — `"Name <email>"` when a display name is set.
    pub fn sender_field(&self) -> String {
        if self.from_name.is_empty() {
            self.from_email.clone()
        } else {
            format!("{} <{}>", self.from_name, self.from_email)
        }
    }

    /// Dev-mode config with the `from_env` defaults, minus anything that would
    /// make a test slow or reach the network (send throttle, MX lookups).
    #[cfg(test)]
    pub fn for_tests() -> Config {
        Config {
            port: 0,
            bind_host: "127.0.0.1".to_string(),
            client_ip_header: ClientIpHeader::ForwardedFor,
            db_path: ":memory:".to_string(),
            cf_account_id: String::new(),
            cf_email_token: String::new(),
            from_email: "noreply@example.com".to_string(),
            from_name: String::new(),
            author_name: "EmNudge".to_string(),
            turnstile_secret: String::new(),
            admin_token: "test-admin-token".to_string(),
            site_origin: "https://site.test".to_string(),
            public_base: "https://api.site.test".to_string(),
            rate_per_min_per_ip: 5,
            global_confirm_cap_per_hour: 200,
            global_confirm_cap_per_day: 500,
            confirm_queue_max: 1000,
            confirm_queue_max_age_hours: 24,
            pending_expiry_days: 7,
            mx_check: false,
            per_ip_confirm_cap_per_hour: 20,
            max_body_bytes: 16 * 1024,
            max_send_body_bytes: 512 * 1024,
            admin_auth_max_failures: 10,
            confirm_resend_cooldown_min: 15,
            suppression_sync_min: 60,
            send_throttle_ms: 0,
            r2: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r2_backups_need_all_four_settings_or_none() {
        assert!(R2Config::from_parts("", "", "", "", "newsletter")
            .unwrap()
            .is_none());

        let r2 = R2Config::from_parts("acct", "key", "secret", "bkt", "/nl/")
            .unwrap()
            .unwrap();
        assert_eq!(r2.endpoint, "https://acct.r2.cloudflarestorage.com");
        assert_eq!((r2.bucket.as_str(), r2.prefix.as_str()), ("bkt", "nl"));

        let err = R2Config::from_parts("acct", "key", "", "", "newsletter").unwrap_err();
        assert!(err.contains("R2_SECRET_ACCESS_KEY, R2_BUCKET"), "{err}");
    }

    #[test]
    fn client_ip_header_accepts_only_the_two_known_headers() {
        assert_eq!(
            ClientIpHeader::parse("x-forwarded-for"),
            Some(ClientIpHeader::ForwardedFor)
        );
        assert_eq!(
            ClientIpHeader::parse(" CF-Connecting-IP "),
            Some(ClientIpHeader::CfConnectingIp)
        );
        for bad in ["", "x-real-ip", "true-client-ip", "forwarded"] {
            assert_eq!(ClientIpHeader::parse(bad), None, "{bad:?}");
        }
    }
}
