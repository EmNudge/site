import preprocess from 'svelte-preprocess';
import adapter from '@sveltejs/adapter-static';
import dotenv from 'dotenv';

dotenv.config();

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: preprocess(),
	
	kit: {
		target: '#svelte',

		adapter: adapter()
	}
};

export default config;
