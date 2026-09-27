//! Newsletter subscription service — Rust port of the original Node/Hono service.
//!
//! Same endpoints, env vars, SQLite schema, and email output as before, so it is a
//! drop-in replacement. Hardening layers added throughout: constant-time admin
//! auth with a brute-force throttle, per-IP + global rate/cost caps, a bot gate,
//! request timeouts, body-size limits, security response headers, and
//! graceful shutdown.

mod auth;
mod backup;
mod cf_email;
mod config;
mod db;
mod email_templates;
mod error;
mod mx;
mod ratelimit;
mod render;
mod suppressions;
mod turnstile;
mod util;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Query, Request, State};
use axum::http::{header, HeaderMap, HeaderValue, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::cf_email::{permanent_bounces, EmailClient, SendArgs};
use crate::config::{ClientIpHeader, Config, ListName};
use crate::db::Db;
use crate::email_templates::PostArgs;
use crate::error::AppError;
use crate::ratelimit::WindowMap;

// ---- shared application state ----

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    db: Db,
    email: EmailClient,
    http: reqwest::Client,
    admin_index: Arc<String>,
    subscribe_rl: Arc<WindowMap>,
    admin_fails: Arc<WindowMap>,
    ip_confirms: Arc<WindowMap>,
    jobs: Arc<Mutex<HashMap<String, SendJob>>>,
    /// Held while an R2 backup runs, so a manual and a scheduled one can't overlap.
    backup_lock: Arc<tokio::sync::Mutex<()>>,
}

// ---- background send jobs (in-memory, per-recipient resume via `deliveries`) ----

#[derive(Serialize, Clone)]
struct SendJob {
    id: String,
    slug: String,
    list: String,
    total: i64,
    processed: i64,
    sent: i64,
    skipped: i64,
    bounced: i64,
    failed: i64,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip)]
    finished_at: Option<Instant>,
    /// Set by `/admin/send-cancel`; the send loop stops before its next recipient.
    #[serde(skip)]
    cancel: bool,
}

struct JobPost {
    slug: String,
    list: ListName,
    title: String,
    author: String,
    date: String,
    url: String,
    content_html: String,
    markdown: String,
}

fn update_job(jobs: &Arc<Mutex<HashMap<String, SendJob>>>, id: &str, f: impl FnOnce(&mut SendJob)) {
    if let Some(j) = jobs.lock().unwrap().get_mut(id) {
        f(j);
    }
}

fn prune_jobs(jobs: &Arc<Mutex<HashMap<String, SendJob>>>) {
    let now = Instant::now();
    jobs.lock().unwrap().retain(|_, j| {
        j.finished_at
            .map(|t| now.duration_since(t) < Duration::from_secs(3600))
            .unwrap_or(true)
    });
}

/// Abort a send job when this many sends fail in a row — a systemic problem (bad
/// payload, revoked token, sending quota used up), not per-recipient trouble.
/// Applies at any point in the blast: carrying on would only burn requests
/// against an API that is refusing them.
const ABORT_AFTER_CONSECUTIVE_FAILURES: i64 = 5;

async fn run_send_job(st: AppState, job_id: String, post: JobPost) {
    let outcome: anyhow::Result<()> = async {
        let list = post.list;
        let recipients = st.db.call(move |c| db::get_confirmed(c, list)).await?;
        update_job(&st.jobs, &job_id, |j| j.total = recipients.len() as i64);
        let mut consecutive_failures = 0i64;

        for sub in recipients {
            let cancelled = st
                .jobs
                .lock()
                .unwrap()
                .get(&job_id)
                .map(|j| j.cancel)
                .unwrap_or(false);
            if cancelled {
                // Leave the post unrecorded: re-sending resumes where this stopped.
                update_job(&st.jobs, &job_id, |j| {
                    j.status = "cancelled";
                    j.finished_at = Some(Instant::now());
                });
                return Ok(());
            }

            // Resume: skip anyone already delivered for this (slug, list).
            let slug = post.slug.clone();
            let email = sub.email.clone();
            let delivered = st
                .db
                .call(move |c| db::was_delivered(c, &slug, list, &email))
                .await?;
            if delivered {
                update_job(&st.jobs, &job_id, |j| {
                    j.skipped += 1;
                    j.processed += 1;
                });
                continue;
            }

            let mail = email_templates::post_email(
                &st.config,
                &PostArgs {
                    title: &post.title,
                    author: &post.author,
                    date: &post.date,
                    list,
                    url: &post.url,
                    content_html: &post.content_html,
                    content_text: &post.markdown,
                    unsub_token: &sub.unsub_token,
                },
            );
            let res = st
                .email
                .send_email_with_retry(SendArgs {
                    to: &sub.email,
                    subject: &mail.subject,
                    html: &mail.html,
                    text: &mail.text,
                    headers: &mail.headers,
                })
                .await;

            if res.ok && permanent_bounces(&res).iter().any(|b| b == &sub.email) {
                // 2xx overall, but CF rejected this recipient — suppress, don't count a send.
                let email = sub.email.clone();
                st.db.call(move |c| db::mark_bounced(c, &email, list)).await?;
                update_job(&st.jobs, &job_id, |j| j.bounced += 1);
                consecutive_failures = 0;
            } else if res.ok {
                let (slug, email, now) = (post.slug.clone(), sub.email.clone(), util::now_iso());
                st.db
                    .call(move |c| db::record_delivery(c, &slug, list, &email, &now))
                    .await?;
                update_job(&st.jobs, &job_id, |j| j.sent += 1);
                consecutive_failures = 0;
            } else {
                // Not a bounce: a 4xx here usually means CF rejected the *request*
                // (size, field, API change), which would hit every recipient alike.
                // Real bounces arrive via `permanent_bounces` + the suppression sync.
                // Don't log the body — it can echo the recipient.
                tracing::error!(to = %util::mask_email(&sub.email), status = res.status, "send failed");
                update_job(&st.jobs, &job_id, |j| j.failed += 1);
                consecutive_failures += 1;
                if consecutive_failures >= ABORT_AFTER_CONSECUTIVE_FAILURES {
                    anyhow::bail!(
                        "{ABORT_AFTER_CONSECUTIVE_FAILURES} sends in a row failed (last status {}); aborting — fix the cause and re-send to resume",
                        res.status
                    );
                }
            }

            update_job(&st.jobs, &job_id, |j| j.processed += 1);
            if st.config.send_throttle_ms > 0 {
                tokio::time::sleep(Duration::from_millis(st.config.send_throttle_ms)).await;
            }
        }

        // Record the post as sent (recipients = skipped + freshly sent) — but only
        // if nobody failed. Otherwise leave it unrecorded so re-sending retries the
        // failures (already-delivered recipients are skipped via `deliveries`).
        let (counted, failed) = {
            let jobs = st.jobs.lock().unwrap();
            jobs.get(&job_id)
                .map(|j| (j.skipped + j.sent, j.failed))
                .unwrap_or((0, 0))
        };
        if failed == 0 {
            let (slug, now) = (post.slug.clone(), util::now_iso());
            st.db
                .call(move |c| db::record_sent(c, &slug, list, &now, counted))
                .await?;
        }

        update_job(&st.jobs, &job_id, |j| {
            j.status = "done";
            j.finished_at = Some(Instant::now());
        });
        Ok(())
    }
    .await;

    if let Err(e) = outcome {
        tracing::error!(error = ?e, job = %job_id, "send job failed");
        update_job(&st.jobs, &job_id, |j| {
            j.status = "error";
            j.error = Some(e.to_string());
            j.finished_at = Some(Instant::now());
        });
    }
}

// ---- confirmation emails ----

enum ConfirmSend {
    Sent,
    /// Cloudflare rejected the recipient outright; the address is now suppressed.
    Bounced,
    /// The send itself failed, with this HTTP status (0 = no response).
    Failed(u16),
}

async fn send_confirmation(
    st: &AppState,
    email: &str,
    lists: &[ListName],
    confirm_token: &str,
) -> anyhow::Result<ConfirmSend> {
    let mail = email_templates::confirmation_email(&st.config, lists, confirm_token);
    let res = st
        .email
        .send_email(SendArgs {
            to: email,
            subject: &mail.subject,
            html: &mail.html,
            text: &mail.text,
            headers: &mail.headers,
        })
        .await;
    if !res.ok {
        // Don't log the raw upstream body (can echo the recipient); mask the email.
        tracing::error!(to = %util::mask_email(email), status = res.status, "confirm send failed");
        return Ok(ConfirmSend::Failed(res.status));
    }
    if permanent_bounces(&res).iter().any(|b| b == email) {
        let em = email.to_string();
        st.db
            .call(move |c| db::suppress_email(c, &em, "hard_bounce"))
            .await?;
        return Ok(ConfirmSend::Bounced);
    }
    Ok(ConfirmSend::Sent)
}

/// Atomically reserve one slot against the global hourly + daily confirmation
/// caps, pruning rows too old to count toward either.
async fn reserve_confirm_slot(st: &AppState) -> anyhow::Result<bool> {
    let now = util::now_iso();
    let hour_ago = util::iso_millis_ago(3_600_000);
    let day_ago = util::iso_millis_ago(24 * 3_600_000);
    let prune_before = util::iso_millis_ago(25 * 3_600_000);
    let (hour_cap, day_cap) = (
        st.config.global_confirm_cap_per_hour,
        st.config.global_confirm_cap_per_day,
    );
    st.db
        .call(move |c| {
            db::prune_send_log_before(c, &prune_before)?;
            db::reserve_confirm_slot(c, &now, &hour_ago, hour_cap, &day_ago, day_cap)
        })
        .await
}

/// Most queued confirmations sent per drain tick (the caps bound it further).
const DRAIN_BATCH: usize = 20;

/// Send confirmations that were queued because a cap was hit at signup, oldest
/// first, as slots free up. A running blast takes priority over the queue.
async fn drain_confirm_queue(st: &AppState) -> anyhow::Result<()> {
    let cutoff = util::iso_millis_ago(st.config.confirm_queue_max_age_hours * 3_600_000);
    let dropped = st
        .db
        .call(move |c| db::drop_stale_queued(c, &cutoff))
        .await?;
    if dropped > 0 {
        tracing::warn!(dropped, "dropped queued confirmations that waited too long");
    }

    for _ in 0..DRAIN_BATCH {
        let blast_running = st
            .jobs
            .lock()
            .unwrap()
            .values()
            .any(|j| j.status == "running");
        if blast_running {
            break;
        }
        let Some(queued) = st.db.call(db::next_queued_confirmation).await? else {
            break;
        };
        if !reserve_confirm_slot(st).await? {
            break;
        }

        let outcome =
            send_confirmation(st, &queued.email, &queued.lists, &queued.confirm_token).await?;
        // A throttled or failing API will refuse the rest too: leave this one
        // queued and try again next tick. Anything else is final for this entry.
        if matches!(outcome, ConfirmSend::Failed(status) if status == 0 || status == 429 || status >= 500)
        {
            break;
        }
        let token = queued.confirm_token;
        st.db
            .call(move |c| db::dequeue_confirmation(c, &token))
            .await?;

        if st.config.send_throttle_ms > 0 {
            tokio::time::sleep(Duration::from_millis(st.config.send_throttle_ms)).await;
        }
    }
    Ok(())
}

/// Periodic retention sweep: forget signups that were never confirmed, and
/// resend-cooldown entries that are long past their cooldown.
async fn run_retention(st: &AppState) -> anyhow::Result<()> {
    let days = st.config.pending_expiry_days;
    if days > 0 {
        let cutoff = util::iso_millis_ago(days * 86_400_000);
        let expired = st.db.call(move |c| db::expire_pending(c, &cutoff)).await?;
        if expired > 0 {
            tracing::info!(expired, "deleted unconfirmed subscriptions");
        }
    }
    let keep_ms = (st.config.confirm_resend_cooldown_min * 60_000).max(86_400_000);
    let cutoff = util::iso_millis_ago(keep_ms);
    st.db
        .call(move |c| db::prune_confirm_sends(c, &cutoff))
        .await?;
    Ok(())
}

// ---- small response helpers ----

fn jr(status: StatusCode, value: Value) -> Response {
    (status, Json(value)).into_response()
}

fn ok_json(value: Value) -> Response {
    jr(StatusCode::OK, value)
}

/// The client IP from whichever header the configured proxy sets.
fn client_ip(config: &Config, h: &HeaderMap) -> String {
    let name = match config.client_ip_header {
        ClientIpHeader::ForwardedFor => "x-forwarded-for",
        // A single address; the same parsing applies.
        ClientIpHeader::CfConnectingIp => "cf-connecting-ip",
    };
    util::client_ip(h.get(name).and_then(|v| v.to_str().ok()))
}

// ---- request bodies / query strings ----
//
// Typed extractors throughout. Besides being shorter than hand-walking JSON,
// `Json<T>` rejects any request not sent as `application/json` — which is what
// stops a cross-site `<form enctype=text/plain>` from forging an admin POST
// (a JSON content type forces a CORS preflight, and /admin has no CORS).

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubscribeReq {
    #[serde(default)]
    email: String,
    #[serde(default)]
    lists: Vec<ListName>,
    #[serde(default)]
    turnstile_token: String,
    /// Honeypot — real users never fill this.
    #[serde(default)]
    website: String,
}

#[derive(Deserialize)]
struct TokenReq {
    #[serde(default)]
    t: String,
}

#[derive(Deserialize)]
struct UnsubReasonReq {
    #[serde(default)]
    t: String,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    note: String,
}

#[derive(Deserialize)]
struct SendReq {
    list: ListName,
    slug: String,
    title: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    date: String,
    url: String,
    body: String,
}

#[derive(Deserialize)]
struct TestSendReq {
    to: String,
    #[serde(flatten)]
    post: SendReq,
}

#[derive(Deserialize)]
struct DeleteSubscriberReq {
    email: String,
}

#[derive(Deserialize)]
struct ListQuery {
    list: ListName,
}

#[derive(Deserialize)]
struct JobQuery {
    #[serde(rename = "jobId", default)]
    job_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum SubStatus {
    Pending,
    Confirmed,
    Unsubscribed,
    Bounced,
}

#[derive(Deserialize)]
struct SubscribersQuery {
    q: Option<String>,
    list: Option<ListName>,
    status: Option<SubStatus>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct TemplateQuery {
    #[serde(rename = "type")]
    kind: String,
    lists: String,
    list: Option<ListName>,
    title: String,
    author: String,
    date: String,
    body: String,
}

fn headers_to_json(headers: &[(String, String)]) -> Value {
    let map: serde_json::Map<String, Value> = headers
        .iter()
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect();
    Value::Object(map)
}

// ---- public handlers ----

async fn health() -> impl IntoResponse {
    Json(json!({ "ok": true }))
}

async fn subscribe(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SubscribeReq>,
) -> Result<Response, AppError> {
    // Honeypot — pretend success, send nothing.
    if !req.website.trim().is_empty() {
        return Ok(ok_json(json!({ "ok": true })));
    }

    let email = req.email.trim().to_lowercase();
    let lists = req.lists;

    if !util::is_valid_email(&email) {
        return Ok(jr(
            StatusCode::BAD_REQUEST,
            json!({ "error": "invalid email" }),
        ));
    }
    if lists.is_empty() {
        return Ok(jr(
            StatusCode::BAD_REQUEST,
            json!({ "error": "no lists selected" }),
        ));
    }

    let ip = client_ip(&st.config, &headers);
    let ip_key = util::rate_key(&ip);
    if st.subscribe_rl.hit_and_check(
        &ip_key,
        Duration::from_secs(60),
        st.config.rate_per_min_per_ip,
    ) {
        return Ok(jr(
            StatusCode::TOO_MANY_REQUESTS,
            json!({ "error": "rate limited" }),
        ));
    }

    // Bot gate BEFORE any email is sent (skipped when no Turnstile secret is set —
    // only possible in dev mode; startup refuses a real deployment without one).
    if !st.config.turnstile_secret.is_empty()
        && !turnstile::verify_turnstile(&st.http, &st.config, &req.turnstile_token, Some(&ip)).await
    {
        return Ok(jr(
            StatusCode::BAD_REQUEST,
            json!({ "error": "verification failed" }),
        ));
    }

    // Don't mail a domain that can't receive it — a typo'd or made-up domain is a
    // guaranteed hard bounce. After the bot gate, so it can't be used to make us
    // issue DNS lookups at will. Skipped in dev mode, like the bot gate.
    if st.config.mx_check && !st.config.email_dev_mode() {
        let domain = email.split_once('@').map(|(_, d)| d).unwrap_or_default();
        if !mx::domain_accepts_mail(&st.http, domain).await {
            return Ok(jr(
                StatusCode::BAD_REQUEST,
                json!({ "error": "invalid email domain" }),
            ));
        }
    }

    let now = util::now_iso();

    // Per-IP hourly send ceiling — one source can't burn the whole global budget.
    // Silently succeeds (don't reveal the cap). In-memory check, no side effect.
    if st.config.per_ip_confirm_cap_per_hour > 0
        && st
            .ip_confirms
            .is_capped(&ip_key, st.config.per_ip_confirm_cap_per_hour)
    {
        return Ok(ok_json(json!({ "ok": true })));
    }

    // Never re-mail an address that hard-bounced or reported us as spam, and don't
    // let a signup reset its suppressed status. Silently succeeds (don't reveal it).
    let em = email.clone();
    if st.db.call(move |c| db::is_suppressed(c, &em)).await? {
        return Ok(ok_json(json!({ "ok": true })));
    }

    // Per-email resend cooldown — blocks inbox-bombing a victim, ATOMICALLY: the
    // claim both checks and records in one statement, so N concurrent requests for
    // the same address yield exactly one send (closes the TOCTOU). Runs BEFORE the
    // global reserve so repeated hits on ONE address can't drain the global budget,
    // and BEFORE the upsert so a blocked attempt leaves any live confirm link intact.
    // Keyed on the normalized mailbox so `victim+1@`, `victim+2@`, … share one cooldown.
    if st.config.confirm_resend_cooldown_min > 0 {
        let cutoff = util::iso_millis_ago(st.config.confirm_resend_cooldown_min * 60_000);
        let (em, now_c) = (util::mailbox_key(&email), now.clone());
        let claimed = st
            .db
            .call(move |c| db::claim_confirm_send(c, &em, &now_c, &cutoff))
            .await?;
        if !claimed {
            return Ok(ok_json(json!({ "ok": true })));
        }
    }

    // Global circuit breaker — atomically RESERVE one slot against the hard hourly
    // and daily ceilings on confirmation-email spend (a single check-and-insert
    // statement, so concurrent requests can't all slip past). With no slot free the
    // signup is still accepted: its confirmation is queued and sent once one frees
    // up (see `drain_confirm_queue`). Only a full queue turns signups away.
    let queued = !reserve_confirm_slot(&st).await?;
    if queued {
        let waiting = st.db.call(db::count_queued_confirmations).await?;
        if waiting >= st.config.confirm_queue_max {
            tracing::warn!(
                waiting,
                "confirmation caps hit and queue full — refusing signup"
            );
            return Ok(jr(
                StatusCode::TOO_MANY_REQUESTS,
                json!({ "error": "temporarily unavailable" }),
            ));
        }
        tracing::warn!(waiting, "confirmation cap hit — queueing confirmation");
    }

    let confirm_token = util::new_token();
    let need_confirm: Vec<ListName> = {
        let (em, ct, now2, lists2) = (
            email.clone(),
            confirm_token.clone(),
            now.clone(),
            lists.clone(),
        );
        st.db
            .call(move |c| {
                let queued_at = queued.then_some(now2.as_str());
                let mut need = Vec::new();
                for list in lists2 {
                    if let db::UpsertResult::Pending =
                        db::upsert_pending(c, &em, list, &ct, &now2, queued_at)?
                    {
                        need.push(list);
                    }
                }
                Ok(need)
            })
            .await?
    };

    // All requested lists already confirmed — nothing to send.
    if need_confirm.is_empty() {
        return Ok(ok_json(json!({ "ok": true, "alreadySubscribed": true })));
    }

    if queued {
        st.ip_confirms.record(&ip_key, Duration::from_secs(3600));
        return Ok(ok_json(json!({ "ok": true, "delayed": true })));
    }

    if let ConfirmSend::Failed(_) =
        send_confirmation(&st, &email, &need_confirm, &confirm_token).await?
    {
        return Ok(jr(
            StatusCode::BAD_GATEWAY,
            json!({ "error": "could not send confirmation" }),
        ));
    }

    // The global slot and per-email cooldown were already reserved atomically above.
    st.ip_confirms.record(&ip_key, Duration::from_secs(3600));

    Ok(ok_json(json!({ "ok": true })))
}

/// Confirmation links in emails sent before confirming moved to the site page
/// point here. Changes nothing — a GET may come from a mail scanner rather than
/// the reader — and hands the token on to that page, where a click confirms.
async fn confirm_redirect(
    State(st): State<AppState>,
    Query(TokenReq { t: token }): Query<TokenReq>,
) -> Response {
    Redirect::to(&format!(
        "{}/newsletter/confirm?t={}",
        st.config.site_origin,
        email_templates::encode_component(&token)
    ))
    .into_response()
}

async fn confirm(
    State(st): State<AppState>,
    Json(TokenReq { t: token }): Json<TokenReq>,
) -> Result<Response, AppError> {
    if token.is_empty() {
        return Ok(jr(
            StatusCode::BAD_REQUEST,
            json!({ "error": "missing token" }),
        ));
    }
    let now = util::now_iso();
    let ok = st
        .db
        .call(move |c| db::confirm_by_token(c, &token, &now))
        .await?;
    Ok(if ok {
        ok_json(json!({ "ok": true }))
    } else {
        jr(StatusCode::NOT_FOUND, json!({ "error": "invalid token" }))
    })
}

async fn unsubscribe(
    State(st): State<AppState>,
    Query(q): Query<TokenReq>,
    body: Bytes,
) -> Result<Response, AppError> {
    // Handles both the JSON call from the site page and the one-click
    // List-Unsubscribe-Post from mail clients (RFC 8058: a form body, token in
    // the query string) — so this one reads the raw body rather than `Json<T>`.
    let token = serde_json::from_slice::<TokenReq>(&body)
        .ok()
        .map(|b| b.t)
        .filter(|t| !t.is_empty())
        .unwrap_or(q.t);
    if token.is_empty() {
        return Ok(jr(
            StatusCode::BAD_REQUEST,
            json!({ "error": "missing token" }),
        ));
    }
    let list = st
        .db
        .call(move |c| db::unsubscribe_by_token(c, &token))
        .await?;
    Ok(match list {
        Some(l) => ok_json(json!({ "ok": true, "list": l.as_str() })),
        None => jr(StatusCode::NOT_FOUND, json!({ "error": "invalid token" })),
    })
}

async fn resubscribe(
    State(st): State<AppState>,
    Json(TokenReq { t: token }): Json<TokenReq>,
) -> Result<Response, AppError> {
    if token.is_empty() {
        return Ok(jr(
            StatusCode::BAD_REQUEST,
            json!({ "error": "missing token" }),
        ));
    }
    let now = util::now_iso();
    let list = st
        .db
        .call(move |c| db::resubscribe_by_token(c, &token, &now))
        .await?;
    Ok(match list {
        Some(l) => ok_json(json!({ "ok": true, "list": l.as_str() })),
        None => jr(StatusCode::NOT_FOUND, json!({ "error": "invalid token" })),
    })
}

async fn unsubscribe_reason(
    State(st): State<AppState>,
    Json(req): Json<UnsubReasonReq>,
) -> Result<Response, AppError> {
    let token = req.t;
    if token.is_empty() {
        return Ok(jr(
            StatusCode::BAD_REQUEST,
            json!({ "error": "missing token" }),
        ));
    }
    let reason: String = req.reason.chars().take(100).collect();
    let note: String = req.note.chars().take(500).collect();
    let changed = st
        .db
        .call(move |c| db::set_unsub_reason(c, &token, &reason, &note))
        .await?;
    Ok(if changed {
        ok_json(json!({ "ok": true }))
    } else {
        jr(StatusCode::NOT_FOUND, json!({ "error": "invalid token" }))
    })
}

// ---- admin handlers (behind require_admin) ----

async fn admin_index_handler(State(st): State<AppState>) -> impl IntoResponse {
    Html((*st.admin_index).clone())
}

async fn admin_config(State(st): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "siteOrigin": st.config.site_origin,
        "authorName": st.config.author_name,
    }))
}

async fn admin_recipient_count(
    State(st): State<AppState>,
    Query(ListQuery { list }): Query<ListQuery>,
) -> Result<Response, AppError> {
    let count = st.db.call(move |c| db::count_confirmed(c, list)).await?;
    Ok(ok_json(json!({ "count": count })))
}

async fn admin_send(
    State(st): State<AppState>,
    Json(req): Json<SendReq>,
) -> Result<Response, AppError> {
    let SendReq {
        list,
        slug,
        title,
        author,
        date,
        url,
        body: markdown,
    } = req;
    let author = if author.is_empty() {
        st.config.author_name.clone()
    } else {
        author
    };
    if slug.is_empty() || title.is_empty() || url.is_empty() || markdown.is_empty() {
        return Ok(jr(
            StatusCode::BAD_REQUEST,
            json!({ "error": "list, slug, title, url and body are required" }),
        ));
    }

    let (slug_check, list_check) = (slug.clone(), list);
    if st
        .db
        .call(move |c| db::has_sent(c, &slug_check, list_check))
        .await?
    {
        return Ok(ok_json(
            json!({ "ok": true, "skipped": true, "reason": "already sent" }),
        ));
    }

    // Kick off the blast in the background and return a job id to poll.
    prune_jobs(&st.jobs);
    let content_html = render::render_markdown(&markdown);
    let job_id: String = util::new_token().chars().take(16).collect();
    let job = SendJob {
        id: job_id.clone(),
        slug: slug.clone(),
        list: list.as_str().to_string(),
        total: 0,
        processed: 0,
        sent: 0,
        skipped: 0,
        bounced: 0,
        failed: 0,
        status: "running",
        error: None,
        finished_at: None,
        cancel: false,
    };
    {
        // Check-and-insert under one lock: a double-click (or the dashboard and the
        // CLI at once) must not start two jobs for the same post, or everyone gets
        // it twice — `deliveries` is only written after each send completes.
        let mut jobs = st.jobs.lock().unwrap();
        if jobs
            .values()
            .any(|j| j.status == "running" && j.slug == slug && j.list == list.as_str())
        {
            return Ok(jr(
                StatusCode::CONFLICT,
                json!({ "error": "a send for this post is already running" }),
            ));
        }
        jobs.insert(job_id.clone(), job);
    }

    let post = JobPost {
        slug,
        list,
        title,
        author,
        date,
        url,
        content_html,
        markdown,
    };
    tokio::spawn(run_send_job(st.clone(), job_id.clone(), post));

    Ok(ok_json(json!({ "ok": true, "jobId": job_id })))
}

/// Send one copy of a post to a single address, to check it in a real inbox
/// before the blast. Records nothing, so it never counts as a delivery.
async fn admin_send_test(
    State(st): State<AppState>,
    Json(req): Json<TestSendReq>,
) -> Result<Response, AppError> {
    let to = req.to.trim().to_lowercase();
    let post = req.post;
    if !util::is_valid_email(&to) {
        return Ok(jr(
            StatusCode::BAD_REQUEST,
            json!({ "error": "invalid test address" }),
        ));
    }
    if post.title.is_empty() || post.url.is_empty() || post.body.is_empty() {
        return Ok(jr(
            StatusCode::BAD_REQUEST,
            json!({ "error": "title, url and body are required" }),
        ));
    }
    let author = if post.author.is_empty() {
        st.config.author_name.clone()
    } else {
        post.author
    };

    let content_html = render::render_markdown(&post.body);
    let mail = email_templates::post_email(
        &st.config,
        &PostArgs {
            title: &post.title,
            author: &author,
            date: &post.date,
            list: post.list,
            url: &post.url,
            content_html: &content_html,
            content_text: &post.body,
            unsub_token: "TEST-UNSUB-TOKEN",
        },
    );
    let res = st
        .email
        .send_email_with_retry(SendArgs {
            to: &to,
            subject: &format!("[TEST] {}", mail.subject),
            html: &mail.html,
            text: &mail.text,
            headers: &mail.headers,
        })
        .await;
    if !res.ok {
        tracing::error!(status = res.status, "test send failed");
        return Ok(jr(
            StatusCode::BAD_GATEWAY,
            json!({ "error": format!("send failed (status {})", res.status) }),
        ));
    }
    Ok(ok_json(json!({ "ok": true })))
}

/// Stop a running blast before its next recipient. Whoever already got the post
/// keeps it; re-sending later resumes with the rest.
async fn admin_send_cancel(
    State(st): State<AppState>,
    Json(q): Json<JobQuery>,
) -> Result<Response, AppError> {
    let mut jobs = st.jobs.lock().unwrap();
    Ok(match jobs.get_mut(&q.job_id) {
        Some(j) if j.status == "running" => {
            j.cancel = true;
            ok_json(json!({ "ok": true }))
        }
        Some(_) => jr(
            StatusCode::CONFLICT,
            json!({ "error": "job is not running" }),
        ),
        None => jr(StatusCode::NOT_FOUND, json!({ "error": "unknown job" })),
    })
}

/// Erase an address and everything stored about it (an erasure request, or
/// clearing out junk signups).
async fn admin_delete_subscriber(
    State(st): State<AppState>,
    Json(req): Json<DeleteSubscriberReq>,
) -> Result<Response, AppError> {
    let email = req.email.trim().to_lowercase();
    if email.is_empty() {
        return Ok(jr(
            StatusCode::BAD_REQUEST,
            json!({ "error": "missing email" }),
        ));
    }
    let (em, mailbox) = (email.clone(), util::mailbox_key(&email));
    let deleted = st
        .db
        .call(move |c| db::delete_subscriber(c, &em, &mailbox))
        .await?;
    if deleted == 0 {
        return Ok(jr(
            StatusCode::NOT_FOUND,
            json!({ "error": "no such subscriber" }),
        ));
    }
    tracing::info!(email = %util::mask_email(&email), deleted, "admin deleted subscriber");
    Ok(ok_json(json!({ "ok": true, "deleted": deleted })))
}

async fn admin_send_status(
    State(st): State<AppState>,
    Query(q): Query<JobQuery>,
) -> Result<Response, AppError> {
    let job = st.jobs.lock().unwrap().get(&q.job_id).cloned();
    Ok(match job {
        Some(j) => ok_json(serde_json::to_value(j)?),
        None => jr(StatusCode::NOT_FOUND, json!({ "error": "unknown job" })),
    })
}

async fn admin_stats(State(st): State<AppState>) -> Result<Response, AppError> {
    let since7d = util::iso_millis_ago(7 * 86_400_000);
    let since30d = util::iso_millis_ago(30 * 86_400_000);
    let stats = st
        .db
        .call(move |c| db::get_stats(c, &since7d, &since30d))
        .await?;
    Ok(ok_json(serde_json::to_value(stats)?))
}

async fn admin_subscribers(
    State(st): State<AppState>,
    Query(q): Query<SubscribersQuery>,
) -> Result<Response, AppError> {
    let query = db::SubscriberQuery {
        q: q.q.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        list: q.list.map(|l| l.as_str().to_string()),
        status: q.status.map(|s| {
            match s {
                SubStatus::Pending => "pending",
                SubStatus::Confirmed => "confirmed",
                SubStatus::Unsubscribed => "unsubscribed",
                SubStatus::Bounced => "bounced",
            }
            .to_string()
        }),
        limit: q.limit.unwrap_or(50).clamp(1, 200),
        offset: q.offset.unwrap_or(0).max(0),
    };
    let page = st.db.call(move |c| db::list_subscribers(c, query)).await?;
    Ok(ok_json(serde_json::to_value(page)?))
}

async fn admin_sent(State(st): State<AppState>) -> Result<Response, AppError> {
    let rows = st.db.call(move |c| db::list_sent_posts(c, 100)).await?;
    Ok(ok_json(json!({ "rows": serde_json::to_value(rows)? })))
}

async fn admin_sync_suppressions(State(st): State<AppState>) -> Result<Response, AppError> {
    let result = suppressions::sync_suppressions(&st.email, &st.db).await?;
    Ok(ok_json(serde_json::to_value(result)?))
}

async fn admin_template(
    State(st): State<AppState>,
    Query(q): Query<TemplateQuery>,
) -> impl IntoResponse {
    fn or(s: String, fallback: impl FnOnce() -> String) -> String {
        if s.is_empty() {
            fallback()
        } else {
            s
        }
    }

    if q.kind == "confirmation" {
        let mut lists: Vec<ListName> = q
            .lists
            .split(',')
            .filter_map(|s| ListName::parse(s.trim()))
            .collect();
        if lists.is_empty() {
            lists.push(ListName::Notes);
        }
        let mail = email_templates::confirmation_email(&st.config, &lists, "PREVIEW-CONFIRM-TOKEN");
        return Json(json!({
            "subject": mail.subject,
            "html": mail.html,
            "text": mail.text,
            "headers": {},
        }));
    }

    let list = q.list.unwrap_or(ListName::Notes);
    let title = or(q.title, || "Sample post title".to_string());
    let author = or(q.author, || st.config.author_name.clone());
    let date = or(q.date, util::now_iso);
    let markdown = or(q.body, || {
        "_(write some markdown to preview the body)_".to_string()
    });
    let url = format!("{}/{}/sample-post", st.config.site_origin, list.as_str());
    let content_html = render::render_markdown(&markdown);
    let mail = email_templates::post_email(
        &st.config,
        &PostArgs {
            title: &title,
            author: &author,
            date: &date,
            list,
            url: &url,
            content_html: &content_html,
            content_text: &markdown,
            unsub_token: "PREVIEW-UNSUB-TOKEN",
        },
    );
    Json(json!({
        "subject": mail.subject,
        "html": mail.html,
        "text": mail.text,
        "headers": headers_to_json(&mail.headers),
    }))
}

// ---- admin auth middleware ----

/// CSRF backstop: browsers attach the session cookie (and cached Basic
/// credentials) to cross-site requests too, so refuse any state-changing request a
/// browser marks as not same-origin. Non-browser clients (the CLI sender) don't
/// send the header.
fn cross_site_refusal(method: &Method, headers: &HeaderMap) -> Option<Response> {
    if method == Method::GET || method == Method::HEAD {
        return None;
    }
    let site = headers.get("sec-fetch-site").and_then(|v| v.to_str().ok());
    matches!(site, Some(s) if s != "same-origin")
        .then(|| (StatusCode::FORBIDDEN, "Cross-site request refused").into_response())
}

/// `Some(response)` when this IP is locked out after too many failed logins.
fn lockout(st: &AppState, ip: &str) -> Option<Response> {
    st.admin_fails
        .is_capped(ip, st.config.admin_auth_max_failures)
        .then(|| {
            tracing::warn!(%ip, "admin auth: throttled — too many failed attempts");
            (StatusCode::TOO_MANY_REQUESTS, "Too many attempts").into_response()
        })
}

fn record_failed_login(st: &AppState, ip: &str, method: &Method, path: &str) {
    st.admin_fails.record(ip, Duration::from_secs(15 * 60));
    tracing::warn!(%ip, %method, %path, "admin auth: failed attempt");
}

async fn require_admin(State(st): State<AppState>, req: Request, next: Next) -> Response {
    if let Some(refused) = cross_site_refusal(req.method(), req.headers()) {
        return refused;
    }
    let ip = util::rate_key(&client_ip(&st.config, req.headers()));
    if let Some(locked) = lockout(&st, &ip) {
        return locked;
    }

    let credentials = req.headers().get(header::AUTHORIZATION);
    let session_ok = req
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(auth::session_from_cookies)
        .is_some_and(|s| {
            auth::check_session(s, &st.config.admin_token, chrono::Utc::now().timestamp())
        });
    let authorized = session_ok
        || auth::check_admin_auth(
            credentials.and_then(|v| v.to_str().ok()),
            &st.config.admin_token,
        );

    if !authorized {
        // Only a wrong credential is a guess; a visit with no (or an expired)
        // session is not, so it doesn't count toward the lockout.
        if credentials.is_some() {
            record_failed_login(&st, &ip, req.method(), req.uri().path());
        }
        // Send a browser opening the dashboard to the login form. API calls get a
        // bare 401 — no `WWW-Authenticate: Basic`, which would make the browser
        // pop its native password dialog (one password managers can't fill).
        let path = req.uri().path();
        if req.method() == Method::GET && (path == "/admin" || path == "/admin/") {
            return Redirect::to("/admin/login").into_response();
        }
        return jr(StatusCode::UNAUTHORIZED, json!({ "error": "unauthorized" }));
    }

    st.admin_fails.clear(&ip);
    next.run(req).await
}

// ---- backups ----

/// A gzipped snapshot of the database, as a file download.
async fn admin_backup_download(State(st): State<AppState>) -> Result<Response, AppError> {
    let body = backup::snapshot_gz(&st.db).await?;
    let name = backup::file_name(chrono::Utc::now());
    tracing::info!(bytes = body.len(), "admin downloaded a backup");
    Ok((
        [
            (header::CONTENT_TYPE, "application/gzip".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{name}\""),
            ),
            (header::CACHE_CONTROL, "no-store".to_string()),
        ],
        body,
    )
        .into_response())
}

async fn admin_backup_status(State(st): State<AppState>) -> Result<Response, AppError> {
    let (last, last_success) = st.db.call(db::backup_status).await?;
    Ok(ok_json(json!({
        "r2Configured": st.config.r2.is_some(),
        "running": st.backup_lock.try_lock().is_err(),
        "last": last,
        "lastSuccess": last_success,
    })))
}

/// Back up to R2 now (the dashboard's button), instead of waiting for the schedule.
async fn admin_backup_run(
    State(st): State<AppState>,
    Json(_): Json<Value>,
) -> Result<Response, AppError> {
    let Some(r2) = st.config.r2.clone() else {
        return Ok(jr(
            StatusCode::CONFLICT,
            json!({ "error": "R2 backups aren't configured (set R2_* in .env)" }),
        ));
    };
    let Ok(_running) = st.backup_lock.try_lock() else {
        return Ok(jr(
            StatusCode::CONFLICT,
            json!({ "error": "a backup is already running" }),
        ));
    };
    let row = backup::run_r2_backup(&st.db, &st.http, &r2).await?;
    Ok(if row.ok {
        ok_json(json!({ "ok": true, "backup": row }))
    } else {
        jr(
            StatusCode::BAD_GATEWAY,
            json!({ "error": row.error, "backup": row }),
        )
    })
}

/// The hourly check: back up to R2 if a day has passed since the last success.
async fn scheduled_backup(st: &AppState, r2: &config::R2Config) -> anyhow::Result<()> {
    let (_, last_success) = st.db.call(db::backup_status).await?;
    let last_at = last_success.as_ref().map(|b| b.finished_at.as_str());
    if !backup::is_due(last_at, chrono::Utc::now()) {
        return Ok(());
    }
    let Ok(_running) = st.backup_lock.try_lock() else {
        return Ok(()); // a manual one is in flight
    };
    let row = backup::run_r2_backup(&st.db, &st.http, r2).await?;
    if row.ok {
        tracing::info!(key = ?row.object_key, bytes = ?row.bytes, "backed up to R2");
    } else {
        tracing::error!(error = ?row.error, "R2 backup failed; retrying next hour");
    }
    Ok(())
}

// ---- dashboard login ----
//
// A plain HTML form (so password managers can save and fill it) that exchanges
// the admin token for a signed session cookie. Lives outside `require_admin`.

#[derive(Deserialize)]
struct LoginForm {
    #[serde(default)]
    password: String,
}

fn login_page(status: StatusCode, error: Option<&str>) -> Response {
    let error = error
        .map(|e| format!(r#"<p class="error" role="alert">{e}</p>"#))
        .unwrap_or_default();
    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Sign in · Newsletter admin</title>
<style>
  body {{ margin:0; min-height:100vh; display:grid; place-items:center; background:#f4f4f2; color:#1a1a1a;
         font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif; }}
  form {{ width:min(340px, calc(100vw - 32px)); background:#fff; border:1px solid #e3e3e0; border-radius:12px; padding:28px; }}
  h1 {{ font-size:20px; margin:0 0 20px; }}
  label {{ display:block; font-size:14px; margin-bottom:14px; }}
  input {{ display:block; box-sizing:border-box; width:100%; margin-top:6px; padding:9px 11px; font:inherit;
          border:1px solid #d4d4d0; border-radius:8px; }}
  button {{ width:100%; padding:10px; font:inherit; font-weight:600; color:#fff; background:#28665f; border:0; border-radius:8px; cursor:pointer; }}
  .error {{ color:#b42318; font-size:14px; margin:0 0 14px; }}
</style>
</head>
<body>
<form method="post" action="/admin/login">
  <h1>Newsletter admin</h1>
  {error}
  <label>Username <input name="username" autocomplete="username" value="admin" required></label>
  <label>Password <input type="password" name="password" autocomplete="current-password" required autofocus></label>
  <button type="submit">Sign in</button>
</form>
</body>
</html>"#
    );
    (status, [(header::CACHE_CONTROL, "no-store")], Html(html)).into_response()
}

fn session_cookie(config: &Config, value: &str, max_age: i64) -> String {
    let secure = if config.public_base.starts_with("https://") {
        "; Secure"
    } else {
        "" // plain-http local dev
    };
    format!(
        "{}={value}; Path=/admin; Max-Age={max_age}; HttpOnly; SameSite=Lax{secure}",
        auth::SESSION_COOKIE
    )
}

async fn admin_login_page() -> Response {
    login_page(StatusCode::OK, None)
}

async fn admin_login(
    State(st): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<LoginForm>,
) -> Response {
    if let Some(refused) = cross_site_refusal(&Method::POST, &headers) {
        return refused;
    }
    let ip = util::rate_key(&client_ip(&st.config, &headers));
    if let Some(locked) = lockout(&st, &ip) {
        return locked;
    }
    if !auth::safe_equal(&form.password, &st.config.admin_token) {
        record_failed_login(&st, &ip, &Method::POST, "/admin/login");
        return login_page(StatusCode::UNAUTHORIZED, Some("Wrong password."));
    }

    st.admin_fails.clear(&ip);
    let session = auth::new_session(&st.config.admin_token, chrono::Utc::now().timestamp());
    (
        [(
            header::SET_COOKIE,
            session_cookie(&st.config, &session, auth::SESSION_TTL_SECS),
        )],
        Redirect::to("/admin"),
    )
        .into_response()
}

async fn admin_logout(State(st): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(refused) = cross_site_refusal(&Method::POST, &headers) {
        return refused;
    }
    (
        [(header::SET_COOKIE, session_cookie(&st.config, "", 0))],
        Redirect::to("/admin/login"),
    )
        .into_response()
}

// ---- startup ----

/// Hard ceiling on distinct keys per in-memory limiter map (bounds memory under
/// source-IP rotation; normal traffic never approaches this).
const MAX_LIMITER_KEYS: usize = 100_000;

/// Minimum ADMIN_TOKEN length outside dev mode (`openssl rand -hex 32` gives 64).
const MIN_ADMIN_TOKEN_LEN: usize = 32;

/// Whether a bind host is a loopback address (safe to expose without a proxy). A
/// non-loopback bind (e.g. 0.0.0.0) means clients may reach the service directly
/// and spoof the client-IP header, so per-IP guards can't be trusted.
fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "::1" | "localhost")
}

fn load_admin_index() -> String {
    std::fs::read_to_string("admin/dist/index.html").unwrap_or_else(|_| {
        "<!doctype html><meta charset=utf-8><body style='font-family:sans-serif;padding:40px'>\
         <h1>Admin UI not built</h1><p>Run <code>npm --prefix admin install &amp;&amp; npm --prefix admin run build</code> in the service directory.</p>"
            .to_string()
    })
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received — draining");
}

/// The full router: public signup routes, health, and the auth-gated admin API.
fn build_app(state: AppState) -> anyhow::Result<Router> {
    // Browser-facing POST routes: CORS to the site origin + a tight body limit.
    let cors = CorsLayer::new()
        .allow_origin(
            HeaderValue::from_str(&state.config.site_origin)
                .map_err(|_| anyhow::anyhow!("SITE_ORIGIN is not a valid header value"))?,
        )
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    let public = Router::new()
        .route("/subscribe", post(subscribe))
        .route("/unsubscribe", post(unsubscribe))
        .route("/unsubscribe-reason", post(unsubscribe_reason))
        .route("/resubscribe", post(resubscribe))
        .route("/confirm", get(confirm_redirect).post(confirm))
        .layer(cors)
        .layer(DefaultBodyLimit::max(state.config.max_body_bytes));

    let misc = Router::new().route("/health", get(health));

    // The login form sits outside the auth gate (it's how you get past it).
    let login = Router::new()
        .route("/admin/login", get(admin_login_page).post(admin_login))
        .route("/admin/logout", post(admin_logout));

    // Every other /admin* route is gated by the auth middleware. No CORS: same-origin only.
    let admin = Router::new()
        .route("/admin", get(admin_index_handler))
        .route("/admin/config", get(admin_config))
        .route("/admin/stats", get(admin_stats))
        .route("/admin/subscribers", get(admin_subscribers))
        .route("/admin/sent", get(admin_sent))
        .route("/admin/recipient-count", get(admin_recipient_count))
        .route("/admin/send", post(admin_send))
        .route("/admin/send-status", get(admin_send_status))
        .route("/admin/send-test", post(admin_send_test))
        .route("/admin/send-cancel", post(admin_send_cancel))
        .route("/admin/delete-subscriber", post(admin_delete_subscriber))
        .route("/admin/template", get(admin_template))
        .route("/admin/sync-suppressions", post(admin_sync_suppressions))
        .route("/admin/backup/download", get(admin_backup_download))
        .route("/admin/backup/status", get(admin_backup_status))
        .route("/admin/backup/run", post(admin_backup_run))
        .nest_service("/admin/assets", ServeDir::new("admin/dist/assets"))
        .layer(middleware::from_fn_with_state(state.clone(), require_admin));

    let app = Router::new()
        .merge(public)
        .merge(misc)
        .merge(login)
        .merge(admin)
        .with_state(state.clone())
        // Hardening layers (outermost last). Security headers, request timeout,
        // an outer body-size ceiling, and request tracing.
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(DefaultBodyLimit::max(state.config.max_send_body_bytes))
        .layer(TraceLayer::new_for_http());

    Ok(app)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = Arc::new(Config::from_env().map_err(anyhow::Error::msg)?);

    // Boot-time safety checks. The per-IP guards only hold if a trusted proxy sets
    // the client-IP header; a non-loopback bind without that is a footgun.
    if !is_loopback_host(&config.bind_host) {
        tracing::warn!(
            bind_host = %config.bind_host,
            "binding to a non-loopback address — a trusted proxy (Cloudflare Tunnel or Caddy) MUST be the only way in and set the CLIENT_IP_HEADER, or clients can spoof their IP and bypass every per-IP guard"
        );
        // Dev mode (no CF creds) logs confirmation links with live tokens and does
        // not actually send — never expose that on a public interface.
        if config.email_dev_mode() && std::env::var("ALLOW_INSECURE_DEV_BIND").as_deref() != Ok("1")
        {
            anyhow::bail!(
                "refusing to start: dev mode (no Cloudflare creds) on a non-loopback bind ({}) would expose confirmation links with live tokens on a public interface. Set BIND_HOST=127.0.0.1, provide CF creds, or set ALLOW_INSECURE_DEV_BIND=1 to override.",
                config.bind_host
            );
        }
    }
    // Outside dev mode, refuse configs that silently weaken the service.
    if !config.email_dev_mode() {
        if config.admin_token.len() < MIN_ADMIN_TOKEN_LEN {
            anyhow::bail!(
                "refusing to start: ADMIN_TOKEN must be at least {MIN_ADMIN_TOKEN_LEN} chars (use `openssl rand -hex 32`)"
            );
        }
        if config.turnstile_secret.is_empty() {
            anyhow::bail!(
                "refusing to start: TURNSTILE_SECRET is unset, which would disable the signup bot gate"
            );
        }
    }

    let db = Db::open(&config.db_path)?;
    let email = EmailClient::new(config.clone())?;
    let http = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .user_agent("newsletter-service")
        .build()?;

    let state = AppState {
        config: config.clone(),
        db,
        email,
        http,
        admin_index: Arc::new(load_admin_index()),
        subscribe_rl: Arc::new(WindowMap::new(MAX_LIMITER_KEYS)),
        admin_fails: Arc::new(WindowMap::new(MAX_LIMITER_KEYS)),
        ip_confirms: Arc::new(WindowMap::new(MAX_LIMITER_KEYS)),
        jobs: Arc::new(Mutex::new(HashMap::new())),
        backup_lock: Arc::new(tokio::sync::Mutex::new(())),
    };

    let app = build_app(state.clone())?;

    // Background maintenance: drop expired limiter entries so the maps stay small.
    {
        let st = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10 * 60));
            interval.tick().await; // fire immediately, then every 10 min
            loop {
                interval.tick().await;
                st.subscribe_rl.sweep();
                st.admin_fails.sweep();
                st.ip_confirms.sweep();
            }
        });
    }

    // Send queued confirmations as cap slots free up.
    {
        let st = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = drain_confirm_queue(&st).await {
                    tracing::error!(error = ?e, "confirmation queue drain failed");
                }
            }
        });
    }

    // Retention: forget unconfirmed signups and stale cooldown entries.
    {
        let st = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3600));
            loop {
                interval.tick().await;
                if let Err(e) = run_retention(&st).await {
                    tracing::error!(error = ?e, "retention sweep failed");
                }
            }
        });
    }

    // Daily database backup to R2, when configured. Checked hourly; the last
    // success is kept in the DB, so restarts neither skip nor repeat one.
    if let Some(r2) = config.r2.clone() {
        let st = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3600));
            loop {
                interval.tick().await;
                if let Err(e) = scheduled_backup(&st, &r2).await {
                    tracing::error!(error = ?e, "R2 backup check failed");
                }
            }
        });
    } else {
        tracing::warn!(
            "R2_* unset — automatic backups are off (the dashboard can still download one)."
        );
    }

    // Poll Cloudflare's suppression list (complaints + hard bounces) into our DB.
    // No-op in dev mode. Runs once on boot, then on the configured interval.
    if !config.email_dev_mode() && config.suppression_sync_min > 0 {
        let st = state.clone();
        let every = config.suppression_sync_min;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(every * 60));
            loop {
                interval.tick().await;
                match suppressions::sync_suppressions(&st.email, &st.db).await {
                    Ok(r) if r.suppressed > 0 => tracing::info!(
                        suppressed = r.suppressed,
                        actionable = r.actionable,
                        scanned = r.scanned,
                        "suppression sync marked rows bounced"
                    ),
                    Ok(_) => {}
                    Err(e) => tracing::error!(error = ?e, "suppression sync failed"),
                }
            }
        });
    }

    let addr = format!("{}:{}", config.bind_host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("newsletter-service listening on {addr}");
    if config.email_dev_mode() {
        tracing::warn!("CF creds unset — dev mode: emails are logged, not sent.");
    }
    if config.turnstile_secret.is_empty() {
        tracing::warn!("TURNSTILE_SECRET unset — signup bot gate is disabled (dev only).");
    }

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
