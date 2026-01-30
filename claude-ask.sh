#!/usr/bin/env bash
# Send text to Clawdbot and capture response
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/config.env"

# Accept input from argument or stdin
if [[ $# -gt 0 ]]; then
    PROMPT="$*"
else
    PROMPT=$(cat)
fi

if [[ -z "$PROMPT" ]]; then
    echo "[clawdbot] Error: No input provided" >&2
    exit 1
fi

echo "[clawdbot] Sending to Clawdbot: ${PROMPT:0:80}..." >&2

# Wrap prompt with instruction to respond in plain spoken text (no markdown)
VOICE_PROMPT="[Voice input — respond in plain conversational speech. No markdown, no bullet points, no headers, no emojis, no special characters. Keep it concise and natural as if speaking aloud.] $PROMPT"

# Send to Clawdbot agent, parse response text from JSON
RAW=$(clawdbot agent --agent main --session-id voice-pipeline --message "$VOICE_PROMPT" --json 2>/dev/null) || true

if [[ -z "$RAW" ]]; then
    echo "[clawdbot] Error: No response from Clawdbot" >&2
    exit 1
fi

# Extract the text payload from JSON response
RESPONSE=$(echo "$RAW" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['result']['payloads'][0]['text'])" 2>/dev/null) || true

if [[ -z "$RESPONSE" ]]; then
    echo "[clawdbot] Error: Could not parse response" >&2
    exit 1
fi

echo "$RESPONSE"
