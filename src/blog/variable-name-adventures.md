---
date: Mar 16, 2020
title: Adventures in Horrible No Good Very Bad Variable Names
summary: Messing around with variable names and pranks!
tags: javascript, coding
icon: technologies/languages/javascript
---


Recently I got a DM on [Discord](https://discordapp.com/). This person did not have much knowledge of JavaScript, but they had seen this rather interesting [snippet of JS](https://jsfiddle.net/scp235o0/1/) which affected tweets on Twitter (now deleted). It changes a couple of very specific tweets, revealing text that wasn't previously there.

They had run this JavaScript snippet in their dev console and wanted me to explain how it worked. For future reference, if you do not fully understand a JavaScript snippet, please do not do this. They can be malicious. 

![](https://i.imgur.com/ZSjcv7x.png)

How did this work? The tweets contained text that wasn't viewable by most font-sets. Many times these icons will show up as missing symbol boxes (▯ or □). These characters simply do not show at all.

The JS snippet got the UTF code point for each character using [`String.prototype.codePointAt()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/codePointAt) and then converted it into an english character using [`String.fromCodePoint()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCodePoint).

These special characters are ones returned from `String.fromCodePoint()` when passed the first 4096 numbers starting from `0xe0000` (917504). You can "see" all of them by running the following:

```javascript
for (let i = 0; i < 4096; i++) {
    const char = String.fromCodePoint(i + 0xe0000);
    console.log(char);
}
```

Most dev tools will combine console logs into one output if they contain the same text. As these are distinct symbols, they will appear as 4096 separate logs. 

As they are distinct symbols, they do indeed contain length. In fact, we could probably artifically increase this article's "reading length" by filling it with these symbols. In between these two arrows are 100 characters. You can copy/paste it into dev tools and check its length to confirm.

`→󠀀󠀁󠀂󠀃󠀄󠀅󠀆󠀇󠀈󠀉󠀊󠀋󠀌󠀍󠀎󠀏󠀐󠀑󠀒󠀓󠀔󠀕󠀖󠀗󠀘󠀙󠀚󠀛󠀜󠀝󠀞󠀟󠀠󠀡󠀢󠀣󠀤󠀥󠀦󠀧󠀨󠀩󠀪󠀫󠀬󠀭󠀮󠀯󠀰󠀱󠀲󠀳󠀴󠀵󠀶󠀷󠀸󠀹󠀺󠀻󠀼󠀽󠀾󠀿󠁀󠁁󠁂󠁃󠁄󠁅󠁆󠁇󠁈󠁉󠁊󠁋󠁌󠁍󠁎󠁏󠁐󠁑󠁒󠁓󠁔󠁕󠁖󠁗󠁘󠁙󠁚󠁛󠁜󠁝󠁞󠁟󠁠󠁡󠁢󠁣←`

Note that using `String.prototype.length` will actually print a length of `202` instead of the expected `102` (almost double) because every character after `0xFFFF` (65,535) (called the BMP or [Basic Multilingual Plane](https://en.wikipedia.org/wiki/Plane_(Unicode))) exceeds the limit of JavaScript's UTF-16 strings. The arrows (in order to be displayed on smaller font sets) has a code point of `0x2192` (8,594). To **actually** retrieve the number of characters in a string, use a `for...of` loop and take advantage of JS iterables!

```javascript
function getNumChars(str) {
    let length = 0;
    for (const char of str) length++;
    return length;
}
```
or, since the spread operator also works on iterables, a bit of a simpler method:
```javascript
[...str].length
```

In general, the intricacies of all of this is a bit more than what I'd like to get into. [Mathias Bynens](https://mathiasbynens.be/notes/javascript-unicode) has a fantastic article on all this, which I highly advise you read for more information.

You can quickly view a character's code point via the following function:

```javascript
function getCharHexCode(char) {
    const hex = char.codePointAt(0).toString(16).toUpperCase();
    return `0x${hex}`;
}
```

"Astral code points" (ones after `0xFFFF`, such as `🡆`) also contain a second index. It will return a code point that is relational to the actual code point defined by the following expression:

```javascript
codePoint + 1024 * Math.floor((codePoint - 0xFFFF) / 1024) + 9216
```

or the following function

```javascript
function getAssociatedCodePoint(codePoint) {
    if (codePoint <= 0xFFFF) return null;
    
    // This is just here. No clue.
    const BASE_DIFF = 9216;
    
    // how many sets of 1024 numbers this is above 0xFFFF
    const index = Math.floor((codePoint - 0xFFFF) / 1024);
    // for every 1024 numbers, the difference jumps 1024
    const levelNum = 1024 * index;
    
    return BASE_DIFF + levelNum + codePoint;
}
```

I honestly do not know why this is so. Drop a comment if you have an explanation.

*6/12/2020 EDIT*: It turns out it's just the right surrogate pair.
One would get the same result doing `'🡆'.codePointAt(1)` as one would doing `'🡆'[1].codePointAt(0)`. `codePointAt` does not remove the right surrogate pair when retrieving the codepoint, just the left one.
Read more about this stuff at: https://dmitripavlutin.com/what-every-javascript-developer-should-know-about-unicode/#24-surrogate-pairs 

While all this might be interesting to some, that wasn't why I wrote this article. I wanted to investigate variable names (hence the title). Could these special characters be used as variable names?

## Variable Names And You

Most people stick to standard conventions when making variable names in JS. 

1. Use English characters (no umlauts or diacritics).
2. Start with `$` for jQuery or `querySelector`-based libraries.
3. Start with `_` for lodash/underscore or unused variable names.

Although these aren't physical limitations, people tend to stick to them. If one developer used diacritics, it would be difficult for developers without specific keyboard layouts to replicate them.

What *I'm* interested in is what are we *physically* bound by. Could we use a number literal as a variable name, for instance? No. We are physically bound from doing that.

```javascript
const 32 = 24;
// Uncaught SyntaxError: Unexpected number
const .32 = 24;
// Uncaught SyntaxError: Unexpected number
```

Some other things we can't use: 
- reserved keywords
    - `if`, `while`, `let`, `const`, etc
- immutable global object properties in the global scope
    - `NaN`, `Infinity`, and `undefined` 
- variable names starting with unicode outside of the [Unicode derived core property `ID_Start`](https://codepoints.net/search?IDS=1) (excluding `$` and `_`).

*Thanks again to [Mathias Bynens](https://mathiasbynens.be/notes/javascript-identifiers-es6) for this info*

Mathias also provided an [online JavaScript variable name validator](https://mothereff.in/js-variables) for if you would like to test things out yourself.

One thing to note is that there is a difference in valid variable names for ES5, ES5-era engines, and ES6. We are using ES6.
Mathias (yet again) has [an article](https://mathiasbynens.be/notes/javascript-identifiers-es6) for this.

What I was interested in was the odd stuff. A theoretical prank.

## The Theoretical Prank

Every now and again this "meme" floats around where it advises pranking a coworker by replacing their semicolons with greek question marks (`;` or `0x037E`).

These days, we have pretty good linters (in most languages) which will catch these. This prank can be found out very quickly. Let's try spicing things up a bit.

What information from our knowledge of valid variable names can we use for our prank?

Well firstly, [Zalgo text](https://zalgo.org/) is fine. Zalgo text is the result of combining a bunch of diacritics to extend text outside of its vertical container. It tends to look like ṱ̶͇̭̖̩̯͚̋͛͗̋h̶̳̳̫͕̄͆̈̍̓̀̉ͅi̷̥̩̼̒̏s̷̰̣̽̇̀͆̀͠ and it's both valid unicode and a valid identifier. 

```javascript
const ṱ̶͇̭̖̩̯͚̋͛͗̋h̶̳̳̫͕̄͆̈̍̓̀̉ͅi̷̥̩̼̒̏s̷̰̣̽̇̀͆̀͠ = 32;

console.log(ṱ̶͇̭̖̩̯͚̋͛͗̋h̶̳̳̫͕̄͆̈̍̓̀̉ͅi̷̥̩̼̒̏s̷̰̣̽̇̀͆̀͠); // > 32
```

Since diacritics are valid in variable names, there's nothing really stopping us from combining them ad infinitum. This isn't very pleasant to look at, but it's still not what I had in mind for a prank.

We previously discussed invisible characters. What if we could create invisible variable names? Are these valid?

```javascript
const 󠀀= 42;
// Uncaught SyntaxError: Invalid or unexpected token
```

It doesn't seem so. And in case you were wondering, there is indeed a character there between `const` and `=`. If there wasn't, we would get a separate error.

```javascript
const = 42;
// Uncaught SyntaxError: Unexpected token '='
```

We could use the aforementioned tool to check valid variable names, but we'd be entering characters one by one. I need a way to automate this. I can copy Mathias's code, using a ton of regex and all that, or....

```javascript
function isValidVariableName(str) {
    try {
        eval(`const ${str} = 42;`);
    } catch {
        return false;
    }
    return true;
}
```

[-"eval is evil"](http://linterrors.com/js/eval-is-evil) but we can make an exception for personal testing. Note that I'm specifically not using `let` since passing a space to `isValidVariableName` will return a false-positive if `let` were used. After all, the following is valid:

```javascript
let = 42;
console.log(let + 8); // 50
```
As `let` along with 8 other words were not considered reserved keywords outside of strict mode.

With that in mind, let's get into a bit of width testing.

## Width Testing

I want to find valid variable names with thin, weird characters.  The easiest way to do this is via your eyes. Looking at characters is a pretty good way to tell how they look. Unfortunately, this is time-consuming. Especially for possibly over 1 million characters.

Let's set up some test code

```javascript
function getSmallestChars() {
    const chars = [];
    const min = { width: Infinity, height: Infinity };

    for (let i = 0; i < 0xFFFF; i++) {
        const char = String.fromCodePoint(i);

        if (!isValidVariableName(char)) continue;
        
        const { width, height  } = getStrSize(char);
        
        if (width > min.width || height > min.height) continue;

        if (width < min.width && height < min.height) {
            min.width = width;
            min.height = height;
            chars.length = 0;
        }
        
        chars.push(char);
    }
    
    return chars;
}
```

The upper bound of `i` is just small for the initial test. The important question is how do we find out how much space a character takes up? The question is font-specific and the DOM generally will not give the specific character size, but rather the space the parent `TextNode` takes up.

For this, we need to use `Canvas`.

```javascript
const cnvs = document.createElement('canvas');
const ctx = cnvs.getContext("2d");

function getStrSize(str) {
    const textMetric = ctx.measureText(str);
    const width = textMetric.actualBoundingBoxRight - textMetric.actualBoundingBoxLeft;
    const height = textMetric.actualBoundingBoxAscent - textMetric.actualBoundingBoxDescent;

    return { 
        width: Math.abs(width), 
        height: Math.abs(height) 
    };
}
```

What you might notice is that we're declaring 2 variables outside of the scope of the function. This is *generally* bad practice, but this function will be called thousands of times and I want to self-optimize a bit, just in case.

If you have worked with `ctx.measureText` before, you might also realize I'm not using its returned `width` property, which should be exactly what I want. Some diacritics actually contain a negative width and the returned `width` will only go as low as `0`. I am calculating it myself to avoid such cases.

You can view the resulting code on [JS Fiddle](https://jsfiddle.net/owrum5qk/2/).

The code takes a while to run, but we (at least on my machine) get an array of 3 characters. 

![](https://i.imgur.com/WaqSQSo.png)

Yup. 3 spaces of varying widths. The canvas must have calculated these to be of zero width. Using these spaces, we can make some funky valid code.

```javascript
const ﾠ= 42;
const ㅤ= 58;
console.log(ﾠ+ㅤ); // 100
```

I'm excluding one of the spaces as it doesn't show up on some devices (such as Android phones or Windows 10 PC's). The other 2 spaces are known as [hangul filler](https://unicodelookup.com/#hangul%20filler/1) characters. One is a half-width, which is why it's thinner.

As an aside, while this test only ran through UTF-16 characters, I have done a test involving all unicode characters and gotten the same results.

At this point, we've gotten the 2 characters that ES6 will allow us to *start* a variable name with, but we haven't explored all the valid variable-naming characters. 

As discussed before, a number cannot be at the beginning of a variable name, although it can be anywhere after the first character. 

```javascript
const 1 = 42;
// Uncaught SyntaxError: Unexpected number
const num = 42;
// good
const 1num = 42;
// Uncaught SyntaxError: Invalid or unexpected token
const num1 = 42; 
// good
```

Our `isValidVariableName` fails to check for this. We can use the same function, but pass in a valid character as the first symbol to fully test this out. In our code, let's change the following code:

```javascript
function getSmallestChars() {
    // snip...

    if (!isValidVariableName(char)) continue;
        
    // snip...
}
```
to

```javascript
function getSmallestChars() {
    // snip...

    if (isValidVariableName(char)) continue;
    if (!isValidVariableName(`h${char}`)) continue;
        
    // snip...
}
```

With this code we are automatically skipping over super valid symbols and only keeping ones that are "kinda valid". We are prepending `h` to the symbol. This way, if it passes, it is valid only after the first character.

Using this change, we get 51 symbols (vs the 3 we originally got).

![](https://i.imgur.com/uCD31JL.png)

The newline (`↵` or `0x21B5`) character is a false-positive. It is not that the newline character is a part of the variable, it is simply getting skipped over. It reads similar to the following:

```javascript
const h
= 42;
```

Which, due to how ASI works, is valid code. Although, only `h`  (not `h↵`) has been set to `42`. We need to modify `isValidVariableName` a bit for this checking.

```javascript
function isValidVariableName(str, kinda = false) {
    try {
        if (kinda) {
            eval(`
                const h = 42;
                const h${str} = 42;
            `);
        } else {
            eval(`const ${str} = 42;`);
        }
    } catch {
        return false;
    }
    return true;
}
```

By already defining `h` before we use the passed string, we can guarantee an error will be thrown if the ASI simply interprets this as whitespace.

Let's also change the previous code to

```javascript
function getSmallestChars() {
    // snip...

    if (isValidVariableName(char)) continue;
    if (!isValidVariableName(char, true)) continue;
        
    // snip...
}
```

Running it we get 27 array elements.
![](https://i.imgur.com/t0tZXiO.png)
That means 24 of our previously returned symbols were whitespace characters. Here are the 27 hex codes:

```javascript
const symbols = ["0x34F", "0x5A2", "0x5BA", "0x5C5", "0x17B4", "0x17B5", "0x180B", "0x180C", "0x180D", "0x200C", "0x200D", "0xFE00", "0xFE01", "0xFE02", "0xFE03", "0xFE04", "0xFE05", "0xFE06", "0xFE07", "0xFE08", "0xFE09", "0xFE0A", "0xFE0B", "0xFE0C", "0xFE0D", "0xFE0E", "0xFE0F"]
```

It's at this point that I might as well mention that I have been doing most of these tests on a MacBook. I switch off between a MacBook and a Windows 10 Desktop PC depending on where I am. Windows 10 comes with a font containing many more unicode characters than other devices (aside for a few Linux distros). 

We want our "prank" to affect the majority of users, so we won't be using the larger 119 characters that my Windows machine gave me and only sticking to the 27 that both machines seem to share.

The first 9 characters are viewable on Windows' default font, so we're going to skip to the following 18.

The first 2 characters (`0x200C` and `0x200D`) are [zero width joiner/non-joiners](https://unicodelookup.com/#zero%20width/1). `0x200B`, the [zero width space](https://unicodelookup.com/#zero%20width%20space/1) (and the one right behind the other 2) was not included. Probably because it's whitespace and not a valid variable name.

The following 16 (from `0xFE00` to `0xFE0F`) are [variation selectors](https://en.wikipedia.org/wiki/Variation_Selectors_(Unicode_block)). There are many more than 16, but the rest are passed `0xFFFF` and thus would not come up in our search.

Here are all those characters: `→‌‍︀︁︂︃︄︅︆︇︈︉︊︋︌︍︎️←`

Running this code with the full extent of unicode doesn't genarate vastly different results. This means our aforementioned invisible tweet characters are not valid variable names. However, Our new characters are.

## Put Into Action

We went over a lot. We have 18 non-starting variable characters and 2 starting blank characters. All within UTF-16 (not that it's strictly needed).

Now for the "prank". Let's create a [Babel](https://babeljs.io/) transformer plugin.

```javascript
module.exports = () => {
  // counts the number of variables we have transformed
  let index = 0;

  return {
    visitor: {
      Identifier({ node }) {
        // convert number to hexidecimal and split into array
        const indexes = index.toString(16).split('');

        // turn each hex into an invisible char
        const chars = indexes.map((hex) => {
          const codePoint = 0xfe00 + parseInt(hex, 16);
          return String.fromCodePoint(codePoint);
        });

        // add these chars onto the node name
        node.name += chars.join('');

        // increase the number of variables we have transformed
        index++;
      }
    }
  };
};
```

This plugin will add invisible characters onto every variable name, making every variable unique. Passing this plugin to a babel transformation will render the code broken. The error messages will be even more cryptic, as nothing will seem to have changed.

![](https://i.imgur.com/uHdPz39.png)

Of course fixing this code manually will be extraordinarily difficult, which is why I've produced the cure as well!

```javascript
module.exports = () => {
  return {
    visitor: {
      Identifier({ node }) {
        const newName = node.name.split('').filter(char => {
          const codePoint = char.codePointAt(0);
          // if it's within this range, it's invisible.
          const isInvisible = codePoint >= 0xfe00 && codePoint <= 0xfe0f;
          // only allow visible characters
          return !isInvisible
        }).join('');

        node.name = newName;
      }
    }
  };
};

```


## Conclusion

I thought ending with a somewhat "practical" application of what we've found through researching unicode might be interesting. 

It goes without saying, but please don't *actually* use the aforementioned babel transformation on an unsuspecting participant's code. This was all in good fun and learning. The resulting output can be extraordinarily aggravating to debug.

#### June 4th Edit:
When discussing this post with a friend, we found it was possible to check valid variable characters using `regex`. This brings with it a significant speed improvement, so I'd advise using it over `try{}catch{}`.
One can find if a character is a valid starting character with `/\p{ID_Start}/u.test(char)` and if it's a valid "continuation" character with `/\p{ID_Continue}/u.test(char)`.