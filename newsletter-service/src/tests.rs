//! Service-level tests: the send job, `/subscribe` + `/confirm`, and admin auth,
//! run against the real router with an in-memory DB and a fake Cloudflare API.

use super::*;

// ---- harness ----

type Responder = Arc<dyn Fn(&str) -> (u16, Value) + Send + Sync>;

/// A stand-in for the Cloudflare send endpoint. Records every payload and
/// answers each one with `respond(recipient)`.
struct FakeCf {
    base: String,
    sent: Arc<Mutex<Vec<Value>>>,
}

impl FakeCf {
    async fn start(respond: impl Fn(&str) -> (u16, Value) + Send + Sync + 'static) -> FakeCf {
        let sent = Arc::new(Mutex::new(Vec::new()));
        let respond: Responder = Arc::new(respond);
        let app = Router::new().fallback({
            let sent = sent.clone();
            move |Json(payload): Json<Value>| {
                let (sent, respond) = (sent.clone(), respond.clone());
                async move {
                    let to = payload["to"].as_str().unwrap_or_default().to_string();
                    sent.lock().unwrap().push(payload);
                    let (status, body) = respond(&to);
                    (StatusCode::from_u16(status).unwrap(), Json(body))
                }
            }
        });
        FakeCf {
            base: serve(app).await,
            sent,
        }
    }

    /// Accepts everything, like a healthy Cloudflare.
    async fn ok() -> FakeCf {
        FakeCf::start(|_| (200, json!({ "success": true }))).await
    }

    fn recipients(&self) -> Vec<String> {
        self.sent
            .lock()
            .unwrap()
            .iter()
            .map(|p| p["to"].as_str().unwrap().to_string())
            .collect()
    }
}

async fn serve(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// App state wired to `cf` (or dev mode when `None`), with `tweak` applied to the
/// test config.
fn state(cf: Option<&FakeCf>, tweak: impl FnOnce(&mut Config)) -> AppState {
    let mut config = Config::for_tests();
    if cf.is_some() {
        config.cf_account_id = "acct".to_string();
        config.cf_email_token = "cf-token".to_string();
    }
    tweak(&mut config);
    let config = Arc::new(config);
    let mut email = EmailClient::new(config.clone()).unwrap();
    if let Some(cf) = cf {
        email = email.with_api_base(&cf.base);
    }
    AppState {
        config,
        db: Db::open(":memory:").unwrap(),
        email,
        http: reqwest::Client::new(),
        admin_index: Arc::new(String::new()),
        subscribe_rl: Arc::new(WindowMap::new(100)),
        admin_fails: Arc::new(WindowMap::new(100)),
        ip_confirms: Arc::new(WindowMap::new(100)),
        jobs: Arc::new(Mutex::new(HashMap::new())),
        backup_lock: Arc::new(tokio::sync::Mutex::new(())),
    }
}

/// Serves the full router for `st`; returns its base URL and a client that
/// doesn't follow redirects.
async fn start(st: &AppState) -> (String, reqwest::Client) {
    let base = serve(build_app(st.clone()).unwrap()).await;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    (base, client)
}

async fn add_confirmed(st: &AppState, email: &str, list: ListName) {
    let (em, token) = (email.to_string(), util::new_token());
    st.db
        .call(move |c| {
            let now = util::now_iso();
            db::upsert_pending(c, &em, list, &token, &now, None)?;
            db::confirm_by_token(c, &token, &now)?;
            Ok(())
        })
        .await
        .unwrap();
}

async fn status_of(st: &AppState, email: &str, list: ListName) -> Option<String> {
    let em = email.to_string();
    st.db
        .call(move |c| {
            use rusqlite::OptionalExtension;
            Ok(c.query_row(
                "SELECT status FROM subscriptions WHERE email = ? AND list = ?",
                rusqlite::params![em, list.as_str()],
                |r| r.get(0),
            )
            .optional()?)
        })
        .await
        .unwrap()
}

async fn has_sent(st: &AppState, slug: &str, list: ListName) -> bool {
    let slug = slug.to_string();
    st.db
        .call(move |c| db::has_sent(c, &slug, list))
        .await
        .unwrap()
}

// ---- send job ----

const SLUG: &str = "my-post";

/// Register a running job for `SLUG` on the blog list and run it to completion.
async fn run_job(st: &AppState, cancel: bool) -> SendJob {
    let id = util::new_token();
    st.jobs.lock().unwrap().insert(
        id.clone(),
        SendJob {
            id: id.clone(),
            slug: SLUG.to_string(),
            list: "blog".to_string(),
            total: 0,
            processed: 0,
            sent: 0,
            skipped: 0,
            bounced: 0,
            failed: 0,
            status: "running",
            error: None,
            finished_at: None,
            cancel,
        },
    );
    let post = JobPost {
        slug: SLUG.to_string(),
        list: ListName::Blog,
        title: "My post".to_string(),
        author: "me".to_string(),
        date: "2026-09-29".to_string(),
        url: "https://site.test/blog/my-post".to_string(),
        content_html: "<p>hi</p>".to_string(),
        markdown: "hi".to_string(),
    };
    run_send_job(st.clone(), id.clone(), post).await;
    let job = st.jobs.lock().unwrap().get(&id).cloned().unwrap();
    job
}

async fn deliveries(st: &AppState) -> Vec<String> {
    st.db
        .call(|c| {
            let mut stmt =
                c.prepare("SELECT email FROM deliveries WHERE slug = ? ORDER BY email")?;
            let rows = stmt
                .query_map([SLUG], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            Ok(rows)
        })
        .await
        .unwrap()
}

#[tokio::test]
async fn send_job_delivers_to_confirmed_subscribers_of_its_list_only() {
    let cf = FakeCf::ok().await;
    let st = state(Some(&cf), |_| {});
    add_confirmed(&st, "a@x.co", ListName::Blog).await;
    add_confirmed(&st, "b@x.co", ListName::Blog).await;
    add_confirmed(&st, "notes-only@x.co", ListName::Notes).await;
    st.db
        .call(|c| db::upsert_pending(c, "pending@x.co", ListName::Blog, "t", "2026-01-01", None))
        .await
        .unwrap();

    let job = run_job(&st, false).await;

    assert_eq!(job.status, "done");
    assert_eq!((job.total, job.sent, job.processed), (2, 2, 2));
    let mut to = cf.recipients();
    to.sort();
    assert_eq!(to, ["a@x.co", "b@x.co"]);
    assert_eq!(deliveries(&st).await, ["a@x.co", "b@x.co"]);
    assert!(has_sent(&st, SLUG, ListName::Blog).await);

    // Each copy carries its recipient's own one-click unsubscribe header.
    let payloads = cf.sent.lock().unwrap().clone();
    let unsub = |p: &Value| {
        p["headers"]["List-Unsubscribe"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert_ne!(unsub(&payloads[0]), unsub(&payloads[1]));
}

#[tokio::test]
async fn resend_skips_everyone_already_delivered() {
    let cf = FakeCf::ok().await;
    let st = state(Some(&cf), |_| {});
    for email in ["a@x.co", "b@x.co", "c@x.co"] {
        add_confirmed(&st, email, ListName::Blog).await;
    }
    let now = util::now_iso();
    st.db
        .call(move |c| db::record_delivery(c, SLUG, ListName::Blog, "b@x.co", &now))
        .await
        .unwrap();

    let job = run_job(&st, false).await;

    assert_eq!((job.sent, job.skipped, job.processed), (2, 1, 3));
    assert!(!cf.recipients().contains(&"b@x.co".to_string()));
    let recorded = st
        .db
        .call(|c| {
            Ok(c.query_row("SELECT recipients FROM sent_posts", [], |r| {
                r.get::<_, i64>(0)
            })?)
        })
        .await
        .unwrap();
    assert_eq!(recorded, 3, "skipped + freshly sent");
}

#[tokio::test]
async fn failures_leave_the_post_unrecorded_so_a_resend_retries_only_them() {
    let failing = Arc::new(Mutex::new(true));
    let cf = FakeCf::start({
        let failing = failing.clone();
        move |to| {
            if to == "flaky@x.co" && *failing.lock().unwrap() {
                (500, json!({ "success": false }))
            } else {
                (200, json!({ "success": true }))
            }
        }
    })
    .await;
    let st = state(Some(&cf), |_| {});
    add_confirmed(&st, "a@x.co", ListName::Blog).await;
    add_confirmed(&st, "flaky@x.co", ListName::Blog).await;

    let job = run_job(&st, false).await;
    assert_eq!(job.status, "done");
    assert_eq!((job.sent, job.failed), (1, 1));
    assert!(!has_sent(&st, SLUG, ListName::Blog).await);
    assert_eq!(deliveries(&st).await, ["a@x.co"]);

    *failing.lock().unwrap() = false;
    cf.sent.lock().unwrap().clear();
    let job = run_job(&st, false).await;
    assert_eq!((job.sent, job.skipped, job.failed), (1, 1, 0));
    assert_eq!(cf.recipients(), ["flaky@x.co"]);
    assert!(has_sent(&st, SLUG, ListName::Blog).await);
}

#[tokio::test]
async fn permanent_bounce_in_a_2xx_marks_the_address_bounced() {
    let cf = FakeCf::start(|to| {
        let bounces: Vec<&str> = if to == "gone@x.co" { vec![to] } else { vec![] };
        (
            200,
            json!({ "success": true, "result": { "permanent_bounces": bounces } }),
        )
    })
    .await;
    let st = state(Some(&cf), |_| {});
    add_confirmed(&st, "a@x.co", ListName::Blog).await;
    add_confirmed(&st, "gone@x.co", ListName::Blog).await;

    let job = run_job(&st, false).await;

    assert_eq!((job.sent, job.bounced, job.failed), (1, 1, 0));
    assert_eq!(
        status_of(&st, "gone@x.co", ListName::Blog).await.as_deref(),
        Some("bounced")
    );
    assert_eq!(deliveries(&st).await, ["a@x.co"]);
    assert!(
        has_sent(&st, SLUG, ListName::Blog).await,
        "a bounce is not a failure"
    );
}

#[tokio::test]
async fn job_aborts_after_consecutive_failures() {
    let cf = FakeCf::start(|_| (400, json!({ "success": false }))).await;
    let st = state(Some(&cf), |_| {});
    for i in 0..8 {
        add_confirmed(&st, &format!("r{i}@x.co"), ListName::Blog).await;
    }

    let job = run_job(&st, false).await;

    assert_eq!(job.status, "error");
    assert_eq!(job.failed, ABORT_AFTER_CONSECUTIVE_FAILURES);
    assert_eq!(
        cf.sent.lock().unwrap().len() as i64,
        ABORT_AFTER_CONSECUTIVE_FAILURES
    );
    assert!(job.error.unwrap().contains("in a row"));
    assert!(!has_sent(&st, SLUG, ListName::Blog).await);
}

#[tokio::test]
async fn a_success_resets_the_failure_streak() {
    // Recipients go out in insertion order: 4 failures, 1 success, 4 failures.
    let cf = FakeCf::start(|to| {
        if to.starts_with("fail") {
            (400, json!({ "success": false }))
        } else {
            (200, json!({ "success": true }))
        }
    })
    .await;
    let st = state(Some(&cf), |_| {});
    for i in 0..4 {
        add_confirmed(&st, &format!("fail{i}@x.co"), ListName::Blog).await;
    }
    add_confirmed(&st, "ok@x.co", ListName::Blog).await;
    for i in 4..8 {
        add_confirmed(&st, &format!("fail{i}@x.co"), ListName::Blog).await;
    }

    let job = run_job(&st, false).await;

    assert_eq!(job.status, "done");
    assert_eq!((job.sent, job.failed, job.processed), (1, 8, 9));
}

#[tokio::test]
async fn cancelled_job_sends_nothing_further_and_records_nothing() {
    let cf = FakeCf::ok().await;
    let st = state(Some(&cf), |_| {});
    add_confirmed(&st, "a@x.co", ListName::Blog).await;

    let job = run_job(&st, true).await;

    assert_eq!(job.status, "cancelled");
    assert!(cf.recipients().is_empty());
    assert!(!has_sent(&st, SLUG, ListName::Blog).await);
}

// ---- /subscribe + /confirm ----

async fn subscribe(base: &str, client: &reqwest::Client, body: Value) -> (StatusCode, Value) {
    let res = client
        .post(format!("{base}/subscribe"))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = StatusCode::from_u16(res.status().as_u16()).unwrap();
    (status, res.json().await.unwrap())
}

fn signup(email: &str) -> Value {
    json!({ "email": email, "lists": ["blog", "notes"] })
}

/// The confirm token from the one confirmation email `cf` received.
fn confirm_token_sent(cf: &FakeCf) -> String {
    let sent = cf.sent.lock().unwrap();
    assert_eq!(sent.len(), 1, "exactly one confirmation email");
    let text = sent[0]["text"].as_str().unwrap();
    let (_, rest) = text.split_once("/newsletter/confirm?t=").unwrap();
    rest.split_whitespace().next().unwrap().to_string()
}

#[tokio::test]
async fn signup_then_confirm_subscribes_to_every_requested_list() {
    let cf = FakeCf::ok().await;
    let st = state(Some(&cf), |_| {});
    let (base, client) = start(&st).await;

    let (status, body) = subscribe(&base, &client, signup(" New@X.co ")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(cf.recipients(), ["new@x.co"], "address is normalized");
    assert_eq!(
        status_of(&st, "new@x.co", ListName::Blog).await.as_deref(),
        Some("pending")
    );

    let token = confirm_token_sent(&cf);

    // A GET (mail scanner, link prefetch) only redirects to the site page.
    let res = client
        .get(format!("{base}/confirm?t={token}"))
        .send()
        .await
        .unwrap();
    assert!(res.status().is_redirection());
    assert_eq!(
        res.headers()["location"],
        format!("https://site.test/newsletter/confirm?t={token}").as_str()
    );
    assert_eq!(
        status_of(&st, "new@x.co", ListName::Blog).await.as_deref(),
        Some("pending")
    );

    let confirm = |t: String| {
        client
            .post(format!("{base}/confirm"))
            .json(&json!({ "t": t }))
            .send()
    };
    assert_eq!(confirm("bogus".into()).await.unwrap().status(), 404);
    assert_eq!(confirm(String::new()).await.unwrap().status(), 400);
    assert_eq!(confirm(token).await.unwrap().status(), 200);
    for list in [ListName::Blog, ListName::Notes] {
        assert_eq!(
            status_of(&st, "new@x.co", list).await.as_deref(),
            Some("confirmed")
        );
    }
}

#[tokio::test]
async fn honeypot_signup_pretends_to_succeed() {
    let cf = FakeCf::ok().await;
    let st = state(Some(&cf), |_| {});
    let (base, client) = start(&st).await;

    let mut body = signup("bot@x.co");
    body["website"] = json!("http://spam.example");
    let (status, _) = subscribe(&base, &client, body).await;

    assert_eq!(status, StatusCode::OK);
    assert!(cf.recipients().is_empty());
    assert_eq!(status_of(&st, "bot@x.co", ListName::Blog).await, None);
}

#[tokio::test]
async fn invalid_signups_are_rejected() {
    let cf = FakeCf::ok().await;
    let st = state(Some(&cf), |_| {});
    let (base, client) = start(&st).await;

    let (status, _) = subscribe(&base, &client, signup("not-an-email")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = subscribe(&base, &client, json!({ "email": "a@x.co", "lists": [] })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let res = client
        .post(format!("{base}/subscribe"))
        .json(&json!({ "email": "a@x.co", "lists": ["secret"] }))
        .send()
        .await
        .unwrap();
    assert!(res.status().is_client_error(), "unknown list name");
    assert!(cf.recipients().is_empty());
}

#[tokio::test]
async fn resend_cooldown_blocks_inbox_bombing_across_aliases() {
    let cf = FakeCf::ok().await;
    let st = state(Some(&cf), |_| {});
    let (base, client) = start(&st).await;

    subscribe(&base, &client, signup("victim@gmail.com")).await;
    let token = confirm_token_sent(&cf);

    for alias in [
        "victim@gmail.com",
        "victim+1@gmail.com",
        "v.ictim@gmail.com",
    ] {
        let (status, _) = subscribe(&base, &client, signup(alias)).await;
        assert_eq!(status, StatusCode::OK, "silently accepted");
    }
    assert_eq!(
        cf.recipients().len(),
        1,
        "no second email inside the cooldown"
    );

    // The blocked attempts left the live link working.
    let res = client
        .post(format!("{base}/confirm"))
        .json(&json!({ "t": token }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn suppressed_addresses_are_never_mailed_or_reset() {
    let cf = FakeCf::ok().await;
    let st = state(Some(&cf), |_| {});
    add_confirmed(&st, "gone@x.co", ListName::Blog).await;
    st.db
        .call(|c| db::suppress_email(c, "gone@x.co", "complaint"))
        .await
        .unwrap();
    let (base, client) = start(&st).await;

    let (status, _) = subscribe(&base, &client, signup("gone@x.co")).await;

    assert_eq!(status, StatusCode::OK);
    assert!(cf.recipients().is_empty());
    assert_eq!(
        status_of(&st, "gone@x.co", ListName::Blog).await.as_deref(),
        Some("bounced")
    );
    assert_eq!(status_of(&st, "gone@x.co", ListName::Notes).await, None);
}

#[tokio::test]
async fn already_confirmed_signup_sends_nothing() {
    let cf = FakeCf::ok().await;
    let st = state(Some(&cf), |_| {});
    add_confirmed(&st, "a@x.co", ListName::Blog).await;
    let (base, client) = start(&st).await;

    let (status, body) = subscribe(
        &base,
        &client,
        json!({ "email": "a@x.co", "lists": ["blog"] }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["alreadySubscribed"], true);
    assert!(cf.recipients().is_empty());
}

#[tokio::test]
async fn cap_hit_queues_the_confirmation_until_the_queue_is_full() {
    let cf = FakeCf::ok().await;
    let st = state(Some(&cf), |c| {
        c.global_confirm_cap_per_hour = 0;
        c.confirm_queue_max = 1;
    });
    let (base, client) = start(&st).await;

    let (status, body) = subscribe(&base, &client, signup("first@x.co")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["delayed"], true);
    assert!(cf.recipients().is_empty(), "queued, not sent");
    assert_eq!(st.db.call(db::count_queued_confirmations).await.unwrap(), 1);

    let (status, _) = subscribe(&base, &client, signup("second@x.co")).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!(cf.recipients().is_empty());
}

#[tokio::test]
async fn per_ip_rate_limit() {
    let st = state(None, |c| c.rate_per_min_per_ip = 2);
    let (base, client) = start(&st).await;

    for i in 0..2 {
        let (status, _) = subscribe(&base, &client, signup(&format!("r{i}@x.co"))).await;
        assert_eq!(status, StatusCode::OK);
    }
    let (status, _) = subscribe(&base, &client, signup("r2@x.co")).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn confirmation_bounce_suppresses_and_send_failure_is_reported() {
    let cf = FakeCf::start(|to| match to {
        "gone@x.co" => (200, json!({ "result": { "permanent_bounces": [to] } })),
        "down@x.co" => (500, json!({ "success": false })),
        _ => (200, json!({ "success": true })),
    })
    .await;
    let st = state(Some(&cf), |_| {});
    let (base, client) = start(&st).await;

    subscribe(&base, &client, signup("gone@x.co")).await;
    assert_eq!(
        status_of(&st, "gone@x.co", ListName::Blog).await.as_deref(),
        Some("bounced")
    );

    let (status, _) = subscribe(&base, &client, signup("down@x.co")).await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
}

// ---- client IP source ----

/// POST a fresh signup with the given extra headers; returns the status.
async fn signup_from(
    base: &str,
    client: &reqwest::Client,
    n: &mut u32,
    headers: &[(&str, &str)],
) -> u16 {
    *n += 1;
    let mut req = client
        .post(format!("{base}/subscribe"))
        .json(&signup(&format!("r{n}@x.co")));
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    req.send().await.unwrap().status().as_u16()
}

#[tokio::test]
async fn cf_mode_limits_each_visitor_by_cf_connecting_ip_and_ignores_xff() {
    let st = state(None, |c| {
        c.client_ip_header = ClientIpHeader::CfConnectingIp;
        c.rate_per_min_per_ip = 1;
    });
    let (base, client) = start(&st).await;
    let n = &mut 0;

    assert_eq!(
        signup_from(&base, &client, n, &[("cf-connecting-ip", "198.51.100.1")]).await,
        200
    );
    assert_eq!(
        signup_from(&base, &client, n, &[("cf-connecting-ip", "198.51.100.1")]).await,
        429
    );
    // A different visitor behind the same Cloudflare edge has their own budget.
    assert_eq!(
        signup_from(&base, &client, n, &[("cf-connecting-ip", "198.51.100.2")]).await,
        200
    );
    // Rotating X-Forwarded-For doesn't mint new budgets in this mode.
    let spoof = [
        ("cf-connecting-ip", "198.51.100.1"),
        ("x-forwarded-for", "203.0.113.9"),
    ];
    assert_eq!(signup_from(&base, &client, n, &spoof).await, 429);
}

#[tokio::test]
async fn xff_mode_ignores_cf_connecting_ip() {
    let st = state(None, |c| c.rate_per_min_per_ip = 1);
    let (base, client) = start(&st).await;
    let n = &mut 0;

    let xff = [("x-forwarded-for", "198.51.100.1")];
    assert_eq!(signup_from(&base, &client, n, &xff).await, 200);
    // A client-supplied CF-Connecting-IP must not buy a fresh budget here.
    let spoof = [
        ("x-forwarded-for", "198.51.100.1"),
        ("cf-connecting-ip", "203.0.113.9"),
    ];
    assert_eq!(signup_from(&base, &client, n, &spoof).await, 429);
}

#[tokio::test]
async fn cf_mode_admin_lockout_hits_only_the_attacker() {
    let st = state(None, |c| {
        c.client_ip_header = ClientIpHeader::CfConnectingIp;
        c.admin_auth_max_failures = 2;
    });
    let (base, client) = start(&st).await;
    let stats = |ip: &'static str, auth: String| {
        client
            .get(format!("{base}/admin/stats"))
            .header("cf-connecting-ip", ip)
            .header("authorization", auth)
            .send()
    };

    for _ in 0..2 {
        let res = stats("203.0.113.66", "Bearer nope".into()).await.unwrap();
        assert_eq!(res.status(), 401);
    }
    assert_eq!(
        stats("203.0.113.66", bearer(&st)).await.unwrap().status(),
        429
    );
    assert_eq!(
        stats("198.51.100.7", bearer(&st)).await.unwrap().status(),
        200
    );
}

// ---- admin auth ----

const ADMIN_ROUTES: &[(&str, &str)] = &[
    ("GET", "/admin"),
    ("GET", "/admin/config"),
    ("GET", "/admin/stats"),
    ("GET", "/admin/subscribers"),
    ("GET", "/admin/sent"),
    ("GET", "/admin/recipient-count?list=blog"),
    ("POST", "/admin/send"),
    ("GET", "/admin/send-status?jobId=x"),
    ("POST", "/admin/send-test"),
    ("POST", "/admin/send-cancel"),
    ("POST", "/admin/delete-subscriber"),
    ("GET", "/admin/template"),
    ("POST", "/admin/sync-suppressions"),
    ("GET", "/admin/backup/download"),
    ("GET", "/admin/backup/status"),
    ("POST", "/admin/backup/run"),
    ("GET", "/admin/assets/index.js"),
];

fn bearer(st: &AppState) -> String {
    format!("Bearer {}", st.config.admin_token)
}

/// The dashboard page sends a browser to the login form; everything else is a
/// bare 401 (no `WWW-Authenticate: Basic`, so no native browser dialog).
fn assert_refused(res: &reqwest::Response, method: &str, path: &str) {
    if (method, path) == ("GET", "/admin") {
        assert_eq!(res.status(), 303, "{method} {path}");
        assert_eq!(res.headers()["location"], "/admin/login");
    } else {
        assert_eq!(res.status(), 401, "{method} {path}");
        assert!(
            res.headers().get("www-authenticate").is_none(),
            "{method} {path}"
        );
    }
}

#[tokio::test]
async fn every_admin_route_requires_auth() {
    // Every probe below is a failed login; keep the lockout out of the way.
    let st = state(None, |c| c.admin_auth_max_failures = 1000);
    let (base, client) = start(&st).await;

    for (method, path) in ADMIN_ROUTES {
        let req = client.request(method.parse().unwrap(), format!("{base}{path}"));
        let res = req.json(&json!({})).send().await.unwrap();
        assert_refused(&res, method, path);

        let res = client
            .request(method.parse().unwrap(), format!("{base}{path}"))
            .header("authorization", "Bearer wrong-token")
            .json(&json!({}))
            .send()
            .await
            .unwrap();
        assert_refused(&res, method, path);
    }

    let res = client
        .get(format!("{base}/admin/stats"))
        .header("authorization", bearer(&st))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn cross_site_admin_posts_are_refused_even_with_credentials() {
    let st = state(None, |_| {});
    add_confirmed(&st, "a@x.co", ListName::Blog).await;
    let (base, client) = start(&st).await;
    let delete = |site: Option<&'static str>| {
        let mut req = client
            .post(format!("{base}/admin/delete-subscriber"))
            .header("authorization", bearer(&st))
            .json(&json!({ "email": "a@x.co" }));
        if let Some(site) = site {
            req = req.header("sec-fetch-site", site);
        }
        req.send()
    };

    for site in ["cross-site", "same-site", "none"] {
        assert_eq!(delete(Some(site)).await.unwrap().status(), 403, "{site}");
    }
    assert!(status_of(&st, "a@x.co", ListName::Blog).await.is_some());

    assert_eq!(delete(Some("same-origin")).await.unwrap().status(), 200);
    assert_eq!(
        delete(None).await.unwrap().status(),
        404,
        "CLI clients send no Sec-Fetch-Site and get through (row already gone)"
    );
}

#[tokio::test]
async fn repeated_bad_logins_lock_out_the_ip() {
    let st = state(None, |c| c.admin_auth_max_failures = 3);
    let (base, client) = start(&st).await;
    let stats = |auth: String| {
        client
            .get(format!("{base}/admin/stats"))
            .header("authorization", auth)
            .send()
    };

    // A success clears the count…
    for _ in 0..2 {
        assert_eq!(stats("Bearer nope".into()).await.unwrap().status(), 401);
    }
    assert_eq!(stats(bearer(&st)).await.unwrap().status(), 200);
    for _ in 0..2 {
        assert_eq!(stats("Bearer nope".into()).await.unwrap().status(), 401);
    }
    assert_eq!(stats(bearer(&st)).await.unwrap().status(), 200);

    // …but reaching the limit locks out even the right token.
    for _ in 0..3 {
        assert_eq!(stats("Bearer nope".into()).await.unwrap().status(), 401);
    }
    assert_eq!(stats(bearer(&st)).await.unwrap().status(), 429);
}

#[tokio::test]
async fn credential_less_visits_do_not_count_toward_the_lockout() {
    let st = state(None, |c| c.admin_auth_max_failures = 2);
    let (base, client) = start(&st).await;

    // A browser opening the dashboard with no session, many visits over.
    for _ in 0..5 {
        let res = client.get(format!("{base}/admin")).send().await.unwrap();
        assert_eq!(res.status(), 303);
        let res = client
            .get(format!("{base}/admin/stats"))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
    }
    let res = client
        .get(format!("{base}/admin/stats"))
        .header("authorization", bearer(&st))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}

// ---- dashboard login ----

async fn login(base: &str, client: &reqwest::Client, password: &str) -> reqwest::Response {
    client
        .post(format!("{base}/admin/login"))
        .form(&[("username", "admin"), ("password", password)])
        .send()
        .await
        .unwrap()
}

/// The `name=value` part of a login response's Set-Cookie.
fn session_from(res: &reqwest::Response) -> String {
    let set = res.headers()["set-cookie"].to_str().unwrap();
    set.split(';').next().unwrap().to_string()
}

#[tokio::test]
async fn login_page_is_a_form_password_managers_can_fill() {
    let st = state(None, |_| {});
    let (base, client) = start(&st).await;

    let res = client
        .get(format!("{base}/admin/login"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.headers()["cache-control"], "no-store");
    let html = res.text().await.unwrap();
    assert!(html.contains(r#"<form method="post" action="/admin/login">"#));
    assert!(html.contains(r#"autocomplete="username""#));
    assert!(html.contains(r#"type="password" name="password" autocomplete="current-password""#));
}

#[tokio::test]
async fn login_sets_a_session_that_opens_the_dashboard() {
    let st = state(None, |_| {});
    let (base, client) = start(&st).await;

    let res = login(&base, &client, "wrong").await;
    assert_eq!(res.status(), 401);
    assert!(res.headers().get("set-cookie").is_none());
    assert!(res.text().await.unwrap().contains("Wrong password."));

    let res = login(&base, &client, &st.config.admin_token).await;
    assert_eq!(res.status(), 303);
    assert_eq!(res.headers()["location"], "/admin");
    let set = res.headers()["set-cookie"].to_str().unwrap().to_string();
    for attr in [
        "Path=/admin",
        "HttpOnly",
        "SameSite=Lax",
        "Secure",
        "Max-Age=2592000",
    ] {
        assert!(set.contains(attr), "{attr} missing from {set}");
    }
    let cookie = session_from(&res);

    for path in ["/admin", "/admin/stats", "/admin/config"] {
        let res = client
            .get(format!("{base}{path}"))
            .header("cookie", &cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200, "{path}");
    }
    // The session also covers state-changing calls from the dashboard.
    let res = client
        .post(format!("{base}/admin/send-cancel"))
        .header("cookie", &cookie)
        .header("sec-fetch-site", "same-origin")
        .json(&json!({ "jobId": "none" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404, "reached the handler");
}

#[tokio::test]
async fn forged_or_cleared_sessions_are_refused() {
    let st = state(None, |_| {});
    let (base, client) = start(&st).await;
    let stats = |cookie: String| {
        client
            .get(format!("{base}/admin/stats"))
            .header("cookie", cookie)
            .send()
    };

    let forged = format!(
        "nl_admin={}.{}",
        chrono::Utc::now().timestamp() + 60,
        "0".repeat(64)
    );
    assert_eq!(stats(forged).await.unwrap().status(), 401);
    let other_token = auth::new_session("some-other-token", chrono::Utc::now().timestamp());
    assert_eq!(
        stats(format!("nl_admin={other_token}"))
            .await
            .unwrap()
            .status(),
        401
    );

    let res = client
        .post(format!("{base}/admin/logout"))
        .header("sec-fetch-site", "same-origin")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 303);
    assert_eq!(res.headers()["location"], "/admin/login");
    let set = res.headers()["set-cookie"].to_str().unwrap();
    assert!(
        set.starts_with("nl_admin=;") && set.contains("Max-Age=0"),
        "{set}"
    );
}

#[tokio::test]
async fn wrong_passwords_at_the_login_form_count_toward_the_lockout() {
    let st = state(None, |c| c.admin_auth_max_failures = 2);
    let (base, client) = start(&st).await;

    for _ in 0..2 {
        assert_eq!(login(&base, &client, "guess").await.status(), 401);
    }
    let res = login(&base, &client, &st.config.admin_token).await;
    assert_eq!(res.status(), 429, "locked out even with the right password");
}

#[tokio::test]
async fn cross_site_login_and_logout_are_refused() {
    let st = state(None, |_| {});
    let (base, client) = start(&st).await;

    let res = client
        .post(format!("{base}/admin/login"))
        .header("sec-fetch-site", "cross-site")
        .form(&[("password", st.config.admin_token.as_str())])
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 403);
    assert!(res.headers().get("set-cookie").is_none());

    let res = client
        .post(format!("{base}/admin/logout"))
        .header("sec-fetch-site", "cross-site")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn plain_http_dev_sessions_are_not_marked_secure() {
    let st = state(None, |c| {
        c.public_base = "http://localhost:8787".to_string()
    });
    let (base, client) = start(&st).await;

    let res = login(&base, &client, &st.config.admin_token).await;
    let set = res.headers()["set-cookie"].to_str().unwrap();
    assert!(!set.contains("Secure"), "{set}");
}

// ---- backups ----

/// A stand-in for R2's S3 API: records each request and answers with `status`.
struct FakeR2 {
    base: String,
    puts: Arc<Mutex<Vec<(String, HeaderMap, Bytes)>>>,
}

impl FakeR2 {
    async fn start(status: u16) -> FakeR2 {
        let puts = Arc::new(Mutex::new(Vec::new()));
        let app = Router::new().fallback({
            let puts = puts.clone();
            move |method: Method, uri: axum::http::Uri, headers: HeaderMap, body: Bytes| {
                let puts = puts.clone();
                async move {
                    assert_eq!(method, Method::PUT);
                    puts.lock()
                        .unwrap()
                        .push((uri.path().to_string(), headers, body));
                    let body = if status == 200 {
                        ""
                    } else {
                        "<Error><Code>AccessDenied</Code></Error>"
                    };
                    (StatusCode::from_u16(status).unwrap(), body)
                }
            }
        });
        FakeR2 {
            base: serve(app).await,
            puts,
        }
    }

    fn config(&self) -> config::R2Config {
        config::R2Config {
            endpoint: self.base.clone(),
            access_key_id: "AKIDTEST".to_string(),
            secret_access_key: "test-secret".to_string(),
            bucket: "backups".to_string(),
            prefix: "nl".to_string(),
        }
    }
}

/// Gunzip a backup and open it as a database; returns its subscription count.
fn subscriptions_in_backup(gz: &[u8]) -> i64 {
    use std::io::Read;
    let mut raw = Vec::new();
    flate2::read::GzDecoder::new(gz)
        .read_to_end(&mut raw)
        .unwrap();
    let path = std::env::temp_dir().join(format!("restore-test-{}.db", util::new_token()));
    std::fs::write(&path, raw).unwrap();
    let conn = rusqlite::Connection::open(&path).unwrap();
    let n = conn
        .query_row("SELECT COUNT(*) FROM subscriptions", [], |r| r.get(0))
        .unwrap();
    drop(conn);
    std::fs::remove_file(&path).unwrap();
    n
}

#[tokio::test]
async fn dashboard_downloads_a_restorable_backup() {
    let st = state(None, |_| {});
    add_confirmed(&st, "a@x.co", ListName::Blog).await;
    add_confirmed(&st, "b@x.co", ListName::Notes).await;
    let (base, client) = start(&st).await;

    let res = client
        .get(format!("{base}/admin/backup/download"))
        .header("authorization", bearer(&st))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.headers()["content-type"], "application/gzip");
    let disposition = res.headers()["content-disposition"]
        .to_str()
        .unwrap()
        .to_string();
    assert!(
        disposition.starts_with("attachment; filename=\"newsletter-")
            && disposition.ends_with(".db.gz\""),
        "{disposition}"
    );
    assert_eq!(subscriptions_in_backup(&res.bytes().await.unwrap()), 2);
}

#[tokio::test]
async fn backup_now_uploads_a_signed_snapshot_to_r2() {
    let r2 = FakeR2::start(200).await;
    let r2_config = r2.config();
    let st = state(None, |c| c.r2 = Some(r2_config));
    add_confirmed(&st, "a@x.co", ListName::Blog).await;
    let (base, client) = start(&st).await;

    let res = client
        .post(format!("{base}/admin/backup/run"))
        .header("authorization", bearer(&st))
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let (path, headers, body) = r2.puts.lock().unwrap()[0].clone();
    assert!(
        path.starts_with("/backups/nl/newsletter-") && path.ends_with(".db.gz"),
        "{path}"
    );
    assert_eq!(headers["content-type"], "application/gzip");
    let body_hash: String = <sha2::Sha256 as sha2::Digest>::digest(&body)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(headers["x-amz-content-sha256"], body_hash.as_str());
    let auth = headers["authorization"].to_str().unwrap();
    assert!(
        auth.starts_with("AWS4-HMAC-SHA256 Credential=AKIDTEST/"),
        "{auth}"
    );
    assert!(auth.contains("/auto/s3/aws4_request, SignedHeaders=content-type;host;x-amz-content-sha256;x-amz-date, Signature="));
    assert_eq!(subscriptions_in_backup(&body), 1);

    let status: Value = client
        .get(format!("{base}/admin/backup/status"))
        .header("authorization", bearer(&st))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["r2Configured"], true);
    assert_eq!(status["lastSuccess"]["ok"], true);
    assert_eq!(status["lastSuccess"]["bytes"], body.len());
    assert_eq!(
        format!(
            "/backups/{}",
            status["lastSuccess"]["objectKey"].as_str().unwrap()
        ),
        path
    );
}

#[tokio::test]
async fn a_failed_r2_upload_is_recorded_and_reported() {
    let r2 = FakeR2::start(403).await;
    let r2_config = r2.config();
    let st = state(None, |c| c.r2 = Some(r2_config));
    let (base, client) = start(&st).await;

    let res = client
        .post(format!("{base}/admin/backup/run"))
        .header("authorization", bearer(&st))
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);

    let status: Value = client
        .get(format!("{base}/admin/backup/status"))
        .header("authorization", bearer(&st))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["last"]["ok"], false);
    let error = status["last"]["error"].as_str().unwrap();
    assert!(
        error.contains("403") && error.contains("AccessDenied"),
        "{error}"
    );
    assert_eq!(status["lastSuccess"], Value::Null);
}

#[tokio::test]
async fn backup_now_without_r2_says_it_is_not_configured() {
    let st = state(None, |_| {});
    let (base, client) = start(&st).await;

    let res = client
        .post(format!("{base}/admin/backup/run"))
        .header("authorization", bearer(&st))
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
    let status: Value = client
        .get(format!("{base}/admin/backup/status"))
        .header("authorization", bearer(&st))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["r2Configured"], false);
}

#[tokio::test]
async fn scheduled_backup_runs_once_a_day() {
    let r2 = FakeR2::start(200).await;
    let r2_config = r2.config();
    let st = state(None, |c| c.r2 = Some(r2_config.clone()));

    scheduled_backup(&st, &r2_config).await.unwrap();
    scheduled_backup(&st, &r2_config).await.unwrap();
    assert_eq!(
        r2.puts.lock().unwrap().len(),
        1,
        "the second check isn't due"
    );
}

// ---- admin send ----

fn send_body() -> Value {
    json!({
        "list": "blog",
        "slug": SLUG,
        "title": "My post",
        "url": "https://site.test/blog/my-post",
        "body": "Hello **world**",
    })
}

#[tokio::test]
async fn admin_send_runs_the_blast_and_refuses_duplicates() {
    let cf = FakeCf::ok().await;
    let st = state(Some(&cf), |_| {});
    add_confirmed(&st, "a@x.co", ListName::Blog).await;
    let (base, client) = start(&st).await;
    let send = || {
        client
            .post(format!("{base}/admin/send"))
            .header("authorization", bearer(&st))
            .json(&send_body())
            .send()
    };

    let body: Value = send().await.unwrap().json().await.unwrap();
    let job_id = body["jobId"].as_str().unwrap().to_string();
    let mut status = Value::Null;
    for _ in 0..100 {
        status = client
            .get(format!("{base}/admin/send-status?jobId={job_id}"))
            .header("authorization", bearer(&st))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if status["status"] != "running" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(status["status"], "done", "{status}");
    assert_eq!(status["sent"], 1);
    let payload = cf.sent.lock().unwrap()[0].clone();
    assert!(payload["html"]
        .as_str()
        .unwrap()
        .contains("<strong>world</strong>"));
    assert!(
        payload["text"].as_str().unwrap().contains("EmNudge"),
        "default author"
    );

    let again: Value = send().await.unwrap().json().await.unwrap();
    assert_eq!(again["skipped"], true, "a sent post is never blasted twice");
    assert_eq!(cf.recipients().len(), 1);
}

#[tokio::test]
async fn admin_send_conflicts_with_a_running_blast_for_the_same_post() {
    let st = state(None, |_| {});
    st.jobs.lock().unwrap().insert(
        "running".to_string(),
        SendJob {
            id: "running".to_string(),
            slug: SLUG.to_string(),
            list: "blog".to_string(),
            total: 1,
            processed: 0,
            sent: 0,
            skipped: 0,
            bounced: 0,
            failed: 0,
            status: "running",
            error: None,
            finished_at: None,
            cancel: false,
        },
    );
    let (base, client) = start(&st).await;

    let res = client
        .post(format!("{base}/admin/send"))
        .header("authorization", bearer(&st))
        .json(&send_body())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);

    let res = client
        .post(format!("{base}/admin/send-cancel"))
        .header("authorization", bearer(&st))
        .json(&json!({ "jobId": "running" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    assert!(st.jobs.lock().unwrap()["running"].cancel);
}
