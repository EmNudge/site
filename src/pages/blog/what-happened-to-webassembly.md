---
pubDate: Dec 31, 2025
title: What Happened To WebAssembly
summary: Where are they now? You won't believe her new look!
tags: coding
layout: ../../layouts/Blog.astro
discussions:
  - { site: lobsters, url: https://lobste.rs/s/elj9pq/what_happened_webassembly }
---

On every WebAssembly discussion, there is inevitably one comment (often near the top) asking what happened.

It seems to have been advertised as a world-changing advancement. Was it just oversold? Was it another JVM applet scenario, doomed to fail?

I’d like to tackle this in a weirdly roundabout way because I think these sorts of questions make a few misplaced assumptions that are critical to clarify.

# In The Real World

Of course, WebAssembly **does** see real-world usage. Let’s list some examples!

- Godot uses WebAssembly to [build games for the web](https://docs.godotengine.org/en/stable/contributing/development/compiling/compiling_for_web.html).
- Squoosh.app uses WebAssembly to [make use of image libraries](https://web.dev/blog/squoosh-v2#new_codecs_support).
- Zellij uses WebAssembly for its [plugin ecosystem](https://zellij.dev/documentation/plugins#plugins).
- Figma uses WebAssembly to [convert their C++ code](https://www.figma.com/blog/webassembly-cut-figmas-load-time-by-3x/) to something usable in a browser
- Stackblitz uses WebAssembly for [their web containers](https://blog.stackblitz.com/posts/introducing-webcontainers/).
- Ruffle uses WebAssembly to run [a flash emulator](https://ruffle.rs/) in your browser

For many of these, WebAssembly is critical to either their entire product or a major feature.

But I think this alone is not very convincing. We don’t yet see major websites entirely built with [webassembly-based frameworks](https://github.com/yewstack/yew). We’re not building our applications directly to WebAssembly for maximum portability. But why not? 

To answer this, we need a good mental model for what WebAssembly is. This will help us qualify where it is most impactful and the limitations we’re up against.

# What Is WebAssembly

In a word, WebAssembly is a language. 

## A Note On Speed

This makes questions like “how fast is WebAssembly” a bit hard to answer. You don’t ask how fast algebraic notation is—it’s not a very sensible question. 

Taken in the context of something like JavaScript, the language is only as fast as the engine running it. JavaScript the language has no speed, but you can benchmark JS engines like V8, SpiderMonkey, and JavaScriptCore. You can benchmark the IO libraries of JS runtimes like Bun, Deno, and Node.

What people actually mean is “how useful are the constructs of this language to efficient mappings of modern hardware” and “what is the current landscape of systems taking advantage of these constructs”.

Through clever-enough engineering, you can make any system sufficiently fast with some trade-offs. If compiling your code directly to C doesn’t bother you, getting “near native” speeds is possible in both [JavaScript](https://hermesengine.dev/) and [WebAssembly](https://github.com/WebAssembly/wabt/blob/main/Wasm2c/README.md).

That’s right, you can compile WebAssembly! You can also choose to interpret it directly—that’ll be up to your runtime, just like every other system.

So let’s ask the actual question of WebAssembly: how useful are the constructs of this language to efficient mappings of modern hardware? Turns out, pretty useful!

## An Efficient Mapping

WebAssembly is a pretty close approximation of an assembly language. Not too close, mind you. It’s higher level than that. But it’s close enough to cleanly compile to most assembly languages without a significant speed trade-off. 

And yes, you can write WebAssembly by hand! I made a rustlings-esque course called [watlings](https://github.com/EmNudge/watlings/) where you can hand-write WAT to solve some basic exercises.

WAT is a very close approximation to Wasm. It is **almost** 1:1 in that you can compile WAT to Wasm and then back to WAT with barely any loss in information (you may lose variable names and some metadata). It looks like this:

```clojure
(module
  ;; import external i32, name it $global_num_import
  (import "env" "global_num" (global $global_num_import i32))

  ;; A function that adds param $a to $global_num_import, returns i32
  (func $add_to_global_num (param $a i32) (result i32)

    ;; The last stack value is the return value
    (i32.add (local.get $a) (global.get $global_num_import))
  )

	;; export local function, name it add_to_global
  (export "add_to_global" (func $add_to_global_num))
)
```

Try reading the code. It will feel both familiar and foreign. 

We have functions and S-expressions. We have imports and exports. But we also have instructions like `i32.add` and implicit stack returns.

Wasm is a bytecode perhaps best compared to *JVMIS* (i.e. JVM bytecode). They have similar goals and constraints, but different landscapes and guarantees.

Compared to JVM bytecode, Wasm has a significantly smaller API and stronger safety guarantees. It has fewer opinions on your memory management strategy and more limitations on what your program can do without permission from its host environment.

It can crunch numbers, but must be explicitly provided its memory and all imports. In this way, it is much different from an actual assembly language (or, a more widely used one).

We’ll wrap back around to this later.

# A compilation target

You can compile many languages to Wasm. 

Notable among them are Rust, C, Zig, Go, Kotlin, Java, and C#. Commonly interpreted languages have even had their runtimes compiled to WebAssembly, such as Python, PHP, and Ruby. There are also many languages that solely compile to WebAssembly, such as [AssemblyScript](https://www.assemblyscript.org/), [Grain](https://grain-lang.org/), and [MoonBit](https://www.moonbitlang.com/).

For many of these, it is important not to require a garbage-collector. For others, it would be helpful to include one. Wasm allows for both (with the GC option being much more recent).

Your browser includes a Wasm “engine”, making this doubly an attractive compilation target. This means without much setup, your phone and laptop can run Wasm programs already. 

Like how JVM can have many implementations of its runner, there are many implementations that run independently of your browser such as [Wasmtime](https://github.com/bytecodealliance/Wasmtime), [WasmEdge](https://github.com/WasmEdge/WasmEdge), and [Wasmer](https://github.com/Wasmerio/Wasmer). 

```bash
$ Wasmer run cowsay "I am cow"
 __________
< I am cow >
 ----------
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
               ||----w |
                ||     ||
```

These languages can output a single artifact without being too specific to your computer’s hardware. You only need a Wasm runner to execute it (note more JVM analogies).

# Security and what it enables

Right now, Wasm is looking really similar to JVM. The main differences seem to be around memory management strategies and how many platforms support it.

The security story is what really starts to drive in the wedge.

WebAssembly maintains a minimal attack surface by treating all external interactions as explicit, host-defined imports. We went over this earlier. Its “deny-by-default” architecture, small instruction set, hidden control-flow stack (i.e. no raw pointers), and linear memory combine to create a **very strong** security story.

It is such that you can ensure process-like isolation within a single process. Cloudflare takes advantage of this aspect within V8 to run untrusted code very efficiently [using V8 isolates](https://blog.cloudflare.com/mitigating-spectre-and-other-security-threats-the-cloudflare-workers-security-model/). This means significant efficiency gains without significant security trade-offs. 

Wasm programs can start 100x faster if you can avoid spinning up a separate process. Fermyon, a company in the Wasm hosting space, advertises [sub-millisecond spinup times](https://www.fermyon.com/serverless-guide/speed-and-execution-time).

In these cases, the performance is a direct result of what the security guarantees enable.

In other cases, security can unlock feature support.

Flash is a multimedia platform that was primarily used for animations and games up until it was dropped from all major browsers in January of 2021 (primarily) due to security concerns. [Ruffle](https://ruffle.rs/) has revived Flash experiences on sites like [Newgrounds](https://www.notion.so/What-Happened-To-WebAssembly-aa6295c947a24fc48de2a31378ced53b?pvs=21) by acting as an interpreter and VM for ActionScript.

Cloudflare allows running Python code with similar security guarantees to its JS code by using [Pyodide](https://pyodide.org/en/stable/), which is a Wasm build of CPython.

Figma runs untrusted user plugins in your browser by running them in [a QuickJS engine that is compiled to Wasm](https://www.figma.com/blog/how-we-built-the-figma-plugin-system/).

Elsewhere, the security allows for extreme embeddability. 

# Portability and Embeddability

We’ve gone over the number of ways you can run Wasm programs. A Wasm runner can be pretty light. Instead of forcing library authors into a specific language (usually Lua or JavaScript), supporting Wasm itself opens the door to a much wider set of choices.

Tools like [Zellij](https://github.com/zellij-org/zellij), [Envoy](https://www.envoyproxy.io/), and [Lapce](https://github.com/lapce/lapce) support Wasm for their plugin ecosystem. 

In environments where a JavaScript engine is already being used, this means access to programs you would not have been able to run otherwise.

This includes [image processing](https://github.com/kleisauke/Wasm-vips), [ocr](https://github.com/naptha/tesseract.js), [physics engines](https://github.com/kripken/ammo.js), [rendering engines](https://skia.org/docs/user/modules/canvaskit/), [media toolkits](https://ffmpegWasm.netlify.app/), [databases](https://github.com/sql-js/sql.js), and [parsers](https://github.com/tree-sitter/tree-sitter/blob/master/lib/binding_web/README.md), among many others.

In a majority of these cases, the use of Wasm will be transparent to you. A library you installed will just be using it somewhere in its dependency tree.

Godot and Figma have codebases written in C++, but are often browser-ready by compiling to (or in combination with) WebAssembly.

It seems the most common use of Wasm is bridging the language gap. Certain ecosystems seem to have suites of tools more common to them. [Squoosh](https://squoosh.app/) would be a much more limited application if it could only choose image compression libraries from NPM. 

# Speed and size revisited

Browsers run WebAssembly with roughly the same pipeline that runs JavaScript. This seemingly puts a hard limit on the performance of Wasm applications, but they will often be more or less performant due to their architecture or domain.

Using languages with richer type systems and more sophisticated optimizing compilers can produce more efficient programs. The JIT model of engines like V8 might prevent optimizations if the cost of optimizing exceeds the gains from running the optimized code. You might avoid [megamorphic functions](https://mrale.ph/blog/2015/01/11/whats-up-with-monomorphism.html) more easily by avoiding JavaScript. 

However, there is a cost to crossing the host-program boundary, especially if cloning memory. [Zaplib’s post-mortem](https://news.ycombinator.com/item?id=31214521) is an interesting read here. Incrementally moving a codebase to Wasm can incur significant costs in boundary crossing, eliminating any benefit in the short term.

A small API surface also means binary bloat as system APIs are more often re-created than imported. There are standards like [WASI](https://wasi.dev/) which aim to help here. Still, there is no native string type ([yet](https://github.com/WebAssembly/stringref)).

Zig seems to produce the smallest Wasm binaries among mainstream languages.

Practical performance of Wasm in native contexts (i.e. outside of a JS engine) seems to suffer for a variety of reasons. Threading and IO of any sort incurs some cost. Memory usage is larger. Cold start is slower.

Still, the performance trade-offs might not be significant enough to matter. For most uses, I’d wager it’s “fast enough”. If you’re in a performance-sensitive context, the benefits of Wasm are likely not as relevant.

# Language development and you

Clearly things are happening.

The [Wasm IO YouTube channel](https://www.youtube.com/@Wasmio) has lots of talks worth watching. 

In fact, standards and language development in Wasm has stirred significant controversy internally. There is a lot of desire for advancement, but standardization means decisions are hard to reverse. For many, things are moving too quickly and in the wrong direction.

There is the “more official” [W3C working group](https://www.w3.org/groups/wg/Wasm/) and then the “less official” [Bytecode Alliance](https://github.com/bytecodealliance) which works much more quickly and is centered around tooling and language development outside of Wasm directly (e.g. on [WIT](https://component-model.bytecodealliance.org/design/wit.html) and the [WebAssembly Component Model](https://component-model.bytecodealliance.org/)).

[Wasm feature proposals](https://webassembly.org/features/) are being quickly advanced and adopted by a wide suite of tools. This is remarkable progress for standardization, but is also scary to watch if you fear large missteps. 

So why do people think nothing has happened?

I figure most are under the impression that the advancement of this technology would have had a more visible impact on their work. That they would intentionally reach for and use Wasm tools. 

Many seem to think there is a path to Wasm replacing JavaScript within the browser—that they might not need to include a `.js` file at all. This is very unlikely.

However, you can use frameworks like [Blazor](https://dotnet.microsoft.com/en-us/apps/aspnet/web-apps/blazor) and [Leptos](https://github.com/leptos-rs/leptos) without being aware or involved in the produced JS artifacts. 

Mostly, Wasm tools have been adopted and used by library authors, not application developers. The internals are opaque. This is fine, probably. 

Separately, I think the community is not helped by the philosophy of purposely obfuscating teaching material around Wasm. This is a fight I lost a few times. 

For now, maybe check out [watlings](https://github.com/EmNudge/watlings). I’ll expand it at some point, surely.