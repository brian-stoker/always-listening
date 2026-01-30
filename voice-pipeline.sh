#!/usr/bin/env bash
# Main voice-to-Claude loop: Record → Transcribe → Claude → TTS
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/config.env"

mkdir -p "$TMP_DIR"

echo "=== Voice-to-Clawdbot Pipeline ==="
echo "Speak to interact with Clawdbot. Say 'goodbye clawdbot' to exit."
echo ""

while true; do
    # 1. Record audio with silence detection
    if ! "$SCRIPT_DIR/record.sh"; then
        echo "[pipeline] No speech detected, listening again..."
        continue
    fi

    # 2. Transcribe with Whisper
    TEXT=$("$SCRIPT_DIR/stt.sh" 2>/dev/null) || true

    if [[ -z "$TEXT" ]]; then
        echo "[pipeline] Transcription empty, listening again..."
        continue
    fi

    echo "[pipeline] You said: $TEXT"

    # 3. Check for exit command (only "goodbye clawdbot" exits the pipeline)
    TEXT_LOWER=$(echo "$TEXT" | tr '[:upper:]' '[:lower:]')
    if [[ "$TEXT_LOWER" == *"goodbye clawdbot"* || "$TEXT_LOWER" == *"goodbye claudbot"* || "$TEXT_LOWER" == *"goodbye claude bot"* ]]; then
        echo "[pipeline] Exit command detected. Goodbye!"
        "$SCRIPT_DIR/tts-ha.sh" "Goodbye!" 2>/dev/null || true
        break
    fi

    # 4. Send to Claude CLI
    echo "[pipeline] Thinking..."
    RESPONSE=$("$SCRIPT_DIR/claude-ask.sh" "$TEXT" 2>/dev/null) || true

    if [[ -z "$RESPONSE" ]]; then
        echo "[pipeline] Claude returned empty response"
        "$SCRIPT_DIR/tts-ha.sh" "Sorry, I didn't get a response." 2>/dev/null || true
        continue
    fi

    echo "[pipeline] Claude: $RESPONSE"

    # 5. Send response to TTS
    "$SCRIPT_DIR/tts-ha.sh" "$RESPONSE" 2>/dev/null || true

    echo ""
done
