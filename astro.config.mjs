import dotenv from "dotenv"
import fs from "fs/promises"

const env = dotenv.parse(await fs.readFile(".env"))
const exposedKeys = ['YOUTUBE_API_KEY', 'YOUTUBE_CHANNEL_ID']
const envDefinitions = Object.fromEntries(exposedKeys.map(key => [`process.env.${key}`, `"${env[key]}"`]))


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