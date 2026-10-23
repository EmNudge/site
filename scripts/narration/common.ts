import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { spawn } from "node:child_process";
import { basename, dirname, extname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseHTML } from "linkedom";
import { getCanonicalText, getNarrationBlocks } from "../../src/components/SpeakAloud/narration";
import type { AlignmentBlock, NarrationAlignment } from "../../src/components/SpeakAloud/types";
import type { AlignmentJob, AsrWord, SentenceAlignmentResult } from "./lib/types";
import type { PipelineDiagnostics } from "./lib/pipeline";

export const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
export const RECORDINGS = join(ROOT, "public/recordings");
export const CACHE = join(ROOT, ".cache/narration");

export interface Post {
  source: string;
  route: string;
  recording: string;
}

export const sha256 = (value: string | Uint8Array) =>
  `sha256:${createHash("sha256").update(value).digest("hex")}`;

export const getAlignmentText = (blocks: { text: string }[]) =>
  blocks.map((block) => block.text).join("\n\n");

export const readPosts = async (): Promise<Post[]> => {
  const posts: Post[] = [];
  for (const section of ["blog", "notes"]) {
    const directory = join(ROOT, "src/pages", section);
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      if (!entry.isFile() || !new Set([".md", ".mdx"]).has(extname(entry.name))) continue;
      const source = join(directory, entry.name);
      const content = await readFile(source, "utf8");
      const frontmatter = content.match(/^---\r?\n([\s\S]*?)\r?\n---/);
      const rawRecording = frontmatter?.[1].match(/^recording:\s*(.+?)\s*$/m)?.[1];
      const recording = rawRecording
        ?.replace(/\s+#.*$/, "")
        .trim()
        .replace(/^(["'])(.*)\1$/, "$2");
      if (recording) {
        posts.push({
          source,
          route: `/${section}/${basename(entry.name, extname(entry.name))}`,
          recording,
        });
      }
    }
  }
  return posts;
};

export const resolvePost = async (input: string): Promise<Post> => {
  const posts = await readPosts();
  const absolute = resolve(ROOT, input);
  const byPath = posts.find((post) => post.source === absolute);
  const byRecording = posts.find((post) => post.recording === input);
  const post = byPath ?? byRecording;
  if (!post) {
    throw new Error(
      `No recorded post found for "${input}". Use a recording slug or Markdown path.`,
    );
  }
  return post;
};

export const getBlocksFromHtml = (post: Post, html: string) => {
  const { document, window } = parseHTML(html);
  Object.assign(globalThis, {
    document,
    NodeFilter: window.NodeFilter,
    Text: window.Text,
  });
  const article = Array.from(document.querySelectorAll("article[data-recording]")).find(
    (element) => element.getAttribute("data-recording") === post.recording,
  );
  if (!article) throw new Error(`Could not find narrated article at ${post.route}`);
  return getNarrationBlocks(article).map((element, index) => ({
    index,
    tag: element.tagName.toLowerCase(),
    text: getCanonicalText(element).text,
  }));
};

export const getRenderedBlocks = async (post: Post) => {
  const { dev } = await import("astro");
  const server = await dev({
    root: ROOT,
    logLevel: "error",
    server: { host: "127.0.0.1", port: 0, open: false },
  });
  try {
    const response = await fetch(`http://127.0.0.1:${server.address.port}${post.route}/`);
    if (!response.ok) throw new Error(`Could not render ${post.route}: HTTP ${response.status}`);
    return getBlocksFromHtml(post, await response.text());
  } finally {
    await server.stop();
  }
};

export const getBuiltBlocks = async (post: Post) => {
  const htmlPath = join(ROOT, "dist", post.route.slice(1), "index.html");
  if (!existsSync(htmlPath)) throw new Error(`Built page not found: ${relative(ROOT, htmlPath)}`);
  return getBlocksFromHtml(post, await readFile(htmlPath, "utf8"));
};

export const downloadAudio = async (recording: string, explicit?: string) => {
  await mkdir(CACHE, { recursive: true });
  if (explicit) {
    const path = resolve(ROOT, explicit);
    if (!existsSync(path)) throw new Error(`Audio file not found: ${path}`);
    return path;
  }
  for (const extension of ["mp3", "ogg"]) {
    const destination = join(CACHE, `${recording}.${extension}`);
    const etagPath = `${destination}.etag`;
    const headers = new Headers();
    if (existsSync(etagPath))
      headers.set("If-None-Match", (await readFile(etagPath, "utf8")).trim());

    let response: Response;
    try {
      response = await fetch(`https://assets.emnudge.com/${recording}.${extension}`, { headers });
    } catch (error) {
      if (existsSync(destination)) {
        console.warn(`Could not refresh ${recording}.${extension}; using cached audio.`);
        return destination;
      }
      throw error;
    }
    if (response.status === 304 && existsSync(destination)) return destination;
    if (!response.ok) continue;
    await writeFile(destination, Buffer.from(await response.arrayBuffer()));
    const etag = response.headers.get("etag");
    if (etag) await writeFile(etagPath, etag);
    return destination;
  }
  throw new Error(`Could not download audio for ${recording} from assets.emnudge.com`);
};

export const run = (command: string, args: string[], cwd: string, input?: string) =>
  new Promise<{ code: number; stdout: string; stderr: string }>((resolvePromise, reject) => {
    const child = spawn(command, args, { cwd, stdio: "pipe" });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => (stdout += chunk));
    child.stderr.on("data", (chunk) => (stderr += chunk));
    child.on("error", reject);
    child.on("close", (code) => resolvePromise({ code: code ?? 1, stdout, stderr }));
    if (input !== undefined) child.stdin.write(input);
    child.stdin.end();
  });

export const getAudioDuration = async (audio: string) => {
  const result = await run(
    "ffprobe",
    [
      "-v",
      "error",
      "-show_entries",
      "format=duration",
      "-of",
      "default=noprint_wrappers=1:nokey=1",
      audio,
    ],
    ROOT,
  );
  const duration = Number(result.stdout.trim());
  if (result.code !== 0 || !Number.isFinite(duration)) {
    throw new Error(result.stderr || `Could not read audio duration for ${audio}`);
  }
  return duration;
};

export interface AsrResult {
  aligner: string;
  version: string;
  model: string;
  duration: number;
  words: AsrWord[];
}

export const runAsr = async (audio: string, model: string): Promise<AsrResult> => {
  const result = await run(
    "uv",
    ["run", "scripts/narration/align.py", "transcribe", "--audio", audio, "--model", model],
    ROOT,
  );
  if (result.code !== 0) throw new Error(result.stderr || "ASR failed");
  try {
    return JSON.parse(result.stdout) as AsrResult;
  } catch {
    throw new Error(`ASR returned invalid JSON:\n${result.stdout}`);
  }
};

export const runSentenceAligner = async (
  audio: string,
  jobs: AlignmentJob[],
  model: string,
): Promise<{ version: string; model: string; results: SentenceAlignmentResult[] }> => {
  const result = await run(
    "uv",
    ["run", "scripts/narration/align.py", "align", "--audio", audio, "--model", model],
    ROOT,
    JSON.stringify({ jobs }),
  );
  if (result.code !== 0) throw new Error(result.stderr || "Sentence alignment failed");
  try {
    return JSON.parse(result.stdout) as {
      version: string;
      model: string;
      results: SentenceAlignmentResult[];
    };
  } catch {
    throw new Error(`Sentence aligner returned invalid JSON:\n${result.stdout}`);
  }
};

export const createAlignment = (
  post: Post,
  recording: string,
  blocks: AlignmentBlock[],
  audioPath: string,
  audioDuration: number,
  stableTsVersion: string,
  asrModel: string,
  alignModel: string,
): NarrationAlignment => {
  const text = getAlignmentText(blocks);
  const audioBytes = readFileSync(audioPath);
  return {
    version: 1,
    recording,
    generatedAt: new Date().toISOString(),
    article: {
      source: relative(ROOT, post.source),
      textHash: sha256(text),
      blockCount: blocks.length,
    },
    audio: { file: basename(audioPath), sha256: sha256(audioBytes), duration: audioDuration },
    aligner: {
      name: "stable-ts",
      version: stableTsVersion,
      model: `${asrModel}+${alignModel}`,
    },
    blocks,
  };
};

export const reportAlignment = async (
  alignment: NarrationAlignment,
  diagnostics?: PipelineDiagnostics,
) => {
  const words = alignment.blocks.flatMap((block) =>
    block.sentences.flatMap((sentence) => sentence.words),
  );
  const lowConfidence = words.filter((word) => word[4] < 0.3);
  const zeroDuration = words.filter((word) => word[3] <= word[2]);
  const unspoken = alignment.blocks.filter((block) => !block.spoken);
  console.log(
    `Aligned ${words.length} words across ${alignment.blocks.length - unspoken.length} spoken blocks ` +
      `(${unspoken.length} unspoken; ${lowConfidence.length} below 0.30 confidence; ` +
      `${zeroDuration.length} zero-duration).`,
  );
  if (diagnostics) {
    const retried = Object.values(diagnostics.retries).filter((count) => count > 0).length;
    const anchorErrors = [...diagnostics.anchorMedianErrors].sort((a, b) => a - b);
    const medianAnchorError = anchorErrors[Math.floor(anchorErrors.length / 2)] ?? 0;
    const p95AnchorError = anchorErrors[Math.floor(anchorErrors.length * 0.95)] ?? 0;
    console.log(
      `${diagnostics.reliableAnchors}/${diagnostics.canonicalWords} reliable ASR anchors; ` +
        `${diagnostics.unspokenWords} inferred unspoken words; ${retried} sentences retried; ` +
        `${diagnostics.wordsPerSecond.toFixed(2)} words/s; anchor error ` +
        `${medianAnchorError.toFixed(2)}s median / ${p95AnchorError.toFixed(2)}s p95.`,
    );
  }

  const legacyPath = join(RECORDINGS, `${alignment.recording}.timestamps`);
  if (!existsSync(legacyPath)) return;
  const legacy = (await readFile(legacyPath, "utf8"))
    .split(/\s+/)
    .filter(Boolean)
    .map((timestamp) => {
      const [minutes, seconds] = timestamp.split(":");
      return Number(minutes) * 60 + Number(seconds);
    });
  const spokenBlocks = alignment.blocks.filter((block) => block.spoken);
  const flagged = spokenBlocks
    .map((block, index) => ({ index: block.index, delta: block.start - legacy[index] }))
    .filter(({ delta }) => Number.isFinite(delta) && Math.abs(delta) > 1.5);
  if (legacy.length !== spokenBlocks.length) {
    console.warn(
      `Legacy marker count differs: ${legacy.length} markers for ${spokenBlocks.length} spoken blocks.`,
    );
  }
  if (flagged.length) {
    console.warn(
      `Review ${flagged.length} block starts more than 1.5s from legacy markers: ` +
        flagged
          .map(({ index, delta }) => `${index} (${delta >= 0 ? "+" : ""}${delta.toFixed(2)}s)`)
          .join(", "),
    );
  }
};

export const writeDiagnostics = async (recording: string, diagnostics: PipelineDiagnostics) => {
  await mkdir(CACHE, { recursive: true });
  const path = join(CACHE, `${recording}.diag.json`);
  await writeFile(path, JSON.stringify(diagnostics, null, 2) + "\n");
  return path;
};

export const writeAlignment = async (recording: string, alignment: NarrationAlignment) => {
  await mkdir(RECORDINGS, { recursive: true });
  const path = join(RECORDINGS, `${recording}.align.json`);
  await writeFile(path, JSON.stringify(alignment) + "\n");
  return path;
};
