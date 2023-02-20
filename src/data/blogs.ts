import type { MarkdownInstance } from "astro";
import { readFile } from 'node:fs/promises';

interface Post {
    draft?: true;
    pubDate: string;
    title: string;
    summary: string;
    url: string;
    minuteLength: number
}

type BlogPostFile = MarkdownInstance<Record<string, any> & Omit<Post, 'minuteLength' | 'url'>>;
export type MdOrMdxFile = Omit<BlogPostFile, 'rawContent'> & { rawContent?: BlogPostFile['rawContent'] }

const sortDate = (a: Post, b: Post) => Number(new Date(b.pubDate)) - Number(new Date(a.pubDate));

function getMinuteLength(text: string) {
    const words = text.split(/\s+/);
    return Math.floor(words.length / 200);
}
export async function getMinuteLengthFromFile(file: MdOrMdxFile | string) {
    if (typeof file !== 'string' && 'rawContent' in file) {
        return getMinuteLength(file.rawContent());
    }

    const filePath = typeof file === 'string' ? file : file.file;

    const fileText = await readFile(filePath, { encoding: 'utf8' })
    return getMinuteLength(fileText);
}

export const getSlugs = async (postFiles: MdOrMdxFile[]): Promise<Post[]> => {
    const postPromises = postFiles
        .filter(post => !post.frontmatter.draft)
        .map(async post => ({ 
            ...post.frontmatter,
            minuteLength: await getMinuteLengthFromFile(post),
            url: post.url
        }));

    const posts = await Promise.all(postPromises)
    posts.sort(sortDate)

    return posts
}
