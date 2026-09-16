#!/usr/bin/env node
// Validate blog/note frontmatter against a fixed schema so a typo'd or unused
// key (e.g. `description` instead of `summary`) fails CI instead of being
// silently ignored. No network; safe to run on every PR.
import { readFile, readdir } from "node:fs/promises";
import { join, dirname, extname } from "node:path";
import { fileURLToPath } from "node:url";

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const POST_EXTS = new Set([".md", ".mdx"]);

// Every key here is consumed somewhere in src/ (layouts, data/blogs.ts, list
// stubs). Add a key here only when something actually reads it.
const REQUIRED = ["title", "pubDate", "summary", "tags", "layout"];
const OPTIONAL = ["icon", "draft", "recording", "discussions"];
const ALLOWED = new Set([...REQUIRED, ...OPTIONAL]);

const DIRS = {
  blog: join(REPO_ROOT, "src/pages/blog"),
  notes: join(REPO_ROOT, "src/pages/notes"),
};

// Top-level frontmatter keys only (indented lines are list items / nested).
function frontmatterKeys(text) {
  const match = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match) return null;
  const keys = [];
  for (const line of match[1].split(/\r?\n/)) {
    if (!line.trim() || /^\s/.test(line)) continue;
    const m = line.match(/^([A-Za-z0-9_]+)\s*:/);
    if (m) keys.push(m[1]);
  }
  return keys;
}

const errors = [];

for (const [kind, dir] of Object.entries(DIRS)) {
  const entries = await readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    if (!entry.isFile() || !POST_EXTS.has(extname(entry.name))) continue;
    const rel = `src/pages/${kind}/${entry.name}`;
    const keys = frontmatterKeys(await readFile(join(dir, entry.name), "utf8"));
    if (!keys) {
      errors.push(`${rel}: no frontmatter block`);
      continue;
    }
    const seen = new Set(keys);
    for (const req of REQUIRED) {
      if (!seen.has(req)) errors.push(`${rel}: missing required key "${req}"`);
    }
    for (const key of keys) {
      if (!ALLOWED.has(key)) {
        errors.push(`${rel}: unknown key "${key}" (allowed: ${[...ALLOWED].join(", ")})`);
      }
    }
    const dupes = keys.filter((k, i) => keys.indexOf(k) !== i);
    for (const d of new Set(dupes)) errors.push(`${rel}: duplicate key "${d}"`);
  }
}

if (errors.length) {
  console.error(`Frontmatter check failed:\n  ${errors.join("\n  ")}`);
  process.exit(1);
}
console.log("✓ all blog/note frontmatter is valid");
