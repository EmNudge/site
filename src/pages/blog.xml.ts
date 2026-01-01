import rss from "@astrojs/rss";
import { SITE as site } from "../data/env";

export const GET = async () => {
	const files = import.meta.glob("./blog/*.md*");
	const blogPosts = Object.values(files).map(async (post) => {
		const file = (await post()) as any;
		const { title, summary, pubDate, draft } = file.frontmatter;
		return { title, description: summary, link: file.url, pubDate, draft };
	});

	const items = await Promise.all(blogPosts)
		.then((items) => items.filter((post) => !post.draft))
		.then((items) =>
			items.sort((a, b) => +new Date(b.pubDate) - +new Date(a.pubDate)),
		);

	return rss({
		title: "EmNudge's Blog",

		description: "Linguistics and Programming",
		customData: `<language>en-us</language>`,
		site,

		items,
	});
};
