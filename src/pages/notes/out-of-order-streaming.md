---
pubDate: April 21, 2024
title: "Out Of Order HTML Streaming"
summary: Just a sprinkle of JavaScript
tags: coding
icon: technologies/languages/javascript
layout: ../../layouts/Blog.astro
---

If you use the `Transfer-Encoding: chunked` header, you can stream in an HTML page. This can be helpful if you're processing parts of the page while sending it down to the client. It gives them an early drip of the content.

We rarely use HTML streaming because, in practice, you don't write code with streaming in mind. Pages are usually sent down the moment they are full compiled. There are some movements in the frontend space, both modern and ancient, that have recognized this as an advantage and built in the ergonomics.

Can we build it from scratch?
## HTML Streaming

It's surprisingly simple to express this with Node.js alone. No `npm` dependencies - straight node.

```js
import { createServer } from "node:http";

createServer(async (_req, res) => {
	res.writeHead(200, {
		"Content-Type": "text/html",
		"Transfer-Encoding": "chunked",
	});

	res.write('<h1>This loads immediately</h1>\n');
	setTimeout(() => {
		res.write('<p>This loads after a while</p>');
		res.end();
	}, 2000)
}).listen(8080);

console.log('server on http://localhost:8080');
```

This won't work in Bun without an extra `res.write()`. There's [an open issue for this](https://github.com/oven-sh/bun/issues/5869).

While this demo works, it doesn't support "out of order" streaming. We're writing information as we have it. We can't go back and rewrite old information. This is an "append-only" system.

What if we have the classic case - a header and footer with variant main content. Do we hold off rendering the footer until we have the main content?
## Out Of Order Streaming

Without JavaScript, accomplishing this is very difficult. A [blog post 3 months back](https://lamplightdev.com/blog/2024/01/10/streaming-html-out-of-order-without-javascript/) explored a solution that used the [declarative shadow DOM](https://developer.chrome.com/docs/css-ui/declarative-shadow-dom), but this is a very new feature with major implications for how the rest of your scripts interact with the page. If all your clients run modern browsers with JS disabled (why??), this could work.

Otherwise, we'll use the same technique most frameworks use.

We send down a placeholder element that we replace using JavaScript with the actual nodes when we send them.

```js
import { createServer } from "node:http";

createServer(async (_req, res) => {
	res.writeHead(200, {
		"Content-Type": "text/html",
		"Transfer-Encoding": "chunked",
	});

	res.write(`
		<header>Hello from header</header>
		<placeholder-element></placeholder-element>
		<footer>Hello from footer</footer>
	`);
	
	setTimeout(() => {
		res.write(`
			<main>This is the new content</main>
			<script type="module">
				const placeholderEl = document.querySelector('placeholder-element');
				const mainEl = document.querySelector('main');
				placeholderEl.replaceWith(mainEl);
			</script>
		`);
		res.end();
	}, 2000)
}).listen(8080);

console.log('server on http://localhost:8080');
```

It's fairly seamless, but not perfect. On very slow computers with more complex layouts and a larger JS bundle, there is a parse step that will stand between the content loading and being swapped.

The other problem might be that it's not as easily generalizable. With some help from [LankyMoose](https://github.com/lankymoose) and [Kai](https://github.com/panthyy), here is a more comprehensive (and complicated) version
## A Comprehensive Version

We start with a function that defines its own placeholder element and method for replacing said content when some task finishes.

```js
const suspend = (placeholder, promise) => {
	const id = "stream-" + Math.random().toString(32).slice(2);
	const inserterTag = `<script 
		src="data:text/javascript," 
		onload="window[&quot;${id}&quot;].replaceWith(this.previousElementSibling), this.remove()"
    ></script>`;
	
	const p = promise.then((content) => `${content}${inserterTag}`);
	p[Symbol.toPrimitive] = () =>
		`<content-placeholder id="${id}">${placeholder}</content-placeholder>`;
	
	return p;
};
```

A little hacky, but I like it. We return a regular promise that resolves into the content. If someone tries to parse us like a regular string, we return the placeholder element instead.

The resolved content and placeholder share a custom ID. The resolved content sits next to a script tag that (when it loads) will automatically replace the placeholder and delete itself.

It's strange, but it was fun to write. 

```js
async function* getStreamingChunks(strings, ...values) {
	yield String.raw(strings, ...values);
	
	const chunkPromises = values.filter((value) => value instanceof Promise);
	const resolvers = [];
	const promiseQueue = Array(chunkPromises.length).fill().map(() => 
		new Promise((res) => resolvers.push(res)));
	
	for (const chunkPromise of chunkPromises) {
		chunkPromise.then((content) => resolvers.shift()(content));
	}
	
	for await (const resolvedChunk of promiseQueue) {
		yield resolvedChunk;
	}
}
```

Speaking of super weird code, this allows you to use a tagged template and returns an async iterable by leveraging a generator function!

We immediately yield the stringified version of our content using [String.raw](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/raw) and then start building the async chunk of our code.

From our variables, we take all promises and construct an identical, but empty, "promise queue". It's a weird bit of code, but those few lines let us create an array of promises that resolve in-order. Don't spend too much time looking, it'll hurt your brain.

We then yield each resolved value from our array as they get come.

Putting it all together lets us write code like this.

```js
const chunks = getStreamingChunks`
	<header>This is the Header</header>
	<main>
	  ${suspend(
      "<h1>This is going to change</h1>",
      sleep(1000).then(() => "<h1>Hello World!</h1>")
	  )}
	  ${suspend(
      "<p>This is going to change</p>",
      sleep(500).then(() => "<p>Hello this is a paragraph</p>")
	  )}
	</main>
	<footer>This is the footer</footer>
`;

for await (const chunk of chunks) {
	res.write(chunk);
}

res.end();
```

Ain't that neat!

You can view the [full code here in a github gist](https://gist.github.com/EmNudge/2a5353488a41b8ea543cf5b7e0f1d31b).

\# end note