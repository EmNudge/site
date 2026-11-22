---
pubDate: Nov 21, 2026
title: "Notes on (LLM) comprehensibility"
tags: llm
summary: How to prompt better communication
layout: ../../layouts/Blog.astro
---

I really enjoy teaching. I also enjoy thinking about teaching. I enjoy the craft of chiseling at an explanation until it becomes obvious enough to impart to anyone.

This is a useful skill to work on for a myriad of reasons, but it's become an especially important topic with the advent of machines writing human-adjacent prose. LLMs often produce subpar output in human-centric content, and it's helpful to have a language to call out why.

Here are some disorganized observations and thoughts.

### Why are LLMs so bad at writing

Better phrased - why do they appear so much better at coding than writing?

My guess is that it is very difficult to build an eval set and there's less value in building this as a target.

The second factor is changing. Anecdotally, GPT models improved on Opus 4.8. Then Opus 5.5 seemed to outperform GPT models. I'm hearing that Gemini Neon on low thinking mode is currently on top. None of this is very consistent. It's quite easy to get terrible prose from even widely praised models. Neon on high thinking mode felt overly dense to me. 

I think the average person is not a good judge of fine writing. People are easily tricked by complex constructions and big words. "It's not this, it's that" is a rhetorical device that easily exploits something in the average reader. Just because something "feels" explanatory does not mean it is. 

As such, getting outputs that actually improve understanding is difficult, even for the labs.

### What works

It's not easy, but I've experimented with some prompts that have improved the legibility of our PR descriptions. As models get better, my prompts can be thinner. 

The prompt is small and includes (roughly):
- \[3 examples of good descriptions]
- Be brief, at most 3 paragraphs
- The first paragraph should be a single sentence.
- Do not comment on the code, only (at most) the motivation and the behavior

Explanations of the code itself are a red herring. It sounds useful but is usually secondary to just reading the code. The pieces that are hard to ascertain by reading specific lines are best consumed as a whole and usually summarized as "intent" and "product". 

Descriptions are skimmed, usually along whitespace boundaries, so separate paragraphs allow for this.

I also make sure to embed [layers of abstraction](https://emnudge.dev/notes/layers-of-abstraction/) - every paragraph should be doing the same thing. It should be explaining the entirety of the change only in increasing amounts of detail. 

Long pieces of text require keeping some information in "human RAM" in case it becomes relevant. This is often expensive. It's similar to how students are often better served by reading the quiz questions at the end of a textbook chapter before reading the chapter itself. It gives you an "attention mechanism" so that you can selectively memorize pieces of the overall text.

I'm struck that I'm using computer terms, which were based on human terms, to describe human ones. It's weird.

### Code tours

We experiment with code tours. I know some people find them valuable. I've had a hard time doing so. I think text explanations of text is the wrong direction. 

It's hard to know that taking the time to read slop will actually deliver on the cost, so it's best to minimize this. No one likes to feel that their time is wasted.

I speculate that an improved form of code explanation uses colors, symbols, and arrows to direct your attention to signals that you can choose to ignore or focus on at will. It should be some compromise between distracting and something you need to toggle on intentionally.

I would love to get takes from [Ink & Switch](https://www.inkandswitch.com/) on how to model this. I fear I am out of my depth on this one. I couldn't convince the [Zed](https://zed.dev/) folks to work on it. 

I'll talk more on code understandability in a later section, but I don't think code tours are the right direction. 

### Surprising finds

I get sent PRs from all sorts of sections of our codebase. Getting up to speed quickly is important.

The PR description summary might itself be pretty taxing. Assuming the same order of "layers of abstraction", the first sentence could appear as:

>Improves frungle bingling for the joncules of the gorter when in a reverse hyper state after latent tortoise explosions.

What we've found to work with today's technology has been narrated videos. Yes, real TikTok-esque. There's something about a combination of visuals and audio that lets your brain relax a little bit while still getting the gist of the impact. 

I usually prompt an LLM to get me a narrated "before-after video" to catch me up on what we're doing. We've also been doing this on a weekly cadence to give high level overviews on what an org shipped.

Honestly, I would not have predicted this is good. My intuition on LLMs' ability to write made me assume ad-hoc construction of explanatory videos would have been miserable. 

### On relative comprehensibility

I work largely within TypeScript. My legibility prompt is huge and contains tons of very specific rules on what I think makes code readable. A lot of this is informed by experience. I like reading code that I would have written.

It's possible that the things I find readable are not universal. It's also possible that there are topics worth learning that make everything after much more readable. 

My guess is that we define things as comprehensible if they mostly follow a line of best fit through strong connections already made in our brain with the slight divergences being the nuggets we call novel information. As such, maybe it's worth learning some functional programming in order to find declarative codebases pleasant to work with.

I buy this argument. I've seen a lot of one-way doors in programming pedagogy. 

Largely, people find declarative code easier to read than imperative code once they get used to reading declarative code. Most engineers, learning independently, seem to catch on to "early returns" as a useful way to reduce complexity and cyclomatic complexity as a burden worth attacking.

I noticed this weird pattern of prose from a few LLMs and I initially diagnosed it as being German. I just remembered from the bits of German that I know that German often pushes the verb to the end of a clause (Subject-Object-Verb), which is hard for English speakers to internalize. Sort of like "the coder complicated code wrote" instead of "the coder wrote complicated code". 

On closer inspection, I was actually noticing "Left-Branching" and "Nominalization". This is more of the same in the category of things that are universally hard to parse. You need to hold onto longer phrases in working memory before you know what to do with them. 

It seems we can pin complexity on cognitive load and interpret cognitive load as the burden on working memory through sentence parsing and where information is scattered. 

Sentences should be conveyed in a way that is familiar (reuses patterns already well shaped by experience), but not redundant. The syntax tree of the sentence should be well balanced. Attention instructions should be front-loaded so that parsing large bits of text becomes tractable. 

The best response I often get from an explanation is when I phrase a new concept as a slight deviation from a familiar concept. This fits into that framework nicely.
