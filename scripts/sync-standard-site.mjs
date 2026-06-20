#!/usr/bin/env node
import { readFile, readdir } from "node:fs/promises";
import { join, dirname, basename, extname } from "node:path";
import { fileURLToPath } from "node:url";

const STANDARD_SITE_DID = "did:plc:tgatoi47bb7xrxexwk7ogx73";
const PDS_URL = "https://bsky.social";

const PUBLICATIONS = {
  blog: {
    rkey: "blog",
    url: "https://emnudge.dev/blog",
    name: "EmNudge Blog",
    description: "Long-form articles on software, language, and the web.",
  },
  notes: {
    rkey: "notes",
    url: "https://emnudge.dev/notes",
    name: "EmNudge Notes",
    description: "Short notes and quick thoughts.",
  },
};

const getDocumentRkey = (kind, slug) => `${kind}-${slug}`;
const getPublicationAtUri = (kind) =>
  `at://${STANDARD_SITE_DID}/site.standard.publication/${PUBLICATIONS[kind].rkey}`;

const HANDLE = process.env.BLUESKY_HANDLE;
const APP_PASSWORD = process.env.BLUESKY_APP_PASSWORD;
const DRY_RUN = process.env.DRY_RUN === "1";

if (!DRY_RUN && (!HANDLE || !APP_PASSWORD)) {
  console.error("Missing BLUESKY_HANDLE or BLUESKY_APP_PASSWORD env vars");
  process.exit(1);
}

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const PAGES = {
  blog: join(REPO_ROOT, "src/pages/blog"),
  notes: join(REPO_ROOT, "src/pages/notes"),
};

const POST_EXTS = new Set([".md", ".mdx"]);

const MAX_TITLE = 300;
const MAX_SUMMARY = 1000;
const MAX_TAGS = 16;
const TAG_RE = /^[a-z0-9-]{1,32}$/i;

function parseFrontmatter(text) {
  const match = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match) return null;
  const block = match[1];
  const out = {};
  let key = null;
  for (const rawLine of block.split(/\r?\n/)) {
    if (!rawLine.trim()) continue;
    const indented = /^\s/.test(rawLine);
    if (indented && key) {
      const v = rawLine.trim();
      if (!Array.isArray(out[key])) out[key] = [];
      if (v.startsWith("-")) out[key].push(v.slice(1).trim());
      continue;
    }
    const m = rawLine.match(/^([A-Za-z0-9_]+)\s*:\s*(.*)$/);
    if (!m) continue;
    key = m[1];
    const rawVal = m[2].trim();
    if (rawVal === "") {
      out[key] = "";
      continue;
    }
    out[key] = unquote(rawVal);
  }
  return out;
}

function unquote(v) {
  if (
    (v.startsWith('"') && v.endsWith('"')) ||
    (v.startsWith("'") && v.endsWith("'"))
  ) {
    return v.slice(1, -1);
  }
  return v;
}

function toIsoDate(value) {
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) {
    throw new Error(`Unparseable pubDate: ${value}`);
  }
  return d.toISOString();
}

function sanitizeString(value, max) {
  if (typeof value !== "string") return "";
  const cleaned = value.replace(/[\x00-\x08\x0B\x0C\x0E-\x1F]/g, "").trim();
  return cleaned.slice(0, max);
}

function sanitizeTags(value) {
  const arr = typeof value === "string" ? [value] : Array.isArray(value) ? value : [];
  return arr
    .filter((t) => typeof t === "string" && TAG_RE.test(t))
    .slice(0, MAX_TAGS);
}

async function loadPosts(kind) {
  const dir = PAGES[kind];
  const entries = await readdir(dir, { withFileTypes: true });
  const files = entries.filter(
    (e) => e.isFile() && POST_EXTS.has(extname(e.name)),
  );
  const posts = [];
  for (const entry of files) {
    const slug = basename(entry.name, extname(entry.name));
    const filePath = join(dir, entry.name);
    const text = await readFile(filePath, "utf8");
    const fm = parseFrontmatter(text);
    if (!fm) {
      console.warn(`skip ${kind}/${slug}: no frontmatter`);
      continue;
    }
    if (fm.draft === "true" || fm.draft === true) continue;
    if (!fm.title || !fm.pubDate) {
      console.warn(`skip ${kind}/${slug}: missing title or pubDate`);
      continue;
    }
    posts.push({
      kind,
      slug,
      title: sanitizeString(fm.title, MAX_TITLE),
      summary: sanitizeString(fm.summary, MAX_SUMMARY),
      tags: sanitizeTags(fm.tags),
      publishedAt: toIsoDate(fm.pubDate),
      path: `/${kind}/${slug}`,
    });
  }
  return posts;
}

async function createSession() {
  const res = await fetch(`${PDS_URL}/xrpc/com.atproto.server.createSession`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ identifier: HANDLE, password: APP_PASSWORD }),
  });
  if (!res.ok) {
    throw new Error(`createSession failed: HTTP ${res.status}`);
  }
  return res.json();
}

async function putRecord(session, collection, rkey, record) {
  if (DRY_RUN) {
    console.log(`[dry-run] putRecord ${collection}/${rkey}`);
    return;
  }
  const res = await fetch(`${PDS_URL}/xrpc/com.atproto.repo.putRecord`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${session.accessJwt}`,
    },
    body: JSON.stringify({
      repo: session.did,
      collection,
      rkey,
      record,
    }),
  });
  if (!res.ok) {
    throw new Error(
      `putRecord ${collection}/${rkey} failed: HTTP ${res.status}`,
    );
  }
}

async function main() {
  const session = DRY_RUN
    ? { did: STANDARD_SITE_DID, accessJwt: "" }
    : await createSession();
  if (session.did !== STANDARD_SITE_DID) {
    throw new Error(
      `DID mismatch: handle resolves to ${session.did} but script expects ${STANDARD_SITE_DID}`,
    );
  }

  for (const kind of ["blog", "notes"]) {
    const pub = PUBLICATIONS[kind];
    await putRecord(session, "site.standard.publication", pub.rkey, {
      $type: "site.standard.publication",
      url: pub.url,
      name: pub.name,
      description: pub.description,
    });
    console.log(`✓ publication ${getPublicationAtUri(kind)}`);
  }

  const all = (
    await Promise.all([loadPosts("blog"), loadPosts("notes")])
  ).flat();
  console.log(`syncing ${all.length} documents`);

  for (const post of all) {
    const rkey = getDocumentRkey(post.kind, post.slug);
    await putRecord(session, "site.standard.document", rkey, {
      $type: "site.standard.document",
      site: getPublicationAtUri(post.kind),
      title: post.title,
      path: post.path,
      description: post.summary || undefined,
      publishedAt: post.publishedAt,
      tags: post.tags.length ? post.tags : undefined,
    });
    console.log(`✓ ${post.kind}/${post.slug}`);
  }

  console.log(`done. ${all.length} documents synced.`);
}

main().catch((err) => {
  console.error(err.message ?? "sync failed");
  process.exit(1);
});
