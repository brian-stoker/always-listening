#!/usr/bin/env bash
# Text-to-Speech via Home Assistant (hass-cli) with macOS fallback
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/config.env"

TEXT="$*"
if [[ -z "$TEXT" ]]; then
    TEXT=$(cat)
fi

if [[ -z "$TEXT" ]]; then
    echo "[tts] Error: No text provided" >&2
    exit 1
fi

# Strip markdown formatting for clean TTS output
TEXT=$(echo "$TEXT" | sed \
    -e 's/^#{1,6} //g' \
    -e 's/\*\*//g' \
    -e 's/\*//g' \
    -e 's/__//g' \
    -e 's/`//g' \
    -e 's/^- //g' \
    -e 's/^• //g' \
    -e 's/^[0-9]*\. //g' \
    -e 's/\[//g' \
    -e 's/\]//g' \
    -e 's/(http[^ ]*)//g' \
    -e 's/→/ /g' \
    -e 's/—/ — /g' \
    -e '/^$/d')

echo "[tts] Speaking: ${TEXT:0:80}..." >&2

# Try Home Assistant ElevenLabs TTS via REST API
if [[ -n "${HASS_TOKEN:-}" && -n "${HA_URL:-}" ]]; then
    # Escape text for JSON
    JSON_TEXT=$(python3 -c "import json,sys; print(json.dumps(sys.argv[1]))" "$TEXT")
    PAYLOAD="{\"entity_id\":\"$HA_TTS_ENTITY\",\"media_player_entity_id\":\"$HA_ENTITY\",\"message\":$JSON_TEXT}"

    if curl -sf -X POST "$HA_URL/api/services/tts/speak" \
        -H "Authorization: Bearer $HASS_TOKEN" \
        -H "Content-Type: application/json" \
        -d "$PAYLOAD" >/dev/null 2>&1; then
        echo "[tts] Sent to Home Assistant ($HA_ENTITY via ElevenLabs)" >&2
        exit 0
    else
        echo "[tts] Home Assistant call failed, falling back to macOS say" >&2
    fi
fi

# Fallback: macOS say
if command -v say &>/dev/null; then
    say "$TEXT"
    echo "[tts] Played via macOS say" >&2
    exit 0
fi

echo "[tts] Error: No TTS backend available" >&2
exit 1
