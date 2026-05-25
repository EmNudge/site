#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <note|blog> <title> [--description <desc>] [--slug <slug>] [--tags <tags>]"
  echo ""
  echo "Examples:"
  echo "  $0 note \"My Cool Note\""
  echo "  $0 blog \"My Blog Post\" --description \"A summary\" --slug my-custom-slug --tags \"coding, philosophy\""
  exit 1
}

if [[ $# -lt 2 ]]; then
  usage
fi

TYPE="$1"
TITLE="$2"
shift 2

if [[ "$TYPE" != "note" && "$TYPE" != "blog" ]]; then
  echo "Error: type must be 'note' or 'blog'"
  exit 1
fi

DESCRIPTION=""
SLUG=""
TAGS=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --description) DESCRIPTION="$2"; shift 2 ;;
    --slug) SLUG="$2"; shift 2 ;;
    --tags) TAGS="$2"; shift 2 ;;
    *) echo "Unknown option: $1"; usage ;;
  esac
done

# Infer slug from title if not provided
if [[ -z "$SLUG" ]]; then
  SLUG=$(echo "$TITLE" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9 -]//g' | tr ' ' '-' | sed 's/--*/-/g' | sed 's/^-//;s/-$//')
fi

# Format today's date like "May 25, 2026"
PUB_DATE=$(date "+%b %-d, %Y")

if [[ "$TYPE" == "note" ]]; then
  DIR="src/pages/notes"
else
  DIR="src/pages/blog"
fi

FILE_PATH="$DIR/$SLUG.md"

if [[ -f "$FILE_PATH" ]]; then
  echo "Error: $FILE_PATH already exists"
  exit 1
fi

# Build frontmatter
{
  echo "---"
  echo "pubDate: $PUB_DATE"
  echo "title: \"$TITLE\""
  if [[ -n "$DESCRIPTION" ]]; then
    echo "summary: $DESCRIPTION"
  fi
  echo "tags: ${TAGS:-general}"
  echo "layout: ../../layouts/Blog.astro"
  echo "---"
  echo ""
} > "$FILE_PATH"

echo "Created $FILE_PATH"
