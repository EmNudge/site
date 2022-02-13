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

const csvParse = (csv: string) => {
    const lines = csv.split('\n');
    const headers = getHeaders(lines[0]);

    return lines.slice(1).map(line => {
        const matches = [...line.matchAll(PARSE_LINE)].map(m => m[2]);
        const entries = matches.map((match, i) => [headers[i], match]);
        return Object.fromEntries(entries);
    });
}

export const reads = csvParse(csvData);