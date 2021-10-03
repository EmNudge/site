import type { RequestHandler } from '@sveltejs/kit';
import type { BlogPostMeta } from '$lib/types';
import { getBlogPosts } from '../../blog/index';

let postMetas: BlogPostMeta[]; 
export async function getPostMetas() {
  postMetas = []
  const posts = await getBlogPosts();

  for (const [_slug, data] of posts) {
    const postMeta = { ...data };
    delete postMeta.html;

    postMetas.push(postMeta as BlogPostMeta);
  }
  
  postMetas.sort((a, b) => Number(new Date(b.date)) - Number(new Date(a.date)));
  
  return postMetas;
}

export const get: RequestHandler<BlogPostMeta[]> = async (_request) => {
  return {
		status: 200,
		body: postMetas || await getPostMetas()
	};
};

