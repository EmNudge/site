import { API_KEY, CHANNEL_ID } from "./env";

const MAX_RESULTS = 4;

const BASE_URL = "https://www.googleapis.com/youtube/v3/search";
type YoutubeAPIResponse =
	| {
			items: {
				id: { videoId: string };
				snippet: {
					title: string;
					thumbnails: {
						medium: { url: string };
					};
				};
			}[];
	  }
	| {
			error: {
				code: 400;
				message: string;
				errors: unknown[];
				status: string;
				details: unknown[];
			};
	  };

const fetchYouTubeVideos = async (): Promise<YoutubeAPIResponse> => {
	const params = {
		key: API_KEY,
		channelId: CHANNEL_ID,
		part: "snippet,id",
		order: "date",
		maxResults: MAX_RESULTS,
	};
	const paramsStr = Object.entries(params)
		.map(([k, v]) => `${k}=${v}`)
		.join("&");

	const url = `${BASE_URL}?${paramsStr}`;

	return fetch(url).then((res) => res.json());
};

export interface Video {
	title: string;
	id: string;
	link: string;
	img: string;
}

let videos: Video[] = null;
export async function getVideos(): Promise<Video[]> {
	if (import.meta.env.DEV) {
		return Array(4).fill({ title: "Example Video" });
	}

	if (videos) return videos;

	const json = await fetchYouTubeVideos();

	if ("error" in json) throw new Error(json.error.message);

	console.log({ json });

	videos = json.items.map(({ snippet, id: snippetId }) => {
		const { title, thumbnails } = snippet;
		const id = snippetId.videoId;
		const link = `https://youtube.com/watch?v=${id}`;
		const img = thumbnails.medium.url;

		return { title, id, link, img } as Video;
	});

	return videos;
}
