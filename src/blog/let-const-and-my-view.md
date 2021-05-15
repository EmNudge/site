---
date: Dec 26, 2019
title: let, const, And My View
summary: JavaScript and its own share of drama
tags: javascript, coding
icon: technologies/languages/javascript
---

"Well this isn't going to be a controversial post" he said, wrongly.

For those unaware, the title is in reference to a little discussion that spanned across web dev twitter and reddit on the 21st of December.

![Alt Text](https://thepracticaldev.s3.amazonaws.com/i/x8mdpvfv5hzaxmt5tmez.png)

This tweet was in reference to [a reddit post of his](https://www.reddit.com/r/reactjs/comments/edj1dr/what_is_javascript_made_of/), which linked to an article he wrote from his fantastic blog, https://overreacted.io/

Many comments on the post critiqued a part of his article which mentioned, in reference to variable bindings, favoring `let` and only using `const` when needed. As people usually act in the opposite way, favoring `const` and only `let` when a mutation is needed, a lot of the comments on the post where in regards to that snippet.

![Alt Text](https://thepracticaldev.s3.amazonaws.com/i/l0g74i6j97sgjwspe7cr.jpg)

Dan defended his views on twitter and had some people from TC39 backing him up - either presently or with links in the replies.

A post which seemed to echo his views by a TC39 member is one by Jamie Builds: https://jamie.build/const

The post uses quite a bit of foul language, for anyone who is sensitive to that, but it's a fun read.

My take? I'm a `const` advocate. I hadn't left any reply on twitter since I tend not to use any social media (at least for social media purposes. I just post jokes on twitter).

With that said, I wasn't going to make a post, but this has been banging around in my head for the past few days and it has gotten to the point where I have to write *something* on the matter. There's a key point that I feel a lot of people are missing and I'd like to provide it.

My posts usually tackle some of the trickier bits on JS, so this is a bit out of my field, but still something I'm going to try and cover.

# `const` vs `let`
Right off the bat, what do they do? Why were they provided in ES6? Didn't we have `var`?

`var` is almost entirely out of use due to an aspect that many find to look much more like a bug. `var` allows for semi-hoisting. I elaborate a bit on this in [my article about for loops](https://dev.to/emnudge/how-for-loops-really-work-4lhj), but essentially, the variable declaration is hoisted to the top of the current function/file scope.

Additionally, `var` is not constrained to block scopes, allowing it to hoist itself out of `for` loops. These 2 annoying aspects brought us the 2 alternatives - `let` and `const`. `let` can be seen as `var`, but without those 2 "bugs", and `const` can be seen as `let`, but with constant variable bindings.

The "constant variable bindings" and not "creates constant values" distinction is important. If you create a variable with `const` that is assigned to some non-primitive data structure, the structure can be mutated, but the variable cannot be re-assigned. This distinction can be conveyed with a single sentence and it would be much easier to teach if people stopped teaching `const` as something to create immutable data structures, but rather immutable bindings.

As a quick example:
```js
const myObj = {};       // {} is a non-primitive
myObj.name = 'EmNudge'; // You can still mutate the object
myObj = 3               // ERROR - you can't reassign it.
```

And that's all there is to it. It's good practice to not use `var` unless you are [Kyle Simpson](https://github.com/getify/You-Dont-Know-JS/blob/2nd-ed/get-started/ch2.md) (all jokes, he does make some nice observations on this very topic, though) and just use `const` when you need to, right?

Not exactly. Coming from `var`, the first instinct is to use `let`, but many javascript developers will advise you differently. First let's discuss *why* people like Dan Abramov and others are pro-let preference.

# prefer-`let`
Dan Abramov actually [wrote an article](https://overreacted.io/on-let-vs-const/) shortly after this debacle unfolded on his justifications. He also compared both sides, but again missed the point I see missing from most posts.

Let's go through his points. His views seem to be inline with most people who share his perspective, so I won't be including anything from other sources.

### Loss of intent
Dan claims that prefer-`const` means we won't be able to know if something has some importance in being constant. Perhaps we have some value for the amount of seconds in a day? This is a value that we'd want to notify developers about being constant.

### Confusion 
Dan also claims that he sees many beginners confuse `const` with immutability. As mentioned before, `const` only serves to make a constant **binding**, not data structure, but that is not immediately obvious to a beginner. 

### Pressure to avoid redeclaring
This is actually an interesting one and a reason why I am an advocate for [do-expressions](https://github.com/tc39/proposal-do-expressions) or at least using an IIFE in place occasionally. This can also be solved by separating some things into a function.

Essentially, what Dan is saying is that people will often use the ternary operator instead of a mutable value and using an `if` statement to redefine it.

### `const` ain't all it's cracked up to be
The next 2 reasons are essentially that `const` doesn't provide any real benefit. It doesn't prevent bugs and there isn't any performance benefit. Any benefit is already made by the engine, even with `let`. 

So what's my defense?
# prefer-`const`
I'm going to tackle these a bit out of order, so bear with me.

### Performance benefits
Yes, there is no engine benefit that I am directly aware of. That's not why we advocate for `const`, but I'll toss it up as a win for the `let` people for now. The people who write browser engines happen to be very good at what they do. It leads to a lot of specific ways to write JS code, like ["no js-specific micro optimizations"](https://tomdale.net/2017/07/adventures-in-microbenchmarking/).

### Confusion
I think this is a bit of an annoying point. JS has **many** weird parts to it and I would love if none of them existed, but we can't ignore them. Using `let` in most cases will only prevent people from seeing `const` right *now* and this aspect is pointless while plenty of codebases *already* do prefer-`const`. 

`const` needs to be taught as constant **bindings** and not constant **values**. It's unfortunate that it's not immutable values, but ["tant pis"](https://www.google.com/search?q=tant+pis). 

If someone is used to `var` and declaration hoisting, `let` may be confusing to them. Closures are confusing. Implicit coercion is confusing. 
Yes, the less confusing aspects introduced the better, but this is not one that we should try to avoid for reasons explained a bit more later.

### No benefit
I'm going to take an interesting stance and not argue wholly with this. `const` is useful to me when used as a **default**. If you use `const` occasionally, you should only really use it when you need to. It's when you use it **always** that the real benefit comes in.

This will play a lot into the next and final point, which will need its own section.

# Intention Has Been Moved

Intent is **not** lost, just moved. A big issue I have is with people assuming that developers are using `const` to declare intent. In reality, people are using `let` to declare intent. 

Let's backtrack a bit.

### Readability 

One of the favored aspects of a particular paradigm of programming is how easy it is to read someone else's code, as that's what you do most as a developer in many jobs. 

If the code is shorter, that's usually easier to read. If the code uses good variable names, that's usually easier to read. If it is declarative rather than specifically imperative, that's usually easier to read.

One of the biggest time-savers in skimming code is reading intent. If I see someone's code using declarative function names, I know at least what their code is **trying** to do. If it isn't doing what it says it's doing, I additionally know it's a bug.

### Intent through code

This extends to other parts as well - not just function names. Comments show intent. The type of `HOF` you might use can show intent. Using backticks vs quotes for strings shows intent. A lot of things show intent and allow me to skim your code and better understand what's going on.

In regards to variable declarations, [`rust`](https://www.rust-lang.org/) has immutability be *default*. I much prefer this way because mutation is actually much more rare than variables made to clear up code.

In rust, variables are declared with `let`. If they're to be mutated late on, they're declared with `let mut`. When I see `mut`, I am expecting to see this variable change. I like this. `mut` shows me intent to mutate. `const` has a separate meaning, but the point is that variable immutability isn't something we need to bother showing intent behind. It's the natural way of things.

When I changed to prefer-`const`, I was first fairly reluctant. I had no idea how many of my variables were truly never changing. These days, my code contains so few `let`s, you might not notice any outside of a `for` loop unless you're **really** searching for them.

In reality, it is `let` that shows intent, not the other way around (as every let-advocate seems to be insinuating or declaring outright). If I see `let`, a fairly unusual keyword, I know to expect something to change soon. 

### You can still use `let`

If someone does *not* want to use ternaries for a new variable, they can very well use `let` and I will know to expect a mutation soon. If you use prefer-`let`, I will not know to expect a change with an `if` statement. You can also take this opportunity to use an intermediate function and still use `const` without involving ternaries.

prefer-`const` does not mean leaving `let`. You can still use `let`, but only when you find it advantageous for the scenario.

Want to *specify* something as const? Use screaming case. `THIS_VARIABLE_NAME` *already* shows me something is specifically constant. I don't need *another* keyword to indicate this. Using prefer-`const` does not remove `const` intent and instead provides us with a *new* `let` intent, which turns out to be *much* more useful.

# Conclusion

As Dan Abramov says, it's usually preferable to stick with the conventions of a specific project rather than pioneer your own. Consistent code is cleaner code. 

Happy holidays!