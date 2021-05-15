---
date: Aug 8, 2020
title: How Do For Loops REALLY Work
summary: I mean REALLY. Think about it.
tags: javascript, coding
icon: technologies/languages/javascript
---

> How does a for loop work?

It seems like a basic question at first. It's an iterator. It iterates. Simple as that. 
What if we confined the question to specifically talking about the `for([initialization]; [condition]; [final-expression])` for loop? How does it **really** work? 

"What do you mean?" you might say. "You initialize a variable, state a condition which evaluates to a boolean, and provide a statement to perform after every iteration. It's simple" you say. 

Well then let's provide a snippet.

# The Problem
You may have seen this question as a part of interview prep or cool tricky JS code snippets. 
```js
for (var i = 0; i < 10; i++) {
    setTimeout(() => console.log(i), 0);
}
```
*Note*: You might also see a variation of this with functions being added to an array. The result and reason behind it is virtually the same. We just require a function to trigger after the loop ends which contains a reference to `i`.

This for loop, despite what we might initially think, outputs `10` 10 times. We expect it to print out the numbers 0-9 inclusive, but it doesn't. We can fix this by using `let` instead of `var`.

As to "why", the explanation usually involves the use of `var` hoisting. However, people many times forget to mention the closure aspect. Even then, people forget that `var` is the **easier** part to explain and that for loops are doing something behind the scenes not even explained on [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for).

What do I mean? Well first, let's provide the usual explanation. Let's pretend this is a basic interview prep question and explain it as we would in an interview.

Let's first go over `var` vs `let`.

# Var vs Let

`var`, unlike its post-ES5 counterpart `let`, is function scoped and [semi-hoisted](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/var#var_hoisting). What does this mean exactly?

I call `var` variables *semi* hoisted because, unlike [function declarations](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function), only their declaration is hoisted. This means that the following 2 snippets are identical:

```js
var myVal = 233;

for (var i = 0; i < 5; i++) {
  var newVal = myVal + i;
  console.log(newVal)
}

function showNumPlusOne(num) {
  var plusOne = num + 1;
  alert(plusOne);
}
```
and 
```js
var myVal, i, newVal;
function showNumPlusOne(num) {
  var plusOne;
  plusOne = num + 1;
  alert(plusOne);
}

myVal = 233;

for (i = 0; i < 5; i++) {
  newVal = myVal + i;
  console.log(newVal)
}
```

The declaration of the variables we define with `var` are "hoisted" up to the top. The "top" will be the top of our global code or the top of the function (if it's declared in one). Also note how the declaration breaks out of the for loop, but not out of the function. As mentioned before, function declarations are "fully" hoisted since their body goes up too, although variables declared with `var` only have their declarations go up.

What this means, is that we can access variables declared with `var` before we declare them. Since they're just declared and not given a value, their value will be `undefined`, but we can still use them. `let` will prevent this and throw an error. 

`let` is also block-scoped, so variables declared with `let` cannot be accessed outside of a for loop if they're declared in one. 

To illustrate, here is `let` vs `var` block-scoping:

```js
{
  var blockVar = 4;
}

console.log(blockVar) // > 4

for (var i = 0; i < 10; i++) {
  // do stuff
}

console.log(i); // > 10
```
vs 

```js
{
  let blockVar = 4;
}

console.log(blockVar) // > Uncaught ReferenceError: blockVar is not defined

for (let i = 0; i < 10; i++) {
  // do stuff
}

console.log(i); // > Uncaught ReferenceError: i is not defined
```

We get an error when trying to access the variables when declared with `let`, but with `var` it works just fine. Now that we have cleared that up, let's show the example again.

# The Explanation - Part 1

This is the point where many explanations online stop. `let` is different than `var` and since `var` isn't block scoped; it jumps out of the for loop. If we were to use `let`, we'd be fine, but `var` makes `i` equal to what it last was (10) and that gets outputted each time.

```js
// i's declaration gets hoisted
var i;
for (i = 0; i < 10; i++) {
    setTimeout(() => console.log(i), 0);
}
// loop is over and setTimeout functions trigger
```

However, this answer is unsatisfactory. We might understand that `var` and `let` are fundamentally different, but it still doesn't explain why the number that we give it changes. Even though `var` makes `i` eventually equal `10` and `setTimeout` makes our function be called after the for loop is finished, this explanation is still missing something.

At face value, this explanation may lead us to believe that `let` should make our output be an error each time. If in both cases our functions are running after the for loop, we might think we would output whatever `i` is equal to then. In our case it would either be `10` or nothing (an error). 

There's more. There has to be.

# Closures

The more astute among you might have guessed it. A [closure](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Closures) describes the situation where a function grabs variables that it didn't declare or receive through arguments. 

The concept of closures is tied to "[Lexical Environment](https://javascript.info/closure)" and how `this` is calculated. Jeremy Liberman explains how `this` is calculated pretty nicely in his article [How does "this" work in JavaScript?](https://jeremyliberman.com/2019/04/01/how-does-this-work-in-javascript.html). 

The concept can get complicated for many, so I'm going to try to skim over some of the more complicated and abstract bits. I will instead explain what we need to understand for our use case.

```js
let myVar = 4;

function showNumPlusOne() {
  console.log(myVar + 1);
}

showNumPlusOne(); // > 5

myVar = 8;

showNumPlusOne(); // > 9
```

That is an example of a closure. Our function `showNumPlusOne()` is grabbing the outside variable `myVar` and logging its value plus one.

The important thing to note about closures is that they don't just grab the value of the variable, they grab a reference to the variable itself. When we change the value in the above snippet, the function's output changes as well.

This can make for some very interesting code. Have a look at this snippet:
```js
let getVar;
let myVar = 4;
{
  let myVar = 8;
  getVar = () => {
    return myVar;
  }
}
console.log(getVar()); // > 8
console.log(myVar);    // > 4
```
*Note*: [function declaration hoisting in blocks](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function#Conditionally_created_functions) is super inconsistent among browsers, which is why I am using a function expression

We declared the variable `myVar` twice, but it's okay because one of them is in another scope. The function is using the concept of closures to grab the **nearest variable** with the name `myVar`.

It's still grabbing the actual variable and not just its value, but it's using a different `myVar` than the one we're using. This plays closely with the concept of `this` and Lexical Environments which we won't get into here.

# The Explanation - Part 2

So now some of the better explanations will include the concept of closures. Let's explain our example with both `var` and closures now.

```js
var i;
for (i = 0; i < 10; i++) {
  // closure referencing the i variable outside of the for loop
  setTimeout(() => console.log(i), 0);
}
```

So since our `var` is outside of the scope and our closure is referencing the literal variable `i`, we're logging the literal variable that we're getting after the loop is over.

`i`, when declared with `var`, is outside of the for loop. If we get its value after the for loop, it will be `10`. It's not about *when* or *where* we run the function. Since it has a reference to the variable `i`, wherever it was declared, it will output `i`.

Let's illustrate this with another snippet

```js
var i = 0;
function myFunc() {
  console.log(i);
}

(function(){
  var i = 4;
  myFunc(); // > 0
})()
```

We're using an IIFE to create a scope for `var` since it isn't block scoped. Even though we defined `i` right before calling the function and gave it the value `4`, the function still logged 0. The reason being that it wasn't referencing the `i` that we declared. It was referencing a completely different `i` - the one at the top of our snippet.

With this concept in mind, we see that the `i` the function inside the `setTimeout` is referencing is a different `i` each iteration since `let` is block scoped. However, since `var` is only function scoped and gets hoisted outside of our loop, it's the same variable that the functions are referencing each time.

Unfortunately, this answer still isn't completely satisfactory. Some of you might be looking at the article in disbelief as we seem to have gotten complicated enough. 

I'd like you to recall what I said at the beginning of this article.

> Even then, people forget that `var` is the **easier** part to explain ...

Yup. It's now clear(ish) why we get the answer we get for `var`, but why don't we get the same answer for `let`? Think about it.

The `i` that we're referencing should still be the same `i` that we started with. The only way that `i` would be different is if we used `let` on every single iteration. Even then, how would we mutate it with what we provided to the for loop parens (i++)? We can't do `let i = i++;` in JS. And then even if this somehow works and we're not mutating, we should be able to use `const` in our for loops!

What's going on?

To answer this, we need to try and simulate a for loop.

# The Making of a For Loop

Our first instinct would probably be to use a `while` loop. Let's do that.

```js
let i = 0;
while (i < 10) {
  setTimeout(() => console.log(i), 0)
  i++;
}
```

This would work like a for loop, but not in this specific context. We're still going to get `10` 10 times even though we're using `let`. Let's put it in a block to stop that.

```js
{
  let i = 0;
  while (i < 10) {
    setTimeout(() => console.log(i), 0)
    i++;
  }
}
```

So now we can't access it ourselves, but the function is still referencing a common variable `i`, so we get the same buggy result as we would with `var`.
There must be something else here. Let's try creating a variable local to the most inner scope. 

```js
{
  let i = 0;
  while (i < 10) {
    // copying i's value into a new variable _i
    let _i = i;
    // using _i wherever we would normally use i
    setTimeout(() => console.log(_i), 0)
    i++;
  }
}
```

Wow...It worked! We're actually getting the numbers 0-9! If we replace `let` with `var` or `const` we also get the same result as if it were a for loop!

It turns out, this is very similar to what happens on the engine side with for loops.
If we take a look at the [spec](https://www.ecma-international.org/ecma-262/10.0/index.html) over at [13.7.4.9 Runtime Semantics: CreatePerIterationEnvironment](https://www.ecma-international.org/ecma-262/10.0/index.html#sec-createperiterationenvironment), we'll see some very confusing language:

<blockquote>
<pre>
g. For each element bn of perIterationBindings, do
&emsp;i. Perform ! thisIterationEnvRec.CreateMutableBinding(bn, false).
&emsp;ii. Let lastValue be ? lastIterationEnvRec.GetBindingValue(bn, true).
&emsp;iii. Perform thisIterationEnvRec.InitializeBinding(bn, lastValue).
</pre>
</blockquote>

This essentially means what our for-simulator did. Our variable is bound to the context each iteration.

Now to fully explain it.

# The Explanation - Part 3

Let's go over everything one more time.

We are unexpectedly getting `10` 10 times, instead of 0-9 inclusive. 

1. `var` will make it so that `i` is outside of the for loop due to `var`'s scoping and hoisting behavior. 
2. Since we are using a function, we are involving closures.
3. Closures reference the literal variable, which differs if we use `var` or `let`.
4. Since we create a new binding context on each iteration, if we use `let`, `i` is always a new variable and equal to the current value of `i` in the iteration.
5. Using `var`, we don't actually have a `var` statement in the for loop due to how `var` isn't block scoped. Thus `var` isn't bound to the context on each iteration.
6. Therefore, `var` will make the closures reference the same variable each iteration, while `let` will force the closures to reference a completely separate variable each time.

There. That's your explanation. That's how for loops work.

Oh? I didn't mention for-in, for-of, and for-await-of? 

Oops.