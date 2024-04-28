import rss from "@astrojs/rss";
import { videos } from "../data/bookmarks";
import { SITE as site } from '../data/env';

export const GET = () =>
  rss({
    title: "Bookmarked Videos",

    description: "Videos you might find interesting",
    customData: `<language>en-us</language>`,
    site,

    items: videos.map(({ title, description, link, uploadedAt }) => ({
      title,
      description,
      link,
      pubDate: new Date(uploadedAt),
      customData: `<guid isPermaLink="true">${link}</guid>`,
    })),
  });
