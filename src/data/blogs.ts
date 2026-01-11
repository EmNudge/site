import type { MarkdownInstance } from "astro"
import { readFile } from "node:fs/promises"

/**
 * Maps discussion site identifiers to their icon paths for dark and light modes.
 * If a markdown file uses a site not listed here, the build will fail.
 */
export const DISCUSSION_SITES = {
  devto: {
    dark: "/icons/discussions/dark/devto.svg",
    light: "/icons/discussions/light/devto.svg",
  },
  hackernews: {
    dark: "/icons/discussions/dark/hackernews.png",
    light: "/icons/discussions/light/hackernews.png",
  },
  lobsters: {
    dark: "/icons/discussions/dark/lobsters.png",
    light: "/icons/discussions/light/lobsters.png",
  },
} as const;

export type DiscussionSite = keyof typeof DISCUSSION_SITES;

export interface Post {
  draft?: true
  tags: string
  pubDate: string
  title: string
  summary: string
  url: string
  minuteLength: number
  recording?: string
  discussions?: { site: string; url: string }[]
}

type BlogPostFile = MarkdownInstance<Record<string, any> & Omit<Post, "minuteLength" | "url">>
export type MdOrMdxFile = Omit<BlogPostFile, "rawContent"> & {
  rawContent?: BlogPostFile["rawContent"]
}

const sortDate = (a: Post, b: Post) =>
  Number(new Date(b.pubDate)) - Number(new Date(a.pubDate))

function getMinuteLength(text: string) {
  const words = text.split(/\s+/)
  return Math.floor(words.length / 200)
}
export async function getMinuteLengthFromFile(file: MdOrMdxFile | string) {
  if (typeof file !== "string" && "rawContent" in file) {
    return getMinuteLength(file.rawContent())
  }

  const filePath = typeof file === "string" ? file : file.file

  const fileText = await readFile(filePath, { encoding: "utf8" })
  return getMinuteLength(fileText)
}

export const getSlugs = async (postFiles: MdOrMdxFile[]): Promise<Post[]> => {
  const postPromises = postFiles
    .filter((post) => !post.frontmatter.draft)
    .map(async (post) => ({
      ...post.frontmatter,
      minuteLength: await getMinuteLengthFromFile(post),
      url: post.url,
    }))

  const posts = await Promise.all(postPromises)
  posts.sort(sortDate)

  return posts
}
