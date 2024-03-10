---
pubDate: Mar 9, 2024
title: "One Of My Best Habits - Daily Brag Documents"
summary: Life Coaches hate him! This one tip will change your life!
tags: productivity
layout: ../../layouts/Blog.astro
---

I'm a bit of a productivity junkie. I obsess not just about being productive, but around crafting systems to be more productive. You've got to be a certain kind of neurotic nerd to sympathize, but I work in tech - there's plenty of us. I'm sure you understand.

It's important to be careful that you're not spending a lot of time organizing productivity systems **instead** of being productive, as is often the case. I've spent a lot of time looking into novel software, notepads, e-ink tablets, pens, scheduling systems, only to realize that really none of it mattered.

A lot of it relied on ideas that were too complex to maintain for extended periods of time and systems that weren't generalizable to different environments. i.e. does it work when I'm traveling? Does it fall apart when I change computers? 

Simple is better. A basic notebook and pen has been really useful to just make todo lists and jot down ideas. 

I used to look into really fancy pens and notebooks, but this another trap. If your productivity system is too nice, you'll try to save it for your "best" ideas. These don't exist. You need to get into the habit of using your system - you can't save it. You need to be okay with most of your notes and ideas being bad. You need to encourage bad ideas. 

# My Current Setup

I got the core of this idea from [Julia Evans' Brag Documents post](https://jvns.ca/blog/brag-documents/), so I started by calling these "brag documents". The idea is to keep a work history in order to better "brag" about your work later down the line. She mentions what looks like a pretty sophisticated system - another trap, I'm afraid.

Here's my "brag document":

```md
# Todo
- item 1

# Done
- item 2

# Notes
-
```

Each document is titled with the current day and I make these documents every day. I often don't include the `Notes` section unless I think I need it. 

Each day I start off with some "todo" items. I will add items to this section in bullet points as I get ideas. We have the full power of markdown, so I will include links very often which can be useful in getting back to a todo list item. 

What do I put in the `Done` section? Absolutely everything. 

Well, anything that feels like work. Here's what an example document might look like at the end of the day:

```md
# Todo
- look into elevator algorithms 
- Buy thing to block window in bedroom
- Take out trash
- learn design patterns?

# Done
- Bought full length mirror
- made pita bread
- bought groceries for tomorrow
- bought tea
- made condensed milk and tea (loose leaf tea + condensed milk + brown sugar)
- finally bought a REAPER license
	- Set up [nvk_search](https://app.gumroad.com/d/ff477a259bb73e7ddebcca3efbce371d) (license key: `D21C68B2-xxxx`)

# Notes
- [Greg Wilson's book](https://www.amazon.com/dp/B00557TMN4/)
- Maybe look into [Cloudflare R2](https://www.cloudflare.com/developer-platform/r2/)
```

This is a real document from a week ago! Perhaps this is a bit revealing into what I do on my weekend, but here you have it.

You'll notice that I put `?` next to todo list items that I'm unsure if I want to pursue. All my items should be actionable. If something feels too big, I'll make it optional. 

Also notice that I said this is the document from the end of the day and yet it still has items I had not done (like taking out the trash).

This is fine. When I create my document for the next day I'll carry over yesterday's todo items. Sometimes I'll re-evaluate and realize I have more pressing things to get to.

I try to fill in the `Done` section with as many "work" items as possible. This is partly to keep a record of things that might be important retrospectively and also to give myself a better feel of accomplishment. I can gaze upon my list of "stuff" and exclaim "Today I was productive!".

The `Done` section also has information that I will probably want to reference, like the license key (I cut off part of it) that I might need to use the next day.

The `Notes` section is usually empty, but I'll sometimes use it for things that I think are interesting, but not something I want to prioritize. 

# Organizing The Documents

I think the most important part of the system is that 
1. It is done daily
2. I add items liberally and constantly

The rest of this is just some conventions which make managing this system easier, but it is not the main part of the system.

I use [Obsidian](https://obsidian.md/) to keep things as plain-text and local as possible. I used to use Notion, but the interface at times was too sluggish. I needed to log in more often than I'd like. I needed internet connection. Querying the notes felt slow.

Obsidian is **far** less pretty, but I best stay away from pretty. I need simple and friction-free.

For syncing my notes between devices I use iCloud. I'd suggest some sort of managed file sync. Git sounds good in theory, but you need to remember to commit often, which can be difficult on a mobile device in general. With my current setup, I don't have to manually sync anything. The folder itself syncs up as often as it can. I'm writing plain markdown - I'll probably never run out of space.

As mentioned earlier, I store each note with the date. To make copying over data simpler, I will duplicate yesterday's document and just increment the day. My iCloud Obsidian vault current looks like this (assuming today is March 5th):

```
Notes/
Articles/
Personal Brags/
  01/
  02/
  2024-03-01
  2024-03-02
  2024-03-03
  2024-03-04
  2024-03-05
```

I use the same vault for drafts on my "Notes" and "Articles" for my website. 

I group a whole month's documents into a sub-folder to help with visual clutter when trying to find documents. I can find a document just by knowing its day and I can know the day of an event or accomplishment by the title of the document.

You'll notice it says "Personal Brags". This is because I also maintain a list of Work Brag documents, mimicking the exact same system, but for work and stored on my work laptop (without an iCloud vault). The work brag documents are often much more extensive with multiple streams of thoughts and links to different documents, but they are similarly useful and are more closely aligned to the actual brag document definition.

I still make it my mission to write them every day. Whenever we have a sync on what work we've been doing - I often just read through bullet points of the last few days and summarize it like that. I also embed links, which makes linking to work items much simpler as well.

# Current Problems

The mobile experience is still lacking a bit. I'm using Obsidian for my phone, but it's not the world's best phone app. Luckily most of this is done on the desktop, but it would really help if I felt like I could jot things down using my phone. 

Since we're just using markdown and a generalized sync, I could actually make my own phone app if I had some great idea, but that would probably work against the "keeping it simple" principle.

Whatever system you choose, just make sure it's simple and accessible enough to do consistently. It should feel rewarding and messy. It should **not** be elegant. Run away from pretty. Creativity and productivity does not thrive in "elegant". You will spend too much time "improving" the system as opposed to getting things done.

\# end note