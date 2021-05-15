import type { RequestHandler } from '@sveltejs/kit';
import type { Video } from '$lib/types';

const key = process.env['YOUTUBE_API_KEY'];
const channel = process.env['YOUTUBE_CHANNEL_ID'];
const MAX_RESULTS = 4;
const YOUTUBE_LINK = `https://www.googleapis.com/youtube/v3/search?key=${key}`
 + `&channelId=${channel}&part=snippet,id&order=date&maxResults=${MAX_RESULTS}`;

let videos: Video[] = null;
export async function getVideos() {
  if (videos) return videos;

  const res = await fetch(YOUTUBE_LINK);
  const json = await res.json();

  videos = json.items.map(({ snippet, id: snippetId }) => {
    const { title, thumbnails } = snippet;
    const id = snippetId.videoId;
    const link = `https://youtube.com/watch?v=${id}`;
    return { title, id, link, thumbnails } as Video;
  });

  return videos;
}
getVideos();

export const get: RequestHandler<Video[]> = async (_request) => {
  return {
		status: 200,
		body: getVideos(),
	};
};

