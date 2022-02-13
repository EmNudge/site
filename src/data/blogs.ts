interface Post {
    date: string;
    title: string;
    summary: string;
    url: string;
    astro: {
        headers: string[];
        source: string;
        html: string;
    };
}

const sortDate = (a: Post, b: Post) => Number(new Date(b.date)) - Number(new Date(a.date));

export function getMinuteLength(text: string) {
    const words = text.split(/\s+/);
    return Math.floor(words.length / 200);
}

export const getSlugs = (posts: Post[]): (Post & { minuteLength: number })[] =>
    posts
        .sort(sortDate)
        .map(post => ({
            ...post,
            minuteLength: getMinuteLength(post.astro.source)
        }));