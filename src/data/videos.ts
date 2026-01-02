import { API_KEY, CHANNEL_ID } from "./env";

const MAX_RESULTS = 4;

const BASE_URL = "https://www.googleapis.com/youtube/v3/search";
type YoutubeAPIResponse =
  | {
      items: {
        id: { videoId: string };
        snippet: {
          title: string;
          publishedAt: string;
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
  publishedAt: string;
}

let videos: Video[] = null;
export async function getVideos(): Promise<Video[]> {
  if (import.meta.env.DEV) {
    return [
      {
        title: "The Implicit Props Pattern - React vs Vue vs Svelte vs Solid",
        id: "qznuoFecZr0",
        link: "https://youtube.com/watch?v=qznuoFecZr0",
        img: "https://i.ytimg.com/vi/qznuoFecZr0/mqdefault.jpg",
        publishedAt: "2022-01-02T16:40:57-08:00",
      },
      {
        title: "Rate Limiting - From Scratch Episode 3",
        id: "IUmC_dC87oc",
        link: "https://youtube.com/watch?v=IUmC_dC87oc",
        img: "https://i.ytimg.com/vi/IUmC_dC87oc/mqdefault.jpg",
        publishedAt: "2021-06-04T17:02:23-07:00",
      },
      {
        title: "Understanding The Standard For Loop",
        id: "lebu6I-qJrg",
        link: "https://youtube.com/watch?v=lebu6I-qJrg",
        img: "https://i.ytimg.com/vi/lebu6I-qJrg/mqdefault.jpg",
        publishedAt: "2021-02-24T10:40:08-08:00",
      },
      {
        title: "World Champion Typist 320wpm - How I did it",
        id: "7DAG74o3p3o",
        link: "https://youtube.com/watch?v=7DAG74o3p3o",
        img: "https://i.ytimg.com/vi/7DAG74o3p3o/mqdefault.jpg",
        publishedAt: "2021-01-30T14:14:20-08:00",
      },
    ];
  }

  if (videos) return videos;

  const json = await fetchYouTubeVideos();

  if ("error" in json) throw new Error(json.error.message);

  videos = json.items.map(({ snippet, id: snippetId }) => {
    const { title, thumbnails, publishedAt } = snippet;
    const id = snippetId.videoId;
    const link = `https://youtube.com/watch?v=${id}`;
    const img = thumbnails.medium.url;

    return { title, id, link, img, publishedAt } as Video;
  });

  return videos;
}
