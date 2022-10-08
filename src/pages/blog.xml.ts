import rss from "@astrojs/rss";
import { SITE as site } from '../data/env';

const blogDir = import.meta.glob("./blog/*.md");
const markdownFiles = await Promise.all(Object.values(blogDir).map((f) => f()));
const sortedFiles = markdownFiles.sort((a, b) => {
  +new Date(b.frontmatter.date) - +new Date(a.frontmatter.date);
});

export const get = () =>
  rss({
    title: "Calvin Kipperman's Blog",

    description: "Linguistics and Programming",
    customData: `<language>en-us</language>`,
    category: "Web development/Linguistics",
    site,

    items: sortedFiles.map(({ frontmatter, url }) => {
      const { title, description, date } = frontmatter;

      return {
        title,
        description,
        pubDate: date,
        link: url,
      };
    }),
  });
