import type { BlogPost, BlogPostMeta } from '$lib/types';

// retrieving files
import { readFile, readdir } from 'fs';
import { promisify } from 'util';
import path from 'path';

// parsing markdown
import matter from 'gray-matter';
import prism from 'prismjs';
import marked from 'marked';

//#region marked modifications
const renderer = new marked.Renderer()
renderer.heading = (text: string, level: number) => {
  const escapedText = text.toLowerCase().replace(/[^\w]+/g, '-');

  return `<h${level}>
    <a name="${escapedText}" class="anchor" href="#${escapedText}">
      <span class="header-link">#</span>
    </a>
    ${text}
  </h${level}>`;
};
renderer.code = (code: string, languageStr: string) => {
  const [language, isRunnable] = languageStr.split(/\s+/).map(str => str.trim());
  const prop = isRunnable ? `is="runnable-code"` : '';

  const parser = prism.languages[language] || prism.languages.html;
  const highlighted = prism.highlight(code, parser, language);

  const codeTag = `<code class="language-${language}">${highlighted}</code>`;
  return `<pre ${prop} class="language-${language}">${codeTag}</pre>`;
}

// @ts-ignore
marked.setOptions({ renderer });
//#endregion

let posts: Map<string, BlogPost> = null;

async function getMarkdownFiles() {
  // __dirname is not defined for some reason
  const blogPostPath = path.resolve(path.join('src', 'blog'));
  const fileNames = await promisify(readdir)(blogPostPath);

  const files: [string, string][] = [];
  for (const fileName of fileNames) {
    if (!fileName.endsWith('.md')) continue;
    const file = await promisify(readFile)(path.join(blogPostPath, fileName), 'utf8');
    files.push([fileName.slice(0, -3), file]);
  }

  return files;
}

export async function getBlogPosts() {
  if (posts) return posts;

  posts = new Map();

  const files = await getMarkdownFiles();
  
  for (const [fileName, file] of files) {
    const { data, content } = matter(file) as unknown as { data: BlogPostMeta, content: string };
    const html = marked(content);

    const slug = fileName;
    const minutes = Math.floor(content.split(/\s+/).length / 200);
    posts.set(slug, { ...data, slug, html, minuteLength: minutes });
  }

  return posts;
}
