# Sveltekit Personal Site

Remade my site (8th times the charm!) using Sveltekit.
Development was pretty smooth and coincided with the release of Notion's public API.

I'm using YouTube's API to pull recent videos, Notion's API to pull recent reads, and running a markdown parser to generate articles.

## Private Keys
`.env`
```sh
YOUTUBE_API_KEY=SOME_KEY_HERE
YOUTUBE_CHANNEL_ID=SOME_KEY_HERE

NOTION_TOKEN=SOME_KEY_HERE
READS_DATABASE=SOME_KEY_HERE
```