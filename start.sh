#!/usr/bin/env bash
# Entry point for Voice Pipeline
# Starts voice-to-Claude loop and optional dictation daemon
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/config.env"

# Cleanup on exit
cleanup() {
    echo ""
    echo "[start] Shutting down..."
    # Kill background jobs
    jobs -p | xargs -r kill 2>/dev/null || true
    # Remove temp files
    rm -rf "$TMP_DIR"
    echo "[start] Cleanup complete."
}
trap cleanup EXIT INT TERM

# Create temp directory and FIFO
mkdir -p "$TMP_DIR"
if [[ ! -p "$CONTROL_FIFO" ]]; then
    mkfifo "$CONTROL_FIFO"
fi

# Verify dependencies
check_dep() {
    if [[ ! -x "$1" ]] && ! command -v "$1" &>/dev/null; then
        echo "[start] Warning: $2 not found at $1" >&2
        return 1
    fi
    return 0
}

echo "=== Voice Pipeline ==="
echo ""
echo "Checking dependencies..."

DEPS_OK=true
check_dep "$WHISPER_BIN" "Whisper CLI" || DEPS_OK=false
check_dep "$CLAUDE_BIN" "Claude CLI" || DEPS_OK=false
check_dep "ffmpeg" "ffmpeg" || DEPS_OK=false

if [[ "$DEPS_OK" != "true" ]]; then
    echo "[start] Some dependencies are missing. Pipeline may not work correctly."
fi

# Default to mode 3 (both). Use --voice-only or --dictate-only for other modes.
MODE="${1:-3}"
case "$MODE" in
    --voice-only) MODE=1 ;;
    --dictate-only) MODE=2 ;;
    *) MODE=3 ;;
esac

case "$MODE" in
    1)
        echo ""
        "$SCRIPT_DIR/voice-pipeline.sh"
        ;;
    2)
        echo ""
        echo "[start] Dictation mode. Press Enter to record, Ctrl+C to exit."
        while true; do
            read -rp "Press Enter to dictate... "
            "$SCRIPT_DIR/dictate.sh" --submit || true
        done
        ;;
    3)
        echo ""
        # Compile KeyListener if needed
        KEYLISTENER="$SCRIPT_DIR/KeyListener"
        if [[ ! -x "$KEYLISTENER" ]]; then
            echo "[start] Compiling KeyListener..."
            swiftc -o "$KEYLISTENER" "$SCRIPT_DIR/KeyListener.swift" -framework Cocoa
            echo "[start] KeyListener compiled."
        fi

        # Start dictation daemon (reads from FIFO)
        (
            while true; do
                if read -r CMD < "$CONTROL_FIFO"; then
                    if [[ "$CMD" == "dictate" ]]; then
                        "$SCRIPT_DIR/dictate.sh" --submit 2>&1 || true
                    fi
                fi
            done
        ) &
        DICTATION_PID=$!
        echo "[start] Dictation daemon started (PID: $DICTATION_PID)"

        # Start KeyListener in background
        "$KEYLISTENER" &
        KEYLISTENER_PID=$!
        echo "[start] KeyListener started (PID: $KEYLISTENER_PID)"

        echo ""
        echo "Press F19 to dictate at cursor."
        echo "Voice-to-Claude loop starting..."
        echo ""

        # Run voice pipeline in foreground
        "$SCRIPT_DIR/voice-pipeline.sh"
        ;;
    *)
        echo "Invalid selection."
        exit 1
        ;;
esac
