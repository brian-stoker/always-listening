#!/usr/bin/env bash
# Record audio from microphone until F18 is pressed (send signal file appears)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/config.env"

mkdir -p "$TMP_DIR"
OUTPUT="${1:-$TMP_DIR/audio_input.wav}"
SEND_SIGNAL="$TMP_DIR/send.signal"

# Clear any stale signal
rm -f "$SEND_SIGNAL"

echo "[record] Listening... (press F18 to send)"

# Record in background with no time limit
ffmpeg -y -f avfoundation -i "$AUDIO_DEVICE" \
    -ar 16000 -ac 1 -sample_fmt s16 \
    "$OUTPUT" 2>"$TMP_DIR/ffmpeg.log" &
FFMPEG_PID=$!

# Wait for send signal (F18 creates this file)
while [[ ! -f "$SEND_SIGNAL" ]]; do
    sleep 0.2
done

# Stop recording
kill "$FFMPEG_PID" 2>/dev/null || true
wait "$FFMPEG_PID" 2>/dev/null || true
rm -f "$SEND_SIGNAL"

# Check if the file exists and has content
if [[ ! -f "$OUTPUT" ]]; then
    echo "[record] No audio file created"
    exit 1
fi

# Check if the file has actual audio content
VOLUME_INFO=$(ffmpeg -i "$OUTPUT" -af "volumedetect" -f null /dev/null 2>&1 | grep "mean_volume" || true)

if [[ -z "$VOLUME_INFO" ]]; then
    echo "[record] No audio detected"
    exit 1
fi

MEAN_VOLUME=$(echo "$VOLUME_INFO" | sed -n 's/.*mean_volume: \([-0-9.]*\) dB/\1/p')

# If mean volume is below -50dB, consider it silence-only
if [[ -n "$MEAN_VOLUME" ]] && (( $(echo "$MEAN_VOLUME < -50" | bc -l) )); then
    echo "[record] Audio too quiet (${MEAN_VOLUME}dB), likely silence only"
    exit 1
fi

echo "[record] Saved to $OUTPUT (${MEAN_VOLUME}dB)"
exit 0
