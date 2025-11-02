// @ts-ignore
import articleData from "./reads.csv?raw";
// @ts-ignore
import bookData from "./bookBookmarks.csv?raw";
// @ts-ignore
import videoData from "./videoBookmarks.csv?raw";
import { csvParse } from "./parseCsv";

interface Article {
	title: string;
	author: string;
	description: string;
	createdAt: string;
	readAt: string;
	link: string;
	tags: string;
}
export const reads = csvParse<Article>(articleData).sort(
	(a, b) => +new Date(b.readAt) - +new Date(a.readAt),
);

interface Book {
	title: string;
	author: string;
	description: string;
	readAt: string;
	link: string;
	tags: string;
}
export const books = csvParse<Book>(bookData).sort(
	(a, b) => +new Date(b.readAt) - +new Date(a.readAt),
);

interface Video {
	title: string;
	speaker: string;
	description: string;
	uploadedAt: string;
	watchedAt: string;
	link: string;
	tags: string;
}
export const videos = csvParse<Video>(videoData).sort(
	(a, b) => +new Date(b.watchedAt) - +new Date(a.watchedAt),
);
