# Astro Site

Website has gone from
```
 ┌───────────────────────────────────────────────────────────┐
 │                                                           │
 │ Vanilla -> Gatsby -> Next -> Svelte -> SvelteKit -> Astro │
 │                                                           ▲
 │                                                           │
 │                  You Are Here!  ─────────────────────────┘
```

Welcome!

## Narration alignment

Posts with `recording` frontmatter can be aligned locally against their
recording. The command renders the page through Astro, transcribes the audio once
to find reliable word anchors, then force-aligns each exact article sentence
inside its own constrained window:

```sh
pnpm narration:align -- writing-better --force
# or use a local recording and explicit pass-one/pass-two models
pnpm narration:align -- writing-better --force \
  --audio ~/Downloads/writing-better.mp3 \
  --asr-model small.en --align-model base.en
```

The first run requires `uv` and `ffmpeg`; downloaded audio is cached under
`.cache/narration/`, which is ignored by git, while stable-ts uses its normal
model cache. Pass-one transcription is cached by the audio hash and model, so
article edits only rerun the faster sentence alignment. The generated
`public/recordings/<recording>.align.json` contains article hashes, sentence
ranges, and word start/end times. Generation writes nothing if any sentence
has collapsed, overlapping, or otherwise untrustworthy timing; diagnostics go
to `.cache/narration/<recording>.diag.json`.

The player defaults to sentence highlighting and lets readers switch between
paragraph, sentence, and word modes. Existing `.timestamps` files remain a
paragraph-level fallback. To verify that committed alignments still describe
the current articles after a build:

```sh
pnpm build
pnpm narration:check
```
