// @ts-ignore
import csvData from './reads.csv?raw';
// @ts-ignore
import videoData from './videoBookmarks.csv?raw';
import { csvParse } from './parseCsv';

interface Article {
    title: string;
    author: string;
    description: string;
    createdAt: string;
    readAt: string;
    link: string;
}
export const reads = csvParse<Article>(csvData)
    .sort((a, b) => +new Date(b.readAt) - +new Date(a.readAt));;

interface Video {
    title: string;
    speaker: string;
    description: string;
    uploadedAt: string;
    watchedAt: string;
    link: string;
}
export const videos = csvParse<Video>(videoData)
    .sort((a, b) => +new Date(b.watchedAt) - +new Date(a.watchedAt));;