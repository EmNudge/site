# newsletter-service

Self-hosted email subscription backend for the site. It collects subscribers
(double opt-in), handles confirm/unsubscribe, and sends full-content post emails through
**Cloudflare Email Service**. It runs on a VPS so that a flood of signups hits a
fixed-cost server instead of a per-request Cloudflare bill — and it gates every
outbound confirmation email behind Turnstile + rate limits + a global hourly cap
so an attack can never run up the Cloudflare email cost.

**Stack**: a single Rust binary — [`axum`](https://github.com/tokio-rs/axum) (HTTP) +
[`rusqlite`](https://github.com/rusqlite/rusqlite) (SQLite) +
[`reqwest`](https://github.com/seanmonstar/reqwest) (rustls) +
[`pulldown-cmark`](https://github.com/pulldown-cmark/pulldown-cmark) (markdown). Chosen for
memory safety and mature, well-audited libraries. The public React admin UI in
`admin/` is unchanged and still built with Vite. The HTTP surface, env vars, JSON
shapes, and SQLite schema are all identical to the previous Node/Hono service, so
this is a drop-in replacement (it opens an existing `newsletter.db` as-is).

**Hardening** (this service is on public GitHub and internet-facing): constant-time
admin-token comparison with a per-IP brute-force throttle, per-IP + global rate/cost
caps, per-email resend cooldown, honeypot + Turnstile bot gate,
strict body-size limits, request timeouts, security response headers, parameterized
SQL only, no-panic error handling (`panic = abort`), graceful shutdown, and a
tightly sandboxed systemd unit (see `deploy/newsletter.service`).

```
signup     site <NewsletterSignup> ──POST /subscribe──▶ this service (VPS) ──REST──▶ Cloudflare Email Service
                                                            │  SQLite (subscribers + deliveries)
confirm    email link ──▶ site /newsletter/confirm ──(click)──POST /confirm──▶ this service
unsub      email link ──▶ site /newsletter/unsubscribe ──POST /unsubscribe──▶ this service
blasts     admin dashboard (React) ──POST /admin/send──▶ background job ──REST──▶ Cloudflare
```

The whole system spans two places: the **static Astro site** (signup form + confirm/unsub
pages, in the parent repo — see "Site-side integration" below) and **this service** (the
API, subscriber DB, Cloudflare sending, and the admin dashboard).

## Endpoints

| Route              | Auth            | Purpose |
| ------------------ | --------------- | ------- |
| `POST /subscribe`  | Turnstile + rate limit | Store pending sub(s), send one confirmation email covering all chosen lists. Returns `{ ok, delayed: true }` when the email was queued behind a send cap. |
| `POST /confirm`    | confirm token   | Confirm. Body `{ t }`. Called by the site's confirm page when the reader clicks the button. |
| `GET /confirm?t=`  | —               | Legacy link from older emails. Changes nothing; **redirects** to `SITE_ORIGIN/newsletter/confirm?t=`. |
| `POST /unsubscribe`| unsub token     | Unsubscribe. Body `{ t }` or `?t=` (one-click `List-Unsubscribe-Post`). Returns `{ ok, list }`. |
| `POST /resubscribe`| unsub token     | Undo an unsubscribe — restores to confirmed. Body `{ t }`. Returns `{ ok, list }`. |
| `POST /unsubscribe-reason` | unsub token | Optional post-unsubscribe feedback. Body `{ t, reason?, note? }`. Surfaced in the dashboard. |
| `POST /admin/send` | admin           | Start a **background blast** to confirmed subscribers. Body `{ list, slug, title, author?, date, url, body }` (`body` = raw markdown). Returns `{ jobId }`, or `{ skipped }` if `(slug,list)` already sent. Resumes via the `deliveries` log. |
| `GET /admin/send-status?jobId=` | admin | Job progress: `{ total, processed, sent, skipped, bounced, failed, status }`. `status` is `running`, `done`, `error` or `cancelled`. |
| `POST /admin/send-test` | admin | Send one copy of a post to a single address. Body is the `/admin/send` body plus `to`. Subject is prefixed `[TEST]`; nothing is recorded. |
| `POST /admin/send-cancel` | admin | Stop a running blast before its next recipient. Body `{ jobId }`. Re-sending resumes with the rest. |
| `POST /admin/delete-subscriber` | admin | Erase an address from every list, with its delivery records. Body `{ email }`. Returns `{ ok, deleted }`. |
| `GET /admin/recipient-count?list=` | admin | Confirmed subscriber count for a list (for the send UI). |
| `POST /admin/sync-suppressions` | admin | Force a Cloudflare suppression-list sync now. Returns `{ scanned, actionable, suppressed }`. |
| `GET /admin`       | admin           | The React admin app (SPA). |
| `GET /admin/config`| admin           | Runtime config for the SPA: `{ siteOrigin, authorName }`. |
| `GET /admin/template` | admin        | Renders an email/page template for the live preview builder. |
| `GET /admin/stats` | admin           | Totals by status, per-list breakdown, emails sent, signups 7d/30d, unsub reasons. |
| `GET /admin/subscribers` | admin     | Searchable/paginated subscriber list. Query: `q`, `list`, `status`, `limit`, `offset`. |
| `GET /admin/sent`  | admin           | Recent sent newsletters. |
| `GET /health`      | —               | Liveness check. |

**Admin auth**: the dashboard has a login form at `/admin/login` — username `admin`,
password the `ADMIN_TOKEN` — which password managers (1Password etc.) can save and
fill. Signing in sets a signed session cookie (`HttpOnly`, `Secure`, `SameSite=Lax`,
30 days); nothing is stored server-side, so sessions survive restarts and changing
`ADMIN_TOKEN` signs everyone out. Scripts skip the form: every `/admin*` route also
accepts `Authorization: Bearer <ADMIN_TOKEN>` (used by `send-newsletter.mjs`) or HTTP
Basic auth with the token as the password.

**Admin UI**: a Vite + React app in `admin/`, built to `admin/dist` and served by the
service (index at `/admin`, assets at `/admin/assets`, runtime config at `/admin/config`).
It's built on a small design system in `admin/src/ui/` — design tokens in `ui.css`
(spacing/type/radius/semantic-color scales) plus primitives (`Button`, `Card`,
`Metric`, `Section`, `DataTable`, `Badge`/`StatusBadge`, `Field`/`Input`/`Select`/`Textarea`,
`Segmented`, `Progress`). Build new screens from these rather than bespoke markup.
Build it before starting the service (it's served from `admin/dist` at runtime):

```sh
npm --prefix admin install && npm --prefix admin run build
```

For UI development, `npm --prefix admin run dev` runs Vite with the JSON API proxied to
`:8787` (keep the service running).

## One-time setup (prerequisites)

1. **Cloudflare Email Service** — on the Workers **Paid** plan ($5/mo, includes
   3,000 emails/mo), onboard a sending domain (recommend a subdomain like
   `mail.emnudge.dev`) and add the DKIM/SPF/DMARC records it generates. Verify
   the sender (`noreply@mail.emnudge.dev`). Until the domain is fully onboarded
   you can only send to **verified destination addresses** — use one of those to
   test for free.
2. **API token** — create a Cloudflare token scoped to the email-send permission.
   → `CF_EMAIL_TOKEN`, and note your `CF_ACCOUNT_ID`.
3. **Turnstile** — create a widget (free). The **site key** goes in the site's
   `PUBLIC_TURNSTILE_SITE_KEY`; the **secret** goes here as `TURNSTILE_SECRET`.
4. **Admin token** — `openssl rand -hex 32` → `ADMIN_TOKEN` (also used by
   `../send-newsletter.mjs`).
5. **Cloudflare Tunnel + Access** — serve `newsletter.emnudge.dev` through a tunnel
   and put `/admin` behind Cloudflare Access (see **Cloudflare Tunnel + Access**
   below). The alternative is Caddy on the VPS with an unproxied DNS record
   (`deploy/Caddyfile`, `CLIENT_IP_HEADER=x-forwarded-for`).

## Run

Requires a Rust toolchain (`rustup`, stable ≥ 1.82) and Node (for the admin UI only).

```sh
cp .env.example .env                          # fill in the values above
npm --prefix admin install && npm --prefix admin run build   # build the admin UI (served at /admin)
cargo run                                     # dev build
# or a hardened optimized build:
cargo build --release && ./target/release/newsletter-service
```

`cargo` compiles SQLite in (the `bundled` feature), so there are no system SQLite or
OpenSSL dependencies — TLS is rustls.

Production (systemd — runs the compiled binary; only the admin UI needs Node). The
VPS keeps the source and the running files apart:

- `/opt/newsletter-src` — a checkout of the site repo (sparse: just `newsletter-service/`)
- `/opt/newsletter-service` — what systemd runs: the binary, `admin/dist`,
  `data/` (the database) and `.env`

`deploy/update.sh` builds from the first and installs into the second. First install,
as root (needs `git`, `npm`, and Rust via rustup):

```sh
useradd --system --no-create-home --shell /usr/sbin/nologin newsletter
git clone --filter=blob:none --sparse https://github.com/EmNudge/site.git /opt/newsletter-src
git -C /opt/newsletter-src sparse-checkout set newsletter-service
SKIP_PULL=1 /opt/newsletter-src/newsletter-service/deploy/update.sh
install -m600 /opt/newsletter-src/newsletter-service/.env.example /opt/newsletter-service/.env
$EDITOR /opt/newsletter-service/.env
cp /opt/newsletter-src/newsletter-service/deploy/newsletter.service /etc/systemd/system/
systemctl daemon-reload && systemctl enable --now newsletter
```

**Updating** afterwards is `sudo /opt/newsletter-src/newsletter-service/deploy/update.sh`:
it pulls, rebuilds, installs, restarts, and checks `/health`. Don't run it while a
newsletter is sending — the restart ends the send (re-sending resumes it).

The systemd unit is sandboxed (`ProtectSystem=strict`, no capabilities, a syscall
allowlist, etc.); it grants write access only to `/opt/newsletter-service/data`, so
keep `DB_PATH` under that directory.

## Cloudflare Tunnel + Access

The service listens only on `127.0.0.1`; `cloudflared` on the VPS connects *out* to
Cloudflare and carries requests in, so the service has no public port at all. That
is what makes it safe to trust `CF-Connecting-IP` (Cloudflare overwrites it on every
request) for the per-IP guards, and it hides the VPS behind Cloudflare's edge.

1. **Tunnel** — Zero Trust dashboard → **Networks → Tunnels → Create a tunnel**
   (type *Cloudflared*), named e.g. `newsletter`, and copy its token. Skip the
   dashboard's `cloudflared service install` command — it writes the generic
   `cloudflared` unit and would replace any tunnel the VPS already runs. Use the
   dedicated unit instead (`cloudflared` itself must be installed):

   ```sh
   sudo install -d -m700 /etc/cloudflared-newsletter
   sudo sh -c 'umask 077; cat > /etc/cloudflared-newsletter/token'   # paste the token, Ctrl-D
   sudo cp deploy/cloudflared-newsletter.service /etc/systemd/system/
   sudo systemctl daemon-reload && sudo systemctl enable --now cloudflared-newsletter
   ```

2. **Public hostname** — on the tunnel, add `newsletter.emnudge.dev` → service
   `HTTP` `127.0.0.1:8787`. Cloudflare creates the (proxied) DNS record; delete any
   existing A/AAAA record for `newsletter` first. If Caddy was serving this
   hostname, remove that block (`deploy/Caddyfile`) and reload Caddy.
3. **Service config** — in `/opt/newsletter-service/.env`, keep `BIND_HOST=127.0.0.1`
   and set `CLIENT_IP_HEADER=cf-connecting-ip`, then `sudo systemctl restart newsletter`.
   Fill in the real Cloudflare Email and Turnstile values **before** the hostname
   goes live: without them the service runs in dev mode (no emails sent, no bot
   gate, confirmation links written to the log).
4. **Access for the dashboard** — Zero Trust → **Access → Applications → Add →
   Self-hosted**: domain `newsletter.emnudge.dev`, path `admin`. Add a policy
   **Allow** → *Emails* → your address. Visitors now sign in with Cloudflare
   (one-time email code, or Google if you add it) before the dashboard's own
   login form. The public routes (`/subscribe`, `/confirm`, `/unsubscribe`,
   `/health`) stay open.
5. **Access for the CLI** (only if you use `send-newsletter.mjs`) — Access →
   **Service credentials → Service tokens → Create**, then add a second policy to
   the same application: action **Service Auth**, include *Service Token* → that
   token. Put its ID and secret in the site repo's `.env.newsletter` as
   `CF_ACCESS_CLIENT_ID` / `CF_ACCESS_CLIENT_SECRET`.
6. **Check it:**

   ```sh
   curl -s https://newsletter.emnudge.dev/health          # {"ok":true}
   curl -sI https://newsletter.emnudge.dev/admin          # 302 to <team>.cloudflareaccess.com
   curl -sI https://newsletter.emnudge.dev/admin/stats    # also 302 — every admin path is covered
   ```

   If `/admin/stats` isn't redirected, change the Access path to `admin*`.

Optional: a WAF **rate limiting rule** on `/subscribe` (if your plan includes one)
drops floods at the edge, before they reach the VPS. The service's own limits stay
in force either way.

## Backups

The service backs itself up. Each backup is a consistent snapshot (SQLite's
`VACUUM INTO`, safe while the service is writing), integrity-checked and gzipped:

- **Download** — the dashboard's **Download backup** button (`GET /admin/backup/download`)
  hands you one on demand, R2 or not.
- **R2, automatically** — with the four `R2_*` settings in `.env`, the service uploads a
  snapshot to R2 about once a day (checked hourly; the last success is kept in the DB,
  so restarts neither skip nor repeat one). The dashboard's Backups card shows the last
  backup and any failure, and has a **Back up to R2 now** button. At this size it stays
  in R2's free tier.

R2 setup:

1. Create a bucket (e.g. `emnudge-backups`) with a **lifecycle rule** that deletes
   objects after e.g. 30 days — the service never deletes anything.
2. Create an R2 API token with **Object Read & Write**, scoped to that bucket only.
3. Add `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY` and `R2_BUCKET` to
   `/opt/newsletter-service/.env` (see `.env.example`) and
   `sudo systemctl restart newsletter`. Setting only some of them is a startup error.

**Restore** from a downloaded backup (or one fetched from the R2 dashboard):

```sh
gunzip newsletter-<timestamp>.db.gz && sqlite3 newsletter-<timestamp>.db "PRAGMA integrity_check;"
sudo systemctl stop newsletter
sudo sh -c 'cd /opt/newsletter-service/data && mkdir -p replaced && mv newsletter.db* replaced/'
sudo install -o newsletter -g newsletter -m600 newsletter-<timestamp>.db /opt/newsletter-service/data/newsletter.db
sudo systemctl start newsletter
```

Move the `-wal`/`-shm` files aside along with the old database: SQLite would try to
apply them to the restored file, and they may hold the old database's latest writes.

## Local development

Leave the Cloudflare and Turnstile env vars **unset** — the service then runs in dev
mode: Turnstile (the signup bot gate) is skipped, and emails are **logged to the
service console** (with their confirm/unsubscribe links) instead of being sent. So the
whole subscribe → confirm flow works with no real credentials.

```sh
# 1) service (from newsletter-service/) — build the admin UI, then start it
npm --prefix admin install && npm --prefix admin run build
ADMIN_TOKEN=dev SITE_ORIGIN=http://localhost:4321 PUBLIC_BASE=http://localhost:8787 \
FROM_EMAIL=noreply@emnudge.dev PORT=8787 DB_PATH=/tmp/nl-dev.db cargo run

# 2) seed sample subscribers so the dashboard has data (run once first to create the schema)
DB_PATH=/tmp/nl-dev.db cargo run --bin seed_dev

# 3) the Astro site (from the parent repo), pointed at the local service
MY_SITE=http://localhost:4321 PUBLIC_NEWSLETTER_URL=http://localhost:8787 pnpm dev
```

- Admin dashboard: <http://localhost:8787/admin> — sign in with password **`dev`** (the `ADMIN_TOKEN`).
- **Testing signup**: use the form on <http://localhost:4321/blog> or `/notes` (no Turnstile widget in dev). After submitting, watch the **service console** for a line like `dev email link link=http://localhost:4321/newsletter/confirm?t=…` — open that link and click **Confirm subscription**; the subscriber shows up in the dashboard.
- Confirm/unsub pages: seeded demo token → <http://localhost:4321/newsletter/unsubscribe?t=DEMO123>; add `?preview=1` to any of them to see the success state with no side effects.

## Sending a newsletter

**Primary way — the dashboard**: open `/admin` → **Templates → New post**, fill in
title / slug / author / date / list / body (markdown), preview it, and click
**Send newsletter**. It runs as a background job with a live progress bar.

**Alternative — the CLI** (`send-newsletter.mjs`, in the parent repo) still works:

```sh
./send-newsletter.mjs src/pages/notes/my-post.md
```

It reads the post's frontmatter, infers the list from the path, and calls
`POST /admin/send` with the `ADMIN_TOKEN` (needs `.env.newsletter`; see the script header).

## Site-side integration (in the parent Astro repo)

The static site holds the reader-facing pieces (all styled by the site, not this service):

- `src/components/NewsletterSignup.astro` — signup form, mounted on the blog + notes
  listing pages (`src/pages/{blog,notes}/[...page].astro`) and at the end of every
  post (`src/layouts/Blog.astro`, prechecking the post's own list).
- `src/pages/newsletter/confirm.astro`, `src/pages/newsletter/unsubscribe.astro` —
  the pages email links land on. Both support `?preview=1` (renders the final state
  with no API call) which the admin builder embeds. Unsubscribe also has the
  Resubscribe undo + "why did you leave?" feedback.
- `src/data/env.ts` — exposes `PUBLIC_NEWSLETTER_URL` (this service's base URL) and
  `PUBLIC_TURNSTILE_SITE_KEY` to the browser. Set these at the site's build/dev time.
- `send-newsletter.mjs` — the optional CLI sender (reads `.env.newsletter`, gitignored).

## Notes

- **Storage** is a single SQLite file (`DB_PATH`). WAL mode is on. It backs itself up (see Backups); never back it up with a plain `cp` of the live file.
- **Client IP**: every per-IP guard (rate limit, per-IP cap, admin brute-force
  throttle) keys off the client IP (IPv6 collapsed to its /64), read from the header
  named by `CLIENT_IP_HEADER`:
  - `cf-connecting-ip` (Cloudflare Tunnel) — set by Cloudflare on every request,
    overwriting whatever the client sent.
  - `x-forwarded-for` (Caddy, the default) — the **rightmost** entry, the one the
    trusted proxy appends. Leftmost entries are attacker-controlled (Caddy *appends*
    to whatever the client sends), so they're ignored; `deploy/Caddyfile` also
    overwrites the header with the real remote host (defense in depth).

  Either header is only trustworthy if the proxy is the **only** way to reach the
  service, so keep `BIND_HOST=127.0.0.1`. The service refuses to start in dev mode on
  a non-loopback bind (override with `ALLOW_INSECURE_DEV_BIND=1`). The in-memory
  limiter maps are bounded (100k keys each) so IP rotation can't exhaust memory.
- **Cost guards**: `RATE_LIMIT_PER_MIN` (per IP), `GLOBAL_CONFIRM_CAP_PER_HOUR` and
  `GLOBAL_CONFIRM_CAP_PER_DAY` (hard ceilings on confirmation emails; the daily one
  keeps a sustained signup flood from using up the Cloudflare sending quota that
  blasts need). The global caps are enforced by an
  **atomic reserve-before-send** (a single check-and-insert SQL statement), so
  concurrent signups can't race past the ceiling. Sends to real subscribers are
  bounded by your confirmed count and stay within the free 3k/mo for a long time.
- **Confirmation queue**: when a global cap is hit, the signup is still accepted —
  its confirmation email is queued and sent once a slot frees up (checked every
  minute, oldest first). A running blast takes priority over the queue. A queued
  confirmation is dropped after `CONFIRM_QUEUE_MAX_AGE_HOURS` (default 24), and once
  `CONFIRM_QUEUE_MAX` (default 1000) are waiting, `/subscribe` answers 429. The
  dashboard's **Queued confirmations** metric shows the backlog; anything above zero
  means a cap was hit.
- **Confirming takes a click**: the link in the confirmation email opens the site's
  confirm page, which confirms only when the reader clicks the button. Mail scanners
  and link prefetchers open the links in an email, so a link that confirmed on GET
  would subscribe addresses nobody opted in.
- **Address checks**: before a confirmation is sent, the address's domain must be
  able to receive mail (an MX record, or failing that an A/AAAA record; looked up
  over Cloudflare's DNS-over-HTTPS). A domain that can't gets a 400. The check fails
  open on a lookup error, is skipped in dev mode, and `MX_CHECK=0` turns it off. An
  address that hard-bounced or reported spam is never mailed again: a new signup for
  it silently succeeds and leaves it suppressed. To let one back in, delete it from
  the dashboard (Cloudflare's own suppression list still applies).
- **Retention**: subscriptions still unconfirmed after `PENDING_EXPIRY_DAYS`
  (default 7; 0 disables) are deleted by an hourly sweep, which also clears
  resend-cooldown entries more than a day old. The dashboard's **Delete** action
  erases an address on request.
- **Inbox-bomb guard**: `CONFIRM_RESEND_COOLDOWN_MIN` (default 15) is a per-email
  cooldown — even spread across many IPs, the same address can't be re-sent a
  confirmation within the window, so a botnet can't flood a victim's inbox. The
  cooldown is keyed on the normalized mailbox (`+tag` stripped; dots stripped for
  Gmail), so aliases of one inbox share it. The
  claim is **atomic** (one `INSERT … ON CONFLICT` that both checks and records), so
  even simultaneous requests for one address yield exactly one send. It's claimed
  before the global reserve (so hammering one address can't drain the global budget)
  and before the DB upsert (so a blocked attempt leaves any live confirm link
  intact). The throttled request silently succeeds.
- **Admin auth**: constant-time token comparison over fixed-length SHA-256 digests
  (leaks neither length nor content), HMAC-signed session cookies, and a per-IP
  failed-attempt throttle (`ADMIN_AUTH_MAX_FAILURES`) that counts only wrong
  credentials, not visits without a session. Admin POSTs (login and logout
  included) are refused when the browser marks them cross-site (`Sec-Fetch-Site`),
  and API POSTs require `Content-Type: application/json`, so a malicious page can't
  ride the session cookie (CSRF).
- **Fail-closed config**: outside dev mode the service refuses to start if
  `ADMIN_TOKEN` is shorter than 32 chars or `TURNSTILE_SECRET` is unset, and any
  unparseable numeric env var is a startup error rather than a silent default.
- **Sending**: only one send job per post runs at a time (a second request gets
  409). A post is recorded as sent only if every recipient succeeded; re-sending
  after failures retries just the failed recipients (`deliveries` tracks who
  already got it). A job aborts when 5 sends in a row fail, at any point in the
  blast (e.g. the sending quota ran out). A running job can be cancelled from the
  dashboard, and **Send test** mails one copy to an address of your choice first.
- **Bounce suppression**: a blast reads Cloudflare's `result.permanent_bounces`
  and marks those addresses `bounced` even on an HTTP 200 (CF can reject a
  recipient while the call succeeds).
- **Complaint feedback loop**: CF has no bounce/complaint *webhook*, so a timer
  (`SUPPRESSION_SYNC_MIN`, default 60; 0 disables) polls
  `GET /accounts/{id}/email/sending/suppressions` and marks any `complaint` /
  `hard_bounce` address `bounced` locally (storing the reason in
  `suppression_reason`), so we stop re-sending to it. Soft bounces (transient)
  and manual/policy entries are ignored. The dashboard's **Sync suppressions**
  button (and `POST /admin/sync-suppressions`) forces an immediate run, and the
  **Complaints** metric surfaces the spam-report count. No-op in dev mode.
- **Throttle**: `SEND_THROTTLE_MS` spaces out individual sends to respect
  Cloudflare's ramping daily quota; the client retries on HTTP 429.
- Confirmation and post sends go out one recipient at a time so each carries a
  personalized unsubscribe link + `List-Unsubscribe` header.
