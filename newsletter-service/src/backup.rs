//! Database backups: a consistent, gzipped snapshot for the dashboard's download
//! button, and — when the R2_* settings are present — the same snapshot uploaded
//! to Cloudflare R2 about once a day.
//!
//! R2 speaks the S3 API, so an upload is one SigV4-signed `PUT`. The signing is
//! done here with `hmac` + `sha2` rather than pulling in an AWS SDK; its test
//! vectors come from botocore.

use std::io::Write;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use flate2::write::GzEncoder;
use flate2::Compression;
use hmac::{Hmac, Mac};
use rusqlite::OpenFlags;
use sha2::{Digest, Sha256};

use crate::config::R2Config;
use crate::db::{self, BackupRow, Db};

/// A consistent, gzipped copy of the live database.
///
/// `VACUUM INTO` writes a transactionally consistent snapshot through the
/// service's own connection — safe mid-write, unlike copying the file — which is
/// integrity-checked before it's handed out.
pub async fn snapshot_gz(db: &Db) -> Result<Vec<u8>> {
    let path = std::env::temp_dir().join(format!(
        "newsletter-snapshot-{}.db",
        crate::util::new_token()
    ));
    let result = async {
        let target = path.to_string_lossy().into_owned();
        db.call(move |c| {
            c.execute("VACUUM INTO ?", [target])?;
            Ok(())
        })
        .await?;

        let snapshot = path.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
            let conn =
                rusqlite::Connection::open_with_flags(&snapshot, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            let check: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
            if check != "ok" {
                bail!("snapshot failed integrity check: {check}");
            }
            drop(conn);

            let mut gz = GzEncoder::new(Vec::new(), Compression::default());
            gz.write_all(&std::fs::read(&snapshot)?)?;
            Ok(gz.finish()?)
        })
        .await?
    }
    .await;
    let _ = std::fs::remove_file(&path);
    result
}

/// `newsletter-2026-09-30T033000Z.db.gz`, the name for a backup taken at `at`.
pub fn file_name(at: DateTime<Utc>) -> String {
    format!("newsletter-{}.db.gz", at.format("%Y-%m-%dT%H%M%SZ"))
}

fn object_key(prefix: &str, at: DateTime<Utc>) -> String {
    if prefix.is_empty() {
        file_name(at)
    } else {
        format!("{prefix}/{}", file_name(at))
    }
}

/// Whether the daily backup is due: never succeeded, or the last success is 23+
/// hours old. (23, not 24, so hourly checks keep it daily instead of drifting.)
pub fn is_due(last_success_at: Option<&str>, now: DateTime<Utc>) -> bool {
    match last_success_at.and_then(|t| DateTime::parse_from_rfc3339(t).ok()) {
        Some(t) => now - t.with_timezone(&Utc) >= chrono::Duration::hours(23),
        None => true,
    }
}

/// Snapshot the database, upload it to R2, and record the attempt.
pub async fn run_r2_backup(db: &Db, http: &reqwest::Client, r2: &R2Config) -> Result<BackupRow> {
    let started = Utc::now();
    let key = object_key(&r2.prefix, started);
    let outcome = async {
        let body = snapshot_gz(db).await?;
        let bytes = body.len() as i64;
        upload(http, r2, &key, body, Utc::now()).await?;
        Ok::<_, anyhow::Error>(bytes)
    }
    .await;

    let row = BackupRow {
        started_at: started.to_rfc3339_opts(SecondsFormat::Millis, true),
        finished_at: crate::util::now_iso(),
        ok: outcome.is_ok(),
        object_key: Some(key),
        bytes: outcome.as_ref().ok().copied(),
        error: outcome.err().map(|e| format!("{e:#}")),
    };
    let record = row.clone();
    db.call(move |c| db::record_backup(c, &record)).await?;
    Ok(row)
}

// ---- S3 PUT with AWS Signature Version 4 ----

const SIGNED_HEADERS: &str = "content-type;host;x-amz-content-sha256;x-amz-date";
const CONTENT_TYPE: &str = "application/gzip";

async fn upload(
    http: &reqwest::Client,
    r2: &R2Config,
    key: &str,
    body: Vec<u8>,
    now: DateTime<Utc>,
) -> Result<()> {
    let endpoint = r2.endpoint.trim_end_matches('/');
    let host = endpoint.split_once("://").map_or(endpoint, |(_, h)| h);
    let path = uri_encode_path(&format!("/{}/{}", r2.bucket, key));
    let payload_hash = hex(&Sha256::digest(&body));
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let authorization = sign_put(
        host,
        &path,
        &payload_hash,
        &amz_date,
        &r2.access_key_id,
        &r2.secret_access_key,
    );

    let res = http
        .put(format!("{endpoint}{path}"))
        .header("content-type", CONTENT_TYPE)
        .header("x-amz-date", &amz_date)
        .header("x-amz-content-sha256", &payload_hash)
        .header("authorization", authorization)
        .body(body)
        // Under the 30s request timeout, so a dashboard-triggered backup still
        // records its outcome.
        .timeout(Duration::from_secs(25))
        .send()
        .await
        .context("R2 upload request failed")?;
    if !res.status().is_success() {
        let status = res.status();
        let detail: String = res
            .text()
            .await
            .unwrap_or_default()
            .chars()
            .take(300)
            .collect();
        bail!("R2 upload failed: HTTP {status}: {detail}");
    }
    Ok(())
}

/// The `Authorization` header for an S3 `PUT` of a gzip body, in R2's `auto` region.
fn sign_put(
    host: &str,
    path: &str,
    payload_hash: &str,
    amz_date: &str,
    access_key_id: &str,
    secret_access_key: &str,
) -> String {
    let date = &amz_date[..8];
    let canonical_request = format!(
        "PUT\n{path}\n\n\
         content-type:{CONTENT_TYPE}\nhost:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n\n\
         {SIGNED_HEADERS}\n{payload_hash}"
    );
    let scope = format!("{date}/auto/s3/aws4_request");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
        hex(&Sha256::digest(canonical_request.as_bytes()))
    );

    let key = hmac(format!("AWS4{secret_access_key}").as_bytes(), date);
    let key = hmac(&key, "auto");
    let key = hmac(&key, "s3");
    let key = hmac(&key, "aws4_request");
    let signature = hex(&hmac(&key, &string_to_sign));

    format!(
        "AWS4-HMAC-SHA256 Credential={access_key_id}/{scope}, SignedHeaders={SIGNED_HEADERS}, Signature={signature}"
    )
}

fn hmac(key: &[u8], data: &str) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts keys of any length");
    mac.update(data.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// S3's URI encoding for an object path: everything but unreserved characters and
/// `/` is percent-encoded (uppercase hex).
fn uri_encode_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for &b in path.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference signatures from botocore's `S3SigV4Auth` (region `auto`,
    /// service `s3`) for the same requests.
    #[test]
    fn signatures_match_botocore() {
        let (key, secret, date) = (
            "AKIDEXAMPLE0123",
            "secret/EXAMPLE+key0123456789",
            "20260930T040202Z",
        );
        let cases = [
            (
                "acct123.r2.cloudflarestorage.com",
                "/backups/newsletter/newsletter-2026-09-30T033000Z.db.gz",
                &b"hello backup"[..],
                "801bc151ed580552fd9ae431c768fc0fb0e0ddd81a08c7bba730338c07fca8a3",
            ),
            (
                "127.0.0.1:9000",
                "/b/x/y.db.gz",
                &b""[..],
                "1e4123f8fdbfeba9ab7b3afe302e2b7da543030562c5f5c4d70392bc951005d3",
            ),
        ];
        for (host, path, body, signature) in cases {
            let payload_hash = hex(&Sha256::digest(body));
            assert_eq!(
                sign_put(host, path, &payload_hash, date, key, secret),
                format!(
                    "AWS4-HMAC-SHA256 Credential={key}/20260930/auto/s3/aws4_request, \
                     SignedHeaders={SIGNED_HEADERS}, Signature={signature}"
                ),
                "{host}{path}"
            );
        }
    }

    #[test]
    fn object_paths_are_s3_encoded() {
        assert_eq!(
            uri_encode_path("/b/nl/newsletter-2026-09-30T033000Z.db.gz"),
            "/b/nl/newsletter-2026-09-30T033000Z.db.gz"
        );
        assert_eq!(uri_encode_path("/b/a b+c:d"), "/b/a%20b%2Bc%3Ad");
    }

    #[test]
    fn keys_carry_the_prefix_and_timestamp() {
        let at = DateTime::parse_from_rfc3339("2026-09-30T03:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(
            object_key("nl", at),
            "nl/newsletter-2026-09-30T033000Z.db.gz"
        );
        assert_eq!(object_key("", at), "newsletter-2026-09-30T033000Z.db.gz");
    }

    #[test]
    fn a_backup_is_due_about_daily() {
        let now = DateTime::parse_from_rfc3339("2026-09-30T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(is_due(None, now), "never backed up");
        assert!(!is_due(Some("2026-09-30T00:00:01.000Z"), now));
        assert!(is_due(Some("2026-09-29T13:00:00.000Z"), now), "23h ago");
        assert!(is_due(Some("garbage"), now));
    }
}
