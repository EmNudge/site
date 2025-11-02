---
pubDate: December 30, 2024
title: "LLMs For Summarizing Legal Documents"
summary: TOS is getting out of hand
tags: productivity
layout: ../../layouts/Blog.astro
---

I present to you both a problem and a solution.

## Terms of Service

TOS (terms of service) is in a weird position. It'll hide a bunch of very exploitative clauses giving the company far more control than you might like. However, the enforceability is murky.

Many lower courts have seen cases around this, ruling on the arbitration clause specifically. The Supreme Court has yet to make a ruling around this, but several circuit courts have reached what seems like consensus. 

The ninth circuit via cases such as [Nguyen v. Barnes & Noble Inc.](https://casetext.com/case/nguyen-v-barnes-noble-inc) have ruled that putting your TOS behind a link with a checkbox for "I agree" does not qualify under "manifestation of assent" or sufficient notice. However, that same circuit in [Dohrmann v. Intuit, Inc.](https://casetext.com/case/dohrmann-v-intuit-inc?) ruled that all you need is some text next to the submit button that states clicking the button asserts implicit agreement with the TOS. 

Other circuit courts have reached a similar conclusion, such as the 2nd circuit in [Edmundson v. Klarna, Inc.](https://casetext.com/case/edmundson-v-klarna-inc-1). As long as you can link the submit button to an agreement on the TOS, it does not matter that it is not commonly read.

Once the case hits arbitration, there's no longer a good public record. It's hard to say what the courts would rule on the specifics of these documents. They're lost to private hard-to-appeal rulings.

Still, many applications have been forcing you to scroll to the bottom of the document and click "I agree" just to make extra sure the courts will enforce the arbitration clause. 

Okay, it's enforceable. What's the problem? 

## The Problem

No one reads it.

It's a long document with often complicated language. We've set up the precedent culturally that you just scroll and click through. It is rare that a company's overly controlling terms of service acts as a deterrent for users of that product. It happens, but very rarely.

Companies are therefore incentivized to make this document difficult to read and full of clauses you would never agree to if read out loud. 

If you want competition capitalism to work properly, these documents must be a part of the product. If you are deciding between product A and B, it would be helpful to know that product B has a clause which removes culpability from the distributor in cases of major harm. 

Companies would very quickly modify their TOS if it became a part of the conversation when discussing their product offering. 

This sounds difficult. Should companies have to compress their TOS? Any way of doing this would create issues in the wording of the document itself. 

## A Solution

I'm going to suggest LLMs, but I need you to hear me out on this one.

An LLM (e.g. ChatGPT, Claude, Gemini, etc) is quite good at a few very specific things. One of them is translation. It can translate between programming languages, formats, written languages, dialects, and speech patterns. It can also summarize, which is another type of translation. 

Translations are rarely exact - there is often some loss of context. Even with this, we still find it useful to translate things. It is particularly useful for cognition. I can't understand someone in speaking in another language, so I'll use some translation software to get the gist. The wording won't be completely correct, but it should be enough.

So we force companies to compress their TOS, but this time with an LLM? That doesn't sound much better.

No, what I am suggesting is an in-browser LLM summarizing a publicly viewable TOS document. It can link to sections of the document that it summarized for those who want to learn more information. These LLMs can differ from device to device, so it would be difficult for companies to game the language of their documents to come out as more favorable without also hurting their chances in courts.

This is doable today and there have already been [attempts to patent](https://www.tdcommons.org/cgi/viewcontent.cgi?article=7219&context=dpubs_series) this pretty simple idea.

What's left is the TOS being publicly accessible in a standardized way. Similar to a `robots.txt` file being available on many domains, making it accessible before the sign-up flow would allow tools to summarize and compare these documents before getting too deep.

