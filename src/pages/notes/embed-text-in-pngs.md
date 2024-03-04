---
pubDate: Mar 3, 2024
title: "Embed Your Code In Your Code Pictures"
summary: We can make pictures of code a bit better
tags: coding
icon: technologies/languages/javascript
layout: ../../layouts/Blog.astro
---

I've been in the unfortunate (but important) business of "docs" recently and pictures have been an absolute nightmare.

Pictures of code, CLIs, interfaces. They all eventually get out of date. With regards to interfaces, this is often very quickly and without notice as visual updates don't require "backwards compatibility" in the same way as programmatic interfaces often do.

You'll often see pictures of code in environments where the support for inline-code samples is very poor. Think confluence or pre-markdown google docs.

Pictures of code are not great for a couple of reasons. It means you have to regenerate the picture with every API change, you can't copy it from the image directly in testing, and you can't even find out if the code might be invalid by pasting it into an environment with a language server. 

These pictures are often generated with web services like [Carbon](https://carbon.now.sh/), [Ray.so](http://ray.so/), and [Chalk.ist](https://chalk.ist/) which all use browser technologies to generate PNGs. This is notable because PNGs actually allow us to embed meta data!

# Behind PNGs

Many different media formats will use a "chunk-based" encoding where different aspects of a file are grouped together in their respective "chunks". These chunks will usually start with some information about their content, such as their name and size.

For example, [WAV files](https://en.wikipedia.org/wiki/WAV#RIFF_WAVE) will contain a "format" and "data" chunk, but may also include "fact", "cue", and "playlist" chunks.

[PNG contains chunks](https://en.wikipedia.org/wiki/PNG#Critical_chunks) with wonderfully descriptive names like "IHDR", "PLTE", "IDAT", and "IEND". Among the 17 optional chunks are "tEXt" and "eXIf". None of the optional chunks are critical for rendering and may be stripped, but in my testing the `tEXt` chunk often remains. 

Nevertheless, if we're getting an image straight from these generators, it should be preserved. This makes for a really good opportunity to embed the text content of the image inside the image itself. 

We could even embed other settings, like the language and style settings to make updating the image much simpler. Image - just drag your image onto the interface and have it load all the original content!

The JavaScript required is relatively trivial with any PNG library (or even done completely from scratch):

```js
import extract from 'png-chunks-extract';
import encode from 'png-chunks-encode';
import text from 'png-chunk-text';

/** @param {File} file @param {string} key @param {string} value */
async function addTextToPng(file, key, value) {
  const buffer = await file.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  const chunks = extract(bytes);
  
  const newChunks = [
    ...chunks.slice(0, -1),
    text.encode(key, value),
    chunks[chunks.length - 1],
  ]
  
  return new File([encode(newChunks)], 'new.png', { type: 'image/png' })
}
```

# What To Do For Now

Unfortunately, [carbon](https://github.com/carbon-app/carbon) hasn't seen an update in a long time, Ray.so isn't open source, and [Chalk.ist](https://github.com/Idered/chalk.ist) has put these sorts of features on hold while they do a large overhaul. Having those tools implement embed the text directly would make things significantly easier.

Optimally you should not use images of code at all. Inline it if possible. 

For now, you can do this process manually. I made a quick Stackblitz project where you can add the code as text input to the image and retrieve it later. Feel free to build your own tools and expand on the concept!

<iframe src="https://stackblitz.com/edit/vitejs-vite-xksnwx?embed=1&file=main.js" style="width: 100%; min-height: 600px"></iframe>

\# end note