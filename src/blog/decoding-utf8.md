---
date: Sep 9, 2020
title: Decoding UTF-8
summary: Jumping into unicode and binary 
tags: coding
icon: unicode
---

I'm a JavaScript guy, so this is a bit of a weird article. I recently went through a month's worth of research into unicode for an [article](https://dev.to/emnudge/adventures-in-horrible-no-good-very-bad-variable-names-20ii) and then subsequent [video](https://www.youtube.com/watch?v=vHkM6RwA7Wk) on variable names. JavaScript uses UTF-16 and I put some more specific info into the [extra video](https://www.youtube.com/watch?v=ZQRAMHiVEzs), rather than the main video.

I'd advise watching it if you're curious. I was recently asked about UTF-8 by another JavaScript developer and so I decided to look into it. Resources online are scarce or not friendly enough that I was asked to turn my personal explanation into an article for future reference.

First off, why UTF-8?

# Encoding Unicode

Every unicode encoding is tasked with a similar problem:

> How do we represent all 1.1M possible code points?

The actual number is `1_111_998`. In binary this takes up 21 bits. That means it's possible to have a single character that **must** require us to use `21`  bits at a minimum to encode.

![Unicode bits](https://dev-to-uploads.s3.amazonaws.com/i/ynl1asdzclxldylzelhy.png)

We stick to powers of 2 for binary encoding, so our first thought might be exactly what [UTF-32](https://en.wikipedia.org/wiki/UTF-32) is today. Let's use 32 bits!

This works, but is very inefficient. If we approach a character that requires 21 bits, we'll need to use 4 bytes (32 bits) anyway, but most characters do not have a codepoint that large. This means we are wasting  the first 11 bits of every codepoint, at a **minimum**. In fact, this article only uses 3 characters larger than 122 (the 3 example characters later on in the article)! That's (for the most part) only 7 bits per character!

In comes [UTF-16](https://en.wikipedia.org/wiki/UTF-16) with a clever trick.

## UTF-16

It recognizes that characters with codepoints that require more than 16 bits are even more rare! 16 bits gives us roughly 65k possible codepoints. What we do is reserve 2 separate ranges of 1024 characters out of our initial 65k codepoints available. These are used for "surrogate pairs" (more info my [unicode extras video](https://www.youtube.com/watch?v=ZQRAMHiVEzs)). Using a bit of math we find that if we combine them, we're able to get 20 more bits (enabling `1_048_576` more characters).

If we go by character by character, as computers usually do, we're able to tell whether the codepoint we are looking at is a lone one or a surrogate pair just by checking what range it is in! 

This way we can shave off 16-bits for most characters! The surrogate pair ranges are purposely empty anyway, so we're not kicking any characters out.

![Unicode Block Infographic](https://dev-to-uploads.s3.amazonaws.com/i/ygcf2t5zhn7co07hp2x9.png)

While JavaScript uses UTF-16, HTML traditionally uses UTF-8. Only 8 bits? It looks like we barely got by with 16 bits! Are we going to reserve another few ranges? How? Out of 256 possible codepoints, which ranges do we reserve?! 

We could use multiple bytes for larger codepoints, but how would we ever know if we were currently looking at a lone character or one that is meant to be read together with others? UTF-16 and UTF-32 gives us the guarantee that we'll be able to start from anywhere in a file and, provided we know our index, we can regain our footing.

## UTF-8

And now the point of the article. UTF-8 takes a different approach. Instead of reserving a specific range, each starting byte will tell us how many **more** bytes to expect. The answer to this can be either 0, 1, 2, or 3, since UTF-8 characters can be composed of anywhere from 1 to 4 characters.

Our base case is 0. 

If we're given a byte that begins with a `0`, then this is the only codepoint required to parse this into a character. Using the codepoint of 103 as an example (the character `g`),  its UTF-8 encoding would be `01100111`. Notice the leading `0`. We can express this codepoint with 7 bits, so we use the leading bit to tell whomever is asking that this is the only number in this sequence. Only one byte!

![7 bit characters](https://dev-to-uploads.s3.amazonaws.com/i/xa87dekrksy2w5ucl4cv.png)

This works fine up until codepoints over 127. In such a case we are forced to use 2 bytes. 

We need to signal this somehow. UTF-8 tells us to start the first byte with `110`. The second byte must start with `10`. Since each byte is 8 bits, we're left with  `(8 - 3) + (8 - 2) =` 11 total bits! This allows us to express codepoints in the range 128-2047.

Therefore the British Pound (`£`), with a codepoint of 163, requires 2 bytes. Its direct binary representation is `10100011`. Expanding it to the full 11 bits turns it to `00010100011`. Combining the required `110` with the first 5 bits gives us `11000010` for our first bit. Our second bit then combines `10` with our remaining 6 bits (`100011`) gives us `10100011`.

![11 bit characters](https://dev-to-uploads.s3.amazonaws.com/i/ej0z66rbeqxmir5dgjdq.png)

But again, this only covers the first 2047 characters. For larger codepoints (up until 65,536) we need 3 bytes. 

This time we use `1110` (notice an extra `1`) at the start and follow it up by 2 numbers which both start with `10`. This is useful for codepoints such as `￫` which have a codepoint of `65_515`. Since this time it leaves us with `(8 - 4) + 2(8 - 2) =` 16 bits to express 63,488 more characters!

And finally we get to the remaining characters. 

For characters such as 😀, with a codepoint of `128_512`, we can use a leading `11110` (another `1`) to signal that this is a part of a 4 byte codepoint. The next 3 characters again start with `10`. Applying our basic math expression, we find this leaves us with `(8 - 5) + 3(8 - 2) =` 21 bits! Exactly enough to express all unicode codepoints!

![16 and 21 bit characters](https://dev-to-uploads.s3.amazonaws.com/i/135p2qbze2d9eyw7c5l7.png)

## The Algorithm

Now that we've gone through the explanation, let's make it concise with some steps. Let's say  we're placed at a random byte in a file which we know to be UTF-8 encoded.

1. Does this byte begin with a `0`? If so this can be parsed by itself and we can move on to the next byte.
2. Does this byte begin with `10`? Whoops! If so, we're in the middle of a character! We need to move forward until this is no longer the case or move backwards to find out what sort of character we were trying to parse.
3. Does this byte begin with 2-4 `1`s followed up by a `0`? If so, provided `n` refers to the number of `1`s, we've approached an `n`-byte codepoint and should parse the next `n - 1` bytes and this one as a single codepoint.
4. Does this byte begin with 5 or more `1`s? This is invalid UTF-8! Well, maybe. Perhaps in the future we have somehow expanded to ~69 Billion codepoints. 

## The Code

Because we're in binary-land, we also have quite a few operators at our disposal to make things more efficient - [bitwise operators](https://developer.mozilla.org/tr/docs/Web/JavaScript/Reference/Operat%C3%B6rler/Bitwise_Operators). They're a bit unintuitive at first, so I suggest going through them at first to fully understand *what* they do before seeing *how* they're used.

I've placed the code for my UTF-8 encoder/decoder into a Svelte REPL over here: [https://svelte.dev/repl/9d89e50badbd458599fc62cde67fc9b3?version=3.24.1](https://svelte.dev/repl/9d89e50badbd458599fc62cde67fc9b3?version=3.24.1)

I won't be going over the majority of the code, since it'd be a bit too much for this article (and a bit too boring, frankly), so let's just clarify 2 common operations that use bitwise operators in the code.

1. Retrieving Part of a Binary
    - To isolate a binary, we create another binary with the bits we want to preserve and use the `&` operator - make every bit you want to keep a `1`. If it starts in the middle, we can use the `>>` operator to move it back to the start.
    - `0b1111 & myBinary` = isolate the first 4 bits
    - `(0b111100 & myBinary) >> 2` = isolate the first 4 bits after the first 2 bits
2. Adding on Bits to a Binary
    - We cannot add on without knowing the binary length. If we do, just create a binary with `n` digits + however many the binary is and use the `|` operator. Make the rest of the bits (first `n` digits) all `0`.
    - `0b10000 | myBinary` = if binary is 3 digits long, adds on `10` to the end.
    - `0b10000000 | ((myBinary >> 6) & 0b111111)` = take the first 6 bits and add on `10` to the end.

# Conclusion - Efficiency

From the looks of it, it seems like UTF-8 is the most efficient encoding so far. However, there are clearly some edge cases where it would be much less efficient than something like UTF-16.

As a closing, here are 2 quick tests:

- On something like Herman Melville's Moby Dick in English, UTF-16 increases the file size just about twofold, from `1.19MB` to `2.39MB` as the majority of its characters (from the text file I found) would be within the first 127 codepoints. That's one byte per character in UTF-8 and 2 in UTF-16). UTF-32 would no doubt have doubled it yet again.
- On Dan Abramov's [My Decade In Review](https://overreacted.io/my-decade-in-review/) in Simplified Chinese, UTF-16 is slightly smaller than the UTF-8 variant (28.8kb vs 36kb). Many of the codepoints are in the ~25k range. This would be 3 bytes in UTF-8 and only 2 in UTF-16 - a roughly 1.5x increase. The markdown file also includes many links and single byte UTF-8 characters, which is why it isn't entirely `1.5x`.

Without looking anything up, we can assert that UTF-8 would be less efficient than UTF-16 for characters with codepoints between 2048-65534. This would favor UTF-16 when such codepoints dominate the medium, like a book rather than a blogpost, where the benefit is only marginal due to the quantity of single-byte characters that UTF-16 forces to be 2 bytes.

UTF-16 would almost always be more space-efficient than UTF-32 unless the Unicode standard opened up codepoints that could be expressed with 22-32 bits. In which case such codepoints would be better expressed with UTF-32.

For most purposes, particularly code in just about any programming language, UTF-8 is more efficient. 

In regards to computational efficiency, the efficiency order would go in reverse - UTF-32, UTF-16, and then UTF-8. 

Lossless compression formats such as `.zip` and `.72` would also likely make the size differences much thinner (or perhaps even reverse the current order). Specially tailored compression formats even more so. While these encoders work on character to character, other compression formats may be able to work on text blobs as a whole.

Critique? Let me know in the comments!