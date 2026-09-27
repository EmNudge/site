#!/usr/bin/env node
// Send a teaser newsletter for a published post to the confirmed subscribers of
// its list (blog or notes). Content is read from the local markdown file, so the
// VPS service never needs repo access — it only holds subscribers + the CF token.
//
// Usage:
//   ./send-newsletter.mjs <path-to-post.md> [--yes]
//
// Environment (put these in .env.newsletter, which is gitignored):
//   ADMIN_TOKEN            Bearer token for the VPS /admin/send endpoint (required)
//   NEWSLETTER_ADMIN_URL   Base URL of the VPS service (default https://newsletter.emnudge.dev)
//   SITE_URL               Public site origin for building post links (default https://emnudge.dev)
//   CF_ACCESS_CLIENT_ID    Cloudflare Access service token, needed once /admin sits
//   CF_ACCESS_CLIENT_SECRET  behind Access (see newsletter-service/README.md)

import { existsSync, readFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createInterface } from "node:readline/promises";
import matter from "gray-matter";
import { config as loadEnv } from "dotenv";

const scriptDir = dirname(fileURLToPath(import.meta.url));

function die(msg) {
  console.error(msg);
  process.exit(1);
}

// ---- args ----
const args = process.argv.slice(2);
let postPath;
let assumeYes = false;
for (const arg of args) {
  if (arg === "--yes" || arg === "-y") assumeYes = true;
  else if (!postPath && !arg.startsWith("-")) postPath = arg;
  else die(`Unknown option: ${arg}\nUsage: send-newsletter.mjs <path-to-post.md> [--yes]`);
}
if (!postPath) die("Usage: send-newsletter.mjs <path-to-post.md> [--yes]");

// ---- config (load .env.newsletter if present, then apply defaults) ----
const envFile = join(scriptDir, ".env.newsletter");
if (existsSync(envFile)) loadEnv({ path: envFile });

const NEWSLETTER_ADMIN_URL = process.env.NEWSLETTER_ADMIN_URL || "https://newsletter.emnudge.dev";
const SITE_URL = process.env.SITE_URL || "https://emnudge.dev";
const ADMIN_TOKEN = process.env.ADMIN_TOKEN;
const { CF_ACCESS_CLIENT_ID, CF_ACCESS_CLIENT_SECRET } = process.env;

if (!ADMIN_TOKEN) die("Error: ADMIN_TOKEN is not set (put it in .env.newsletter or export it).");
if (!CF_ACCESS_CLIENT_ID !== !CF_ACCESS_CLIENT_SECRET)
  die("Error: set both CF_ACCESS_CLIENT_ID and CF_ACCESS_CLIENT_SECRET, or neither.");
if (!existsSync(postPath)) die(`Error: no such file: ${postPath}`);

// ---- infer list + slug from the path ----
let list;
if (postPath.includes("/notes/")) list = "notes";
else if (postPath.includes("/blog/")) list = "blog";
else die("Error: could not infer list — path must contain /blog/ or /notes/");

const slug = basename(postPath).replace(/\.mdx?$/, "");
const url = `${SITE_URL}/${list}/${slug}`;

// ---- parse frontmatter + body (gray-matter, same as Astro) ----
const { data: frontmatter, content: body } = matter(readFileSync(postPath, "utf8"));
const title = frontmatter.title ? String(frontmatter.title) : "";
const date = frontmatter.pubDate ? String(frontmatter.pubDate) : "";

if (frontmatter.draft === true) die("Refusing to send: post is marked draft: true");
if (!title) die("Error: post has no title in frontmatter");
if (!body.trim()) die("Error: post has no body content");

// ---- confirm ----
console.log("About to send a newsletter:");
console.log(`  list:    ${list}`);
console.log(`  title:   ${title}`);
console.log(`  date:    ${date || "(none)"}`);
console.log(`  url:     ${url}`);
console.log(`  body:    ${body.split("\n").length} lines`);
console.log(`  to:      ${NEWSLETTER_ADMIN_URL}/admin/send`);
console.log("");

if (!assumeYes) {
  const rl = createInterface({ input: process.stdin, output: process.stdout });
  const reply = await rl.question("Send now? [y/N] ");
  rl.close();
  if (!/^y$/i.test(reply.trim())) {
    console.log("Aborted.");
    process.exit(0);
  }
}

// ---- send ----
console.log("Sending…");
const res = await fetch(`${NEWSLETTER_ADMIN_URL}/admin/send`, {
  method: "POST",
  headers: {
    Authorization: `Bearer ${ADMIN_TOKEN}`,
    "content-type": "application/json",
    ...(CF_ACCESS_CLIENT_ID && {
      "CF-Access-Client-Id": CF_ACCESS_CLIENT_ID,
      "CF-Access-Client-Secret": CF_ACCESS_CLIENT_SECRET,
    }),
  },
  body: JSON.stringify({ list, slug, title, date, url, body }),
});

const text = await res.text();
console.log(`HTTP ${res.status}`);
console.log(text);
if (res.status !== 200) process.exit(1);
