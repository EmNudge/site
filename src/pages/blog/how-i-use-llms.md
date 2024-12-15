---
pubDate: Dec 15, 2024
title: How I Use LLMs
summary: Be the wielder of the technology, not its servant
tags: coding
icon: technologies/languages/javascript
layout: ../../layouts/Blog.astro
---

AI? In my codebase? 

You'll get some very polarizing opinions on this one. You shouldn't "trust" code coming out of an LLM. Therefore you should reject all of it. But you can't trust junior devs either, so you can just lay off half your workforce and only use LLMs. But you'll have a higher maintenance burden and what if subtle bugs cause massive damage and where is the blame in our pipeline and what if about the paperclip problem and what if... or... is there... a third opinion?

Calling LLMs "AI" has done some damage to the discourse. These models do not think in the same way we do. Large language models have some limitations, but a black box statistical model for language is still very useful.

## Translation

LLMs can on their own translate between languages. 

They do this more roughly than a service like Google Translate, but the liberties they take can often make their translations more understandable. They'll do word and phrase substitutions that make more sense in context. While not marketed for this, it's pretty fantastic that you can get a multi-language context-based translator running locally on your computer with [ollama](https://ollama.com/).

This might not feel very useful when applied to code, but it does as good of a job translating between formats.

I've used LLMs to translate Objective-C to Swift because I found it very difficult to read some important source code in the language it was written in. I've translated JSON schemas to TypeScript types, [Tersed](https://terser.org/) JavaScript to readable TypeScript, python to rust, one library to another, one format to the next.

When used pedagogically, you don't need to verify the veracity of the conversion. For actual conversion that you plan on using, you'll need to do line-by-line analysis to ensure the conversion was done properly. This might take longer than hand-writing it all, but it can be a time saver if you're normally a slow typer.

You might find simpler to have the LLM generate a script to do the conversion. This can be easier to verify.

Here's a real-world use-case I had at work:

We had a system for documenting CLI flags. There were over 100 different flags which took different arguments. The types of input they expected would vary and we had associated documentation with each flag in its own JSON file.

A significant portion of the file was dedicated to the markdown documentation. Editing this was painful. I decided it was best to have a majority of the document dedicated to markdown with [YAML frontmatter](https://docs.github.com/en/contributing/writing-for-github-docs/using-yaml-frontmatter) for all the other fields.

This conversion is both simple and annoying. You can do this all with a script, but the original markdown also had a bunch of formatting issues that I wanted to solve while I did all this work. For example, it repeated the title and often had empty sections. It was inconsistent in section ordering. Things would be bold that shouldn't have been. 

I used an LLM to translate the initial JSON to Markdown. After verifying this went smoothly, I did another pass with an LLM to fix up all the pervious formatting inconsistencies. This was for documentation and we had git history, so having a subtle mistake here wasn't a major issue. 

## Experimentation

It's hard to trust the process when you don't know what that process is. I wouldn't "LGTM" a PR made by [Devin](https://devin.ai/) the same way I might to a PR from a coworker. 

But not all code goes to production.

A question like "How does the browser deal with nested forms" or "does this router library re-render when the new route matches the current page" can be solved by simply writing code. I've got a bit of a [process](https://emnudge.dev/notes/any-repl-with-bash/) for this, but it's significantly quicker to ask an LLM to generate something.

Without feedback loops like editor intellisense and runtime errors, you'll often go back and forth between your editor and the LLM with informative conversations like "wrong, please try again" or "Error: missing semicolon line 32". I'm sure this will change within a year, but your JS-based questions can go through closed-source tools like [Stackblitz](https://stackblitz.com/)'s [Bolt](https://bolt.new/) if you're willing to pay when your credits run out. 

I've used LLMs to write sample rust code to understand libraries. I've had it generate HTML in weird formats with varying setups to understand the browser. I once wanted to learn more about the distribution of [PRNG](https://en.wikipedia.org/wiki/Pseudorandom_number_generator) algorithms, so I had an LLM generate an HTML canvas to display them. 

All work that isn't technically complex, but it's annoying to spend an hour on boilerplate that you'll throw away as soon as your question is answered. 

Here's a real-world use-case I had at work:

We removed confetti from our application because it didn't fit well with our brand, but we could add it back if we could turn the confetti into something more finance-related. What about shredded receipts?

Receipts animate much differently than confetti. I can't run a physics sim, but it might be helpful to see how modifying publicly available libraries or writing some custom canvas code would fair. I had Bolt generate a few variations on this idea before deciding that any naive approach wasn't convincing enough to be worth the effort. 

## One-off scripts

The theme you'll find for the last 2 sections is that LLMs are good to save typing on low mental-overhead tasks. This section is no different.

The code to rename JS files with JSX in them to use the `.jsx` file extension is not incredibly difficult to write. Any language  with file system APIs can do this. 

What's the way I'm supposed to read files again? Okay so I need to read the files in the folder and then get the name and call this function which pipes it into a buffer and then use a text parser... Why didn't I read the whole file? There's a parameter for this - why isn't that the default? Did it break? Okay so there's a function which reads to a string directly, so I assume it does text parsing. What's the regex for this again? I need to not match JSX in strings. Ugh, this is annoying.

Or! I can have the LLM generate something and then give it a look-over to ensure it does what I want. Anything more destructive will require a greater degree of scrutiny, but I've already saved a greater deal of effort.

Figuring these things out on your own can be a more effective way to learn a new language. But I already know JavaScript. I've been doing it for a damn long time. And still, I need to look up Node.js docs for this. This exercise is not useful to me - I just want to *do the thing*.

Here's a real-world use-case I had at work:

We had a monorepo where teams would keep their projects. I supported these teams and wanted to know how many used specific library patterns we'd wanted to consolidate. Instead of spending an hour fiddling with variations of this script, I had an LLM generate something that I could verify and use to advocate for resourcing. 

## Creating Documentation

Documentation is a legitimately difficult problem. There are bare minimums here, but I don't think an LLM is the correct approach quite yet. Don't write your docs with an LLM.

Instead, what I'm referring to here is generating documentation for yourself. A lot of terminal tools will just throw another flag into the manpages and have you figure it out yourself. Oh you want to view the file? You just need `yagn -csns w ii -v1` unless you're running on arm in which case it's a little different in that it's impossible and a different program entirely is required. 

[TLDR](https://tldr.sh/) is a fantastic tool for getting CLI usage examples - I use it through [tlrc](https://github.com/tldr-pages/tlrc). You can also just use an LLM. 

The danger here is that it can be wrong. You can probably feed it the manpages right before asking for examples and it would do a better job. Still, I try to understand the flags it's using before running. I'll ask the LLM to break down the flags for me one-by-one and end up learning quite a bit too!

Here's a real-world use-case I had at work:

We had PDF issues because of course we did. I wanted to know if there were compression techniques I could use on a PDF I had to fix the render without significant quality loss. I don't know any PDF CLI tools, so I asked an LLM for its top 5 and then for the commands on each and some variations.

After verifying the flags made sense and that I had a backup copy of the PDF, I ran the commands and found that [Ghostscript](https://www.ghostscript.com/) did the best job. It beat out [pdftk](https://www.pdflabs.com/tools/pdftk-the-pdf-toolkit/) and some other lesser-known tools. Also, [Nix](https://nixos.org/) had the only usable installation, so I learned about that to install the darn thing. It's pretty neat.

## Contributions With Strong Guarantees

We're after all the things with low mental overhead. What if you're tasking it to do a complex problem? Should I trust it? Should I throw it away? Well, you shouldn't trust it, but that doesn't mean it's not usable. 

I trust my coworkers will view a feature they develop while they're developing it. I don't believe the same of the LLM. That means I need verifiable evidence that the code will not bring down production. 

I can run it? I can spin up a server and look around. Looks fine. I can't test everything, but nothing here looks broken. I can read it? The PR looks good. Proper technique. No glaring problems, but that doesn't necessarily mean anything. I can test it? Oh it failed a test. Oh that's bad. That's very bad.

Accepting large contributions without strong tests can be dangerous. Subtle bugs are always an issue, but they can be a larger issue when coming from a machine that thinks differently than you and I. You can't predict the kinds of bugs it might make. You need thorough testing.

The best kinds of changes are in contexts where the type system itself would prevent a merge. If your types are strong enough, subtle bugs might be rather difficult. In these contexts, the wrong code would look different enough to the right code for it to be easily caught in code review. 

This includes making type changes to a complex interface in TypeScript. It includes making changes to a lifetime in Rust. It does not include making runtime changes to a python script with a less-than-comprehensive test suite. 

Here's a real-world use-case I had at work:

We had this massive function which made calls to other functions. I needed to "prop-drill" some new data and decided the whole thing would be better if it were re-written as a hook that returned components instead of a function that returned data. This is rarely correct, but it was the right move here.

Doing this is very annoying manually. Trusting the LLM would be dangerous. Luckily, our linter and comprehensive test suite made errors with this transformation unlikely assuming it past all our checks in CI. It saved hours of work and that migration never caused a problem.

## In Editor

A lot of minor code editing tasks would take longer if you asked an LLM. You pass it your old code, ask it to modify something, paste it into your edit, find a syntax problem, go back, re-prompt it, try it again, check for accidental changes, test it, and then continue on.

It's not really worth it.

But if a program could predict what you're about to do and suggest it, accepting it with a key click would be much faster than writing it yourself. This is how I use [Cursor](https://www.cursor.com/). I don't prompt it to generate new complex code, but it's incredible at predicting alterations to code I'm about to perform. 

I edit a function to be named similarly to a hook in some other file. My editor suggests I call the hook at the top of the component. I accept. It suggests I import the hook. I accept. It suggests I change all call sites in this component. I accept. It suggests I delete the import for the now deprecated function. I accept.

A lot of this workflow can be done by other tools. I still use those. Auto-imports, LSP renaming - all solved with tools your editor already has. But pressing `tab` can still be faster, especially combined with more advanced edits.

Code doesn't have to be typed. My LLM is not ruining the joy of software construction by shortening the number of keystrokes required to program. It's just making my job easier.

## Conclusion

Imbuing the LLM with personhood is a dangerous game. It is not a person. You cannot blame the machine. Do not make the machine your scapegoat, absorber of sin.

The human closest to the problem takes the blame. If you merge an LLM PR, you are now responsible. It is as if you wrote this code in its entirety. You are not allowed to excuse its faults by its origin. This is your code. You copied it from Stackoverflow? I don't care - you are the owner now.

The danger in LLM contributions comes when you afford them similar responsibility to engineers in high-trust systems. It is not a person. It cannot be trusted. It is a tool. Do you trust your hammer? You trust it not to be a wrench. You don't let it decide whether you file a W2 or 1099.

But with the bumpers up and someone taking watch, it's a very useful tool. It can do a lot of things. It's kind of incredible. Sometimes people ask the hammer to do their taxes. This is confusing, but it's a really cool hammer. People tend to do that with cool hammers I guess.
