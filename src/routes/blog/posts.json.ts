import type { RequestHandler } from '@sveltejs/kit';
import type { BlogPostMeta } from '$lib/types';
import { getBlogPosts } from '../../blog/index';

export async function getPostMetas() {
  const postMetas: BlogPostMeta[] = []
  for (const [_slug, data] of await getBlogPosts()) {
    const postMeta = { ...data };
    delete postMeta.html;

    postMetas.push(postMeta as BlogPostMeta);
  }
  
  return postMetas;
}

export const get: RequestHandler<BlogPostMeta[]> = async (_request) => {
  return {
		status: 200,
		body: await getPostMetas()
	};
};

