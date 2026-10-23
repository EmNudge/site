#!/usr/bin/env -S pnpm exec tsx
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { parseArgs } from "node:util";
import {
  CACHE,
  RECORDINGS,
  createAlignment,
  downloadAudio,
  getAudioDuration,
  getBuiltBlocks,
  getRenderedBlocks,
  reportAlignment,
  resolvePost,
  runAsr,
  runSentenceAligner,
  sha256,
  writeAlignment,
  writeDiagnostics,
} from "./common";
import { asrCachePath, readAsrCache, writeAsrCache } from "./lib/asr-cache";
import { NarrationPipelineError, runTwoPassPipeline } from "./lib/pipeline";

const STABLE_TS_VERSION = "2.19.1";

async function main() {
  const { values, positionals } = parseArgs({
    args: process.argv.slice(2).filter((argument) => argument !== "--"),
    options: {
      audio: { type: "string" },
      model: { type: "string" },
      "asr-model": { type: "string", default: "small.en" },
      "align-model": { type: "string" },
      "no-asr-cache": { type: "boolean", default: false },
      force: { type: "boolean", default: false },
      "no-build": { type: "boolean", default: false },
    },
    allowPositionals: true,
  });

  const input = positionals[0];
  if (!input) {
    throw new Error(
      "Usage: pnpm narration:align -- <recording-slug|post.md> [--audio path] [--asr-model small.en] [--align-model base.en]",
    );
  }
  const asrModel = values["asr-model"] ?? "small.en";
  const alignModel = values["align-model"] ?? values.model ?? "base.en";
  const post = await resolvePost(input);
  const outputPath = join(RECORDINGS, `${post.recording}.align.json`);
  if (existsSync(outputPath) && !values.force) {
    throw new Error(`${outputPath} already exists; pass --force to regenerate it`);
  }

  const blocks = values["no-build"] ? await getBuiltBlocks(post) : await getRenderedBlocks(post);
  if (!blocks.length) throw new Error(`No narrated paragraphs found in ${post.source}`);
  const audio = await downloadAudio(post.recording, values.audio);
  const audioDuration = await getAudioDuration(audio);
  const audioHash = sha256(await readFile(audio));
  const cachePath = asrCachePath(CACHE, audioHash, asrModel, STABLE_TS_VERSION);
  let asr = values["no-asr-cache"] ? null : await readAsrCache(cachePath);
  if (asr) {
    console.log(`Using cached ${asrModel} ASR words from ${cachePath}`);
  } else {
    console.log(`Transcribing ${post.recording} with ${asrModel}...`);
    asr = await runAsr(audio, asrModel);
    await writeAsrCache(cachePath, asr);
    console.log(`Cached ${asr.words.length} ASR words at ${cachePath}`);
  }

  try {
    const pipeline = await runTwoPassPipeline(
      blocks,
      asr.words,
      audioDuration,
      async (jobs) => (await runSentenceAligner(audio, jobs, alignModel)).results,
    );
    const artifact = createAlignment(
      post,
      post.recording,
      pipeline.blocks,
      audio,
      audioDuration,
      asr.version,
      asrModel,
      alignModel,
    );
    await reportAlignment(artifact, pipeline.diagnostics);
    const output = await writeAlignment(post.recording, artifact);
    console.log(`Wrote ${output}`);
  } catch (error) {
    if (error instanceof NarrationPipelineError) {
      const diagnosticPath = await writeDiagnostics(post.recording, error.diagnostics);
      throw new Error(`${error.message}\nDiagnostics: ${diagnosticPath}`);
    }
    throw error;
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
