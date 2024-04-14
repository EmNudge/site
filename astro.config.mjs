import { defineConfig } from "astro/config";
import svelte from "@astrojs/svelte";
import mdx from "@astrojs/mdx";
import rehypeSlug from "rehype-slug";
import rehypeAutolinkHeadings from "rehype-autolink-headings";
import rehypeToc from "@jsdevtools/rehype-toc";
import expressiveCode from "astro-expressive-code";
import { codeRunnerPlugin } from "./src/components/CodeRunner/plugin";

// https://astro.build/config
export default defineConfig({
  integrations: [
    svelte(),
    expressiveCode({ plugins: [codeRunnerPlugin()] }),
    mdx(),
  ],
  markdown: {
    syntaxHighlight: false,
    rehypePlugins: [
      rehypeSlug,
      [
        rehypeToc,
        {
          customizeTOC(node) {
            node.tagName = "details";
            const summary = {
              type: "element",
              tagName: "summary",
              children: [
                {
                  type: "text",
                  value: "Table Of Contents",
                },
              ],
            };
            node.children.unshift(summary);
            return node;
          },
        },
      ],
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
