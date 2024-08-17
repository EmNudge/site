---
pubDate: August 17, 2024
title: "I'm Using a Markdown Database"
summary: Simple text stores
tags: productivity
layout: ../../layouts/Blog.astro
---

I've become a little obsessed with simple systems and shaving off complexity. That usually means using a tool that already exists instead of making one myself. Sometimes it means opting for rigid infrastructure with text-based data stores. This is such a time.

I wrote [before](https://emnudge.dev/notes/daily-brags/) about my use of Obsidian as my notes app. I use it for (among others) brag documents, blog posts, fitness records, and to-watch lists.

I take recommendations pretty seriously if they're from someone I enjoy spending time with - I will at a minimum write it down. I won't necessarily read it, but I will at least look into it. I maintain a pretty large and active list of these recommendations when it comes to TV shows and movies. If I want to watch something, my first stop is that list.

Wouldn't it be cool if I made it public?

## Syncing

I point Obsidian at an iCloud folder which is synced automatically between my phone and laptop. Between my laptop and a linux VPS, I use [`syncthing`](https://syncthing.net/). While familiar, `git` would have introduced more complexity and manual work than I needed - syncing software like these makes things pretty seamless. 

So I change text on my phone which changes the text on my laptop. A program running on my laptop syncs it to my linux VPS. My linux VPS is responsible for displaying the website.

This setup means changes from my phone are not instantly reflected on my VPS if my computer is not running. It would be better for all these devices to share the same syncing software. This would mean [Mobius Sync](https://mobiussync.com/) or [Secure ShellFish](https://secureshellfish.app/) to get syncthing on iOS. That's a bit annoying and so a task for future-me.

## Running The Site

I'm using [Deno](https://deno.com/) to take this local markdown text and render it as HTML on each request. It combines the output with a local template file so the site looks a bit nicer.

I have Cloudflare as the DNS and Caddy locally as the reverse-proxy. It's a pretty simple concept!

## Check It Out

I also added a reviews page because I'm somehow too lazy for Letterboxd. 

https://shows.5t.rip/

\# end note