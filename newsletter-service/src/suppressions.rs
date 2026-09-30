//! Cloudflare suppression-list sync, ported from `suppressions.ts`.
//!
//! Cloudflare auto-suppresses complaints / hard bounces but exposes no webhook,
//! so polling this list is our complaint-feedback-loop: it stops us from
//! re-sending to addresses that complained or permanently failed, which protects
//! sender reputation over time.

use serde::Serialize;

use crate::cf_email::EmailClient;
use crate::db::{self, Db};

// Reasons worth acting on. Soft bounces are transient; manual/policy entries
// aren't a subscriber-health signal.
const ACTIONABLE: [&str; 2] = ["complaint", "hard_bounce"];

#[derive(Serialize)]
pub struct SuppressionSyncResult {
    pub scanned: usize,
    pub actionable: usize,
    pub suppressed: usize,
}

pub async fn sync_suppressions(
    email: &EmailClient,
    db: &Db,
) -> anyhow::Result<SuppressionSyncResult> {
    let entries = email.list_suppressions().await;
    let scanned = entries.len();

    // Emails are stored lowercased, so lowercase before matching.
    let actionable: Vec<(String, String)> = entries
        .into_iter()
        .filter(|e| ACTIONABLE.contains(&e.reason.as_str()))
        .map(|e| (e.email.to_lowercase(), e.reason))
        .collect();
    let actionable_count = actionable.len();

    let suppressed = db
        .call(move |conn| {
            let mut n = 0usize;
            for (addr, reason) in &actionable {
                n += db::suppress_email(conn, addr, reason)?;
            }
            Ok(n)
        })
        .await?;

    Ok(SuppressionSyncResult {
        scanned,
        actionable: actionable_count,
        suppressed,
    })
}
