---
pubDate: March 22, 2024
title: "A Dead Simple, Actually Useful ToDo List using val.town"
summary: Ignoring REST for GET request based APIs
tags: coding
icon: technologies/languages/javascript
layout: ../../layouts/Blog.astro
---

I love repl environments because they lower the barrier for iteration. They let you test out ideas quickly and gain reliable information to fix your mental model about the systems you work with.

I’ve been fascinated with [val town](https://www.val.town/) for a while. They take this in-browser snippet idea and make it a request-response thing. You write a route for a server and perform some action on certain `HTTP` methods. They then give you a real working subdomain that you can quickly test with!

You can also remix/fork other people’s snippets. There are some great ideas there. It’s a product that I so desperately want to survive. I wish them success.

On a completely separate stream of thought - I need a way to write down “todo” items that are globally synced across my devices.

[Ferb, I know what we're doing today!](https://www.youtube.com/watch?v=djm4h7nv2oI)

# Val.town

In just a few lines, you can get a completely functional system.

```jsx
import { sqlite } from "<https://esm.town/v/std/sqlite>";
import { Hono } from "npm:hono@3";

sqlite.execute(`create table if not exists todos(
  id text unique,
  todo text
)`);

app.post("/", async (c) => {
  const text = await c.req.text();
  sqlite.execute({
    sql: `insert into todos(id, todo) values (:id, :todo)`,
    args: { id: crypto.randomUUID(), todo: text },
  });

  return c.text(`added todo: "${text}"`);
});

app.delete("/:id", async (c) => {
  const id = c.req.param("id");
  await sqlite.execute({
    sql: "delete from todos where id=:id",
    args: { id },
  });

  return c.text(`deleted todo with id of ${id}`);
});

app.get("/", async c => {
  const { rows } = await sqlite.execute(`select todo from todos`);
  const todos = rows.map(items => items[0]);
  return c.json(todos as unknown);
});

app.get("/with-id", async c => {
  const { rows } = await sqlite.execute(`select id, todo from todos`);
  const todos = rows.map(row => ({ id: row[0], todo: row[1] }));
  return c.json(todos as unknown);
});

export default app.fetch;
```

And it just works!

You now have a sqlite database that retains all posted items and returns them to you when you ask.

I can add items with a `curl` and retrieve them the same way. Perfect!

I mean, almost.

A ToDo list shouldn’t require curl. It’s really not the most convenient way to deal with this system. I made some shell scripts to make the curl process easier, so I could just write

```sh
add-todo "some item here"
```

but this requires making a shell script. This doesn’t feel very globally synced.

# Make Everything a GET Request

Okay, so here’s a weird idea - let’s remove any special methods. No `POST`, no `DELETE` - just `GET` .

```js
// used to be a POST
app.get("/add", async (c) => {
  const text = (new URL(c.req.url)).searchParams.get("q");
  await sqlite.execute({
    sql: `insert into todos(id, todo) values (:id, :todo)`,
    args: { id: crypto.randomUUID(), todo: text },
  });

  return c.redirect("/");
});

// used to be a DELETE
app.get("/delete/:id", async (c) => {
  const id = c.req.param("id");
  await sqlite.execute({
    sql: "delete from todos where id=:id",
    args: { id },
  });

  return c.redirect("/");
});
```

I know, it looks a little weird. I’m not doing REST correctly - whatever.

But now we can get the same things done with a browser! To add an item I just go to

```
my-site.com/add?q=make some handmade pasta tonight
```

And it's added!

This means we can ditch the shell scripts and potentially even edit these on our phone. The main drawback is that deleting items based off of their ID isn’t great. I need to go to my `with-id` route, copy it, and edit my current URL.

Editing the URL on a phone is really difficult and it’s still not the best option on desktop.

Check this out:

```ts
app.get("/delete", async (c) => {
  const { rows } = await sqlite.execute(`select id, todo from todos`);
  const todos = rows.map(row => ({
    deleteLink: `/delete/${row[0]}`,
    todo: row[1],
  }));

  return c.text(JSON.stringify(todos as unknown, null, 2));
});
```

When going to the `delete` route (without specifying an ID), I display every item and an associated link to delete it. This looks something like:

```js
[
  {
    "deleteLink": "/delete/3c6a1e05-6422-4d97-b5ff-0b89bc88c36e",
    "todo": "look into tailscale"
  },
  {
    "deleteLink": "/delete/df6d9b8f-db71-48b1-9591-c1a2c1c76020",
    "todo": "read database internals by alex petrov"
  },
  {
    "deleteLink": "/delete/007addc3-8e5e-4c89-9616-4ec845d7e958",
    "todo": "make coldbrew coffee"
  },
  {
    "deleteLink": "/delete/3308491a-f57e-4c59-bffa-1ac99be84fc5",
    "todo": "prototype css highlight API for syntax highlighting"
  }
]
```

And these links are clickable with some [JSON viewer chrome extensions](https://chromewebstore.google.com/detail/json-viewer/gbmdgpbipfallnflgajpaliibnhdgobh)!

So we have

- `/` - view all todo items
- `/delete` - get delete button for each item
- `/add?q=<some text>` add item

And we can do this all from the browser on any computer!

We're relying on our unlisted private link being the key to privacy here instead of some kind of authentication. You can probably build this in by forking some other val.town script.

It also wouldn't be too difficult to return some HTML to get a better interface, but this method allows you to consume the output in other contexts instead of just the browser. 

Maybe we can check the user agent and return different content like some other tools?

