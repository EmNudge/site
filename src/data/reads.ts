// @ts-ignore
import csvData from './reads.csv?raw';

const PARSE_LINE = /("?)(.+?)\1(?:,|$)/gm;
const getHeaders = (line: string) => {
    const headers = [];

    const iter = line.toLowerCase().trim().matchAll(PARSE_LINE);
    for (const match of iter) {
        const header = match[2].replace(/\s+(.)/g, c => c[1].toUpperCase());
        headers.push(header);
    }

    return headers;
}

interface CsvRow {
    title: string;
    author: string;
    description: string;
    createdAt: string;
    readAt: string;
    link: string;
}

const csvParse = (csv: string): CsvRow[] => {
    const lines = csv.split('\n');
    const headers = getHeaders(lines[0]);

    return lines
        .slice(1)
        .map(line => {
            const matches = [...line.matchAll(PARSE_LINE)].map(m => m[2]);
            const entries = matches.map((match, i) => [headers[i], match]);
            return Object.fromEntries(entries);
        })
        .sort((a, b) => +new Date(b.readAt) - +new Date(a.readAt));
}

export const reads: CsvRow[] = csvParse(csvData);