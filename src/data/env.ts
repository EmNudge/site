// We double up the env keys because Vercel uses its own env keys

// @ts-ignore
export const SITE = import.meta.env.SITE ?? process.env.SITE
// @ts-ignore
export const API_KEY = import.meta.env.YOUTUBE_API_KEY ?? process.env.YOUTUBE_API_KEY
// @ts-ignore
export const CHANNEL_ID = import.meta.env.YOUTUBE_CHANNEL_ID ?? process.env.YOUTUBE_CHANNEL_ID
