import type { RequestHandler } from '@sveltejs/kit';
import type { BlogPost, BlogPostMeta } from '$lib/types';
import { getBlogPosts } from '../../blog/index';

getBlogPosts();

export const get: RequestHandler<BlogPost> = async (request) => {
  const { slug } = request.params;
  const posts = await getBlogPosts();
  
  return {
		status: 200,
		body: posts.get(slug)
	};
};

