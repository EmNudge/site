import { defineConfig } from "astro/config"
import svelte from "@astrojs/svelte"
import mdx from "@astrojs/mdx";
import rehypeSlug from 'rehype-slug';
import rehypeAutolinkHeadings from 'rehype-autolink-headings';

export default defineConfig({
    integrations: [
        svelte(),
        mdx(),
    ],
    markdown: {
        rehypePlugins: [
            rehypeSlug, [rehypeAutolinkHeadings, {
                behavior: 'prepend',
                content: {
                    type: 'element',
                    tagName: 'span',
                    properties: { className: 'header-link' },
                    children: [{ type: 'text', value: '#' }]
                }
            }]
        ]
    },
});
