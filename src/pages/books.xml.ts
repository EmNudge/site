import rss from "@astrojs/rss";
import { books } from "../data/bookmarks";
import { SITE as site } from '../data/env';

export const get = () =>
  rss({
    title: "Recent Books",

    description: "Books you might find interesting",
    customData: `<language>en-us</language>`,
    site,

    items: books.map(({ title, description, readAt, link }) => ({
      title,
      description,
      link,
      pubDate: new Date(readAt),
    })),
  });
