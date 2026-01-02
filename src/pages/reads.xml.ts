import rss from "@astrojs/rss";
import { reads } from "../data/bookmarks";
import { SITE as site } from "../data/env";

export const GET = () =>
  rss({
    title: "Recent Reads",

    description: "Articles you might find interesting",
    customData: `<language>en-us</language>`,
    site,

    items: reads.map(({ title, description, link, createdAt }) => ({
      title,
      description,
      link,
      pubDate: new Date(createdAt),
    })),
  });
