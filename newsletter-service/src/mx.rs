//! "Can this domain receive mail?" check, run before a confirmation is sent.
//!
//! A confirmation to a typo'd or made-up domain is a guaranteed hard bounce, and
//! hard bounces are what sender reputation is judged on. Lookups go through
//! Cloudflare's DNS-over-HTTPS JSON API, reusing the HTTP client we already have
//! rather than adding a resolver dependency.
//!
//! Unlike Turnstile this FAILS OPEN: it is a quality filter, not a security gate,
//! so a DNS outage must not block real signups. Only a definite answer — the
//! domain doesn't exist, publishes a null MX, or has no MX/A/AAAA record — rejects.

use std::time::Duration;

use serde_json::Value;

const DOH_URL: &str = "https://cloudflare-dns.com/dns-query";
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(3);

const RCODE_NXDOMAIN: u64 = 3;
const TYPE_A: u64 = 1;
const TYPE_MX: u64 = 15;
const TYPE_AAAA: u64 = 28;

#[derive(Debug, PartialEq, Eq)]
enum Lookup {
    /// At least one usable record of the requested type.
    Found,
    /// The domain exists but has no record of this type.
    Empty,
    /// A definite "no mail here": NXDOMAIN, or a null MX (RFC 7505).
    Rejects,
    /// Couldn't tell (network error, SERVFAIL, unexpected shape).
    Unknown,
}

fn classify(body: &Value, record_type: u64) -> Lookup {
    let Some(status) = body.get("Status").and_then(Value::as_u64) else {
        return Lookup::Unknown;
    };
    if status == RCODE_NXDOMAIN {
        return Lookup::Rejects;
    }
    if status != 0 {
        return Lookup::Unknown;
    }

    let records: Vec<&str> = body
        .get("Answer")
        .and_then(Value::as_array)
        .map(|answers| {
            answers
                .iter()
                .filter(|a| a.get("type").and_then(Value::as_u64) == Some(record_type))
                .filter_map(|a| a.get("data").and_then(Value::as_str))
                .collect()
        })
        .unwrap_or_default();

    if records.is_empty() {
        return Lookup::Empty;
    }
    // Null MX: a single record with target "." — the domain accepts no mail.
    if record_type == TYPE_MX
        && records
            .iter()
            .all(|r| r.split_whitespace().last() == Some("."))
    {
        return Lookup::Rejects;
    }
    Lookup::Found
}

async fn lookup(http: &reqwest::Client, domain: &str, name: &str, record_type: u64) -> Lookup {
    let res = http
        .get(DOH_URL)
        .query(&[("name", domain), ("type", name)])
        .header("accept", "application/dns-json")
        .timeout(LOOKUP_TIMEOUT)
        .send()
        .await;
    match res {
        Ok(res) if res.status().is_success() => match res.json::<Value>().await {
            Ok(body) => classify(&body, record_type),
            Err(e) => {
                tracing::warn!(error = %e, "dns lookup parse failed");
                Lookup::Unknown
            }
        },
        Ok(res) => {
            tracing::warn!(status = %res.status(), "dns lookup failed");
            Lookup::Unknown
        }
        Err(e) => {
            tracing::warn!(error = %e, "dns lookup request failed");
            Lookup::Unknown
        }
    }
}

/// Whether mail to `domain` can be delivered. With no MX record, mail falls back
/// to the domain's address records (RFC 5321 §5.1), so those are checked too.
pub async fn domain_accepts_mail(http: &reqwest::Client, domain: &str) -> bool {
    match lookup(http, domain, "MX", TYPE_MX).await {
        Lookup::Found | Lookup::Unknown => return true,
        Lookup::Rejects => return false,
        Lookup::Empty => {}
    }
    for (name, record_type) in [("A", TYPE_A), ("AAAA", TYPE_AAAA)] {
        match lookup(http, domain, name, record_type).await {
            Lookup::Found | Lookup::Unknown => return true,
            Lookup::Rejects => return false,
            Lookup::Empty => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn classifies_dns_answers() {
        let mx = json!({ "Status": 0, "Answer": [{ "type": 15, "data": "10 mx.example.org." }] });
        assert_eq!(classify(&mx, TYPE_MX), Lookup::Found);

        let nxdomain = json!({ "Status": 3 });
        assert_eq!(classify(&nxdomain, TYPE_MX), Lookup::Rejects);

        let null_mx = json!({ "Status": 0, "Answer": [{ "type": 15, "data": "0 ." }] });
        assert_eq!(classify(&null_mx, TYPE_MX), Lookup::Rejects);

        let no_records = json!({ "Status": 0 });
        assert_eq!(classify(&no_records, TYPE_MX), Lookup::Empty);

        // A CNAME in the answer section is not a record of the requested type.
        let only_cname =
            json!({ "Status": 0, "Answer": [{ "type": 5, "data": "other.example.org." }] });
        assert_eq!(classify(&only_cname, TYPE_A), Lookup::Empty);

        let servfail = json!({ "Status": 2 });
        assert_eq!(classify(&servfail, TYPE_MX), Lookup::Unknown);
        assert_eq!(classify(&json!("garbage"), TYPE_MX), Lookup::Unknown);
    }
}
