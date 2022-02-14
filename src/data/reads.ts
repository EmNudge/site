// @ts-ignore
import csvData from './reads.csv?raw';

const PARSE_LINE_REGEX = /(?:"((?:.|\n)+?)"|(.+?))(\r?\n|,|$)/g;
function* parseCsvRow(text: string) {
    for (const match of text.matchAll(PARSE_LINE_REGEX)) {
        // must have either 1 or 2
        const { 1: quoteMatch, 2: regMatch, 3: terminator } = match;
        const item = regMatch ?? quoteMatch.replace(/""/g, '"');
        const isLast = terminator !== ',';

        yield { item, isLast, match };
    }
}
const parseCsvLine = (text: string): [string[], number] => {
    const items = [];
    for (const { item, isLast, match} of parseCsvRow(text)) {
        items.push(item);
        
        if (!isLast) continue;
        const finishIndex = match.index + match[0].length;
        return [items, finishIndex];
    }

    return [items, text.length];
}

interface CsvRow {
    title: string;
    author: string;
    description: string;
    createdAt: string;
    readAt: string;
    link: string;
}

const csvParse = (csv: string): any[] => {
    const [ headersDirty, index ] = parseCsvLine(csv);
    const headers = headersDirty.map(item => {
        return item
            .toLowerCase()
            .trim()
            .replace(/\s+(.)/g, m => m[1].toUpperCase())
    });

    const restOfCsvIter = parseCsvRow(csv.slice(index))

    const lines: any[] = [];
    const lineData = [];
    for (const { item, isLast } of restOfCsvIter) {
        lineData.push(item);
        if (!isLast) continue;
        
        // reset linkData and add to lines
        const entries = lineData.map((item, i) => [headers[i], item]);
        lines.push(Object.fromEntries(entries));
        lineData.length = 0;
    }
    lines.sort((a, b) => +new Date(b.readAt) - +new Date(a.readAt));

    return lines;
}

export const reads: CsvRow[] = csvParse(csvData) as CsvRow[];