---
pubDate: April 22, 2024
title: "A REPL For Any Language Using Bash Functions"
summary: Local first and ethically farmed
tags: coding, productivity
layout: ../../layouts/Blog.astro
---

I really like how fast I can prototype an idea in JavaScript. It's one of my favorite parts of the language.

If I have a question about some runtime feature of the language, I can open dev tools on any page in the browser and quickly sketch something out. It's just `Cmd + Shift + I` and I'm off to the races.

I never understood how people can handle being required to set up an entire project to test out an idea. I'm using `rust`
 a lot more and this has really been troubling me.
## An Annoying Workflow

I ran into an issue with HTTP certs on a GET made with with [reqwest](https://github.com/seanmonstar/reqwest). Is this a problem with the package or something about the environment I'm running? I want to test out making a request with the `reqwest` outside of my project. How do I do that?

I need to find a folder I can use a scratchpad. I run `cargo new test-proj-thing` (hopefully an unused name) and then `cd test-proj-thing`. Now I can run `cargo add reqwest` and open up the folder in my editor of choice (for rust codebases, it has been [Zed](https://zed.dev/) these days).

This is really annoying. The steps are simple, but I'll likely forget to clean up this project. I'm going to collect cruft. At some point I'll have 30 "test project" folders with a few being significant.

It's gotten to the extent that somehow [repl.it](https://repl.it/) feels faster for prototyping than my own computer. I don't like this. I shouldn't need an internet connection or an account for prototyping.
## Temporary Directories

If you're running a Unix machine, you may have used `mktemp`. Maybe not, I hadn't. 

You can use `cd $(mktemp -d)` which will create a directory with some random text and `cd` into it. It's almost perfect. Except on MacOS it uses `/var/folders`, which is long-lived and it puts a `.` in the name. You can change a lot of this with CLI options, but at that point, you're making a bash script anyway.

I've been using the following.

```sh
function tmp() {
	local rand_name="r$(LC_ALL=C tr -dc a-z0-9 </dev/urandom | head -c 6)"
	local temp_folder="/tmp/trash-dirs/${1:+$1-}$rand_name"
	mkdir -p "$temp_folder" && cd "$temp_folder" || return 1
}
```

I can type `tmp` and I'll automatically be placed in a newly created folder in `/tmp/trash-dirs/` with a random name. `/tmp` should get cleaned up on reboot, but you can also just `rm -rf` the whole directory if you ever want to clean up preemptively. 

If you do `tmp rust` it will name the folder with that prefix, so you'll `cd` into something like `/tmp/trash-dirs/rust-ra3ij4`.

I start the name with `r` and exclude certain characters to ensure it's a valid project name for most languages. This lets you run `cargo init` instead of `cargo new $name`. It skips the additional step of coming up with a name and going into this new directory.
## Graduating A Directory

Eventually you'll want to move a folder out of this ephemeral space. I have another bash script for this.

Most of my projects exist in a `~/Workspace` directory, but you can change that folder to the space where the rest of your code usually lies.

```sh
function graduate() {
	folder_name=${1:-$(basename "$PWD")}

	new_path="$HOME/Workspace/$folder_name"
	if [ -d $new_path ]; then
    echo "Error: Folder $new_path already exists. Aborting."
    exit 1
	fi

	mv "$PWD" $new_path
	cd $new_path
}
```

## The REPL

So my workflow is usually

```sh
$ tmp 
$ cargo init
$ zed .
# leave project or decide it's worth keeping
$ graduate rust-proj
```

Quick. Efficient. Local.

