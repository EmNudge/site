import rss from "@astrojs/rss";
import { SITE as site } from '../data/env';

console.log({ site })
export const get = () =>
  rss({
    title: "Calvin Kipperman's Blog",

    description: "Linguistics and Programming",
    customData: `<language>en-us</language>`,
    site,

    items: import.meta.glob('./blog/*.md'),
  });