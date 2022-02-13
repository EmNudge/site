import dotenv from "dotenv"
import fs from "fs/promises"

const exposedKeys = ['YOUTUBE_API_KEY', 'YOUTUBE_CHANNEL_ID']
const envDefinitions = await fs.readFile(".env")
    .then(file => {
        const env = dotenv.parse(file);
        const entries = exposedKeys.map(key => [`process.env.${key}`, `"${env[key]}"`])
        return Object.fromEntries(entries);
    }).catch(_err => {
        const entries = exposedKeys.map(key => [`process.env.${key}`, `"${process.env[key]}"`])
        return Object.fromEntries(entries);
    })

// @type-check enabled!
// VSCode and other TypeScript-enabled text editors will provide auto-completion,
// helpful tooltips, and warnings if your exported object is invalid.
// You can disable this by removing "@ts-check" and `@type` comments below.

// @ts-check
export default /** @type {import('astro').AstroUserConfig} */ ({
    renderers: ['@astrojs/renderer-svelte'],
    buildOptions: {
        site: 'https://kipperman.co',
    },
    vite: {
        define: {
            ...envDefinitions
        }
    },
});