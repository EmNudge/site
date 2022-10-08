import { defineConfig } from "astro/config"
import svelte from "@astrojs/svelte"

export default defineConfig({
    integrations: [
        svelte(),
    ],
    markdown: {
        rehypePlugins: [
            'rehype-slug', ['rehype-autolink-headings', {
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
