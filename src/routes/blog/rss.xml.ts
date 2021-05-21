import type { RequestHandler } from '@sveltejs/kit';
import { getPostMetas } from './posts.json';

const getRssFromXmlStr = (str: string) => `
<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0">

<channel>
  ${str}
</channel>

</rss>
`.trim();

// gets array-like XML from object using key-value pairs.
function getXmlFromObj(obj: Object): string {
  const fields: string[] = [];
  for (const [key, value] of Object.entries(obj)) {
    if (Array.isArray(value)) {
      for (const item of value) {
        fields.push(`<${key}>\n${getXmlFromObj(item)}\n</${key}>`);
      }
      continue;
    }
    fields.push(`<${key}>${value}</${key}>`);
  }

  return fields.join('\n');
}

let rss: string;

export async function getRss() {
  if (rss) return rss;

  const postMetas = await getPostMetas();
  const rssData = { 
    title: 'Calvin Kipperman\'s Blog',
    link: 'https://kipperman.co/blog',
    description: 'Linguistics and Programming',
    category: 'Web development/Linguistics',
    language: 'en-us',
    item: postMetas.map(postMeta => {
      const link = `https://kipperman.co/blog/${postMeta.slug}`;
      const { title, summary: description, date: pubDate } = postMeta;
      return { title, description, pubDate, link };
    })
  };

  const xml = getXmlFromObj(rssData);
  rss = getRssFromXmlStr(xml);
  return rss;
}

export const get: RequestHandler<string> = async (_request) => {
  return {
		status: 200,
    headers: { 'content-type': 'application/rss+xml' },
		body: await getRss()
	};
};
