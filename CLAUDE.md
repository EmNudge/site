# Site

Personal blog/notes site built with Astro.

## Creating new posts

Use `./new-post.sh` to scaffold a new blog or note:

```
./new-post.sh <note|blog> "<title>" [--description "<summary>"] [--slug <slug>] [--tags "<tags>"]
```

- Slug is inferred from the title if omitted
- Date is set to today automatically
- Tags default to `general` if omitted (tags are required for notes listing page)
- Posts go to `src/pages/notes/` or `src/pages/blog/`
