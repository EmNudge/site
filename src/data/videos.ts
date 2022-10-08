import { API_KEY, CHANNEL_ID } from './env';

const MAX_RESULTS = 4;

const BASE_URL = 'https://www.googleapis.com/youtube/v3/search';
function getYoutubeUrl() {
    const params = {
        key: API_KEY,
        channelId: CHANNEL_ID,
        part: 'snippet,id',
        order: 'date',
        maxResults: MAX_RESULTS
    };
    const paramsStr = Object.entries(params).map(([k, v]) => `${k}=${v}`).join('&')

    return `${BASE_URL}?${paramsStr}`;
}

export interface Video {
    title: string;
    id: string;
    link: string;
    img: string;
}

let videos: Video[] = null;
export async function getVideos() {
    if (videos) return videos;

    const json = await fetch(getYoutubeUrl()).then(res => res.json());

    videos = json.items.map(({ snippet, id: snippetId }) => {
        const { title, thumbnails } = snippet;
        const id = snippetId.videoId;
        const link = `https://youtube.com/watch?v=${id}`;
        const img = thumbnails.medium.url;

        return { title, id, link, img } as Video;
    });

    return videos;
}
