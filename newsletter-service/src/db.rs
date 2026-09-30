//! SQLite storage, ported from `db.ts`.
//!
//! Schema, table names and column names are IDENTICAL to the Node service so this
//! binary opens an existing `newsletter.db` unchanged (drop-in). A few extra
//! indexes are added (they only speed up admin queries — non-breaking).
//!
//! rusqlite is synchronous; to keep it off the async runtime, all access goes
//! through [`Db::call`], which runs the closure on a blocking thread. SQLite
//! allows one writer at a time anyway, so a single mutex-guarded connection is
//! all this workload needs (no pool).

use std::sync::{Arc, Mutex};

use anyhow::Result;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
use serde::Serialize;

use crate::config::ListName;

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl Db {
    /// Open (creating parent dir + schema if needed) the SQLite database.
    pub fn open(path: &str) -> Result<Db> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;\
             PRAGMA busy_timeout = 5000;\
             PRAGMA foreign_keys = ON;\
             PRAGMA synchronous = NORMAL;",
        )?;
        init_schema(&conn)?;

        Ok(Db {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Run a closure with the connection on a blocking thread. All DB access in
    /// the service goes through here.
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap_or_else(|e| e.into_inner());
            f(&conn)
        })
        .await?
    }
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS subscriptions (
            id            INTEGER PRIMARY KEY,
            email         TEXT NOT NULL,
            list          TEXT NOT NULL CHECK (list IN ('blog','notes')),
            status        TEXT NOT NULL CHECK (status IN ('pending','confirmed','unsubscribed','bounced')),
            confirm_token TEXT NOT NULL,
            unsub_token   TEXT NOT NULL,
            created_at    TEXT NOT NULL,
            confirmed_at  TEXT,
            UNIQUE(email, list)
        );

        CREATE INDEX IF NOT EXISTS idx_sub_confirm ON subscriptions(confirm_token);
        CREATE INDEX IF NOT EXISTS idx_sub_unsub   ON subscriptions(unsub_token);
        CREATE INDEX IF NOT EXISTS idx_sub_list    ON subscriptions(list, status);

        CREATE TABLE IF NOT EXISTS sent_posts (
            slug       TEXT NOT NULL,
            list       TEXT NOT NULL,
            sent_at    TEXT NOT NULL,
            recipients INTEGER NOT NULL,
            PRIMARY KEY (slug, list)
        );

        CREATE TABLE IF NOT EXISTS send_log (
            ts TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS deliveries (
            slug    TEXT NOT NULL,
            list    TEXT NOT NULL,
            email   TEXT NOT NULL,
            sent_at TEXT NOT NULL,
            PRIMARY KEY (slug, list, email)
        );

        -- Per-email confirmation-send ledger, keyed by email so the resend
        -- cooldown can be claimed atomically BEFORE a send (independent of whether
        -- a subscription row exists yet). This is what makes the anti-inbox-bomb
        -- guard race-free under concurrent /subscribe requests for one address.
        CREATE TABLE IF NOT EXISTS confirm_sends (
            email     TEXT PRIMARY KEY,
            last_sent TEXT NOT NULL
        );

        -- One row per automatic R2 backup attempt (the latest 100 are kept).
        CREATE TABLE IF NOT EXISTS backups (
            id          INTEGER PRIMARY KEY,
            started_at  TEXT NOT NULL,
            finished_at TEXT NOT NULL,
            ok          INTEGER NOT NULL,
            object_key  TEXT,
            bytes       INTEGER,
            error       TEXT
        );
        "#,
    )?;

    // Lightweight migrations: add columns to existing DBs.
    ensure_column(conn, "subscriptions", "unsub_reason", "unsub_reason TEXT")?;
    ensure_column(conn, "subscriptions", "unsub_note", "unsub_note TEXT")?;
    ensure_column(
        conn,
        "subscriptions",
        "last_confirm_sent_at",
        "last_confirm_sent_at TEXT",
    )?;
    ensure_column(
        conn,
        "subscriptions",
        "suppression_reason",
        "suppression_reason TEXT",
    )?;
    // Set while a confirmation email is waiting for a free send slot (a cap was
    // hit at signup); cleared once it is sent or dropped.
    ensure_column(
        conn,
        "subscriptions",
        "confirm_queued_at",
        "confirm_queued_at TEXT",
    )?;

    // Added (non-breaking) indexes to keep the admin analytics/search fast as the
    // table grows. The Node service didn't have these; they only help.
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_sub_email      ON subscriptions(email);\
         CREATE INDEX IF NOT EXISTS idx_sub_created    ON subscriptions(created_at);\
         CREATE INDEX IF NOT EXISTS idx_send_log_ts    ON send_log(ts);",
    )?;

    Ok(())
}

fn ensure_column(conn: &Connection, table: &str, column: &str, ddl: &str) -> Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let existing: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<_>>()?;
    if !existing.iter().any(|c| c == column) {
        conn.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {ddl}"))?;
    }
    Ok(())
}

// ---- subscribe / confirm / unsubscribe ----

pub enum UpsertResult {
    Pending,
    AlreadyConfirmed,
}

/// Upsert a `(email, list)` subscription into the `pending` state using the
/// caller-supplied confirm token (shared across all lists in one signup). Returns
/// whether a confirmation still needs to be sent, or that it's already confirmed.
///
/// `queued_at` is set when the confirmation email can't go out yet (a send cap was
/// hit) and is left for the queue drain; `None` means it is being sent right now.
pub fn upsert_pending(
    conn: &Connection,
    email: &str,
    list: ListName,
    confirm_token: &str,
    now: &str,
    queued_at: Option<&str>,
) -> Result<UpsertResult> {
    let existing: Option<(i64, String)> = conn
        .query_row(
            "SELECT id, status FROM subscriptions WHERE email = ? AND list = ?",
            params![email, list.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;

    match existing {
        Some((_, status)) if status == "confirmed" => Ok(UpsertResult::AlreadyConfirmed),
        Some((id, _)) => {
            // pending / unsubscribed -> restart the double opt-in flow. (Suppressed
            // addresses never get here: `/subscribe` checks `is_suppressed` first.)
            // `last_confirm_sent_at` is when this pending request started — the
            // clock `expire_pending` runs on, since `created_at` may be years old.
            conn.execute(
                "UPDATE subscriptions SET status = 'pending', confirm_token = ?, confirmed_at = NULL,
                        last_confirm_sent_at = ?, confirm_queued_at = ? WHERE id = ?",
                params![confirm_token, now, queued_at, id],
            )?;
            Ok(UpsertResult::Pending)
        }
        None => {
            conn.execute(
                "INSERT INTO subscriptions
                 (email, list, status, confirm_token, unsub_token, created_at, last_confirm_sent_at, confirm_queued_at)
                 VALUES (?, ?, 'pending', ?, ?, ?, ?, ?)",
                params![
                    email,
                    list.as_str(),
                    confirm_token,
                    crate::util::new_token(),
                    now,
                    now,
                    queued_at
                ],
            )?;
            Ok(UpsertResult::Pending)
        }
    }
}

/// True if any subscription for this address is suppressed (hard bounce or spam
/// complaint). Such an address must not be mailed again, confirmations included.
pub fn is_suppressed(conn: &Connection, email: &str) -> Result<bool> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM subscriptions WHERE email = ? AND status = 'bounced' LIMIT 1",
            params![email],
            |r| r.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

// ---- queued confirmations (a send cap was hit at signup) ----

pub struct QueuedConfirmation {
    pub email: String,
    pub confirm_token: String,
    pub lists: Vec<ListName>,
}

/// Queued confirmation emails — one per signup, however many lists it covers.
pub fn count_queued_confirmations(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(DISTINCT confirm_token) FROM subscriptions
         WHERE status = 'pending' AND confirm_queued_at IS NOT NULL",
        [],
        |r| r.get(0),
    )?)
}

/// The longest-waiting queued confirmation, with every list it covers.
pub fn next_queued_confirmation(conn: &Connection) -> Result<Option<QueuedConfirmation>> {
    let head: Option<(String, String)> = conn
        .query_row(
            "SELECT email, confirm_token FROM subscriptions
             WHERE status = 'pending' AND confirm_queued_at IS NOT NULL
             ORDER BY confirm_queued_at LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((email, confirm_token)) = head else {
        return Ok(None);
    };

    let mut stmt = conn.prepare_cached(
        "SELECT list FROM subscriptions
         WHERE email = ? AND confirm_token = ? AND status = 'pending' AND confirm_queued_at IS NOT NULL",
    )?;
    let lists = stmt
        .query_map(params![email, confirm_token], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .iter()
        .filter_map(|l| ListName::parse(l))
        .collect();

    Ok(Some(QueuedConfirmation {
        email,
        confirm_token,
        lists,
    }))
}

/// Take a confirmation off the queue (it was sent, or is being dropped).
pub fn dequeue_confirmation(conn: &Connection, confirm_token: &str) -> Result<()> {
    conn.execute(
        "UPDATE subscriptions SET confirm_queued_at = NULL WHERE confirm_token = ?",
        params![confirm_token],
    )?;
    Ok(())
}

/// Drop queued confirmations that have waited since before `cutoff_iso`. The
/// pending rows stay until `expire_pending` removes them. Returns rows changed.
pub fn drop_stale_queued(conn: &Connection, cutoff_iso: &str) -> Result<usize> {
    Ok(conn.execute(
        "UPDATE subscriptions SET confirm_queued_at = NULL
         WHERE confirm_queued_at IS NOT NULL AND confirm_queued_at < ?",
        params![cutoff_iso],
    )?)
}

// ---- retention ----

/// Delete subscriptions that were never confirmed and whose latest signup request
/// predates `cutoff_iso`. Returns rows deleted.
pub fn expire_pending(conn: &Connection, cutoff_iso: &str) -> Result<usize> {
    Ok(conn.execute(
        "DELETE FROM subscriptions
         WHERE status = 'pending' AND COALESCE(last_confirm_sent_at, created_at) < ?",
        params![cutoff_iso],
    )?)
}

/// Drop resend-cooldown ledger entries older than `cutoff_iso` — past the cooldown
/// they only serve to retain addresses.
pub fn prune_confirm_sends(conn: &Connection, cutoff_iso: &str) -> Result<usize> {
    Ok(conn.execute(
        "DELETE FROM confirm_sends WHERE last_sent < ?",
        params![cutoff_iso],
    )?)
}

/// Erase everything stored about an address: its subscriptions (all lists), its
/// delivery records, and its resend-cooldown entry (`mailbox` is the normalized
/// key that ledger uses). Returns the number of subscriptions deleted.
pub fn delete_subscriber(conn: &Connection, email: &str, mailbox: &str) -> Result<usize> {
    let deleted = conn.execute("DELETE FROM subscriptions WHERE email = ?", params![email])?;
    conn.execute("DELETE FROM deliveries WHERE email = ?", params![email])?;
    conn.execute(
        "DELETE FROM confirm_sends WHERE email IN (?, ?)",
        params![email, mailbox],
    )?;
    Ok(deleted)
}

/// Confirm every pending subscription sharing this confirm token. Returns true if
/// the token matched any row (idempotent — already-confirmed rows still count).
pub fn confirm_by_token(conn: &Connection, confirm_token: &str, now: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM subscriptions WHERE confirm_token = ?",
        params![confirm_token],
        |r| r.get(0),
    )?;
    if count == 0 {
        return Ok(false);
    }
    conn.execute(
        "UPDATE subscriptions SET status = 'confirmed', confirmed_at = ?, confirm_queued_at = NULL
         WHERE confirm_token = ? AND status = 'pending'",
        params![now, confirm_token],
    )?;
    Ok(true)
}

fn list_by_unsub(conn: &Connection, unsub_token: &str) -> Result<Option<(i64, String, String)>> {
    Ok(conn
        .query_row(
            "SELECT id, list, status FROM subscriptions WHERE unsub_token = ?",
            params![unsub_token],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?)
}

/// Unsubscribe by unsub token. Returns the affected list, or None if no match.
pub fn unsubscribe_by_token(conn: &Connection, unsub_token: &str) -> Result<Option<ListName>> {
    let Some((id, list, status)) = list_by_unsub(conn, unsub_token)? else {
        return Ok(None);
    };
    if status != "unsubscribed" {
        conn.execute(
            "UPDATE subscriptions SET status = 'unsubscribed' WHERE id = ?",
            params![id],
        )?;
    }
    Ok(ListName::parse(&list))
}

/// Resubscribe (undo) by unsub token — restores the row straight to `confirmed`.
pub fn resubscribe_by_token(
    conn: &Connection,
    unsub_token: &str,
    now: &str,
) -> Result<Option<ListName>> {
    let Some((id, list, status)) = list_by_unsub(conn, unsub_token)? else {
        return Ok(None);
    };
    if status != "confirmed" {
        conn.execute(
            "UPDATE subscriptions SET status = 'confirmed', confirmed_at = ? WHERE id = ?",
            params![now, id],
        )?;
    }
    Ok(ListName::parse(&list))
}

/// Attach an optional unsubscribe reason + free-text note to a row. Returns true
/// if the token matched a row.
pub fn set_unsub_reason(
    conn: &Connection,
    unsub_token: &str,
    reason: &str,
    note: &str,
) -> Result<bool> {
    let changed = conn.execute(
        "UPDATE subscriptions SET unsub_reason = ?, unsub_note = ? WHERE unsub_token = ?",
        params![
            (!reason.is_empty()).then_some(reason),
            (!note.is_empty()).then_some(note),
            unsub_token
        ],
    )?;
    Ok(changed > 0)
}

// ---- sending ----

pub struct ConfirmedRecipient {
    pub email: String,
    pub unsub_token: String,
}

pub fn get_confirmed(conn: &Connection, list: ListName) -> Result<Vec<ConfirmedRecipient>> {
    let mut stmt = conn.prepare_cached(
        "SELECT email, unsub_token FROM subscriptions WHERE list = ? AND status = 'confirmed'",
    )?;
    let rows = stmt
        .query_map(params![list.as_str()], |r| {
            Ok(ConfirmedRecipient {
                email: r.get(0)?,
                unsub_token: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

/// Count confirmed subscribers for a list without materializing the rows (used by
/// the admin recipient-count endpoint).
pub fn count_confirmed(conn: &Connection, list: ListName) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM subscriptions WHERE list = ? AND status = 'confirmed'",
        params![list.as_str()],
        |r| r.get(0),
    )?)
}

pub fn mark_bounced(conn: &Connection, email: &str, list: ListName) -> Result<()> {
    conn.execute(
        "UPDATE subscriptions SET status = 'bounced' WHERE email = ? AND list = ?",
        params![email, list.as_str()],
    )?;
    Ok(())
}

pub fn has_sent(conn: &Connection, slug: &str, list: ListName) -> Result<bool> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM sent_posts WHERE slug = ? AND list = ?",
            params![slug, list.as_str()],
            |r| r.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

pub fn record_sent(
    conn: &Connection,
    slug: &str,
    list: ListName,
    now: &str,
    recipients: i64,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO sent_posts (slug, list, sent_at, recipients) VALUES (?, ?, ?, ?)",
        params![slug, list.as_str(), now, recipients],
    )?;
    Ok(())
}

pub fn was_delivered(conn: &Connection, slug: &str, list: ListName, email: &str) -> Result<bool> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM deliveries WHERE slug = ? AND list = ? AND email = ?",
            params![slug, list.as_str(), email],
            |r| r.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

pub fn record_delivery(
    conn: &Connection,
    slug: &str,
    list: ListName,
    email: &str,
    now: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO deliveries (slug, list, email, sent_at) VALUES (?, ?, ?, ?)",
        params![slug, list.as_str(), email, now],
    )?;
    Ok(())
}

// ---- global hourly confirmation cap (send_log) ----

pub fn prune_send_log_before(conn: &Connection, iso: &str) -> Result<()> {
    conn.execute("DELETE FROM send_log WHERE ts < ?", params![iso])?;
    Ok(())
}

/// Atomically reserve one slot against the global confirmation caps: insert a
/// `send_log` row IFF fewer than `hour_cap` rows exist since `hour_ago_iso` AND
/// fewer than `day_cap` since `day_ago_iso`. Returns true if a slot was reserved
/// (i.e. we're under both ceilings), false if either cap is hit.
///
/// The counts and the insert happen in a single SQL statement, so concurrent
/// `/subscribe` requests can't all observe "under cap" and then each send — this
/// is what makes the cap a genuine hard ceiling rather than a soft, race-prone
/// check. Reserving BEFORE the send means a failed send still consumes a slot
/// (conservative — errs toward under-spending, which is the safe direction).
pub fn reserve_confirm_slot(
    conn: &Connection,
    now: &str,
    hour_ago_iso: &str,
    hour_cap: i64,
    day_ago_iso: &str,
    day_cap: i64,
) -> Result<bool> {
    let changed = conn.execute(
        "INSERT INTO send_log (ts)
         SELECT ?1 WHERE (SELECT COUNT(*) FROM send_log WHERE ts >= ?2) < ?3
                     AND (SELECT COUNT(*) FROM send_log WHERE ts >= ?4) < ?5",
        params![now, hour_ago_iso, hour_cap, day_ago_iso, day_cap],
    )?;
    Ok(changed > 0)
}

// ---- per-email resend cooldown (atomic claim) ----

/// Atomically claim the right to send a confirmation to `email`: succeeds (returns
/// true) only if no confirmation was sent to this address at or after `cutoff_iso`
/// (i.e. the cooldown has elapsed) or none was ever sent. The claim is recorded in
/// the same statement, so N concurrent requests for the SAME address produce
/// exactly ONE winner — closing the TOCTOU that would otherwise let a botnet
/// inbox-bomb a victim with simultaneous requests. Keyed by email, so it works
/// even before a subscription row exists.
pub fn claim_confirm_send(
    conn: &Connection,
    email: &str,
    now: &str,
    cutoff_iso: &str,
) -> Result<bool> {
    let changed = conn.execute(
        "INSERT INTO confirm_sends (email, last_sent) VALUES (?1, ?2)
         ON CONFLICT(email) DO UPDATE SET last_sent = ?2 WHERE confirm_sends.last_sent < ?3",
        params![email, now, cutoff_iso],
    )?;
    Ok(changed > 0)
}

// ---- backups ----

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BackupRow {
    pub started_at: String,
    pub finished_at: String,
    pub ok: bool,
    pub object_key: Option<String>,
    pub bytes: Option<i64>,
    pub error: Option<String>,
}

pub fn record_backup(conn: &Connection, row: &BackupRow) -> Result<()> {
    conn.execute(
        "INSERT INTO backups (started_at, finished_at, ok, object_key, bytes, error)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![
            row.started_at,
            row.finished_at,
            row.ok,
            row.object_key,
            row.bytes,
            row.error
        ],
    )?;
    conn.execute(
        "DELETE FROM backups WHERE id NOT IN (SELECT id FROM backups ORDER BY id DESC LIMIT 100)",
        [],
    )?;
    Ok(())
}

fn backup_row(r: &rusqlite::Row) -> rusqlite::Result<BackupRow> {
    Ok(BackupRow {
        started_at: r.get(0)?,
        finished_at: r.get(1)?,
        ok: r.get(2)?,
        object_key: r.get(3)?,
        bytes: r.get(4)?,
        error: r.get(5)?,
    })
}

/// The most recent backup attempt, and the most recent successful one.
pub fn backup_status(conn: &Connection) -> Result<(Option<BackupRow>, Option<BackupRow>)> {
    const COLS: &str = "started_at, finished_at, ok, object_key, bytes, error";
    let last = conn
        .query_row(
            &format!("SELECT {COLS} FROM backups ORDER BY id DESC LIMIT 1"),
            [],
            backup_row,
        )
        .optional()?;
    let last_ok = conn
        .query_row(
            &format!("SELECT {COLS} FROM backups WHERE ok = 1 ORDER BY id DESC LIMIT 1"),
            [],
            backup_row,
        )
        .optional()?;
    Ok((last, last_ok))
}

// ---- suppression sync ----

/// Mark every subscription for this address as bounced with the given Cloudflare
/// suppression reason, skipping rows already bounced. Returns rows changed.
pub fn suppress_email(conn: &Connection, email: &str, reason: &str) -> Result<usize> {
    Ok(conn.execute(
        "UPDATE subscriptions SET status = 'bounced', suppression_reason = ? WHERE email = ? AND status != 'bounced'",
        params![reason, email],
    )?)
}

// ---- admin analytics ----

#[derive(Serialize)]
pub struct Totals {
    #[serde(rename = "uniqueEmails")]
    pub unique_emails: i64,
    pub confirmed: i64,
    pub pending: i64,
    pub unsubscribed: i64,
    pub bounced: i64,
    pub complaints: i64,
    /// Confirmation emails waiting for a free send slot. Non-zero means a send cap
    /// was hit — the signal of a signup flood.
    #[serde(rename = "queuedConfirmations")]
    pub queued_confirmations: i64,
}

#[derive(Serialize, Default)]
pub struct ListCounts {
    pub confirmed: i64,
    pub pending: i64,
    pub unsubscribed: i64,
    pub bounced: i64,
}

#[derive(Serialize)]
pub struct ByList {
    pub blog: ListCounts,
    pub notes: ListCounts,
}

#[derive(Serialize)]
pub struct Newsletter {
    #[serde(rename = "postsSent")]
    pub posts_sent: i64,
    #[serde(rename = "emailsDelivered")]
    pub emails_delivered: i64,
}

#[derive(Serialize)]
pub struct Signups {
    pub last7d: i64,
    pub last30d: i64,
}

#[derive(Serialize)]
pub struct UnsubReason {
    pub reason: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct Stats {
    pub totals: Totals,
    #[serde(rename = "byList")]
    pub by_list: ByList,
    pub newsletter: Newsletter,
    pub signups: Signups,
    #[serde(rename = "unsubReasons")]
    pub unsub_reasons: Vec<UnsubReason>,
}

fn apply_status(counts: &mut ListCounts, status: &str, c: i64) {
    match status {
        "confirmed" => counts.confirmed = c,
        "pending" => counts.pending = c,
        "unsubscribed" => counts.unsubscribed = c,
        "bounced" => counts.bounced = c,
        _ => {}
    }
}

pub fn get_stats(conn: &Connection, since7d: &str, since30d: &str) -> Result<Stats> {
    let mut totals = Totals {
        unique_emails: 0,
        confirmed: 0,
        pending: 0,
        unsubscribed: 0,
        bounced: 0,
        complaints: 0,
        queued_confirmations: count_queued_confirmations(conn)?,
    };

    {
        let mut stmt =
            conn.prepare("SELECT status, COUNT(*) FROM subscriptions GROUP BY status")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
        for row in rows {
            let (status, c) = row?;
            match status.as_str() {
                "confirmed" => totals.confirmed = c,
                "pending" => totals.pending = c,
                "unsubscribed" => totals.unsubscribed = c,
                "bounced" => totals.bounced = c,
                _ => {}
            }
        }
    }

    totals.unique_emails =
        conn.query_row("SELECT COUNT(DISTINCT email) FROM subscriptions", [], |r| {
            r.get(0)
        })?;
    // Complaints are a subset of `bounced` (marked bounced with a reason); surfaced
    // separately because they're the reputation-critical signal.
    totals.complaints = conn.query_row(
        "SELECT COUNT(*) FROM subscriptions WHERE suppression_reason = 'complaint'",
        [],
        |r| r.get(0),
    )?;

    let mut by_list = ByList {
        blog: ListCounts::default(),
        notes: ListCounts::default(),
    };
    {
        let mut stmt =
            conn.prepare("SELECT list, status, COUNT(*) FROM subscriptions GROUP BY list, status")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in rows {
            let (list, status, c) = row?;
            match list.as_str() {
                "blog" => apply_status(&mut by_list.blog, &status, c),
                "notes" => apply_status(&mut by_list.notes, &status, c),
                _ => {}
            }
        }
    }

    let (posts, delivered): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(recipients), 0) FROM sent_posts",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    let signups_since = |since: &str| -> Result<i64> {
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM subscriptions WHERE created_at >= ?",
            params![since],
            |r| r.get(0),
        )?)
    };

    let unsub_reasons = {
        let mut stmt = conn.prepare(
            "SELECT unsub_reason, COUNT(*) FROM subscriptions
             WHERE unsub_reason IS NOT NULL AND unsub_reason != '' GROUP BY unsub_reason ORDER BY COUNT(*) DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(UnsubReason {
                reason: r.get(0)?,
                count: r.get(1)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    Ok(Stats {
        totals,
        by_list,
        newsletter: Newsletter {
            posts_sent: posts,
            emails_delivered: delivered,
        },
        signups: Signups {
            last7d: signups_since(since7d)?,
            last30d: signups_since(since30d)?,
        },
        unsub_reasons,
    })
}

#[derive(Serialize)]
pub struct SubscriberRow {
    pub email: String,
    pub list: String,
    pub status: String,
    pub created_at: String,
    pub confirmed_at: Option<String>,
}

#[derive(Serialize)]
pub struct SubscriberPage {
    pub rows: Vec<SubscriberRow>,
    pub total: i64,
}

pub struct SubscriberQuery {
    pub q: Option<String>,
    pub list: Option<String>,
    pub status: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

pub fn list_subscribers(conn: &Connection, opts: SubscriberQuery) -> Result<SubscriberPage> {
    // Build the WHERE clause with bound parameters — never string-interpolate
    // user input into SQL.
    let mut where_clauses: Vec<&str> = Vec::new();
    let mut binds: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(q) = &opts.q {
        where_clauses.push("email LIKE ?");
        binds.push(Box::new(format!("%{q}%")));
    }
    if let Some(list) = &opts.list {
        where_clauses.push("list = ?");
        binds.push(Box::new(list.clone()));
    }
    if let Some(status) = &opts.status {
        where_clauses.push("status = ?");
        binds.push(Box::new(status.clone()));
    }
    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM subscriptions {where_sql}"),
        params_from_iter(binds.iter().map(|b| b.as_ref())),
        |r| r.get(0),
    )?;

    let sql = format!(
        "SELECT email, list, status, created_at, confirmed_at FROM subscriptions
         {where_sql} ORDER BY created_at DESC LIMIT ? OFFSET ?"
    );
    let mut stmt = conn.prepare(&sql)?;
    // Append limit/offset after the WHERE binds, in order.
    let mut row_binds: Vec<Box<dyn rusqlite::ToSql>> = binds;
    row_binds.push(Box::new(opts.limit));
    row_binds.push(Box::new(opts.offset));
    let rows = stmt
        .query_map(
            params_from_iter(row_binds.iter().map(|b| b.as_ref())),
            |r| {
                Ok(SubscriberRow {
                    email: r.get(0)?,
                    list: r.get(1)?,
                    status: r.get(2)?,
                    created_at: r.get(3)?,
                    confirmed_at: r.get(4)?,
                })
            },
        )?
        .collect::<rusqlite::Result<_>>()?;

    Ok(SubscriberPage { rows, total })
}

#[derive(Serialize)]
pub struct SentPostRow {
    pub slug: String,
    pub list: String,
    pub sent_at: String,
    pub recipients: i64,
}

pub fn list_sent_posts(conn: &Connection, limit: i64) -> Result<Vec<SentPostRow>> {
    let mut stmt = conn.prepare(
        "SELECT slug, list, sent_at, recipients FROM sent_posts ORDER BY sent_at DESC LIMIT ?",
    )?;
    let rows = stmt
        .query_map(params![limit], |r| {
            Ok(SentPostRow {
                slug: r.get(0)?,
                list: r.get(1)?,
                sent_at: r.get(2)?,
                recipients: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    fn status_of(conn: &Connection, email: &str) -> Option<String> {
        conn.query_row(
            "SELECT status FROM subscriptions WHERE email = ?",
            params![email],
            |r| r.get(0),
        )
        .optional()
        .unwrap()
    }

    #[test]
    fn confirm_caps_are_hourly_and_daily() {
        let conn = test_conn();
        let (hour_ago, day_ago) = ("2026-01-02T11:00:00.000Z", "2026-01-01T12:00:00.000Z");
        // Two sends earlier in the day, outside the last hour.
        for ts in ["2026-01-02T01:00:00.000Z", "2026-01-02T02:00:00.000Z"] {
            conn.execute("INSERT INTO send_log (ts) VALUES (?)", params![ts])
                .unwrap();
        }
        let now = "2026-01-02T12:00:00.000Z";
        let reserve = |hour_cap, day_cap| {
            reserve_confirm_slot(&conn, now, hour_ago, hour_cap, day_ago, day_cap)
        };

        assert!(!reserve(5, 2).unwrap(), "daily cap already reached");
        assert!(reserve(1, 5).unwrap());
        assert!(!reserve(1, 5).unwrap(), "hourly cap now reached");
    }

    #[test]
    fn queued_confirmations_drain_oldest_first() {
        let conn = test_conn();
        let (t1, t2) = ("2026-01-01T00:00:00.000Z", "2026-01-01T00:05:00.000Z");
        upsert_pending(&conn, "b@x.co", ListName::Blog, "tok-b", t2, Some(t2)).unwrap();
        upsert_pending(&conn, "a@x.co", ListName::Blog, "tok-a", t1, Some(t1)).unwrap();
        upsert_pending(&conn, "a@x.co", ListName::Notes, "tok-a", t1, Some(t1)).unwrap();
        upsert_pending(&conn, "now@x.co", ListName::Blog, "tok-now", t1, None).unwrap();
        assert_eq!(count_queued_confirmations(&conn).unwrap(), 2);

        let head = next_queued_confirmation(&conn).unwrap().unwrap();
        assert_eq!(head.email, "a@x.co");
        assert_eq!(head.lists.len(), 2);

        dequeue_confirmation(&conn, &head.confirm_token).unwrap();
        let head = next_queued_confirmation(&conn).unwrap().unwrap();
        assert_eq!(head.email, "b@x.co");

        assert_eq!(
            drop_stale_queued(&conn, "2026-01-02T00:00:00.000Z").unwrap(),
            1
        );
        assert!(next_queued_confirmation(&conn).unwrap().is_none());
        assert_eq!(status_of(&conn, "b@x.co").as_deref(), Some("pending"));
    }

    #[test]
    fn expiry_runs_on_the_latest_signup_request() {
        let conn = test_conn();
        let (old, recent) = ("2025-01-01T00:00:00.000Z", "2026-01-10T00:00:00.000Z");
        let cutoff = "2026-01-05T00:00:00.000Z";

        upsert_pending(&conn, "stale@x.co", ListName::Blog, "t1", old, None).unwrap();
        // Signed up long ago, unsubscribed, and asked to re-subscribe recently.
        upsert_pending(&conn, "back@x.co", ListName::Blog, "t2", old, None).unwrap();
        confirm_by_token(&conn, "t2", old).unwrap();
        upsert_pending(&conn, "kept@x.co", ListName::Blog, "t4", old, None).unwrap();
        confirm_by_token(&conn, "t4", old).unwrap();
        let unsub: String = conn
            .query_row(
                "SELECT unsub_token FROM subscriptions WHERE email = 'back@x.co'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        unsubscribe_by_token(&conn, &unsub).unwrap();
        upsert_pending(&conn, "back@x.co", ListName::Blog, "t3", recent, None).unwrap();

        assert_eq!(expire_pending(&conn, cutoff).unwrap(), 1);
        assert_eq!(status_of(&conn, "stale@x.co"), None);
        assert_eq!(status_of(&conn, "back@x.co").as_deref(), Some("pending"));
        assert_eq!(status_of(&conn, "kept@x.co").as_deref(), Some("confirmed"));
    }

    #[test]
    fn suppression_and_delete() {
        let conn = test_conn();
        let now = "2026-01-01T00:00:00.000Z";
        upsert_pending(&conn, "a@x.co", ListName::Blog, "t1", now, None).unwrap();
        upsert_pending(&conn, "a@x.co", ListName::Notes, "t1", now, None).unwrap();
        record_delivery(&conn, "post", ListName::Blog, "a@x.co", now).unwrap();
        claim_confirm_send(&conn, "a@x.co", now, now).unwrap();

        assert!(!is_suppressed(&conn, "a@x.co").unwrap());
        suppress_email(&conn, "a@x.co", "complaint").unwrap();
        assert!(is_suppressed(&conn, "a@x.co").unwrap());

        assert_eq!(delete_subscriber(&conn, "a@x.co", "a@x.co").unwrap(), 2);
        assert!(!is_suppressed(&conn, "a@x.co").unwrap());
        assert!(!was_delivered(&conn, "post", ListName::Blog, "a@x.co").unwrap());
        let left: i64 = conn
            .query_row("SELECT COUNT(*) FROM confirm_sends", [], |r| r.get(0))
            .unwrap();
        assert_eq!(left, 0);
    }

    fn tokens_of(conn: &Connection, email: &str, list: ListName) -> (String, String) {
        conn.query_row(
            "SELECT confirm_token, unsub_token FROM subscriptions WHERE email = ? AND list = ?",
            params![email, list.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
    }

    fn list_status(conn: &Connection, email: &str, list: ListName) -> String {
        conn.query_row(
            "SELECT status FROM subscriptions WHERE email = ? AND list = ?",
            params![email, list.as_str()],
            |r| r.get(0),
        )
        .unwrap()
    }

    #[test]
    fn resend_cooldown_has_one_winner_per_window() {
        let conn = test_conn();
        let (t0, t1, t2) = (
            "2026-01-01T00:00:00.000Z",
            "2026-01-01T00:10:00.000Z",
            "2026-01-01T00:20:00.000Z",
        );
        assert!(claim_confirm_send(&conn, "v@x.co", t0, "2025-12-31T23:45:00.000Z").unwrap());
        // 10 minutes later, 15-minute cooldown: cutoff is before the last send.
        assert!(!claim_confirm_send(&conn, "v@x.co", t1, "2025-12-31T23:55:00.000Z").unwrap());
        assert!(claim_confirm_send(&conn, "w@x.co", t1, "2025-12-31T23:55:00.000Z").unwrap());
        // The losing claim must not push the window forward.
        assert!(claim_confirm_send(&conn, "v@x.co", t2, "2026-01-01T00:05:00.000Z").unwrap());
        assert!(!claim_confirm_send(&conn, "v@x.co", t2, "2026-01-01T00:05:00.000Z").unwrap());
    }

    #[test]
    fn upsert_restarts_opt_in_but_never_touches_confirmed_rows() {
        let conn = test_conn();
        let now = "2026-01-01T00:00:00.000Z";
        let later = "2026-01-02T00:00:00.000Z";
        upsert_pending(&conn, "a@x.co", ListName::Blog, "t1", now, None).unwrap();
        confirm_by_token(&conn, "t1", now).unwrap();
        let (_, unsub) = tokens_of(&conn, "a@x.co", ListName::Blog);

        let again = upsert_pending(&conn, "a@x.co", ListName::Blog, "t2", later, None).unwrap();
        assert!(matches!(again, UpsertResult::AlreadyConfirmed));
        assert_eq!(list_status(&conn, "a@x.co", ListName::Blog), "confirmed");
        assert_eq!(
            tokens_of(&conn, "a@x.co", ListName::Blog),
            ("t1".into(), unsub.clone())
        );

        unsubscribe_by_token(&conn, &unsub).unwrap();
        let back = upsert_pending(&conn, "a@x.co", ListName::Blog, "t3", later, None).unwrap();
        assert!(matches!(back, UpsertResult::Pending));
        assert_eq!(list_status(&conn, "a@x.co", ListName::Blog), "pending");
        // New confirm token; the unsubscribe token in old emails stays valid.
        assert_eq!(
            tokens_of(&conn, "a@x.co", ListName::Blog),
            ("t3".into(), unsub)
        );

        // A pending re-request replaces the confirm token, killing the old link.
        upsert_pending(&conn, "a@x.co", ListName::Blog, "t4", later, None).unwrap();
        assert!(!confirm_by_token(&conn, "t3", later).unwrap());
        assert!(confirm_by_token(&conn, "t4", later).unwrap());
    }

    #[test]
    fn confirm_token_covers_every_list_in_the_signup_and_is_idempotent() {
        let conn = test_conn();
        let now = "2026-01-01T00:00:00.000Z";
        upsert_pending(&conn, "a@x.co", ListName::Blog, "t1", now, None).unwrap();
        upsert_pending(&conn, "a@x.co", ListName::Notes, "t1", now, None).unwrap();
        upsert_pending(&conn, "b@x.co", ListName::Blog, "t2", now, None).unwrap();

        assert!(!confirm_by_token(&conn, "nope", now).unwrap());
        assert!(!confirm_by_token(&conn, "", now).unwrap());
        assert!(confirm_by_token(&conn, "t1", now).unwrap());
        assert_eq!(list_status(&conn, "a@x.co", ListName::Blog), "confirmed");
        assert_eq!(list_status(&conn, "a@x.co", ListName::Notes), "confirmed");
        assert_eq!(list_status(&conn, "b@x.co", ListName::Blog), "pending");
        assert!(
            confirm_by_token(&conn, "t1", now).unwrap(),
            "second click still ok"
        );
    }

    #[test]
    fn old_confirm_link_cannot_undo_an_unsubscribe() {
        let conn = test_conn();
        let now = "2026-01-01T00:00:00.000Z";
        upsert_pending(&conn, "a@x.co", ListName::Blog, "t1", now, None).unwrap();
        confirm_by_token(&conn, "t1", now).unwrap();
        let (_, unsub) = tokens_of(&conn, "a@x.co", ListName::Blog);
        unsubscribe_by_token(&conn, &unsub).unwrap();

        confirm_by_token(&conn, "t1", now).unwrap();
        assert_eq!(list_status(&conn, "a@x.co", ListName::Blog), "unsubscribed");
    }

    #[test]
    fn unsubscribe_and_resubscribe_touch_only_their_own_list() {
        let conn = test_conn();
        let now = "2026-01-01T00:00:00.000Z";
        upsert_pending(&conn, "a@x.co", ListName::Blog, "t1", now, None).unwrap();
        upsert_pending(&conn, "a@x.co", ListName::Notes, "t1", now, None).unwrap();
        confirm_by_token(&conn, "t1", now).unwrap();
        let (_, blog_unsub) = tokens_of(&conn, "a@x.co", ListName::Blog);

        assert_eq!(
            unsubscribe_by_token(&conn, &blog_unsub).unwrap(),
            Some(ListName::Blog)
        );
        assert_eq!(list_status(&conn, "a@x.co", ListName::Blog), "unsubscribed");
        assert_eq!(list_status(&conn, "a@x.co", ListName::Notes), "confirmed");
        assert_eq!(get_confirmed(&conn, ListName::Blog).unwrap().len(), 0);
        assert_eq!(unsubscribe_by_token(&conn, "nope").unwrap(), None);

        assert!(set_unsub_reason(&conn, &blog_unsub, "too-often", "").unwrap());
        assert!(!set_unsub_reason(&conn, "nope", "too-often", "").unwrap());

        assert_eq!(
            resubscribe_by_token(&conn, &blog_unsub, now).unwrap(),
            Some(ListName::Blog)
        );
        assert_eq!(list_status(&conn, "a@x.co", ListName::Blog), "confirmed");
        assert_eq!(resubscribe_by_token(&conn, "nope", now).unwrap(), None);
    }

    #[test]
    fn deliveries_are_recorded_once_per_post_list_and_address() {
        let conn = test_conn();
        let now = "2026-01-01T00:00:00.000Z";
        record_delivery(&conn, "p", ListName::Blog, "a@x.co", now).unwrap();
        record_delivery(&conn, "p", ListName::Blog, "a@x.co", now).unwrap();
        assert!(was_delivered(&conn, "p", ListName::Blog, "a@x.co").unwrap());
        assert!(!was_delivered(&conn, "p", ListName::Notes, "a@x.co").unwrap());
        assert!(!was_delivered(&conn, "q", ListName::Blog, "a@x.co").unwrap());
    }

    #[test]
    fn schema_upgrades_a_database_from_the_node_service() {
        let conn = Connection::open_in_memory().unwrap();
        // The original schema, before any `ensure_column` migrations.
        conn.execute_batch(
            "CREATE TABLE subscriptions (
                id INTEGER PRIMARY KEY, email TEXT NOT NULL,
                list TEXT NOT NULL CHECK (list IN ('blog','notes')),
                status TEXT NOT NULL CHECK (status IN ('pending','confirmed','unsubscribed','bounced')),
                confirm_token TEXT NOT NULL, unsub_token TEXT NOT NULL,
                created_at TEXT NOT NULL, confirmed_at TEXT, UNIQUE(email, list));
             INSERT INTO subscriptions (email, list, status, confirm_token, unsub_token, created_at)
             VALUES ('old@x.co', 'blog', 'confirmed', 'c', 'u', '2024-01-01T00:00:00.000Z');",
        )
        .unwrap();

        init_schema(&conn).unwrap();
        init_schema(&conn).unwrap(); // every boot runs it again

        assert_eq!(get_confirmed(&conn, ListName::Blog).unwrap().len(), 1);
        assert_eq!(
            unsubscribe_by_token(&conn, "u").unwrap(),
            Some(ListName::Blog)
        );
        assert!(set_unsub_reason(&conn, "u", "moved", "note").unwrap());
        assert_eq!(suppress_email(&conn, "old@x.co", "complaint").unwrap(), 1);
        let now = "2026-01-01T00:00:00.000Z";
        upsert_pending(&conn, "new@x.co", ListName::Notes, "t", now, Some(now)).unwrap();
        assert_eq!(count_queued_confirmations(&conn).unwrap(), 1);
    }
}
