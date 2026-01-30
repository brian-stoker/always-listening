#!/usr/bin/env bash
# Dictation mode: Record → Transcribe → Type at cursor
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/config.env"

mkdir -p "$TMP_DIR"

echo "[dictate] Dictation mode active — speak now..."

# 1. Record audio
if ! "$SCRIPT_DIR/record.sh" "$TMP_DIR/dictation_audio.wav"; then
    echo "[dictate] No speech detected"
    exit 1
fi

# 2. Transcribe
TEXT=$("$SCRIPT_DIR/stt.sh" "$TMP_DIR/dictation_audio.wav" 2>/dev/null) || true

if [[ -z "$TEXT" ]]; then
    echo "[dictate] No text transcribed"
    exit 1
fi

echo "[dictate] Transcribed: $TEXT"

# 3. Type text at cursor using AppleScript System Events
# Escape double quotes and backslashes for AppleScript
ESCAPED_TEXT=$(echo "$TEXT" | sed 's/\\/\\\\/g; s/"/\\"/g')

osascript -e "tell application \"System Events\" to keystroke \"$ESCAPED_TEXT\""

echo "[dictate] Text typed at cursor"

# 4. Optionally press Enter to submit
if [[ "${1:-}" == "--submit" ]]; then
    sleep 0.1
    osascript -e 'tell application "System Events" to keystroke return'
    echo "[dictate] Enter pressed"
fi
