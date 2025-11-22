---
pubDate: November 22, 2025
title: The vertical slice
summary: AKA the MVP
tags: general
icon: technologies/languages/javascript
layout: ../../layouts/Blog.astro
---

I bring this up a lot at work. It’s a concept I took from game development. In generalized software development, the closest analogue is the MVP.

Say you’re given a project and expected to deliver estimates for its completion. Not uncommon.

There are a couple routes you can take. You can take a week to scope out all the edge cases. You can find resourcing and figure out all the ways things can go wrong. And then you can deliver an estimate that is still wildly inaccurate, but such is software, I suppose.

Or you can build the MVP. 

I don’t like the term MVP much, it’s too muddied with context from my industry so as to become less useful as a descriptor. I prefer the term (borrowed from game dev) “vertical slice”.

But the concept is identical. You’ll find hundreds of issues with any project. The unknown unknowns are many and will drag the delivery dates for any project regardless of how much you pre-anticipate. 

Start-ups already know this. They need to know or this or they do not survive.

You’ll find inefficiencies in large organizations the second teams start to view themselves as a cog of a larger system instead of as independent startups all working on MVPs of a related product category.

So what do start-ups do?

They build. Your immediate goal is not to plan, but to build aggressively. You build and build and throw away feature after feature until you are left with the bare bones version of the thing you wanted to build. You work this way because any other method of development will take longer than your runway allows for.

There are a couple really cool attributes to this process.

Firstly, unknown discovery is heavily accelerated. The spec writing process becomes intertwined with the development process. As you build, you find questions. Still, your goal is not shifted much from the changing requirements.

Your goal is to build the most basic, buggy, broken version of the idea of the thing, not the thing itself. Find another edge case? Cool, add it to the list and ignore it for now.

You get an immense amount of visibility into the planning. Far more than pre-planning and building directly.

Secondly, at any point in the process where you decide to stop developing, you will still have the thing. Development can hit a variety of hiccups. The tech lead might get burnout. You might have shifting product requirements. Scope creep. There could be a bigger rock that needs moving.

By defining the exact vertical slice up front you set a reasonable and easily estimated goal. From there on, each addition becomes cleanup and polish rather than the thing itself. The tech debt is obvious and the pain is felt. It becomes easier to estimate the next leg of work.

The danger to this strategy is divergent understandings of complexity.

This is inevitable. Product has no insight into how hard an engineering task is. The relationship works similarly the other way. It is very difficult to tell someone a task is easy if you personally are unable to do the task. 

Truthfully, if you cannot do something, you should forbid yourself from ever guessing at its effort. You will not only be wrong, but come across as extraordinarily disrespectful.

Still, you will now find your managers and PMs looking upon your 1 week deliverable and exclaiming “oh nice, so the hard part is done!” much to your chagrin. It’s really important to be upfront about the expectations when executing this strategy.

Just to add in a quick catch-all for some generalizations I’ve made above - yes, this is product centered. When developing work more technically demanding and possibly novel, it is harder to create an MVP in a comparable time frame to that of a product-focused feature.

Still, the same underlying philosophy applies - if you are facing issues with time estimates, change the form of development. Do less planning and more building. The deliverable is given in chunks of completeness instead of correctness or experience. 

Do not talk, then plan, then review, then build. Instead, build while you plan so that you can plan from the build. The vertical slice becomes the discovery process.

I hate giving examples, but I love reading them in long-winded high-level articles, so let me provide one.

I was helping out a friend with a personal project. It was a twitter clone with a novel angle. The app would artificially restrict posting cadence so that each post felt more intentional. 

He began with the Figma mockups. He made the session storage layer, profiles, search functionality, resiliency layers, likes, and the app did not yet work.

We sat down and took a look at what we were building. This was a personal project. The risk of abandonment for these is **high**. Anything this complicated was never going to be finished. Since part of the motivation for this project was resume padding, it was important that it was finished. So we implemented the vertical slice. 

Taking a step back, what was the most minimal version of the product? It was a list with some kind of temporal lock. Okay, build that first. He built the most primitive version of the thing and then every step after that was cleanup and feature extensions.

Like every personal project, it was eventually abandoned, but this time the project was functional no matter when he decided to give up.

