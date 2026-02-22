---
pubDate: Feb 22, 2026
title: "The Quickest Trick in Design"
summary: just be consistent
tags: design
layout: ../../layouts/Blog.astro
---

I struggle a lot with making pretty UIs, which is a shame because I do FrontEnd full-time. 

[Refactoring UI](https://refactoringui.com/) is one of my favorite books on this topic. It's definitely helped me quite a bit, but I always ignored the "design systems part" of the book. It felt like designers making a suggestion on code organization. I could handle that, I thought.

Turns out, I could not handle that.

The quickest visual improvements to most of my personal projects has been asking claude to create and enforce a design system. Simply being consistent upgraded the UI almost immediately.

I think this is why I much prefer the TUIs to the GUIs for LLM chat interfaces. I really don't like the ones in my editor and prefer `claude code` or `gemini` cli over what is in Cursor or Antigravity.

The terminal enforces a consistent font size and color scheme. Your steps in color change aren't nearly as smooth as you'd like, so these choices become deliberate.

So here's the trick: make a design system.

I've been going through some old projects and asking claude code to take into account my entire component library and create a design system around it. Just being consistent in colors, font sizing, weight, spacing, and interaction has improved the design.

I can do this somewhat mechanically without needing especially good taste. Just make it consistent.
