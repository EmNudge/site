---
pubDate: June 30, 2024
title: "Moving away from Discord as a productivity platform"
summary: Whatever can replace this technology
tags: coding
layout: ../../layouts/Blog.astro
---

This title will sound strange if you weren’t already familiar with my full suite of productivity systems. I’ve been using [Discord](https://discord.com) chats as a productivity tool for the last 5 years.

It’s remarkably effective.

## My Current System

Discord has a myriad of tools and integrations for operating large community servers. You can have channels with specific topics, bots with commands, webhooks, role-based access, channel groups, etc. A lot of these tools also work for productivity platforms.

I have a Discord server with just myself and a few bots as a member. It has channels with various topics - `#spam`, `#general`, `#grocery`, `#pics`, `#bookmarks`, `#ideas`, `#to-buy`, `#to-watch`, `#to-read`.

Because I can access Discord both on my phone and desktop, I can send quickly send content from wherever the content is closer. I get a text and need to copy a code on my computer? I send it in a Discord channel and copy it on my desktop.

Sometimes I’m in a reading or watching mood. I go to my `#to-read`/`#to-watch` channel and see what I’m feeling.

Any short-lived content goes in `#spam`. Anything I know I’ll need longer term (next week) goes in `#general`.

Still, chat platforms seem like a weird idea for this until you realize they implement one of the best UX patterns for stack-based tasks. Content is sorted automatically by most-recently sent, which is what I usually want most. I don’t need to worry about cleaning or checking off anything.

I can also search through Discord and find links I might have sent years ago. I can also search in specific channels to narrow down the topic of the content I’m searching for.

It might still sound pretty weird, but it really did work quite well.

So why am I changing?

## The problem with Discord

Unfortunately, Discord is just plain distracting. I’ve become very conscious about where I spend my time. Discord was a convenient platform since I was already on there all the time. I had a lot of friends who used the platform and I liked talking to people in programming-related servers.

Over time, I realize it’s more often a place I use to avoid work. It’s not a constructive use of my time. Sure, it’s better than scrolling through YouTube shorts, but I don’t feel happy after realizing I spent 3 hours reading chat messages today.

When I’ve had conversations about these kinds of addictive and unpleasant platform behaviors, I usually get back something like “just don’t use that part of the platform”. I hear a lot of “just don’t” in these discussions. If only I were like that. If only you were like that. We are not like that.

I don’t think I have OCD, but I do get “into things” a lot. A fun video game would keep my attention until 2am back in the day. A good book or an idea still keeps me occupied even today. I learned to remove negative distractions instead of actively fighting them. It’s not a battle I am going to win.

So I deleted my video games. I leave my bedroom to work if I can help it. I delete TikTok off my phone (I needed it for work up until recently).

I am very close to deleting Discord...

## How this can be done

I’ll need to replace Discord with some other platforms. For daily tasks, I am now firmly on [local markdown documents](https://emnudge.dev/notes/daily-brags/). I can also use this for transient info - I put it into a “notes” section.

Searchability is a bit worse. I can’t search by “topic” anymore. It’s not stack-based. It’s clearly inferior in a few areas. Hmmmm...

Luckily, I am a software engineer. I have a technical problem. This is solvable.

I need replacements for the following

- A simple interface to send transient information
- A simple interface to send long-term “tagged” information
- An interface for saving long-term documents such as ZIP files and pictures

This system should be stack-based and accessible from both my desktop and phone.

It’s also important that the system for sending text be **simple**. An advantage of any productivity system is getting out of the way. The reason people often have problems with systems like Jira that are used to organize work is that if they’re complicated, they force a context switch. This can be really dangerous if your sole goal is doing the work you’re logging and not logging the work.

I have no interest in re-creating an entire platform. In recent times I’ve seen some very clever uses of email and text for products that you’d normally require an interface for. At some point, I’ll probably want a database and S3 bucket, but it’s not necessary for every aspect.

There are a few other features I’ve built around Discord that I also want to transition.

- Sending data to my knowledge base from an unlisted public domain
- Scheduling reminders

These might be trickier.

## No-code solutions

I mentioned text and email earlier. Using these, I can already cover some of my use cases.

I can start a text conversation with myself. This reuses the same chat UX, just without categories. This can be good for transient info.

I can “tag” info with some unique syntax, such as `!movie` for movies. I currently use iMessage which doesn’t let you search specific chats (why??), but if the formatting is unique enough, this won’t matter.

If I’m trying to store media, I can use email threads. The UX is not the best here, but I don’t upload media as often and it’s not important to the experience for it to be quick.

## Text-based high-code integration

[Ramp](https://ramp.com) has a really compelling text experience. The interface for texting is so simple that it got me thinking about building something similar. What if I could text some number and have some actions performed on my behalf?

The rise of cheap LLMs also makes NLP work a breeze. I can take an unstructured request and transform it into JSON for some backend system.

So the user flow might look like:

- Text special number
- If processed successfully, react with 👍
- If the category is unclear, ask questions with a 15m timeout on processing the response

Transient messages would be simple texts. I could tag messages and get responses for ambiguity. Reminders would be texts sent back to me. Elegant.

A lot of this requires task scheduling (a.k.a. dynamic cronjobs) and various API integrations, so I would probably opt for a VPS instead of edge functions.

Unfortunately, using a texting service requires a ton of regulatory oversight and is expensive for what I'm getting. This also would not play well with iMessage and I unfortunately switched over from Android a couple of months ago.

However, if I'm solely using iMessage anyway, I might want to check out self-hosting Beeper.

\# end note