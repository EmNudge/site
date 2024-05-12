---
pubDate: May 12, 2024
title: "Minimal Live Reload"
summary: It's surprisingly simple
tags: coding
layout: ../../layouts/Blog.astro
---

It's been pretty useful to try making web servers with as little framework usage as possible. With Node, you can get pretty far just using the included `http` package.

Every front-end dev setup comes included with a live-reload function - the page reloading in reaction to file changes. It's pretty useful and in theory not terribly complicated to make from scratch.

I was wondering what the simplest implementation of this would look like. Here's what I came up with.

## Minimal Live Reload

First, we set up our listeners/subscribers - pages that are waiting for reload signals.

```js
const listeners = new Set();
const addSubscriber = (res) => {
  res.writeHead(200, {
    "Content-Type": "text/event-stream",
    Connection: "keep-alive",
    "Cache-Control": "no-cache",
  });

  listeners.add(res);
};
```

When a reload signal is received, we tell all listeners to reload.

```js
const triggerReload = (res) => {
  for (const lRes of listeners) lRes.write("data: reload\n\n");
  res.end();
};
```

On pages that we want to reload, we set up a [SSE](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events) listener to some path and wait for a signal. On receiving the signal, we trigger a reload.

```html
<script>
  new EventSource("/some-path-here").onmessage = () => location.reload();
</script>
```

I'm using [watchexec](https://github.com/watchexec/watchexec) to listen for file changes and chose the path `/~_` for the reload signal because it looks strange enough to be unused.

```sh
watchexec -e html,css,js 'curl -X POST http://localhost:3000/~_'
```

## The Full Code

Putting it all together, we can live reload a local `index.html` file with the following code.

```js
const listeners = new Set();
const addSubscriber = (res) => {
  res.writeHead(200, {
    "Content-Type": "text/event-stream",
    Connection: "keep-alive",
    "Cache-Control": "no-cache",
  });

  listeners.add(res);
};
const triggerReload = (res) => {
  for (const lRes of listeners) lRes.write("data: reload\n\n");
  res.end();
};
const getReloadScript = (path) => `<script>
  new EventSource("${path}").onmessage = () => location.reload();
</script>`;

require("http")
  .createServer((req, res) => {
    if (req.url === "/~_" && req.method === "GET") {
      addSubscriber(res);
    } else if (req.url === "/~_" && req.method === "POST") {
      triggerReload(res);
    } else if (
      req.method === "GET" &&
      (req.url === "/index.html" || req.url === "/")
    ) {
      res.writeHead(200, { "Content-Type": "text/html" });
      res.write(`${require("fs").readFileSync('./index.html')} ${getReloadScript("/~_")}`);
      res.end();
    } else {
      res.writeHead(404);
      res.end();
    }
  })
  .listen(3000, () => {
    console.log("Server listening on port http://localhost:3000");
  });
```

[The gist is available here](https://gist.github.com/EmNudge/5d20e433919d5d36d78a03d41a2f3afe).

A technique more common these days is "[Hot Module Replacement](https://webpack.js.org/concepts/hot-module-replacement/)". This requires your code to have an understanding of **where** updates happened to reload a component of the page without reloading the page itself.

For applications that don't use a customized framework, adding in something like this would be considerably more complicated. You can also [just use Vite](https://github.com/EmNudge/audio-visualizer-playground/blob/main/src/main.ts#L48-L71).

\# end note