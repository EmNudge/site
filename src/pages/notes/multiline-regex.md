---
pubDate: Feb 19, 2024
title: "In Pursuit of a JavaScript Multiline Regex"
summary: Getting a regex you can read
tags: coding
icon: technologies/languages/javascript
layout: ../../layouts/Blog.astro
---

I love regex, but it definitely feels like more of a write-only language than I'm comfortable with. It's powerful, but terse and completely alien to anyone who isn't deeply familiar with the syntax.

For the record, I think Regex is far simpler than it looks. Most characters can be literally translated as referring to that character (the regex "a:>#@q" matches only the literal "a:>#@q"). There are only a handful of characters with special meaning, but you wouldn't know that unless you've already written a fair bit of regex. 

Annoyingly, Regex is often the right tool if your problem space isn't worthy of a full parser. I try to avoid Regex, being aware of how it may look to others, but my alternatives are often much more computationally expensive (In JavaScript), significantly longer, and less forgiving. 

# A Good Regex

As an example, here is an emoji-aware string split.

```js
const emojiSplitRe = /(?:\p{Emoji}\p{Emoji_Modifier}?\u{200d}\p{Emoji}\p{Emoji_Modifier}?\u{fe0f}?)|(?:\p{Emoji}\p{Emoji_Modifier})|./giu
```

Say we want to split a string by "visible" characters for common-ish inputs (this is not the focus of this article, I am aware there are still edge cases). Simply doing the following will give us an undesirable result:

```js
'hi 💩 ✊🏾'.split('') // ['h', 'i', ' ', '\uD83D', '\uDCA9', ' ', '✊', '\uD83C', '\uDFFE']
```

We have split both our surrogates (a signature of our text encoding format) and our emoji.

We can solve the surrogates issue by iterating through the string, which is defined to not split surrogates.

```js
[...'hi 💩 ✊🏾'] // ['h', 'i', ' ', '💩', ' ', '✊', '🏾']
```

This gives us distinct unicode characters, but some emojis are made up of multiple codepoints, such as this closed-fist which uses a skin-tone modifier.

To split this text without splitting up the emoji, we can use the previous regex.

```js
'hi 💩 ✊🏾'.match(emojiSplitRe) // ['h', 'i', ' ', '💩', ' ', '✊🏾']
```

We've at least established that our regex is useful, but it still is far from readable. How do we remedy this? Many approach this by pulling in an entire library which allows expressing the regex as a less terse series of steps in english. I prefer a multi-line regex.

# Multi-Line Regex

Virtually every modern language (aside from JavaScript) has the ability to use a multi-line regex. We must usually signify we're in "extended" (x) mode in which we ignore whitespace and start allowing `#` prefixed comments.

```rust
// A rust example
let re = regex::Regex::new(r#"
	# Match 'a' one or more times 
	a+ 
	# Match 'b' zero or more times 
	b*
"#).unwrap();
```

To capture whitespace, we must explicitly escape spaces, which is a bit annoying, but definitely a worthy trade-off for clarity.

It's a little frustrating that JavaScript does not yet have this seemingly common-sense feature. There is a [3 year old TC39 proposal](https://github.com/tc39/proposal-regexp-x-mode) to add it, but there has been 0 activity since then.

We can write a small utility function to bring this to javascript.

```js
const regExtend = flags => str => 
    new RegExp(
        String.raw(str)
            .replace(/(?:#.+)|\s+/gm, ''),
        flags
    )
```

This allows us to generate our emoji-split regex with:

```js
const emojiSplitRe = regExtend('mgui')`
# emoji sequence
(?:
    \p{Emoji}
    \p{Emoji_Modifier}?
    \u{200d}
    \p{Emoji}
    \p{Emoji_Modifier}?
    \u{fe0f}?
)
# regular emojis
|(?:
	\p{Emoji}
	\p{Emoji_Modifier}
)
# anything else
|.
`
```

While we've got a significant clarity bump, we're still dealing with strings. I would much prefer instead looking at a regex literal.

# Regex Literals

While most languages have a regex extended mode, rarely do they have "regex literals". I'm only aware of 4 languages in popular use with this primitive: JavaScript, Perl, Ruby, and Elixir.

```perl
# Perl Code
my $regex_pattern = qr/ 
	# Match 'a' one or more times 
	a+ 
	# Match 'b' zero or more times 
	b* 
/x;
```

Of the 4 languages with regex primitives, JavaScript is the only one missing an extended mode. The [TC39 proposal from earlier](https://github.com/tc39/proposal-regexp-x-mode) does not seem interested in introducing this either, I'm assuming due to parser semantics. It only covers string-based multiline regex.

Why does this matter to me? Simply put: **syntax highlighting**.

As a terse language, syntax highlighting is a god-send. We can often expect syntax highlighting environments to highlight language primitives. Strings rarely get this treatment as it can be difficult to detect their environment or to standardize this behavior.

So what's a good workaround? There are a couple of avenues.

Earlier we made use of template literals. These allow us to embed expressions that we can make use of later on. 

```js
const regExtend = flags => (strs, ...exprs) => 
    new RegExp(exprs.map(r => r.source).join(''), flags);

regExtend()`
	# Match 'a' one or more times 
	${/a+/}
	# Match 'b' zero or more times 
	${/b*/}
`
```

This solution puts a bit more focus on the comments than the regex itself, which I think is probably not the right direction here.

[A comment from brad4d](https://github.com/tc39/proposal-regexp-x-mode/issues/2#issue-1037518153) on the tc39 proposal provides a utility method to simply join an array of regex strings. Let's tweak this to use regex literals.

```js
const zipRegex = (re, flags) => 
	new RegExp(re.map(re => re.source).join(''), flags)

const re = zipRegex([
	// Match 'a' one or more times 
	/a+/,
	// Match 'b' zero or more times 
	/b*/,
]);
```

I really like this solution, but it's still not great for our emoji regex which makes use of groups. 

```js
const re = zipRegex([
    // emoji sequence
    /(?:\p{Emoji}\p{Emoji_Modifier}?\u{200d}\p{Emoji}\p{Emoji_Modifier}?\u{fe0f}?)/,
    // regular emojis
    /|(?:\p{Emoji}\p{Emoji_Modifier})/,
    // anything else
    /|./,
], 'mgui');
```

It does improve clarity somewhat, but we cannot split our groups. Doing so would result in regex compilation errors.

# Combining Approaches

While slightly more complex than the previous options, we can combine our previous approaches by using a regular string for larger portions, but allow ourselves to escape into regex literals when we want syntax highlighting.

```js
const regExtend =
  (flags) =>
  (strings, ...reArrs) => {
    const raw = strings.map((str) => str.replace(/(?:#.+)|\s+/gm, ""));
    const regexStrings = reArrs.map((reArr) => reArr.map((re) => re.source).join(""));
    return new RegExp(String.raw({ raw }, ...regexStrings), flags);
  };

const emojiSplitRe = regExtend("mgui")`
# emoji sequence
(?:
    ${[
      /\p{Emoji}/,
      /\p{Emoji_Modifier}?/,
      /\u{200d}/,
      /\p{Emoji}/,
      /\p{Emoji_Modifier}?/,
      /\u{fe0f}?/,
    ]}
)
# regular emojis
|(?:
    ${[/\p{Emoji}/, /\p{Emoji_Modifier}/]}
)
# anything else
|.
`;
```

\# end note