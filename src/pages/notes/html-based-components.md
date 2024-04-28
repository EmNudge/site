---
pubDate: April 14, 2024
title: HTML-Based Single-Use Components
summary: Vanilla JavaScript components with native HTML
tags: coding
layout: ../../layouts/Blog.astro
---

I'm writing [logpipe](https://github.com/EmNudge/logpipe) without a build system. Any improvements I make in DX I'll have to do by hand. Annoyingly, I'm making a somewhat complex frontend application with raw JS, so it's hard to compartmentalize things.

We regularly think of components as multi-use pieces of compartmentalized code. This means their state is usually attached to the component instance, so you can make many instances of a component without any issues. I don't really need this. I know exactly how many UI elements and state handlers I need. I just want to clean up my code.

This is a locally run tool. A lot of assumptions we make about building web apps change when we talk about fringe environments. Bundling is usually better for application download times, but when the server is on your computer, that part is negligible. 

With that in mind, here's what I'm doing.

# Project Layout

I added 2 big features - the context menu and command palette.

They have associated UI elements, styling, and functionality. This makes for the oft-seen webdev triple, but this time as separate browser-native files.

```
command-palette/
	index.html
	index.js
	style.css
context-menu/
	index.html
	index.js
	style.css
index.html
index.js
```

I'm running a single-page application, so these sub-folders are components, not pages. If my project was a bit different, I'd probably want to label this by putting them in a "components" folder or something.

HTML files are the web entry point, so unlike modern frameworks, I have them do the importing. They link to the associated style.css and JS files. This lets them be flexible with what they include.

```html
<body>
	<script type="modue" src="./index.js"></script>
	<link rel="stylesheet" href="./style.css">
	<h1>Hello from component</h1>
</body>
```

We import them by making a fetch request to the raw HTML and inlining it directly.

```js
async function addComponent(url) {
	const html = await fetch(url).then(res => res.text());
	const span = document.createElement('span');
	span.innerHTML = html;
	document.body.append(...span.children);
}
```

While we often put them in the `<head>` tag, you can insert `<style>` and `<script>` tags pretty much anywhere, so we don't need to do any re-ordering.

But this actually won't work. 

# Script Security

The script tags are marked as external and your browser won't load them. No error is thrown, so this part might be hard to track down.

We can get around this by manually importing them.

```js
const modules = [...span.querySelectorAll("script")].map((el) => el.src);

document.body.append(...span.children);

// Only run after inserting so that the DOM is loaded
await Promise.all(modules.map((src) => import(src)));
```

In my own code, I allow relative imports from the HTML, which breaks down if we simply inline it as I did here. Instead, my function takes a folder name and remaps the relative imports to absolute ones.

Here is the final code I used.

```js
/**
 * Loads a folder and its associated code.
 * @param {string} folder component folder name
 */
export async function loadHtmlComponent(folder) {
  const html = await fetch(`/${folder}/index.html`).then((res) => res.text());

  // temp element used to deserialize HTML
  const span = document.createElement("span");
  // replace relative imports with absolute imports
  span.innerHTML = html.replace(/"\.\/(.+?)"/g, (_, path) => `/${folder}/${path}`);

  // Extract script tag sources because external scripts are blocked by the browser when appending
  const modules = [...span.querySelectorAll("script")].map((el) => el.src);

  document.body.append(...span.children);

  // We need to wait for the HTML to be added to the document before requesting
  await Promise.all(modules.map((src) => import(src)));
}
```

\# end note