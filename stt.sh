#!/usr/bin/env bash
# Speech-to-Text using Whisper CLI
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/config.env"

INPUT="${1:-$TMP_DIR/audio_input.wav}"

if [[ ! -f "$INPUT" ]]; then
    echo "[stt] Error: Audio file not found: $INPUT" >&2
    exit 1
fi

# Run Whisper CLI (openai-whisper: positional audio arg, underscore flags)
RESULT=$("$WHISPER_BIN" \
    --model "$WHISPER_MODEL" \
    --language "$WHISPER_LANGUAGE" \
    --output_format txt \
    --output_dir "$TMP_DIR" \
    "$INPUT" 2>/dev/null)

# Whisper CLI writes to a .txt file alongside the input
TXT_FILE="$TMP_DIR/$(basename "${INPUT%.*}").txt"

if [[ -f "$TXT_FILE" ]]; then
    # Strip leading/trailing whitespace and any timestamp markers [00:00.000 --> ...]
    TEXT=$(sed 's/\[.*\]//g' "$TXT_FILE" | sed '/^$/d' | xargs)
    rm -f "$TXT_FILE"
else
    # Some whisper builds output to stdout
    TEXT=$(echo "$RESULT" | sed 's/\[.*\]//g' | sed '/^$/d' | xargs)
fi

if [[ -z "$TEXT" || "$TEXT" == "(blank audio)" || "$TEXT" == "[BLANK_AUDIO]" ]]; then
    echo "[stt] No speech detected" >&2
    exit 1
fi

echo "$TEXT"
