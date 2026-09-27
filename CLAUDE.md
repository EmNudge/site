# Site

Personal blog/notes site built with Astro.

## Creating new posts

Use `./new-post.sh` to scaffold a new blog or note:

```
./new-post.sh <note|blog> "<title>" [--description "<summary>"] [--slug <slug>] [--tags "<tags>"]
```

- Slug is inferred from the title if omitted
- Date is set to today automatically
- Tags default to `general` if omitted (tags are required for notes listing page)
- Posts go to `src/pages/notes/` or `src/pages/blog/`

## Email newsletter

Readers can subscribe by email to the blog and/or notes. The system spans **two halves**:

1. **This static site** — the reader-facing pieces:
   - `src/components/NewsletterSignup.astro` — signup form, mounted on `src/pages/{blog,notes}/[...page].astro` and at the end of every post (`src/layouts/Blog.astro`).
   - `src/pages/newsletter/confirm.astro` + `unsubscribe.astro` — where email links land (support `?preview=1`).
   - `src/data/env.ts` — `PUBLIC_NEWSLETTER_URL` + `PUBLIC_TURNSTILE_SITE_KEY` (set at build/dev time).
   - `send-newsletter.mjs` — optional CLI sender (reads `.env.newsletter`; sending is normally done from the admin dashboard).

2. **`newsletter-service/`** — a self-hosted VPS backend (Rust: axum + rusqlite/SQLite +
   Cloudflare Email Service) with a React admin dashboard. It owns the subscriber DB,
   sending, and analytics. Build/run it with `cargo`; the admin UI is still built with npm.

**Full setup, architecture, endpoints, and the local dev workflow (running both halves
together, admin login, seeding) are documented in `newsletter-service/README.md` — read it
before touching anything newsletter-related.**
