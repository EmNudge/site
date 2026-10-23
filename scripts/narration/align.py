#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.9"
# dependencies = [
#     "stable-ts==2.19.1",
# ]
# ///
"""Two-pass narration audio worker. JSON is read from stdin and written to stdout."""

import argparse
import json
import sys

import stable_whisper

SAMPLE_RATE = 16_000


def field(value, name, default=None):
    if isinstance(value, dict):
        return value.get(name, default)
    return getattr(value, name, default)


def serialize_word(word):
    return {
        "text": field(word, "word", ""),
        "start": float(field(word, "start", 0.0) or 0.0),
        "end": float(field(word, "end", 0.0) or 0.0),
        "probability": float(field(word, "probability", 0.0) or 0.0),
    }


def transcribe(args):
    print(f"Loading stable-ts ASR model {args.model}...", file=sys.stderr)
    model = stable_whisper.load_model(args.model)
    result = model.transcribe(
        args.audio,
        language="en",
        word_timestamps=True,
        condition_on_previous_text=False,
        regroup=False,
        temperature=0,
        verbose=False,
    )
    words = [
        serialize_word(word)
        for segment in result.segments
        for word in (field(segment, "words", []) or [])
        if field(word, "word", "")
    ]
    duration = max((word["end"] for word in words), default=0.0)
    json.dump(
        {
            "aligner": "stable-ts",
            "version": getattr(stable_whisper, "__version__", "unknown"),
            "model": args.model,
            "duration": duration,
            "words": words,
        },
        sys.stdout,
    )
    sys.stdout.write("\n")


def align(args):
    payload = json.load(sys.stdin)
    jobs = payload.get("jobs", [])
    print(f"Loading stable-ts alignment model {args.model} for {len(jobs)} jobs...", file=sys.stderr)
    model = stable_whisper.load_model(args.model)
    audio = stable_whisper.audio.load_audio(args.audio, sr=SAMPLE_RATE, verbose=False)
    results = []
    for job in jobs:
        identifier = job["id"]
        start = float(job["windowStart"])
        end = float(job["windowEnd"])
        try:
            if not (0 <= start < end):
                raise ValueError(f"invalid window {start:.3f}..{end:.3f}")
            clip = audio[int(start * SAMPLE_RATE) : int(end * SAMPLE_RATE)]
            text = " ".join(job["words"])
            result = model.align(
                clip,
                text,
                language="en",
                original_split=True,
                remove_instant_words=False,
            )
            if result is None:
                raise RuntimeError("stable-ts returned no alignment")
            words = []
            for segment in result.segments:
                for word in field(segment, "words", []) or []:
                    serialized = serialize_word(word)
                    serialized["start"] += start
                    serialized["end"] += start
                    words.append(serialized)
            results.append({"id": identifier, "ok": True, "words": words})
        except Exception as error:
            results.append({"id": identifier, "ok": False, "words": [], "error": str(error)})
    json.dump(
        {
            "aligner": "stable-ts",
            "version": getattr(stable_whisper, "__version__", "unknown"),
            "model": args.model,
            "results": results,
        },
        sys.stdout,
    )
    sys.stdout.write("\n")


def main():
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)

    transcribe_parser = subparsers.add_parser("transcribe")
    transcribe_parser.add_argument("--audio", required=True)
    transcribe_parser.add_argument("--model", default="small.en")
    transcribe_parser.set_defaults(handler=transcribe)

    align_parser = subparsers.add_parser("align")
    align_parser.add_argument("--audio", required=True)
    align_parser.add_argument("--model", default="base.en")
    align_parser.set_defaults(handler=align)

    args = parser.parse_args()
    args.handler(args)


if __name__ == "__main__":
    main()
