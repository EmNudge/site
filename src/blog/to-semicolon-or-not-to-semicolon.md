---
date: Aug 15, 2019
title: To Semicolon, Or Not To Semicolon;
summary: Wait, do we need them or not?
tags: javascript, coding
icon: technologies/languages/javascript
---

"To Semicolon, Or Not To Semicolon" - Shakespeare or something

The debate springs up and dies down every once in a while. I remember my first time learning that semicolons weren't required in JS. I was baffled. 

I had spent my entire JS career (figuratively - I hadn't gotten a JS job yet) up until that point believing that semi-colons were vital to JS code working correctly. I then had to ask "so why do we do it?"

Why did everyone I knew and every example I saw use semicolons? If they weren't necessary, why use them at all?

Let's first discuss ASI - the system that allows us to ignore them in the first place.

# Automatic Semicolon Insertion (ASI)
Automatic Semicolon Insertion, often abbreviated as ASI, is the process by which the parser inserts semicolons for us. It makes our use of semicolons "optional", but it does it *mainly* based on 3 rules (taken from [Dr. Axel Rauschmayer's Speaking JS](http://speakingjs.com/es5/ch07.html#automatic_semicolon_insertion)):
1. By a new line when the next line starts with an "illegal token"
2. By a closing brace
3. At the end of the file

&nbsp;#1 will be the more important one here.

It should be noted that it's not *literally* adding a semicolon. It just makes the engine function as if it did. ASI lets the engine know when a statement/line has "ended".

The following code would function properly since ASI adds in the semicolons we needed.

```js
let myVar = 3
myVar++
console.log(myVar) // > 4
```

ASI sees a newline after `3` and knows that `let myVar = 3 myVar++` is illegal, so it places a semicolon in between them. By the next newline, it knows that `myVar++ console.log(myVar)` is illegal as well and places a semicolon.

We see that ASI is not just checking for new lines, it's checking if there's a new line **and** the next token would be "illegal" if there was no semicolon between them. What if it *wasn't* illegal if there was no semicolon between them?

```js
let myVar = 1 +
3
console.log(myVar) // > 4
```

ASI sees there is a newline, so it checks if `let myVar = 1 + 3` is illegal. It isn't, so it doesn't add a semicolon. Therefore this code works perfectly.

You might have also seen some other code unaffected by ASI - method chaining.
```js
const newArr = oldArr
                 .map(mapFunc1)
                 .reduce(reduceFunc)
                 .split(' ')
                 .map(mapFunc2)
```
One might think that the newlines would make it so that ASI would add semicolons, thus break our chaining up, but it does not. 

ASI checks for illegal tokens and sees that it is completely valid if there was no semicolon. It thus leaves it alone. Some of you might be questioning whether [2, 4, 2]&emsp;&emsp;.map(x => x * 2) (with all those spaces) is actually valid. It is! We can keep those spaces and it'll function just fine.

Now that we have learned how it works, let's discuss cases where we *think* it would work one way, but it actually messes us up.

# ASI - Unexpected Lack of Insertion

There are cases where ASI won't add semicolons because the subsequent token isn't illegal.

```js
console.log(4) // > 4
['foo', 'bar'].forEach(el => console.log(el)) // > Uncaught TypeError: Cannot read property 'bar' of undefined
```

We might think that this code would log `4` and then log `'foo'` and `'bar'`, but instead we're getting an error. This is because ASI sees `console.log(4)['foo', 'bar']` as completely valid. 

ASI believes we're calling a function that returns an object and we're then trying to get this object's `bar` property. `'foo', 'bar'` converts to just `bar` due to the [Comma Operator](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comma_Operator) (cool, I know).

In this case, ASI really doesn't know that we didn't want that syntax. We'd have to add in a semicolon or not write array literals at the start of a new line in our code.

[The JS spec](https://www.ecma-international.org/ecma-262/10.0/index.html#sec-automatic-semicolon-insertion) also gives an example with parentheses that results in a similar effect. It wasn't valid JS (funny enough), so here is a more "practical" example.

```js
let s1 = "Hello "
let s2 = "World"
let myStr = s1 + s2
(s1 + s2).split('').forEach(char => console.log(char))
// Uncaught TypeError: s2 is not a function
```
We want `myStr` to equal `"Hello World"` and then to log every character of `"Hello World"` using a `forEach` loop. Instead, we get an error telling us that `s2` isn't a function.

Similar to the last example, ASI is checking if `s1(s1 + s2)` is valid syntax. Since it **is** valid syntax (as if we're calling a function called s1), no semicolon is added. 

ASI sometimes will **add in** semicolons which hurt us as well.

# ASI - Unexpected Insertion

Some programmers prefer [Allman brace style](https://en.wikipedia.org/wiki/Indentation_style) and carry over this preference to other languages. This means that every brace gets its own line. This works fine (but is generally avoided by JS programmers) until you encounter a case of ASI.

```js
// Allman brace style
function myFunc(val) 
{
  return
  {
    value: val
  }
}

console.log(myFunc(4)) // > undefined
```

While the function itself recognizes the braces as the function body, the object is not returned, because ASI places a semicolon after `return`. It should also be noted that there is no `object` in that function.

Yup. What else can this be aside from an object? A [block](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/block) with a [label](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/label).

We can make blocks in JS by surrounding some statements with `{ }`. We can also make labels in JS using `foo:bar` where `foo` represents any string (without quotes) and `bar` represents any expression or statement.

Labels are generally avoided and expression labels are almost wholly useless, but the engine doesn't care. We wrote useless code, but no one was hurt, so it lets it slide.

How does the engine differentiate between blocks and objects? If an opening brace is at the start of the line or after a statement requiring a block, it's a block. That's it. 

What that means, is even if we make the block look like an object, it is still treated like a block. 

```js
function myFunc(val) {
  return
  {
    value: val,
    name: 'Greg',
  }
}

// > Uncaught SyntaxError: Unexpected token :
```

labels cannot have a comma after them. This is most certainly therefore not a label. JS sees a block and then sees this weird quasi-label syntax. It throws an error, telling us our block has semicolons. 

**However** this was a fun aside, but it still doesn't fully explain the behavior of ASI in this instance due to the following behavior:
```js
function myFunc1(val) {
  return
  ({
    value: val,
    name: 'Greg',
  })
}
console.log(myFunc1(4)) // > undefined

function myFunc2(val) {
  return
  4
}
console.log(myFunc2()) // > undefined
```
We see that `return` is ignoring rule #1. Even though it'd be valid on the same line, a semicolon is added and we're returning `undefined`.

This is due to a special rule regarding something called "restricted productions" by the spec. You can read about it [here](https://www.ecma-international.org/ecma-262/10.0/index.html#prod-asi-rules-ReturnStatement), but it is essentially an exception to the rule. 

Other notable "restricted productions" include `continue`, `break`, and `throw`. If those are on their own line, regardless of what comes next, there is a semicolon added.

# To Semicolon?

After seeing how weird ASI can be, should we start adding semicolons?
Well the only situations where ASI doesn't add semicolons (and it harms us) seems to be where we use parentheses and brackets at the start of a line.

There are some other small offenders, but in general, you won't be writing expressions in your code on their own line. They're utterly useless and lend themselves to bad programming habits. You won't find a dangling `4` or `(13 + 4)` on its own line as it doesn't do anything and helps no one.

The practical times where they **do** matter, such as with IIFE's and array destructuring statements, are often caught with a linter.

If you ever need to start a line with parens or brackets, consider using a `;` right before it, as so many have adopted. Although, there is rarely a case where this would become a necessity.

The only case where ASI **really** seems to harm us is when it adds semicolons where it otherwise shouldn't, such as when "restricted productions" are followed up by an expression. 

At that point, it doesn't matter whether you regularly add semicolons or not - ASI is still creating problems. 

The solution most tend to go for is to use a linter which catches these subtle mistakes. Many linters will force semicolons by default, but you can disable this. Linters will also notify you of unreachable code. If you return early and have code after, the linter will let you know to remove or update it.

# Conclusion

If you're aware of the pitfalls and practice good programming habits, there doesn't seem to be a very strong case for using semicolons. It's at the point that [some style guides](https://github.com/standard/standard) even avoid them (VueJS doesn't include them by default as well).

Dan Abramov does make a [good case](https://github.com/standard/standard/issues/78) for keeping semicolons, but the pros seem to outweigh the cons in my experience. If you have ever had to enclose an expression in parens where the line ended with a semicolon, you may know what I mean.

You either click precisely right before the semicolon or delete it and then add it back. Without using semicolons, you could have just clicked somewhere at the end of the line and added a closing paren. It's small, but it adds to the amount of work in performing an extremely simple operation.

As for me? I surprisingly do use semicolons. Despite the occasional frustration, I like how it looks; it feels cleaner. I strongly believe it to still be a stylistic preference at the end of the day.

What do you think?
