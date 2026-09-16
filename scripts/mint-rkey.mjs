#!/usr/bin/env node
// Assign a stable TID rkey to a document and persist it to the committed map.
// Run at authoring time (see new-post.sh) so the slug -> TID binding is in the
// repo before the site is built and before records are published. Idempotent:
// an existing valid TID is left untouched.
//
//   node scripts/mint-rkey.mjs <blog|notes> <slug>
import { readFile, writeFile } from "node:fs/promises";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { genTid, isTid } from "./tid.mjs";

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const RKEYS_PATH = join(REPO_ROOT, "scripts/standard-site-rkeys.json");

const [kind, slug] = process.argv.slice(2);
if ((kind !== "blog" && kind !== "notes") || !slug) {
  console.error("Usage: node scripts/mint-rkey.mjs <blog|notes> <slug>");
  process.exit(1);
}

const map = JSON.parse(await readFile(RKEYS_PATH, "utf8"));
map.documents ??= {};

const mapKey = `${kind}/${slug}`;
if (isTid(map.documents[mapKey])) {
  console.log(`${mapKey} already has rkey ${map.documents[mapKey]}`);
  process.exit(0);
}

const rkey = genTid();
map.documents[mapKey] = rkey;
await writeFile(RKEYS_PATH, JSON.stringify(map, null, 2) + "\n");
console.log(`minted ${mapKey} -> ${rkey}`);
