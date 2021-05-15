---
date: Jul 27, 2019
title: "[] == ![] - WTFJS And Coercion"
summary: Explaining the frustration of JavaScript
tags: javascript, coding
icon: technologies/languages/javascript
---


WTFJS is a term first coined by [Brian Leroux](https://twitter.com/brianleroux), but I first heard it from [James Padolsey](https://twitter.com/padolsey)'s twitter. The title to this article is from a [github repo by the same name](https://github.com/denysdovhan/wtfjs).

Now, as a preface, I have written this article about 3 or 4 times by now. Each time longer than last. None published.

It's a difficult topic, but I'm going to attempt to write my shortest version, while still communicating the principles effectively.

# Coercion

The first and most important point on this topic is that coercion exists. We can convert a unit of one type to another accidentally or on purpose. These are usually called "implicitly" and "explicitly". 

You can convert a string to a number *explicitly* by using the function `Number()` or *implicitly* by subtracting it from another number. Here are those in code.

```js
7 + Number("4") // -> 11
7 - "4"         // -> 3
```

Most languages have explicit coercion. In these languages, we're usually doing things like converting an integer into a decimal. In JS there isn't a difference between the 2, so our concern is converting numbers into strings, strings into booleans, etc.

Implicit conversion can be a bit tricky. It's definitely useful, but it can lead to messy results. Here's a fairly famous example.
```js
7 + "4"   // -> "74"
7 + + "4" // -> 11
```
We might have forgotten that `+`, when one of the operators are a string, will coerce the non-string into a string and concatenate them.

We might have also made a typo, adding an extra `+`. We might have also forgotten that there is something called a unary `+` which actually does explicit coercion. `+"4" == 4` so our example was like doing `7 + 4`.

So implicit coercion is bad? Well not really. We use it all the time. Especially with booleans. Here's an example.

```js
const myArr = [1, 2, 3, 4, 5];
if (myArr && myArr.length) {
  console.log("My arr is: " + myArr);
}
```
this code outputs `my arr is: 1,2,3,4,5`. We used implicit coercion 3 times in this short example.

Our `if` statement first checks if `myArr` even exists. Then it checks if `myArr` has any elements inside of it. If it does, it prints out the stringified version of `myArr` in a console log.

This might seem intuitive to a lot of developers, but we're actually doing a lot of coercion here. We're coercing the array itself into a boolean, then the length property into a boolean, and then the array itself into a string!

It's important to know what coerces into what. I won't be going through everything here (like I did in previous versions), but I will touch on the basics.

# The Basics

Probably the most important and frequent coercion topic in JS is [falsy values](https://developer.mozilla.org/en-US/docs/Glossary/Falsy). Falsy values are the 6 things that, when coercing to a boolean, will evaluate to `false`. It should be noted that it does not mean they're **equal** to `false`, just that if we use `Boolean()` or implicit coercion, they will evaluate to `false` (this will be important for later).

These falsy values are `false`, `0`, empty strings (`''`), `undefined`, `null`, and `NaN`. If we run any of these into a `Boolean()` we will get `false`. 

**EDIT**: A friend of mine urged me to add in `-0` ([See here](https://dev.to/emnudge/identifying-negative-zero-2j1o)), `0n` ([BigInt](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt)), and `document.all`. That last one has a [super interesting story](https://stackoverflow.com/questions/10350142/why-is-document-all-falsy) and can be found in the spec [here](https://tc39.es/ecma262/#sec-IsHTMLDDA-internal-slot).

In our previous example, we were able to tell if our array existed and if it had any elements due to falsy values. If our array didn't exist, `myArr` would be `undefined`, which is falsy. If it had no elements, `myArr.length` would be `0`, which is falsy. If they're *not* falsy, they're truthy and the expression will evaluate to `true`.

Number coercion is probably the second most important. It's hardly as important, but it's still the second.

All you need to know there is that `null`, `false`, and empty arrays coerce to `0`, [strings in number format will turn into their corresponding number](https://dev.to/emnudge/converting-strings-to-numbers-in-js-subtleties-secrets-and-slip-ups-478k), `true` is 1, and everything else is `NaN`.


String coercion mostly does what you expect. Object coercion is cool, but irrelevant. Arrays will stringify themselves when coerced.

You can always test these by using their corresponding function. Want to know what `false` is in number form again? Open dev console and type `Number(false)`!

# Application
Now that we touched all the basics, let's apply them to really nail it in. Try not to read on too quickly. Take a second to think about what this will log to the console.

```js
const foo = "4a";
const bar = Number(Boolean(Number(foo)));
const baz = Number(String(Boolean(Number(foo))));

console.log(foo, bar, baz);
```

The answer is `4a 0 NaN`. 
`foo` is `"4a"` which at first looks like it'd be able to parse into a number, but we're not using `parseInt()` here. It would coerce into `NaN`. 

`NaN` is falsy, so when coercing into a boolean it'd be `false`. If we then coerce that into a number we get `0`, but if we make it a string in between, we get `NaN` since `"false"` cannot be coerced into a number.

Now that we got all that down, let's move on to some subtler parts.

# Equality Coercion and The Spec

I never thought I'd check the JS spec. Why would I? [MDN](https://developer.mozilla.org/en-US/) works fine. Well, there are some instances where the spec actually makes things clearer.

I'm going to show some really confusing results. Try to take a minute and figure out why they evaluate to what they do. This is actually going to be our first bit of WTFJS.

```js
false == ''        // -> true
false == null      // -> false
false == undefined // -> false
```

Let's think about it together now. Our first instinct might be to say "oh! these 3 are falsy! We're using loose equality so we're coercing!"

And you'd be on the right track, but still a bit off. Loose equality (2 equals signs) *does* force coercion, but not always how we'd like it to.

If we used 3 equals signs, there would be no coercion. Each one of these would be false. Loose equality first tries to coerce the values into the same type before checking equality.

The one problem here is that both our `null` and `undefined` expressions are evaluating to `false` when compared with `false`. Those 2 are falsy, so they *should* be equal to `false` when coerced, right? Let's take just the null example and try to break it down a bit.

```js
false == null                   // -> false      
Boolean(false) == Boolean(null) // -> true
false == Boolean(null)          // => true
```

Okay, so we've just verified that `null` is in fact falsy. So we're not crazy. What must be happening here is that we're not actually coercing them into booleans. Let's try other types.

```js
Number(false) // -> 0
Number(null)  // -> 0
Number(false) == Number(null) // -> true
String(false) == String(null) // -> false
```

Okay, so it can't be Number, but it also can't be String since we found out that `false == ''` from before is `true`. I'm missing something. Time to check the [spec](https://www.ecma-international.org/ecma-262/10.0/index.html)! 

We're going to need to read through the section labeled [**Abstract Equality Comparison**](https://www.ecma-international.org/ecma-262/10.0/index.html#sec-abstract-equality-comparison). By "abstract" they mean what we mean when we say "loose".


> 1. If Type(x) is the same as Type(y), then

Well our types are different, so let's skip down to a number which described our initial state.

> 6. If Type(x) is Boolean, return the result of the comparison ! ToNumber(x) == y.

Note: The `!` does not mean the same thing as it does in JS. It refers to operations that don't result in an [abrupt completion](https://www.ecma-international.org/ecma-262/10.0/index.html#sec-completion-record-specification-type) and is irrelevant to us for now. We can ignore this in the spec when we see it in this article.

So when our first value is a boolean, regardless of what the second value is, convert **only the first value to a number** using some built-in function called `ToNumber()` and perform the algorithm again.

We can substitute `ToNumber()` for good ole `Number()` here. This gives us `Number(false) == null`, not `Number(false) == Number(null)`. This evaluates to `0 == null`. Let's read on in the spec for what to do now.

Well the only option that discusses what to do if we have a Number is the one that reads

> 4. If Type(x) is Number and Type(y) is String, return the result of the comparison x == ! ToNumber(y).

We're not dealing with a String, we're dealing with `null`. None of the options address this. Let's go down to the last option

> 10. return false

Huh. How about that. If a number is being compared with anything that isn't a string (and isn't an object that can be stringified), it evaluates to `false`. We're just getting `false` by default after the first step.

Now let's address the other 2.
Well with `false == ''` we would first coerce `false` using `ToNumber`/`Number`, so we're doing `0 == ''`, like before. This time, however, option #4 actually *does* apply to us. We have a string! Now it's basically `0 == Number('')` which is `0 == 0` and that's `true`!

Wow, that cleared up a lot of confusion. Let's move on to the last one.
We "numberify" the boolean, so `false == undefined` becomes `0 == undefined` and then since `undefined` is not a string, it's `false` by default!

Whew. The spec cleared things up, but that was still a bit weird. Welcome to WTFJS!

# WTFJS
WTFJS is essentially just the tricky parts of JS that make you say "WTF JS?!"

It many times has something to do with coercion or some other weird part of JS. Regardless, looking over WTF JS examples can really give you a much better understanding of JS. Let's tackle the problem in the title.

```js
[] == ![] // -> true
```

Alright. Something equals the opposite of itself? That can't be right. We know `[] === []` would evaluate to `false` since objects/arrays are compared by reference, but `true` is an odd result. Let's break this down.

So, first off, when we see `==` (Abstract/Loose equality), we know we're going to be dealing with coercion, at least in WTFJS examples. 

Before we delve into coercion however, we must first evaluate each side. We're using the boolean NOT operator (`!`) which, [according to the spec](https://www.ecma-international.org/ecma-262/10.0/index.html#sec-logical-not-operator), performs coercion on non-booleans before flipping the boolean to the opposite.

Now we have `[] == !Boolean([])`. Since `[]` isn't falsy, it'll evaluate to `true`. the `!` will make it `false`, so we then have `[] == false`.
As we saw before, if one of the operands is a boolean, we first convert it to a number before moving on. That'd be `[] == Number(false)` which would then be `[] == 0`.

Okay, so since `[]` is not a string, wouldn't this evaluate to false? We got `true`! That can't be it.

Reading on in the [**Abstract Equality Comparison**](https://www.ecma-international.org/ecma-262/10.0/index.html#sec-abstract-equality-comparison) section in the spec, we see that we glossed over option #9.

> 9. If Type(x) is Object and Type(y) is either String, Number, or Symbol, return the result of the comparison ToPrimitive(x) == y.

This rule says that we convert an object to a primitive value when dealing with strings, numbers, or [symbols](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol) on the opposite side (not booleans).

"But `[]` is an array, not an object!" I hear you say. Not so fast. `typeof []` actually returns `"object"`. Arrays are of type `"object"` and thus follow any object-based rule.

Well it says to use `ToPrimitive`. Which function to we use to get that? Is it a number, string, or boolean? Back to the spec!

Let's look at the section labeled [**ToPrimitive**](https://www.ecma-international.org/ecma-262/10.0/index.html#sec-toprimitive). It tells us to call a function called [OrdinaryToPrimitive](https://www.ecma-international.org/ecma-262/10.0/index.html#sec-ordinarytoprimitive) which requires a `hint`. This tells us which primitive to convert it into. If none is specified, like in our case, it uses `"number"`.

When this function is called, if `hint` isn't "string" (ours is "number") it tries to call `.valueOf()`. If that doesn't return a primitive (it doesn't), it calls `.toString()`. If that doesn't return a primitive (it does), it throws an error.

In our case, `[].valueOf()` returns `[]`, which is not a primitive. `[].toString()` returns `""`, which *is* a primitive.

Wow. Okay, so now we have `"" == 0`. From before, we know that if we compare a number and a string, we coerce the string into a number. It follows a bunch of rules, but as we saw from a section before, if the string is empty we get `0`.

Now. `0 == 0`. That's true. That's why `[] == ![]` is `true`.

# Conclusion

What did we learn here? I hope a whole bunch.

JS is confusing and sometimes it feels unintuitive, but all of its features (not bugs) can be seen in the spec. Coercion is also a messy topic. A lot of coercion seems so odd, we throw it in [WTFJS](https://github.com/denysdovhan/wtfjs#-is-equal-). 

I urge you to, before you give the spec a look, go to the WTFJS repo and have a look. [Their own reasoning](https://github.com/denysdovhan/wtfjs#-is-equal-) behind their examples may skip a *bunch* of steps, but it will help you get a better understanding regardless.

I hope I encouraged a few devs to give the spec a try. It's really a lot more readable than it looks.