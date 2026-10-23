#!/usr/bin/env -S pnpm exec tsx
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { RECORDINGS, getAlignmentText, getBuiltBlocks, readPosts, sha256 } from "./common";
import { validateArtifactTimings } from "./lib/quality";
import type { NarrationAlignment } from "../../src/components/SpeakAloud/types";

async function main() {
  const posts = await readPosts();
  const errors: string[] = [];
  for (const post of posts) {
    const path = join(RECORDINGS, `${post.recording}.align.json`);
    let alignment: NarrationAlignment;
    try {
      alignment = JSON.parse(await readFile(path, "utf8")) as NarrationAlignment;
    } catch {
      errors.push(`${post.recording}: missing ${path}`);
      continue;
    }
    if (alignment.version !== 1) errors.push(`${post.recording}: unsupported alignment version`);
    const blocks = await getBuiltBlocks(post);
    const text = blocks.map((block) => block.text).join("\n\n");
    if (alignment.article?.textHash !== sha256(text)) {
      errors.push(`${post.recording}: article text changed; regenerate ${path}`);
      continue;
    }
    if (alignment.blocks?.length !== blocks.length) {
      errors.push(`${post.recording}: block count changed; regenerate ${path}`);
      continue;
    }
    if (getAlignmentText(alignment.blocks) !== text) {
      errors.push(`${post.recording}: alignment text does not match built article`);
    }
    for (const [index, block] of blocks.entries()) {
      if (alignment.blocks[index]?.hash !== sha256(block.text)) {
        errors.push(`${post.recording}: block ${index} changed; regenerate ${path}`);
      }
    }
    for (const timingError of validateArtifactTimings(alignment)) {
      errors.push(`${post.recording}: ${timingError}`);
    }
  }

  if (errors.length) throw new Error(`Narration alignment check failed:\n  ${errors.join("\n  ")}`);
  console.log(
    `✓ ${posts.length} narration alignment files match their articles and timing invariants`,
  );
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
