#!/usr/bin/env node
import { readFile, writeFile, mkdir, readdir } from "node:fs/promises";
import { join, dirname, basename, extname } from "node:path";
import { fileURLToPath } from "node:url";

const STANDARD_SITE_DID = "did:plc:tgatoi47bb7xrxexwk7ogx73";
const PDS_URL = "https://bsky.social";

const PUBLICATIONS = {
  blog: {
    url: "https://emnudge.dev/blog",
    name: "EmNudge Blog",
    description: "Long-form articles on software, language, and the web.",
  },
  notes: {
    url: "https://emnudge.dev/notes",
    name: "EmNudge Notes",
    description: "Short notes and quick thoughts.",
  },
};

const PUBLICATION_COLLECTION = "site.standard.publication";
const DOCUMENT_COLLECTION = "site.standard.document";

const getPublicationAtUri = (rkey) =>
  `at://${STANDARD_SITE_DID}/${PUBLICATION_COLLECTION}/${rkey}`;

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
const RKEYS_PATH = join(REPO_ROOT, "scripts/standard-site-rkeys.json");
const WELL_KNOWN_DIR = join(
  REPO_ROOT,
  "public/.well-known/site.standard.publication",
);

const POST_EXTS = new Set([".md", ".mdx"]);

const MAX_TITLE = 300;
const MAX_SUMMARY = 1000;
const MAX_TAGS = 16;
const TAG_RE = /^[a-z0-9-]{1,32}$/i;

// ---- TID generation ---------------------------------------------------------
// standard.site lexicons declare `key: tid`, so record keys must be valid TIDs
// (13 chars, base32-sortable). Slug rkeys fail validation and never render as
// enhanced Bluesky cards, so we mint a TID per record and persist the mapping.
const S32 = "234567abcdefghijklmnopqrstuvwxyz";
const TID_RE = /^[234567abcdefghij][234567a-z]{12}$/;

function s32encode(n) {
  let s = "";
  while (n > 0n) {
    s = S32[Number(n % 32n)] + s;
    n = n / 32n;
  }
  return s.padStart(13, "2");
}

let lastTidMicros = 0;
const clockid = Math.floor(Math.random() * 1024);
function genTid() {
  let micros = Date.now() * 1000;
  if (micros <= lastTidMicros) micros = lastTidMicros + 1;
  lastTidMicros = micros;
  return s32encode((BigInt(micros) << 10n) | BigInt(clockid));
}

const isTid = (rkey) => typeof rkey === "string" && TID_RE.test(rkey);

// ---- frontmatter parsing ----------------------------------------------------
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

// ---- rkey mapping -----------------------------------------------------------
async function loadRkeys() {
  try {
    const raw = await readFile(RKEYS_PATH, "utf8");
    const parsed = JSON.parse(raw);
    return {
      publications: parsed.publications ?? {},
      documents: parsed.documents ?? {},
    };
  } catch {
    return { publications: {}, documents: {} };
  }
}

async function writeRkeys(map) {
  await writeFile(RKEYS_PATH, JSON.stringify(map, null, 2) + "\n");
}

// Reuse an existing TID for a stable key, otherwise mint a new one. Anything
// that isn't already a valid TID (e.g. legacy slug rkeys) is replaced.
function resolveRkey(previous, key) {
  const existing = previous[key];
  return isTid(existing) ? existing : genTid();
}

// ---- PDS access -------------------------------------------------------------
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
    body: JSON.stringify({ repo: session.did, collection, rkey, record }),
  });
  if (!res.ok) {
    throw new Error(`putRecord ${collection}/${rkey} failed: HTTP ${res.status}`);
  }
}

async function listRecordRkeys(session, collection) {
  const rkeys = [];
  let cursor;
  do {
    const url = new URL(`${PDS_URL}/xrpc/com.atproto.repo.listRecords`);
    url.searchParams.set("repo", session.did);
    url.searchParams.set("collection", collection);
    url.searchParams.set("limit", "100");
    if (cursor) url.searchParams.set("cursor", cursor);
    const res = await fetch(url);
    if (!res.ok) throw new Error(`listRecords ${collection} failed: HTTP ${res.status}`);
    const data = await res.json();
    for (const r of data.records ?? []) rkeys.push(r.uri.split("/").pop());
    cursor = data.cursor;
  } while (cursor);
  return rkeys;
}

async function deleteRecord(session, collection, rkey) {
  if (DRY_RUN) {
    console.log(`[dry-run] deleteRecord ${collection}/${rkey}`);
    return;
  }
  const res = await fetch(`${PDS_URL}/xrpc/com.atproto.repo.deleteRecord`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${session.accessJwt}`,
    },
    body: JSON.stringify({ repo: session.did, collection, rkey }),
  });
  if (!res.ok) {
    throw new Error(`deleteRecord ${collection}/${rkey} failed: HTTP ${res.status}`);
  }
}

// Remove any record whose rkey we no longer intend to keep — old slug-keyed
// records from before the TID migration, plus documents for deleted posts.
async function pruneStale(session, collection, intended) {
  if (DRY_RUN) {
    console.log(`[dry-run] skip prune of ${collection} (needs network)`);
    return;
  }
  const existing = await listRecordRkeys(session, collection);
  for (const rkey of existing) {
    if (!intended.has(rkey)) {
      await deleteRecord(session, collection, rkey);
      console.log(`✗ pruned ${collection}/${rkey}`);
    }
  }
}

async function writeWellKnown(kind, rkey) {
  await mkdir(WELL_KNOWN_DIR, { recursive: true });
  await writeFile(join(WELL_KNOWN_DIR, kind), getPublicationAtUri(rkey) + "\n");
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

  const previous = await loadRkeys();
  const nextMap = { publications: {}, documents: {} };
  const intendedPublications = new Set();
  const intendedDocuments = new Set();

  for (const kind of ["blog", "notes"]) {
    const pub = PUBLICATIONS[kind];
    const rkey = resolveRkey(previous.publications, kind);
    nextMap.publications[kind] = rkey;
    intendedPublications.add(rkey);
    await putRecord(session, PUBLICATION_COLLECTION, rkey, {
      $type: PUBLICATION_COLLECTION,
      url: pub.url,
      name: pub.name,
      description: pub.description,
    });
    await writeWellKnown(kind, rkey);
    console.log(`✓ publication ${getPublicationAtUri(rkey)} (${kind})`);
  }

  const all = (await Promise.all([loadPosts("blog"), loadPosts("notes")])).flat();
  console.log(`syncing ${all.length} documents`);

  for (const post of all) {
    const mapKey = `${post.kind}/${post.slug}`;
    const rkey = resolveRkey(previous.documents, mapKey);
    nextMap.documents[mapKey] = rkey;
    intendedDocuments.add(rkey);
    await putRecord(session, DOCUMENT_COLLECTION, rkey, {
      $type: DOCUMENT_COLLECTION,
      site: getPublicationAtUri(nextMap.publications[post.kind]),
      title: post.title,
      path: post.path,
      description: post.summary || undefined,
      publishedAt: post.publishedAt,
      tags: post.tags.length ? post.tags : undefined,
    });
    console.log(`✓ ${post.kind}/${post.slug} -> ${rkey}`);
  }

  await writeRkeys(nextMap);
  console.log(`wrote ${RKEYS_PATH}`);

  await pruneStale(session, DOCUMENT_COLLECTION, intendedDocuments);
  await pruneStale(session, PUBLICATION_COLLECTION, intendedPublications);

  console.log(`done. ${all.length} documents synced.`);
}

main().catch((err) => {
  console.error(err.message ?? "sync failed");
  process.exit(1);
});
