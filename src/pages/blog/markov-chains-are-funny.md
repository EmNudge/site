---
pubDate: August 18, 2024
title: Markov chains are funnier than LLMs
summary: Predictability machines and personality
tags: general
layout: ../../layouts/Blog.astro
---

Before explaining any of these terms, let’s try to establish this anecdotally.

> 12:2 And I will make all my goodness pass before thee, and our sins be upon us, because of our use of not and lisp-value.

And

> In the beginning was the lambda expression, and the lambda expression was with Scheme, and the lambda expression was Scheme.

One of these is the result of a Markov chain trained on the dataset of the King James Bible and a Computer Science textbook[^1]. The other is ChatGPT 3.5 given instructions to form a similar output.

These 2 examples are somewhat cherry-picked, but each was given a fair fight. I tried to choose the best candidate from both sides. Can you tell which one was which?

You have a good chance of guessing correctly. The sentence with funky semantics is the Markov chain. It almost makes sense, but makes a right turn into gibberish before you reach the end of the sentence.

The second sentence is Chat GPT.

My goal in this article is to convince you that humor is measurable and that Markov chains are funny. To do this, I will need to define both Markov chains and humor itself. Let’s start with the easier one.

# What is a Markov chain

When LLMs first came on the scene, people would describe them as a very smart Markov chain. These days, people are more familiar with LLMs than Markov chains, so I’ll describe a Markov chain as a very dumb LLM.

If it wasn’t clear earlier, ChatGPT is a kind of LLM - a Large Language Model. We have LLMs that are very large (greater than 300GB) and ones that are much smaller (less than 10GB), but you wouldn’t call them a “small language model” - they’re just a small LLM.

You could call Markov chains a very very small, very simple, very naive LLM.

Like an LLM, it predicts the next word based off the current context. However, it doesn’t take into account semantics, dimensionality, and a whole bunch of other specialized vector math. It’s a really primitive statistical model.

Have you ever used those “next word suggestions” on the top of your phone’s keyboard? That’s generally built using a Markov chain. It’s cheap to run and can be easily updated with suggestions more common to your own texting style.

I could go into depth talking about how both LLMs and Markov chains work, but I only need you to know this - Markov chains are worse at doing the work that LLMs are used for. If you ask it to generate a sentence with a certain goal in mind, an LLM will often outperform the Markov chain.

But accuracy is not what makes something funny.

# What is funny

This is a weird obsession of mine. I’ve wrote at length about it several times, so just to summarize - humor is about unserious surprise.

The best jokes involve a pleasant and significant “snap”. I use “snap” instead of “punchline” to avoid the semantic baggage. A snap is the whiplash received from the surprise. The less surprise, the less funny.

This is why jokes you’ve heard many times become less funny. It’s why “random” humor feels humorless - although the exact words are unpredictable, the expectation of unpredictability is predictable. If you truly expect the unexpected, you probably won’t laugh.

You can strengthen the snap by reusing common patterns with predictable endings and violating the user’s expectation. As an example, the word coupling “banana, apple, orange, vehicular manslaughter” sets up a pattern of single-word fruit and violates the expectation with a crime.

Joke writing is mostly about violating a pattern.

The snap is also made stronger via the “realization of scene”. If you use more original or descriptive language, you can make the scene appear more real. Instead of “he was shot”, you might say “he was pierced by a 35mm”. Instead of “he fell”, perhaps “his face met the ground”.

You also might want to try starting from the middle of a scene. Imagining what happened before will also help with realization. For example “a urinal cake? I’m not falling for that one again” implies a scene we have to imagine, increasing its realization.

You’ll notice many parallels between good _joke_ writing and good writing in general. Both sorts of authors have similar goals. Using cliche is just wasting words because you leave your scenes *unrealized*.

We call humor subjective because what qualifies as “unserious surprise” is not universal. Crass humor might not be funny because it is not unserious; it’s blasphemy! Or, it might not be funny because it’s not a surprise; it’s expected.

Anti-jokes are only funny if the joke structure itself is predictable. Absurdism has to be bought into. You can violate cultural norms, but the violation must be well understood unserious.

As a monolingual American, I still managed to make (successful) jokes in a very non-english environment by using the word “no” in ways that were culturally unexpected. I refused certain gifts in an unserious way.

Jokes are varied and humor is subject. Yet, at the end of the day, it is unserious surprise.

# The predictability of LLMs

To predict a sentence successfully, you need a lot of context. Large language models have a lot of context. They do tons of fancy math and find the most probable next token. A “better” LLM will necessarily be more predictable if its corpus is sensible speech.

This makes LLMs a really poor choice for creative writing. Without a ton of “prompt engineering”, it’s actually quite easy to spot if some paragraph was LLM generated. It sounds soulless. It’s the most average thing you could have possibly produced given the context.

Asking an LLM for an “original thought” is almost oxymoronic, if not just moronic. It was built with the express purpose of not doing that.

To generate a joke, an LLM would have to be surprising. It would need to take a common turn of phrase and shift its meaning in an unusual way. Good LLMs don’t do this.

I’ve heard people claim that comedy cannot be generated by an algorithm. If it wasn’t obvious, I disagree with this. I think comedy can be analyzed and measured. I think, given a large enough grant, we could probably generate comedy on demand. I think it’s doable - not that we should do it.

The LLMs we have today are just the wrong tool for this task.

LLMs were funniest at their earliest stages. Image generation was funniest here as well. Remember those "trail cam" images we got from *Dall-e mini*? As our systems got better, the humor was lost.

A really good predictability machine is not very helpful in artistic expression. There is still much use, sure, but it's not the perfect tool for the job. An LLM will often miss interesting concepts that a child can readily offer.

I’m open to the idea that, given this framework, we can probably build a kind of language model which takes this task into account. It would just need to be categorically different than the kinds of LLMs we have today. It would be different enough that we probably wouldn't call it an LLM.

# Why this is interesting

This is a weird thing to get passionate about. It's manifestly true, but I think it speaks to something deeper. 

Again, I am not making the spiritual man versus machine argument. It's simply a flaw you'll see come up time and time again when interacting with these models, no matter how advanced they seem to be getting. It's a leaky abstraction, baring its inner architecture when it meant to masquerade as an anthropomorphism. 

This is why every message from ChatGPT reads like a high school essay. It has taken the most average kind of output and reproduced it. It has been stripped of personality and toughened with scholastic rigor. It is bland, corporate speak. 

It's easy to spot fake Amazon reviews these days by thinking "would I ever write something like this?". Would you add an intro and conclusion to your experiences with Oxiclean dish wipes? Would you thank the manufacturers and acknowledge their dedication to customer service? 

Our LLM detection models, like an on-screen captcha, must soon start screening for personality. 

[^1]: The example comes from a [famous tumblr blog](https://kingjamesprogramming.tumblr.com/) which was recently resurrected!