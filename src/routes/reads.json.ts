import type { RequestHandler } from '@sveltejs/kit';
import type { Read } from '$lib/types';

const url = `https://api.notion.com/v1/databases/${process.env['READS_DATABASE']}/query`;
let reads: Read[] = [];

export async function getReads() {
  if (reads.length) return reads;

  const res = await fetch(url, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${process.env['NOTION_TOKEN']}`,
      'Content-Type': 'application/json',
    },
  });

  const { results } = await res.json();
  reads = results
    .map(getReadRowData)
    .sort((a: any, b: any) => Number(new Date(b.readAt)) - Number(new Date(a.readAt)));

  return reads;
}
getReads();

export const get: RequestHandler<Read[]> = async (_request) => {
  return {
		status: 200,
		body: await getReads(),
	};
};

function getKindaLocaleDate(dateStr: string) {
  const date = new Date(dateStr).toString();
  const [month, day, year] = date.split(' ').slice(1, 4);
  return `${month} ${day}, ${year}`;
}

function getReadRowData(readRow: { properties: any }) {
  const { properties: props } = readRow;
  
  if (!props.Title) return null;

  const title: string = props.Title.title[0].plain_text;
  const author: string = props.Author.text[0].plain_text;
  const description: string = props.Description.text[0].plain_text;
  const link: string = props.Link.url;
  const createdAt: string = props['Created At'] 
    ? getKindaLocaleDate(props['Created At'].date.start)
    : 'NA';
  const readAt: string = props['Read At'].date.start;
  
  return { author, link, description, title, readAt, createdAt };
}