import { OGImageRoute } from "astro-og-canvas";

// Build-time OG images: one plain card per blog/note, plus a site default.
// Titles/summaries come straight from each post's frontmatter.
const modules = import.meta.glob<{ frontmatter: Record<string, any> }>(
  ["../blog/*.md", "../blog/*.mdx", "../notes/*.md", "../notes/*.mdx"],
  { eager: true },
);

const pages: Record<string, { title: string; description: string }> = {
  site: {
    title: "EmNudge",
    description: "Long-form articles and notes on software, language, and the web.",
  },
};

for (const [file, mod] of Object.entries(modules)) {
  const match = file.match(/\/(blog|notes)\/(.+)\.mdx?$/);
  if (!match) continue;
  const [, kind, slug] = match;
  const fm = mod.frontmatter ?? {};
  if (fm.draft) continue;
  pages[`${kind}/${slug}`] = {
    title: fm.title ?? slug,
    description: fm.summary ?? "",
  };
}

export const { getStaticPaths, GET } = OGImageRoute({
  param: "route",
  pages,
  getImageOptions: (_path, page) => ({
    title: page.title,
    description: page.description,
    bgGradient: [[19, 22, 25]], // --background #131619
    border: { color: [79, 193, 241], width: 24, side: "inline-start" }, // accent #4fc1f1
    padding: 80,
    font: {
      title: { color: [255, 255, 255], weight: "Bold", size: 72, families: ["Merriweather"] },
      description: { color: [168, 178, 188], size: 36, families: ["Merriweather"] },
    },
    fonts: ["./public/fonts/Merriweather-Black.ttf", "./public/fonts/Merriweather-Regular.ttf"],
    format: "PNG",
  }),
});
