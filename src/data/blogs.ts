import type { MarkdownInstance } from "astro";


interface Post {
    date: string;
    title: string;
    summary: string;
    url: string;
    minuteLength: number
}

export type BlogPostFile = MarkdownInstance<Record<string, any> & Omit<Post, 'minuteLength' | 'url'>>

const sortDate = (a: Post, b: Post) => Number(new Date(b.date)) - Number(new Date(a.date));

export function getMinuteLength(text: string) {
    const words = text.split(/\s+/);
    return Math.floor(words.length / 200);
}

export const getSlugs = async (postFiles: BlogPostFile[]): Promise<Post[]> => {
    const postPromises = postFiles.map(async post => {
		const postData = (await post.default())
        const minuteLength = getMinuteLength(postData.props.rawContent())

        return { ...post.frontmatter, minuteLength, url: post.url }
    })

    const posts = await Promise.all(postPromises)
    posts.sort(sortDate)

    return posts
}
