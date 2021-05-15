import type { RequestHandler } from '@sveltejs/kit';
import type { BlogPostMeta, Video, Read } from '$lib/types';
import { getVideos } from './youtube.json';
import { getPostMetas } from './blog/posts.json';
import { getReads } from './reads.json';

let mainData: { reads: Read[], videos: Video[], posts: BlogPostMeta[] } = null;
async function getMainData() {
	if (mainData) return mainData;

	mainData = {
		reads: (await getReads()).slice(0, 5),
		videos: await getVideos(),
		posts: (await getPostMetas()).slice(0, 5)
	};

	return mainData;
}
getMainData();

export const get: RequestHandler<{ videos: Video[], posts: BlogPostMeta[] }> = async (_request) => {
  return {
		status: 200,
		body: await getMainData(),
	};
};

