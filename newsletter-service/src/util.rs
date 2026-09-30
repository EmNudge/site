//! Small shared helpers: timestamps, tokens, and email validation.

use base64::Engine;
use chrono::{SecondsFormat, Utc};
use rand::RngCore;

/// Current time as an ISO-8601 / RFC-3339 string with millisecond precision and a
/// `Z` suffix — byte-for-byte identical to JavaScript's `new Date().toISOString()`.
/// This exact format matters: timestamps are stored as TEXT and compared
/// lexicographically (e.g. the hourly send-cap window), so it must match the
/// format already present in an existing database.
pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// An ISO timestamp `ms` milliseconds before now.
pub fn iso_millis_ago(ms: i64) -> String {
    (Utc::now() - chrono::Duration::milliseconds(ms)).to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// A fresh unguessable token: 32 random bytes, base64url without padding — the
/// same shape (`randomBytes(32).toString("base64url")`, 43 chars) the TS service
/// produced, so old and new tokens are interchangeable.
pub fn new_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Conservative email shape check: ASCII-only, bounded length, one `@`, a dotted
/// domain. Deliberately stricter than RFC 5322 — it rejects quoted local parts,
/// display names (`x<a@b.c>`) and exotic characters that could let one mailbox be
/// spelled many ways. Deliverability is enforced by the double opt-in flow.
pub fn is_valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    email.len() <= 254
        && (1..=64).contains(&local.len())
        && email
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._%+-@".contains(&b))
        && !domain.contains('@')
        && domain.contains('.')
        && !domain.starts_with(['.', '-'])
        && !domain.ends_with(['.', '-'])
        && !domain.contains("..")
}

/// The key the per-email resend cooldown is tracked under: providers deliver
/// `user+tag@` (and, for Gmail, `u.s.e.r@`) to the same inbox, so each spelling
/// must share one cooldown or a victim can be inbox-bombed via aliases.
pub fn mailbox_key(email: &str) -> String {
    let email = email.to_ascii_lowercase();
    let Some((local, domain)) = email.split_once('@') else {
        return email;
    };
    let local = local.split('+').next().unwrap_or(local);
    let local = if matches!(domain, "gmail.com" | "googlemail.com") {
        local.replace('.', "")
    } else {
        local.to_string()
    };
    format!("{local}@{domain}")
}

/// The client IP, read from the X-Forwarded-For chain (or from `CF-Connecting-IP`,
/// a single address, which parses the same way).
///
/// SECURITY: we take the **rightmost** entry, not the leftmost. `X-Forwarded-For`
/// is a comma-separated list that each proxy *appends* to — Caddy's `reverse_proxy`
/// adds the real peer's address to the end of whatever the client sent. The
/// leftmost entries are therefore fully attacker-controlled (a client can send
/// `X-Forwarded-For: 1.2.3.4` and Caddy turns it into `1.2.3.4, <real-ip>`).
/// Trusting exactly one proxy hop (our Caddy), the last entry is the only one we
/// can rely on. This is what every per-IP guard (rate limit, caps,
/// admin brute-force throttle, via [`rate_key`]) keys off, so getting it right is load-bearing.
pub fn client_ip(xff: Option<&str>) -> String {
    xff.and_then(|h| h.rsplit(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("unknown")
        .to_string()
}

/// The bucket key for per-IP limits. IPv6 clients routinely control a whole /64,
/// so keying on the full address would hand each of them ~2^64 fresh buckets;
/// collapse IPv6 to its /64 prefix instead. IPv4 addresses are used as-is.
pub fn rate_key(ip: &str) -> String {
    match ip.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V6(v6)) => {
            let s = v6.segments();
            format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
        }
        _ => ip.to_string(),
    }
}

/// Mask an email for logs: keep the first character of the local part and the
/// domain, redact the rest — e.g. `alice@example.com` -> `a***@example.com`. Keeps
/// logs useful for debugging without writing full subscriber PII to disk.
pub fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((local, domain)) => {
            let first = local.chars().next().unwrap_or('*');
            format!("{first}***@{domain}")
        }
        None => "***".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_validation() {
        assert!(is_valid_email("a.b+tag@example.co.uk"));
        for bad in [
            "",
            "a@b",
            "@b.co",
            "a@@b.co",
            "a@b..co",
            "a@.b.co",
            "x<a@b.co>",
            "\"a b\"@b.co",
            "a\r\n@b.co",
            "ü@b.co",
        ] {
            assert!(!is_valid_email(bad), "{bad:?}");
        }
        assert!(!is_valid_email(&format!("{}@b.co", "a".repeat(65))));
        assert!(!is_valid_email(&format!("a@{}.co", "b".repeat(260))));
    }

    #[test]
    fn mailbox_key_collapses_aliases() {
        assert_eq!(mailbox_key("V.ictim+1@Gmail.com"), "victim@gmail.com");
        assert_eq!(mailbox_key("a.b+x@example.com"), "a.b@example.com");
    }

    #[test]
    fn rate_key_buckets_ipv6_by_64() {
        assert_eq!(
            rate_key("2001:db8:1:2:aaaa::1"),
            rate_key("2001:db8:1:2:bbbb::9")
        );
        assert_ne!(rate_key("2001:db8:1:2::1"), rate_key("2001:db8:1:3::1"));
        assert_eq!(rate_key("203.0.113.7"), "203.0.113.7");
    }
}
