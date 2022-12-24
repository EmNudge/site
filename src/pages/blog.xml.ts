import rss from "@astrojs/rss";
import { SITE as site } from '../data/env';

export const get = () =>
  rss({
    title: "EmNudge's Blog",

    description: "Linguistics and Programming",
    customData: `<language>en-us</language>`,
    site,

    items: import.meta.glob('./blog/*.md'),
  });
