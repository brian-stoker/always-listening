# Build Always-Listening Voice Pipeline Tray App

## 1. Feature Overview
**Feature Name:** Always-Listening Voice Pipeline Tray App
**Owner:** TBD
**Status:** Draft
**Target Release:** TBD

### Summary
A cross-platform (macOS and Windows) system tray application that wraps the existing Always-Listening voice pipeline, exposing its three operating modes (Voice-to-Claude, Dictation, and Combined) as toggleable menu items. The app provides a native, lightweight interface where users can double-click a mode to toggle it on or off, configure optional Home Assistant integration through a Preferences window, and install from the macOS App Store or Microsoft Store. By packaging the current shell-based pipeline into a distributable tray app, users gain a polished, always-available voice assistant without needing terminal familiarity.

---

## 2. Problem Statement

### What problem are we solving?
The existing voice pipeline requires launching shell scripts from a terminal, manually selecting modes via command-line flags, and managing background processes by hand. This makes it inaccessible to non-technical users and cumbersome even for developers who want the pipeline running persistently in the background. There is no GUI for toggling modes, no visual status indicator, and no guided setup for optional Home Assistant integration (which currently requires manual Docker installation, HA configuration, and token management).

### Who is affected?
- **Primary users:** Anyone who wants a continuous voice-to-AI assistant or voice dictation tool on their desktop without interacting with a terminal.
- **Secondary users:** Home automation enthusiasts who want to connect the voice pipeline to Home Assistant for ElevenLabs TTS via Google Cast speakers but find the manual Docker and HA setup process daunting.
- **Developers and power users:** Those already using the shell scripts who want a more ergonomic way to manage and toggle modes throughout their workday.

### Why now?
Voice-driven AI interaction is rapidly becoming mainstream. The underlying pipeline (Whisper STT, Claude/Clawdbot AI, ElevenLabs TTS) is functional but locked behind a terminal-only workflow. Packaging it as a native tray app with App Store distribution lowers the barrier to adoption significantly and positions the project for a broader user base. Cross-platform support (macOS + Windows) maximizes reach.

---

## 3. Goals & Success Metrics

### Goals
1. **Provide a native system tray app** that runs on macOS and Windows, displaying a menu with the three pipeline modes as toggleable items.
2. **Enable one-click mode toggling** -- double-clicking a mode in the tray menu starts or stops it, with visual indicators (checkmarks, status text) showing active state.
3. **Offer a Preferences window** with a guided Home Assistant integration flow that handles Docker installation, Home Assistant setup, entity configuration, and token management.
4. **Distribute via App Stores** -- submit and maintain listings on the macOS App Store and Microsoft Store.
5. **Preserve existing functionality** -- the tray app must orchestrate the same underlying pipeline scripts/logic without regressions.
6. **Run without Home Assistant by default** -- the app must work out of the box using local Whisper STT and macOS `say` / Windows SAPI TTS fallback, with HA as a purely optional enhancement.

### Success Metrics
| Metric | Target |
|--------|--------|
| App Store approval (macOS) | Accepted within first 2 submission attempts |
| Microsoft Store approval | Accepted within first 2 submission attempts |
| Mode toggle latency | Under 2 seconds from click to pipeline process start/stop |
| Cold start to ready | Under 5 seconds on modern hardware |
| HA integration setup completion rate | 80% of users who begin the flow complete it successfully |
| Crash-free sessions | 99%+ across both platforms |

---

## 4. User Experience & Scope

### In Scope
- **System tray icon and menu** with three mode items: "Voice-to-Claude" (Mode 1), "Dictation" (Mode 2), "Combined (Voice + Dictation)" (Mode 3). Each item is toggleable via double-click; active modes show a checkmark or highlight.
- **Mode management**: Starting a mode launches the corresponding pipeline process(es). Stopping a mode gracefully kills associated processes. Only one mode can be active at a time (matching existing `start.sh` behavior).
- **Status indicators**: Tray icon changes appearance (e.g., color, badge) to reflect idle vs. active vs. recording states.
- **Preferences window** accessible from the tray menu, containing:
  - Audio device selection (microphone input).
  - Whisper model selection (tiny/base/small/medium/large).
  - Home Assistant integration toggle with guided setup:
    - Docker installation check and install flow (Docker Desktop on macOS, Docker Desktop on Windows).
    - Home Assistant container provisioning and startup.
    - HA URL, entity, TTS entity, and Long-Lived Access Token configuration.
    - Connection test and validation.
  - Hotkey configuration (default F18/F19, user-customizable).
  - Launch at login toggle.
- **Dependency bundling or guided install**: Whisper CLI, ffmpeg, and Claude/Clawdbot CLI must either be bundled or the app must guide first-run installation.
- **Cross-platform support**: macOS (12+) and Windows (10/11).
- **App Store distribution**: Signed, sandboxed (where feasible), and packaged for macOS App Store and Microsoft Store.
- **Auto-update mechanism** via App Store update channels.

### Out of Scope
- Linux support (future consideration).
- Mobile (iOS/Android) companion apps.
- Custom wake-word detection (always-listening trigger without hotkey).
- Streaming/real-time transcription display in the UI.
- Multi-language UI localization (English only for initial release).
- Cloud-hosted Whisper (local Whisper CLI only).
- Building a new AI backend -- the app wraps the existing Claude/Clawdbot integration as-is.

---

## 5. Assumptions & Constraints

### Assumptions
- Users have a working microphone and grant the app microphone access permissions.
- On macOS, users grant Accessibility permissions for the global hotkey listener (required by the existing KeyListener.swift approach).
- The Whisper CLI, ffmpeg, and Claude/Clawdbot CLI can either be bundled within the app package or installed via a first-run setup wizard.
- App Store review will accept an app that manages subprocess lifecycles (shell scripts, ffmpeg recording) and optionally installs Docker.
- Users opting into Home Assistant integration are comfortable with Docker running on their machine.

### Constraints
- **macOS App Store sandboxing**: The app sandbox restricts subprocess execution, file system access, and network calls. Entitlements for microphone, accessibility, and outgoing network connections are required. The sandbox may limit the ability to run arbitrary shell scripts; the pipeline logic may need to be re-implemented as native code or use a helper tool installed outside the sandbox.
- **Windows platform gap**: The existing pipeline relies on macOS-specific tools (AVFoundation for audio capture, `osascript` for keystroke injection, `say` for TTS, `swiftc` for KeyListener). Windows equivalents must be implemented (e.g., Windows Audio Session API, SendInput for keystrokes, SAPI for TTS fallback).
- **Code signing**: Both platforms require paid developer accounts (Apple Developer Program $99/year, Microsoft Partner Center).
- **Docker installation**: Automating Docker Desktop installation requires elevated privileges and user consent; the flow must handle cases where Docker is already installed, partially installed, or installation fails.
- **Process lifecycle**: The tray app must reliably manage child processes (ffmpeg, whisper, claude CLI) across mode toggles and app restarts, ensuring no orphan processes remain.

---

## 6. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| **macOS App Store rejection due to sandboxing constraints** | High | High | Investigate XPC Services or a privileged helper tool pattern. Consider distributing outside the App Store (direct download + notarization) as a fallback. Research similar apps (e.g., audio tools, automation utilities) that have been approved. |
| **Windows port complexity underestimated** | Medium | High | Prioritize macOS as the primary platform. Use a cross-platform framework (Electron, Tauri, or .NET MAUI) to share UI code. Isolate platform-specific audio/input code behind clear abstractions. |
| **Docker installation flow fails on diverse user environments** | Medium | Medium | Provide clear error messages and manual fallback instructions. Test across macOS versions (Intel + Apple Silicon) and Windows editions. Include a "Skip" option that lets users install Docker manually. |
| **Whisper CLI performance varies by model and hardware** | Low | Medium | Default to the "base" model. Show estimated performance in Preferences. Warn users selecting larger models about resource usage. |
| **Child process orphaning on crash or forced quit** | Medium | Medium | Implement a watchdog that tracks PIDs and cleans up on launch. Use process groups so killing the parent kills all children. Register signal handlers for graceful shutdown. |
| **Global hotkey conflicts with other apps** | Low | Low | Make hotkeys user-configurable in Preferences. Document default keys (F18/F19) and suggest alternatives. |

---

## 7. Dependencies

### Internal Dependencies
- Existing voice pipeline scripts (`voice-pipeline.sh`, `dictate.sh`, `record.sh`, `stt.sh`, `claude-ask.sh`, `tts-ha.sh`).
- `KeyListener.swift` global hotkey listener (macOS) -- needs a Windows equivalent.
- `config.env` configuration format and values.

### External Dependencies
| Dependency | Purpose | Required? |
|-----------|---------|-----------|
| **Whisper CLI** (openai-whisper) | Speech-to-text transcription | Yes |
| **ffmpeg** | Audio recording and processing | Yes |
| **Claude / Clawdbot CLI** | AI agent interaction | Yes |
| **Docker Desktop** | Container runtime for Home Assistant | No (optional HA feature) |
| **Home Assistant** | Smart home platform for ElevenLabs TTS | No (optional) |
| **Apple Developer Account** | macOS App Store distribution and code signing | Yes (for macOS App Store) |
| **Microsoft Partner Center Account** | Microsoft Store distribution | Yes (for Windows Store) |
| **Cross-platform UI framework** (Tauri, Electron, or Swift+WinUI) | Tray app shell and Preferences window | Yes |

### Platform-Specific Dependencies
- **macOS**: AVFoundation (audio capture), Cocoa (event tap / hotkeys), `say` (TTS fallback), `osascript` (keystroke injection), `swiftc` (compile KeyListener).
- **Windows**: Windows Audio Session API or DirectShow (audio capture), Windows Input API / SendInput (keystroke injection), SAPI (TTS fallback), global keyboard hook (hotkeys).

---

## 8. Open Questions

1. **Framework choice**: Should the tray app use Tauri (Rust + web frontend, small binary), Electron (larger but proven ecosystem), or native per-platform (Swift for macOS, C#/WinUI for Windows)? Trade-offs between bundle size, development speed, and native feel need evaluation.
2. **App Store sandbox feasibility**: Can the pipeline's subprocess model (spawning ffmpeg, whisper, claude CLI) work within the macOS App Store sandbox, or will a helper tool / XPC service be required? Should we also offer direct download distribution as a parallel channel?
3. **Dependency bundling vs. installation**: Should Whisper, ffmpeg, and Claude CLI be bundled inside the app package (larger download, simpler UX) or installed separately via a first-run wizard (smaller app, more setup friction)?
4. **Mutual exclusivity of modes**: The current `start.sh` runs one mode at a time. Should the tray app enforce this (only one mode active), or allow Voice-to-Claude and Dictation to run simultaneously as independent toggles?
5. **Docker installation UX on Windows**: What is the cleanest way to automate Docker Desktop installation on Windows, given UAC prompts, WSL2 requirements, and Hyper-V dependencies?
6. **Pricing model**: Will the app be free, freemium, or paid? This affects App Store listing strategy and review expectations.
7. **Auto-launch behavior**: Should the app auto-launch a default mode on startup, or always start in idle state waiting for the user to toggle a mode?
8. **Windows keystroke injection**: The dictation feature uses `osascript` to type at cursor. What is the most reliable Windows equivalent that works across applications (SendInput, UI Automation, or clipboard-paste)?

---

## 9. Non-Goals

- **Building a new AI model or backend** -- the app wraps the existing Claude/Clawdbot CLI; no new AI capabilities are being developed.
- **Replacing Home Assistant** -- the app does not aim to be a smart home platform; HA integration is a pass-through for TTS routing.
- **Real-time streaming transcription UI** -- the app does not display a live transcript; it operates in batch mode (record, transcribe, respond).
- **Custom voice training or voice cloning** -- TTS output uses ElevenLabs via HA or system TTS as-is.
- **Plugin or extension system** -- the initial release does not support third-party plugins or custom pipeline stages.
- **Self-hosted cloud deployment** -- the app is a local desktop application only.
- **Accessibility audit / WCAG compliance** -- while best practices will be followed, a formal accessibility audit is not in scope for v1.

---

## 10. Notes & References

### Problem Description
This project originates from the need to wrap the existing Always-Listening voice pipeline (a collection of shell scripts for voice-to-AI and dictation workflows) into a native, distributable desktop application with system tray integration, toggleable modes, and an optional guided Home Assistant setup flow.

### Codebase References
- **Repository root:** `/Users/stoked/work/always-listening/`
- **Entry point:** `start.sh` -- orchestrates modes via `--voice-only`, `--dictate-only`, or default combined mode.
- **Core pipeline:** `voice-pipeline.sh` (record-transcribe-Claude-TTS loop), `dictate.sh` (record-transcribe-type-at-cursor).
- **Supporting scripts:** `record.sh` (ffmpeg audio capture with silence/signal detection), `stt.sh` (Whisper CLI wrapper), `claude-ask.sh` (Clawdbot agent CLI wrapper), `tts-ha.sh` (HA ElevenLabs TTS with macOS `say` fallback).
- **Hotkey listener:** `KeyListener.swift` (Cocoa event tap for F18/F19 global hotkeys).
- **Configuration:** `config.env` (audio device, Whisper model, HA URL/entities, silence thresholds).

### Technical Notes
- The pipeline currently uses FIFO (`/tmp/voice-pipeline/control.fifo`) for inter-process communication between KeyListener and the dictation daemon.
- Recording uses ffmpeg with AVFoundation on macOS; a Windows port will need an alternative audio capture backend.
- The TTS script falls back gracefully from Home Assistant ElevenLabs to macOS `say`; Windows will need SAPI or equivalent fallback.
- The Clawdbot CLI is invoked with `--json` output and parsed with Python; this dependency chain (Python 3 for JSON parsing) should be noted for bundling.
