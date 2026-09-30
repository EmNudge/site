//! Seed the local dev database with sample subscribers, a couple of sent posts,
//! and a demo confirm/unsub token — so the admin dashboard has data to show.
//! Rust port of the old `scripts/seed-dev.mjs`.
//!
//! The service creates the schema on boot, so start it once first, then:
//!   DB_PATH=/tmp/nl-dev.db cargo run --bin seed_dev

use chrono::{Duration, SecondsFormat, Utc};
use rusqlite::{params, Connection};

fn iso(days_ago: i64) -> String {
    (Utc::now() - Duration::days(days_ago)).to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn main() {
    let path = std::env::var("DB_PATH").unwrap_or_else(|_| "/tmp/nl-dev.db".to_string());
    let conn = Connection::open(&path).expect("open database");
    // Wait for the write lock instead of erroring out — the service normally holds
    // the DB open (WAL) while we seed.
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .expect("set busy timeout");

    // Bail out clearly if the schema hasn't been created yet.
    let has_table: bool = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='subscriptions'",
            [],
            |_| Ok(true),
        )
        .unwrap_or(false);
    if !has_table {
        eprintln!(
            "No tables in {path}. Start the service once first (it creates the schema), then re-run."
        );
        std::process::exit(1);
    }

    let mut counter = 0u32;
    let mut tok = || {
        let t = format!("seed{counter}");
        counter += 1;
        t
    };

    // [email, list, status, ageDays]
    let rows: &[(&str, &str, &str, i64)] = &[
        ("ada@example.com", "notes", "confirmed", 1),
        ("ada@example.com", "blog", "confirmed", 1),
        ("grace@example.com", "notes", "confirmed", 2),
        ("alan@example.com", "blog", "confirmed", 3),
        ("linus@example.com", "notes", "confirmed", 4),
        ("linus@example.com", "blog", "confirmed", 4),
        ("margaret@example.com", "notes", "confirmed", 6),
        ("katherine@example.com", "blog", "confirmed", 9),
        ("dennis@example.com", "notes", "confirmed", 12),
        ("ken@example.com", "blog", "confirmed", 20),
        ("barbara@example.com", "notes", "pending", 0),
        ("hedy@example.com", "blog", "pending", 1),
        ("radia@example.com", "notes", "unsubscribed", 15),
        ("spammer@bots.xyz", "notes", "bounced", 3),
    ];

    for (email, list, status, age) in rows {
        let confirmed_at = if *status == "pending" {
            None
        } else {
            Some(iso(*age))
        };
        conn.execute(
            "INSERT OR REPLACE INTO subscriptions
             (email, list, status, confirm_token, unsub_token, created_at, confirmed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![email, list, status, tok(), tok(), iso(*age), confirmed_at],
        )
        .expect("insert subscription");
    }

    // Stable demo tokens for eyeballing the confirm / unsubscribe pages locally:
    //   http://localhost:4321/newsletter/unsubscribe?t=DEMO123
    conn.execute(
        "INSERT OR REPLACE INTO subscriptions
         (email, list, status, confirm_token, unsub_token, created_at, confirmed_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![
            "demo@example.com",
            "notes",
            "confirmed",
            "DEMOCONFIRM",
            "DEMO123",
            iso(0),
            iso(0)
        ],
    )
    .expect("insert demo subscription");

    for (slug, list, age, recipients) in [
        ("on-effort", "notes", 2i64, 6i64),
        ("beware-the-moralizers", "notes", 5, 5),
        ("how-i-use-llms", "blog", 8, 4),
    ] {
        conn.execute(
            "INSERT OR REPLACE INTO sent_posts (slug, list, sent_at, recipients) VALUES (?, ?, ?, ?)",
            params![slug, list, iso(age), recipients],
        )
        .expect("insert sent post");
    }

    println!(
        "Seeded {} subscriptions + 3 sent posts into {path}",
        rows.len() + 1
    );
}
