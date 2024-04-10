---
pubDate: April 7, 2024
title: "Your Terminal Might Support Links?"
summary: Markdown-style links MIGHT be supported! Find out here!
tags: productivity
layout: ../../layouts/Blog.astro
---

I've been working on [logpipe](https://github.com/EmNudge/logpipe) over the weekend. It's a tool to display logs directly from CLI programs by just piping the output directly into it.

I apply some heuristics to provide automatic syntax-highlighting and grouping. It's neat. You should check it out.

One of the issues I ran up against was tools that don't plan on having their output piped. They preserve their syntax highlighting through ANSI color codes. It's how you do color on the terminal in general.

Most tools will remove it if they detect their output is going to something that isn't a terminal, but some tools don't suspect that will ever happen, like the CLI output from [Shuttle](https://www.shuttle.rs/).

So you get something like this.

![logpipe without ANSI](https://assets.emnudge.com/logpipe_ansi_preparse.png)

Which is not ideal. I could simply remove all these ANSI color codes, but it's probably better if I just parse them. The colors already included are likely better than ones I would be guessing at.

# Parsing ANSI

There's a lot going on, honestly. People have made libraries for this stuff. I'm going to keep it simple and only parse a common subset. I'll expand it as bug reports are filed.

The syntax you're looking for is `\x1B[` + number sequence + `"m"`. For example:
```
\x1B[31m Hello There! \x1B[0m
```
Which outputs:
<div style="color: red; white-space: pre"> Hello There!</div>

The `\x1B\[0m` at the end is a "reset" which just removes all previous modifications. Try not to forget this. You can accidentally mess up the rest of your output if you don't reset the styles.

You can test this out directly in Node.js console logs or using `echo -e`.
```sh
echo -e "\x1B[31m Hello there! \x1B[0m"
```

You can stack modifications, like making text green, bold, and underlined.

```
\x1B[32m \x1B[1m \x1B[4m Hello \x1B[0m
```
<div style="color: green; text-decoration: underline; font-weight: bold; white-space: pre"> Hello </div>

There's a lot of repetition there. We can combine modifiers by semicolon separating them.

```
\x1B[31;1;4m Hello \x1B[0m
```
<div style="color: green; text-decoration: underline; font-weight: bold; white-space: pre"> Hello </div>

I'd recommend checking out [this StackOverflow answer](https://stackoverflow.com/questions/4842424/list-of-ANSI-color-escape-sequences) for a larger explanation. It's surprisingly thorough.

If you ever wondered how console coloring libraries like [chalk](https://github.com/chalk/chalk) work, this is how. Just ANSI escapes. 

# Embedding Links

Okay, so this is the part was really surprising to me. I built my parser and found some text not captured by my code.

```
\x1B]8;;https://doc.rust-lang.org/cargo/reference/profiles.html#default-profiles\x1B\\`dev` profile [unoptimized + debuginfo]\x1B]8;;\x1B\\
```

It's hard to tell with all that other text there. Here are the escapes without the text (replaced with `<name>`).

```
\x1B]8;; <link> \x1B\\  <text> \x1B]8;;\x1B\\
```

The syntax here is different. We're used to seeing the left `[` bracket, but what is this `\x1B]8;;`? And then sometime later there's a lone `\`  with `\x1B\\`? It looks bizarre. Well, I guess the previous syntax was also strange.

From the context I recognized this as a markdown-like link, but with the link before the text. A lot of these orderings make more sense when you think about performance characteristics. It's not there for readability. 

The link above is directly from `cargo build`. It looks something like this:

<div style="background: black; color: white; white-space: pre; padding: 0 12px; border-radius: 4px; overflow: auto;">
<span style="color: yellow">warning</span>: `inspect_wav` (bin "inspect_wav") generated 3 warnings
    <span style="color: limegreen">Finished</span> <a href="https://doc.rust-lang.org/cargo/reference/profiles.html#default-profiles" style="text-decoration: none; color: inherit;">`dev` profile [unoptimized + debuginfo]</a> target(s) in 0.04s
<span></span>
</div>

Yeah, I had no idea that middle bit was clickable either! 

# Comparing Terminals

Many terminals either don't highlight this behavior or don't make it clickable at all. I went through a couple. 

These are the terminals with default settings. It's possible they contain some secret toggle which I didn't have time to find.

## Disappointments

Terminal.app, [Hyper](https://hyper.is/), and [Warp](https://www.warp.dev/) all do not display special formatting and do not let you click these links. Nothing breaks - it just appears as regular text.

## Passable

Then some terminals like [Wezterm](https://wezfurlong.org/wezterm/index.html), [Alacritty](https://alacritty.org), and [Kitty](https://sw.kovidgoyal.net/kitty/) let you click the link! 
They display an underline when you hover over the clickable text, but not before. This makes discovery a bit difficult, but at least it works!

![kitty link while hovering](https://assets.emnudge.com/kitty_link.png)

Notably, cargo will not output the special link text for Alacritty for some reason. I don't think it's special-cased, so there's some context being used. Wezterm and Kitty get the upper hand here.

## Great

[iterm2](https://iterm2.com/) comes in with an almost perfect design. It gives special syntax, but the link is not clickable unless you are holding `cmd`. When you do, it shows a link preview, which is better than the previous 3 terminals.

Annoyingly, it suffers from the same issue as Alacritty. Cargo does not seem to think it supports links, so it won't supply the ANSI escape codes.

![iterm2 while command-hovering](https://assets.emnudge.com/iterm2_link.png)

## Perfect

The winner absolutely no one predicted - [VS Code](https://code.visualstudio.com/)'s integrated terminal!

![VS Code before hovering](https://assets.emnudge.com/vscode_link_prehover.png)

By far the best experience. It displays this dotted underline for clickable links. It works with Cargo. When you hover over, the underline becomes solid and it shows the destination.

![VS Code right after hovering](https://assets.emnudge.com/vscode_link_posthover.png)

# logpipe

My tool displays terminal content. I have a chance to do the correct thing. So I do!

If I detect a terminal markdown-like link, I make it clickable.

![logpipe after parsing ANSI links](https://assets.emnudge.com/logpipe_ansi_postparse.png)

\# end note