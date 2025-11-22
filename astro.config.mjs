import { defineConfig } from "astro/config";
import svelte from "@astrojs/svelte";
import mdx from "@astrojs/mdx";
import rehypeSlug from "rehype-slug";
import rehypeAutolinkHeadings from "rehype-autolink-headings";
import rehypeCitation from "rehype-citation";
import expressiveCode from "astro-expressive-code";
import { codeRunnerPlugin } from "./src/components/CodeRunner/plugin";
// https://astro.build/config
export default defineConfig({
	integrations: [
		svelte(),
		expressiveCode({
			plugins: [codeRunnerPlugin()],
		}),
		mdx(),
	],
	markdown: {
		syntaxHighlight: false,
		rehypePlugins: [
			rehypeSlug,
			rehypeCitation,
			[
				rehypeAutolinkHeadings,
				{
					behavior: "prepend",
					content: {
						type: "element",
						tagName: "span",
						properties: {
							className: "header-link",
						},
						children: [
							{
								type: "text",
								value: "#",
							},
						],
					},
				},
			],
		],
	},
});
