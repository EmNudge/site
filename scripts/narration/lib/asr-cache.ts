import { existsSync } from "node:fs";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import type { AsrWord } from "./types";

export const PIPELINE_VERSION = 2;

export interface AsrCacheEntry {
  version: string;
  model: string;
  duration: number;
  words: AsrWord[];
}

const safe = (value: string) => value.replace(/[^A-Za-z0-9_.-]/g, "_");

export const asrCachePath = (
  cacheRoot: string,
  audioHash: string,
  model: string,
  stableTsVersion = "2.19.1",
) =>
  join(
    cacheRoot,
    "asr",
    `${audioHash.replace(/^sha256:/, "").slice(0, 24)}-${safe(model)}-ss${safe(stableTsVersion)}-p${PIPELINE_VERSION}.json`,
  );

export const readAsrCache = async (path: string): Promise<AsrCacheEntry | null> => {
  if (!existsSync(path)) return null;
  try {
    return JSON.parse(await readFile(path, "utf8")) as AsrCacheEntry;
  } catch {
    return null;
  }
};

export const writeAsrCache = async (path: string, entry: AsrCacheEntry) => {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, JSON.stringify(entry) + "\n");
};
