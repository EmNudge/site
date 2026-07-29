---
pubDate: Jul 28, 2026
title: "Layers of Abstraction"
summary: technical writing
tags: general
layout: ../../layouts/Blog.astro
---

I do a lot of technical writing. It's easy to make something complicated. Often I'll write large portions of an article over and over again - each time cutting out large swaths of text until I'm left with what feels like a trivial few paragraphs.

In explaining deep concepts with lots of technical baggage, I've landed on a framing that I'm really pleased with - "layers of abstraction".

Let's try to demonstrate this with a sample technical piece on a complex subject - diagnosing a slow website.

> Websites can feel slow for a variety of different reasons. One such reason is render performance. A state update can trigger large CPU tasks in even figuring out what part of the UI to update. The update strategies of frameworks like React and Solid differ enough (VDOM vs fine-grained reactivity) to make for significant divergences in their performance characteristics in mature codebases. Even when done correctly, your choice of CSS property mutation can greatly influence whether layout geometry computations occur. The layout engine (Blink, WebKit, Gecko) will have an influence on the efficiency of this process, but simply changing a `position: relative; left: 2px` to a `translate-x: 2px` can have a dramatic impact on your output FPS. Your JS engine (V8, JavaScriptCore, and SpiderMonkey) has comparatively less of an impact on the experience. Browser divergences are not only annoying to fix for, they're rarely core to the issue.
> 
> 
> ...
> 

Assume this goes on for a while.

Nothing here is egregious, but notice that the point of the article was about slow websites and I just went on a tirade about render performance. Render performance is a **deep** topic. This paragraph can easily become its own article and it's easy to let that happen.

Large articles and long videos are not always a mark of quality; they can often be a signal of the opposite. It's easier to talk than it is to trim.

But the length isn't the issue. It's that reading this is a commitment, and if you stop reading early, you walk away without a complete (or accurate) picture of the problem domain or solution.

This is becoming a much larger problem with the onslaught of AI-generated docs. With no guard rails, the LLM just outputs miles of text with useless analogies and 'isms. You read blocks of text without knowing whether there's any value, but very sure that the value it contains should not take this long to read.

Technical writing is actually quite difficult and it'll take a very smart model with great post-training and a few solid prompts to get something serviceable. In some of my efforts to remedy this, I've had to dig into my own principles on technical writing - the things I know, but don't know I know. This is one of them.

Layers of abstraction.

The trick is to model the answer in a manner where you don't move through the solution in order. Really, there is no order. You're playing with uncountable infinities here - you can't explain a concept in depth with this approach - it's like listing all numbers between 1 and 2.

Instead, structure the answer such that you can stop at (almost) any paragraph already with a complete model of the problem and solution.

Why keep reading? To get deeper. There's nuance and we can take our time to deliver it, but it doesn't need to be upfront.

Here's another take on our previous article.

> Websites can feel slow for a variety of different reasons. Your device and browser are one part of the equation, but the majority of cases are to do with the network. This includes the network conditions between your device and the host, but also the architecture of the website and its relationship to the network.
> 
> 
> ...
> 

Within the first paragraph I've already given you the meat of the article. It's the kind of thing you might stick in the "in conclusion" part of your high school essay. Or perhaps even the intro.

This kind of writing can feel less interesting to write and sometimes less engaging to read. But it's easier and quicker to consume.

Not easier and quicker in the way that fast food is easy and quick - it is easy and quick in the way going for a walk is easier and quicker than changing career paths after your first bout of conflict at work.

Sometimes easy and quick is better.