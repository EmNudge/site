// We double up the env keys because Cloudflare's build passes its env vars on process.env

// @ts-ignore
export const SITE = import.meta.env.MY_SITE ?? process.env.MY_SITE;
// @ts-ignore
export const API_KEY = import.meta.env.YOUTUBE_API_KEY ?? process.env.YOUTUBE_API_KEY;
// @ts-ignore
export const CHANNEL_ID = import.meta.env.YOUTUBE_CHANNEL_ID ?? process.env.YOUTUBE_CHANNEL_ID;

// Newsletter: base URL of the VPS subscription service (e.g. https://newsletter.emnudge.dev)
// and the public Turnstile site key. Both are safe to expose to the browser.
// @ts-ignore
export const NEWSLETTER_URL =
  import.meta.env.PUBLIC_NEWSLETTER_URL ??
  process.env.PUBLIC_NEWSLETTER_URL ??
  "https://newsletter.emnudge.dev";
// @ts-ignore
export const TURNSTILE_SITE_KEY =
  import.meta.env.PUBLIC_TURNSTILE_SITE_KEY ?? process.env.PUBLIC_TURNSTILE_SITE_KEY ?? "";
