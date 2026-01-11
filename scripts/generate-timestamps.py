#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "openai-whisper",
# ]
# ///
"""
Generate timestamp files for audio recordings of articles.

Uses Whisper for speech recognition and matches transcription to paragraphs.

Usage:
    uv run scripts/generate-timestamps.py <audio_file> <markdown_file> [output_file]

Example:
    uv run scripts/generate-timestamps.py ~/Downloads/article.mp3 src/pages/blog/optimism-as-pragmatism.md

Requires: ffmpeg (brew install ffmpeg)
"""

import sys
import re
from pathlib import Path

import whisper


def extract_paragraphs(markdown_path: Path) -> list[str]:
    """
    Extract paragraph text from a markdown file.
    Returns list of cleaned paragraph strings.
    """
    content = markdown_path.read_text()
    lines = content.split('\n')

    paragraphs = []
    in_frontmatter = False
    frontmatter_count = 0

    for line in lines:
        stripped = line.strip()

        # Handle frontmatter
        if stripped == '---':
            frontmatter_count += 1
            if frontmatter_count == 1:
                in_frontmatter = True
            elif frontmatter_count == 2:
                in_frontmatter = False
            continue

        if in_frontmatter:
            continue

        # Skip empty lines
        if not stripped:
            continue

        # Skip headers
        if stripped.startswith('#'):
            continue

        # Skip image-only lines
        if re.match(r'^!\[.*\]\(.*\)$', stripped):
            continue

        # Clean markdown formatting
        clean_text = stripped

        # Strip blockquote prefixes (handles nested blockquotes like "> > text")
        clean_text = re.sub(r'^(?:>\s*)+', '', clean_text)
        clean_text = re.sub(r'\*\*([^*]+)\*\*', r'\1', clean_text)  # bold
        clean_text = re.sub(r'\*([^*]+)\*', r'\1', clean_text)      # italic
        clean_text = re.sub(r'\[([^\]]+)\]\([^)]+\)', r'\1', clean_text)  # links
        clean_text = re.sub(r'!\[([^\]]*)\]\([^)]+\)', '', clean_text)    # images

        if clean_text.strip():
            paragraphs.append(clean_text.strip())

    return paragraphs


def normalize_text(text: str) -> str:
    """Normalize text for matching."""
    text = text.lower()
    text = re.sub(r'[^\w\s]', '', text)  # Remove punctuation
    text = re.sub(r'\s+', ' ', text).strip()
    return text


def get_first_words(text: str, n: int = 5) -> str:
    """Get first N words of text, normalized."""
    words = normalize_text(text).split()
    return ' '.join(words[:n])


def format_timestamp(seconds: float) -> str:
    """Convert seconds to MM:SS format."""
    minutes = int(seconds // 60)
    secs = int(seconds % 60)
    return f"{minutes:02d}:{secs:02d}"


def find_paragraph_timestamps(paragraphs: list[str], segments: list[dict]) -> list[float]:
    """
    Match paragraph starts to Whisper segment timestamps.
    Returns list of timestamps (one per paragraph).
    """
    timestamps = []
    segment_idx = 0

    for para in paragraphs:
        first_words = get_first_words(para, 6)

        # Search through segments to find where this paragraph starts
        best_match_idx = segment_idx
        best_match_score = 0

        # Look ahead in segments to find the best match
        for i in range(segment_idx, min(segment_idx + 20, len(segments))):
            seg_text = normalize_text(segments[i]['text'])

            # Check if paragraph starts within this segment
            if first_words in seg_text:
                best_match_idx = i
                best_match_score = len(first_words)
                break

            # Partial match - check word overlap
            para_words = set(first_words.split())
            seg_words = set(seg_text.split())
            overlap = len(para_words & seg_words)
            if overlap > best_match_score:
                best_match_idx = i
                best_match_score = overlap

        timestamps.append(segments[best_match_idx]['start'])
        segment_idx = best_match_idx

    return timestamps


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        sys.exit(1)

    audio_path = Path(sys.argv[1]).expanduser()
    markdown_path = Path(sys.argv[2]).expanduser()

    if len(sys.argv) >= 4:
        output_path = Path(sys.argv[3])
    else:
        article_name = markdown_path.stem
        output_path = Path('public/recordings') / f'{article_name}.timestamps'

    if not audio_path.exists():
        print(f"Error: Audio file not found: {audio_path}", file=sys.stderr)
        sys.exit(1)

    if not markdown_path.exists():
        print(f"Error: Markdown file not found: {markdown_path}", file=sys.stderr)
        sys.exit(1)

    print(f"Extracting paragraphs from {markdown_path.name}...")
    paragraphs = extract_paragraphs(markdown_path)
    print(f"Found {len(paragraphs)} paragraphs")

    print(f"Transcribing audio with Whisper...")
    model = whisper.load_model("base")
    result = model.transcribe(str(audio_path), word_timestamps=False)
    segments = result['segments']
    print(f"Got {len(segments)} segments from Whisper")

    print(f"Matching paragraphs to timestamps...")
    timestamps = find_paragraph_timestamps(paragraphs, segments)

    content = '\n'.join(format_timestamp(ts) for ts in timestamps) + '\n'

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(content)

    print(f"Written timestamps to {output_path}")


if __name__ == '__main__':
    main()
