import type { RequestHandler } from '@sveltejs/kit';
import type { Read } from '$lib/types';
import { Client } from '@notionhq/client';

const notion = new Client({
  auth: process.env['NOTION_TOKEN'],
});

let reads: Read[] = [];

export async function getReads() {
  if (reads.length) return reads;

  const myPage = await notion.databases.query({
    database_id: process.env['READS_DATABASE'],
  });

  reads = myPage.results.map(getReadRowData);
  reads.sort((a, b) => Number(new Date(b.readAt)) - Number(new Date(a.readAt)));

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
  const author: string = props.Author.rich_text[0].plain_text;
  const description: string = props.Description.rich_text[0].plain_text;
  const link: string = props.Link.url;
  const createdAt: string = props['Created At'] 
    ? getKindaLocaleDate(props['Created At'].date.start)
    : 'NA';
  const readAt: string = props['Read At'].date.start;

  return { author, link, description, title, readAt, createdAt };
}