# Product Requirements Document (Sequential)

## 0. Source Context
**Derived From:** Feature Brief
**Feature Name:** Always-Listening Voice Pipeline Tray App
**PRD Owner:** TBD
**Last Updated:** 2026-01-30

### Feature Brief Summary
A cross-platform (macOS and Windows) system tray application that wraps the existing Always-Listening voice pipeline, exposing its three operating modes (Voice-to-Claude, Dictation, and Combined) as toggleable menu items. The app provides a native, lightweight interface where users can double-click a mode to toggle it on or off, configure optional Home Assistant integration through a Preferences window, and install from the macOS App Store or Microsoft Store. The tray app replaces the current terminal-based workflow (`start.sh` with `--voice-only`, `--dictate-only`, or combined mode flags) with a polished GUI that manages pipeline subprocess lifecycles, global hotkey listeners, and audio recording transparently.

The existing pipeline consists of: `record.sh` (ffmpeg audio capture with F18 send-signal), `stt.sh` (Whisper CLI transcription), `claude-ask.sh` (Clawdbot agent invocation with JSON parsing), `tts-ha.sh` (Home Assistant ElevenLabs TTS with macOS `say` fallback), `dictate.sh` (record-transcribe-type-at-cursor via osascript), and `KeyListener.swift` (Cocoa CGEvent tap for F18/F19 global hotkeys). Configuration lives in `config.env` with audio device, Whisper model, HA connection details, and silence thresholds.

---

## 1. Objectives & Constraints

### Objectives
1. **Native tray app presence** -- Deliver a system tray application for macOS (12+) and Windows (10/11) that launches at startup, displays an icon in the system tray, and exposes a context menu for mode selection and preferences.
2. **One-click mode toggling** -- Users can start or stop any of the three pipeline modes (Voice-to-Claude, Dictation, Combined) with a single interaction. The tray icon and menu reflect the current state (idle, active, recording).
3. **Transparent subprocess management** -- The app manages all child processes (ffmpeg, Whisper CLI, Clawdbot CLI, KeyListener) with proper lifecycle handling: startup, graceful shutdown, orphan cleanup, and crash recovery.
4. **Preferences and configuration GUI** -- Replace manual `config.env` editing with a Preferences window for audio device selection, Whisper model, hotkey customization, launch-at-login, and optional Home Assistant integration.
5. **Home Assistant guided setup** -- Provide an optional, step-by-step wizard for Docker installation, Home Assistant container provisioning, entity configuration, and token management.
6. **App Store distribution** -- Package, sign, and submit the app to both the macOS App Store and Microsoft Store.

### Constraints
1. **macOS App Store sandbox** -- The sandbox restricts subprocess execution and filesystem access. Pipeline logic that spawns ffmpeg, Whisper, and Clawdbot CLI processes may require an XPC Service or privileged helper tool. Entitlements for microphone, accessibility (global hotkeys), and outgoing network connections are mandatory.
2. **Windows platform parity** -- The existing pipeline uses macOS-specific APIs: AVFoundation (audio capture), `osascript` (keystroke injection), `say` (TTS fallback), `swiftc`/Cocoa (KeyListener). Each requires a Windows equivalent (WASAPI/DirectShow, SendInput, SAPI, low-level keyboard hook).
3. **Code signing costs** -- Apple Developer Program ($99/year) and Microsoft Partner Center account are required for store distribution.
4. **Single mode at a time** -- Matching existing `start.sh` behavior, only one mode can be active at any time. Activating a new mode must cleanly stop the currently running mode first.
5. **Dependency availability** -- Whisper CLI, ffmpeg, python3, and Clawdbot CLI must be available on the user's system. The app must either bundle them or provide a first-run installation wizard.
6. **Process group management** -- All child processes must be placed in a process group so that terminating the parent cleanly terminates all children, preventing orphan ffmpeg or Whisper processes.

---

## 2. Execution Phases

> Phases below are ordered and sequential.

---

## Phase 1: Foundation
**Purpose:** Establish the project scaffolding, select and configure the cross-platform framework, and deliver a minimal tray app with an icon and static menu that runs on macOS and Windows.

### 1.1 Project Scaffolding and Framework Setup
Set up the cross-platform project using Tauri (Rust backend + web frontend). Tauri is selected for its small binary size (~10MB vs Electron's ~150MB), native system tray support, Rust-based process management capabilities, and suitability for App Store distribution. Initialize the repository structure, build toolchain, and CI pipeline.

**Implementation Details**
- Systems affected: New `tray-app/` directory within the existing repository root (`/Users/stoked/work/always-listening/`).
- Initialize a Tauri v2 project with TypeScript/React frontend and Rust backend.
- Configure `tauri.conf.json` with app metadata (name: "Always Listening", identifier: `com.alwayslistening.app`, version: `1.0.0`).
- Set up platform-specific build targets: macOS universal binary (x86_64 + aarch64) and Windows x64.
- Configure GitHub Actions CI with build/test jobs for both platforms.
- Add ESLint, Prettier, and Clippy linting for frontend and backend respectively.

**Acceptance Criteria**
- AC-1.1.a: Running `cargo tauri dev` on macOS launches a development build without errors.
- AC-1.1.b: Running `cargo tauri dev` on Windows launches a development build without errors.
- AC-1.1.c: Running `cargo tauri build` produces a `.dmg` on macOS and an `.msi` on Windows.
- AC-1.1.d: CI pipeline completes successfully on both platform targets.
- AC-1.1.e: Repository structure includes `tray-app/src-tauri/` (Rust backend), `tray-app/src/` (React frontend), and `tray-app/tauri.conf.json`.

**Acceptance Tests**
- Test-1.1.a: Clone the repo on a clean macOS machine, run `cargo tauri dev`, verify the app window appears.
- Test-1.1.b: Clone the repo on a clean Windows machine, run `cargo tauri dev`, verify the app window appears.
- Test-1.1.c: Push a commit to the CI branch, verify both macOS and Windows build jobs pass.
- Test-1.1.d: Run `cargo clippy` and `npm run lint` with zero warnings.

---

### 1.2 System Tray Icon and Static Menu
Implement the system tray icon with a static context menu displaying the three pipeline modes and utility items (Preferences, Quit). The modes are non-functional in this phase -- they display labels only. The tray icon must appear in the macOS menu bar and Windows system tray notification area.

**Implementation Details**
- Systems affected: Tauri Rust backend (`tray-app/src-tauri/src/main.rs` or `lib.rs`), Tauri config.
- Use Tauri's `SystemTray` API to register a tray icon with a context menu.
- Menu items: "Voice-to-Claude" (Mode 1), "Dictation" (Mode 2), "Combined" (Mode 3), separator, "Preferences...", separator, "Quit".
- Include three tray icon states as assets: idle (gray microphone), active (green microphone), recording (red microphone). Use PNG/ICO format per platform requirements.
- Wire the "Quit" menu item to gracefully exit the application.
- On macOS, the app should be an "agent" application (no Dock icon, menu bar only) by setting `LSUIElement = true` in `Info.plist`.

**Acceptance Criteria**
- AC-1.2.a: On macOS, a microphone icon appears in the menu bar when the app is running.
- AC-1.2.b: On Windows, a microphone icon appears in the system tray notification area when the app is running.
- AC-1.2.c: Clicking the tray icon displays a context menu with "Voice-to-Claude", "Dictation", "Combined", "Preferences...", and "Quit" items.
- AC-1.2.d: Clicking "Quit" terminates the application process cleanly (exit code 0).
- AC-1.2.e: On macOS, the application does not appear in the Dock.

**Acceptance Tests**
- Test-1.2.a: Launch the app on macOS, visually confirm the tray icon is present in the menu bar.
- Test-1.2.b: Launch the app on Windows, visually confirm the tray icon is present in the notification area.
- Test-1.2.c: Click the tray icon, verify all five menu items are visible with correct labels.
- Test-1.2.d: Click "Quit", verify the process exits and the tray icon disappears.
- Test-1.2.e: On macOS, verify that `LSUIElement` is set and no Dock icon appears.

---

### 1.3 Tray Icon State Management
Implement a state machine in the Rust backend that tracks the current app state (Idle, Active, Recording) and updates the tray icon and menu item appearances accordingly. Mode menu items display a checkmark when active. This establishes the event architecture that Phase 2 will connect to real pipeline processes.

**Implementation Details**
- Systems affected: Tauri Rust backend, tray icon assets.
- Define an enum `AppState { Idle, Active(Mode), Recording(Mode) }` where `Mode` is `VoiceToClaude | Dictation | Combined`.
- Clicking a mode menu item toggles between Idle and Active for that mode. If a different mode is already active, it transitions to the new mode (stop old, start new).
- The tray icon asset swaps to match the current state: gray (Idle), green (Active), red (Recording).
- Menu items for modes show a checkmark (native `NSMenuItem` state on macOS, equivalent on Windows) when that mode is active.
- Emit Tauri custom events (`mode-changed`, `state-changed`) so the frontend (Preferences window) can observe state changes.

**Acceptance Criteria**
- AC-1.3.a: Clicking "Voice-to-Claude" when idle sets that mode as active, displays a checkmark next to it, and changes the tray icon to green.
- AC-1.3.b: Clicking the already-active mode deactivates it, removes the checkmark, and returns the tray icon to gray.
- AC-1.3.c: Clicking a different mode while one is active deactivates the current mode and activates the new one atomically (no intermediate state visible to the user).
- AC-1.3.d: Only one mode can have a checkmark at any time, or none when idle.
- AC-1.3.e: State transitions complete in under 100ms (UI responsiveness, no pipeline process yet).

**Acceptance Tests**
- Test-1.3.a: Click "Voice-to-Claude", verify checkmark appears and icon turns green. Click again, verify checkmark removed and icon turns gray.
- Test-1.3.b: Activate "Dictation", then click "Combined". Verify "Dictation" checkmark is removed and "Combined" checkmark appears.
- Test-1.3.c: Activate each mode in sequence, verify only one checkmark is ever visible at a time.
- Test-1.3.d: Rapidly toggle modes 20 times, verify no visual glitches, double-checkmarks, or crashes.

---

### 1.4 Application Logging Infrastructure
Set up structured logging for the Rust backend and pipeline subprocess output. Logs are essential for debugging process lifecycle issues in later phases. Log to a rotating file in the platform-appropriate application support directory.

**Implementation Details**
- Systems affected: Tauri Rust backend.
- Use the `tracing` crate with `tracing-appender` for async file logging with daily rotation.
- Log directory: `~/Library/Logs/AlwaysListening/` on macOS, `%APPDATA%/AlwaysListening/logs/` on Windows.
- Log levels: `error`, `warn`, `info`, `debug`, `trace`. Default level: `info`. Configurable via environment variable `ALWAYS_LISTENING_LOG`.
- All subprocess stdout/stderr is captured and logged at `debug` level with a prefix identifying the source process (e.g., `[ffmpeg]`, `[whisper]`, `[clawdbot]`).
- Add a "Show Logs" item to the tray context menu that opens the log directory in Finder/Explorer.

**Acceptance Criteria**
- AC-1.4.a: On app startup, a log file is created at the platform-appropriate path with a timestamped entry.
- AC-1.4.b: State transitions (mode toggle, app quit) produce `info`-level log entries.
- AC-1.4.c: Log files rotate daily and retain the last 7 days of logs.
- AC-1.4.d: "Show Logs" menu item opens the log directory in the system file manager.
- AC-1.4.e: Setting `ALWAYS_LISTENING_LOG=debug` produces additional debug-level output.

**Acceptance Tests**
- Test-1.4.a: Launch the app, toggle a mode, quit. Open the log directory and verify the log file contains startup, mode-change, and shutdown entries.
- Test-1.4.b: Run the app for 2+ days (or mock the date), verify a new log file is created and old ones are retained.
- Test-1.4.c: Click "Show Logs" in the tray menu, verify Finder (macOS) or Explorer (Windows) opens to the correct directory.
- Test-1.4.d: Set the `ALWAYS_LISTENING_LOG=debug` environment variable, restart the app, verify additional log output.

---

## Phase 2: Core Pipeline Integration
**Purpose:** Connect the tray app state machine to the actual voice pipeline processes. Implement subprocess lifecycle management, inter-process communication, and platform-specific audio capture so that toggling a mode in the tray menu starts and stops the real pipeline.

### 2.1 Configuration Loading and Migration
Load configuration values from `config.env` (or a new JSON/TOML equivalent) into the Rust backend at startup. Provide defaults for all values so the app works out of the box. This replaces the `source "$SCRIPT_DIR/config.env"` pattern used by all existing shell scripts.

**Implementation Details**
- Systems affected: Tauri Rust backend, new config module.
- Define a `Config` struct in Rust matching all current `config.env` values: `audio_device`, `whisper_bin`, `claude_bin`, `ha_entity`, `ha_url`, `ha_tts_entity`, `record_duration`, `silence_threshold`, `silence_duration`, `whisper_model`, `whisper_language`, `tmp_dir`, `control_fifo`.
- Add tray-app-specific fields: `hotkey_send` (default: F18), `hotkey_dictate` (default: F19), `launch_at_login` (default: false), `ha_enabled` (default: false), `hass_token` (default: empty).
- Config file location: `~/Library/Application Support/AlwaysListening/config.toml` on macOS, `%APPDATA%/AlwaysListening/config.toml` on Windows.
- On first launch, if the legacy `config.env` file exists in the repository directory, migrate its values to the new `config.toml` format.
- Config changes are persisted immediately when modified through the Preferences window.

**Acceptance Criteria**
- AC-2.1.a: On first launch with no existing config, the app creates `config.toml` with sensible defaults and starts without error.
- AC-2.1.b: On first launch with a legacy `config.env` present, values are migrated to `config.toml` and the app uses the migrated values.
- AC-2.1.c: All `config.env` keys from the existing file are represented in the `Config` struct with documented defaults.
- AC-2.1.d: Modifying a config value programmatically persists the change to disk and is reflected on next read.
- AC-2.1.e: Invalid or missing config values fall back to defaults without crashing.

**Acceptance Tests**
- Test-2.1.a: Delete any existing config, launch the app, verify `config.toml` is created with all default values.
- Test-2.1.b: Place a `config.env` with custom values (e.g., `WHISPER_MODEL="large"`) alongside the app, launch, verify `config.toml` contains `whisper_model = "large"`.
- Test-2.1.c: Corrupt the `config.toml` (e.g., invalid TOML syntax), launch the app, verify it falls back to defaults and logs a warning.
- Test-2.1.d: Programmatically change `whisper_model` to `"small"`, restart the app, verify the persisted value is `"small"`.

---

### 2.2 Subprocess Manager and Process Group Handling
Implement a `ProcessManager` module in Rust that spawns, monitors, and terminates child processes. This is the core engine that replaces the shell-script orchestration in `start.sh`. It must handle process groups, PID tracking, graceful shutdown (SIGTERM then SIGKILL), and orphan cleanup on app start.

**Implementation Details**
- Systems affected: Tauri Rust backend, new `process_manager.rs` module.
- `ProcessManager` tracks all spawned child processes by name and PID.
- Processes are spawned in a new process group (`setsid` on Unix, job objects on Windows) so that killing the group terminates all descendants.
- `stop_all()` sends SIGTERM to each process group, waits up to 3 seconds, then sends SIGKILL to any remaining processes.
- On app startup, scan for stale PID files in `tmp_dir` and kill any orphaned processes from a previous crash.
- Stdout and stderr of child processes are captured asynchronously via `tokio::process::Command` and forwarded to the logging infrastructure with process-name prefixes.
- The module exposes: `spawn(name, command, args) -> Result<PID>`, `stop(name) -> Result<()>`, `stop_all() -> Result<()>`, `is_running(name) -> bool`.

**Acceptance Criteria**
- AC-2.2.a: `spawn("test", "sleep", ["30"])` starts a sleep process and `is_running("test")` returns `true`.
- AC-2.2.b: `stop("test")` terminates the sleep process within 4 seconds and `is_running("test")` returns `false`.
- AC-2.2.c: If a spawned process exits unexpectedly, the `ProcessManager` detects this within 1 second and updates its internal state.
- AC-2.2.d: On app startup, any orphaned processes from a previous session are detected and terminated.
- AC-2.2.e: `stop_all()` terminates all tracked processes within 5 seconds, including their child processes.
- AC-2.2.f: On Windows, process groups are managed via job objects with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.

**Acceptance Tests**
- Test-2.2.a: Spawn an ffmpeg process via `ProcessManager`, verify it appears in `ps` / Task Manager. Call `stop()`, verify it disappears.
- Test-2.2.b: Spawn a process that itself spawns children (e.g., a shell script that runs `sleep`), call `stop()`, verify all descendants are terminated.
- Test-2.2.c: Spawn a process, force-kill the tray app (SIGKILL / Task Manager End Process), relaunch the app, verify orphan cleanup runs and the stale process is terminated.
- Test-2.2.d: Spawn a process that exits on its own (e.g., `echo hello`), wait 2 seconds, verify `is_running()` returns `false`.

---

### 2.3 Voice-to-Claude Mode Integration
Wire the "Voice-to-Claude" mode menu item to launch and manage the voice pipeline loop: record audio via ffmpeg, transcribe via Whisper CLI, send to Clawdbot, and speak the response via TTS. This replicates the logic of `voice-pipeline.sh` using the `ProcessManager`.

**Implementation Details**
- Systems affected: Tauri Rust backend, `ProcessManager`, tray state machine.
- Activating Mode 1 starts a Rust async task that runs the record-transcribe-ask-speak loop.
- **Record step**: Spawn `ffmpeg -y -f avfoundation -i <audio_device> -ar 16000 -ac 1 -sample_fmt s16 <tmp_dir>/audio_input.wav` (macOS). On Windows, use `-f dshow` or `-f wasapi` with the configured device. Wait for F18 send signal (file-based signaling as in `record.sh`).
- **Transcribe step**: Spawn `whisper --model <model> --language <lang> --output_format txt --output_dir <tmp_dir> <audio_file>`. Parse the resulting `.txt` file for transcribed text. Skip blank/silent audio (matching `stt.sh` logic).
- **Ask step**: Spawn `clawdbot agent --agent main --session-id voice-pipeline --message "<prompt>" --json`. Parse the JSON response using `serde_json` to extract `result.payloads[0].text` (replacing the `python3 -c` call in `claude-ask.sh`).
- **TTS step**: If HA is configured, send HTTP POST to `<ha_url>/api/services/tts/speak` with the response text. Otherwise, use `say` on macOS or SAPI on Windows (matching `tts-ha.sh` fallback logic).
- **Exit detection**: Check transcribed text for "goodbye clawdbot" (case-insensitive) to stop the pipeline and return to Idle state.
- Loop continues until the user deactivates the mode via the tray menu or says the exit phrase.

**Acceptance Criteria**
- AC-2.3.a: Activating "Voice-to-Claude" begins the recording step -- the tray icon changes to red (recording state), and ffmpeg is running.
- AC-2.3.b: Pressing F18 stops recording, triggers transcription, and the transcribed text is logged.
- AC-2.3.c: Non-empty transcribed text is sent to Clawdbot, and the response is spoken via the configured TTS backend.
- AC-2.3.d: After TTS completes, the pipeline automatically returns to the recording step for the next utterance.
- AC-2.3.e: Deactivating the mode via the tray menu stops all pipeline processes within 3 seconds and returns to Idle.
- AC-2.3.f: Saying "goodbye clawdbot" stops the pipeline and returns to Idle, matching `voice-pipeline.sh` behavior.
- AC-2.3.g: Empty transcriptions or Clawdbot errors are handled gracefully with a fallback TTS message and loop continuation.

**Acceptance Tests**
- Test-2.3.a: Activate Mode 1, speak a question, press F18. Verify the response is spoken aloud and the pipeline returns to listening.
- Test-2.3.b: Activate Mode 1, press F18 without speaking. Verify the pipeline logs "No speech detected" and returns to listening.
- Test-2.3.c: Activate Mode 1, say "goodbye clawdbot", press F18. Verify the pipeline says "Goodbye!" and returns to Idle.
- Test-2.3.d: Activate Mode 1, then click "Voice-to-Claude" in the tray menu again. Verify the pipeline stops and the icon returns to gray.
- Test-2.3.e: Activate Mode 1, force-disconnect the network. Verify Clawdbot failure is handled gracefully (TTS says "Sorry, I didn't get a response") and the loop continues.

---

### 2.4 Dictation Mode Integration
Wire the "Dictation" mode menu item to launch the dictation workflow: record audio, transcribe, and type the text at the current cursor position. This replicates `dictate.sh` with `--submit` behavior.

**Implementation Details**
- Systems affected: Tauri Rust backend, `ProcessManager`, tray state machine.
- Activating Mode 2 starts a loop that waits for the F19 hotkey, then runs: record, transcribe, type-at-cursor.
- **Recording**: Same ffmpeg invocation as Mode 1, but output to `<tmp_dir>/dictation_audio.wav`. Triggered by F19 hotkey via FIFO or signal file.
- **Transcription**: Same Whisper invocation as Mode 1.
- **Type-at-cursor (macOS)**: Use `osascript -e 'tell application "System Events" to keystroke "<text>"'` with escaped quotes and backslashes (matching `dictate.sh` logic).
- **Type-at-cursor (Windows)**: Use the Windows `SendInput` API via Rust FFI or the `enigo` crate to simulate keystrokes.
- **Auto-submit**: After typing, simulate pressing Enter (matching the `--submit` flag behavior).
- The mode remains active and loops, waiting for the next F19 press, until deactivated via the tray menu.

**Acceptance Criteria**
- AC-2.4.a: Activating "Dictation" enables the F19 hotkey listener and changes the tray icon to green (active).
- AC-2.4.b: Pressing F19 begins recording (icon turns red). Speaking and pressing F18 stops recording and types the transcribed text at the cursor.
- AC-2.4.c: After typing, an Enter keystroke is simulated to submit the text.
- AC-2.4.d: The dictation loop returns to waiting for F19 after each cycle.
- AC-2.4.e: Deactivating the mode via the tray menu stops all processes and returns to Idle.
- AC-2.4.f: Special characters in the transcribed text (quotes, backslashes) are escaped correctly and typed accurately.

**Acceptance Tests**
- Test-2.4.a: Activate Mode 2, open a text editor, press F19, speak a sentence, press F18. Verify the text appears in the editor followed by a newline.
- Test-2.4.b: Dictate text containing quotes and apostrophes. Verify they are typed correctly without breaking the keystroke injection.
- Test-2.4.c: Activate Mode 2, then click "Dictation" in the tray menu. Verify the mode deactivates and F19 no longer triggers recording.
- Test-2.4.d: On Windows, activate Mode 2 and dictate into Notepad. Verify text is typed correctly using SendInput.

---

### 2.5 Combined Mode Integration
Wire the "Combined" mode menu item to run both Voice-to-Claude and Dictation concurrently, matching the behavior of `start.sh` with no flags (default mode 3). F18 controls the voice pipeline send signal, F19 triggers dictation.

**Implementation Details**
- Systems affected: Tauri Rust backend, `ProcessManager`, tray state machine, hotkey listener.
- Activating Mode 3 starts both the voice pipeline loop (2.3) and the dictation daemon (2.4) concurrently.
- The global hotkey listener (Rust native implementation replacing `KeyListener.swift`) dispatches F18 to the voice pipeline (create send signal file) and F19 to the dictation daemon (write "dictate" to FIFO or trigger via internal channel).
- On macOS, use `CGEvent.tapCreate` via Rust FFI (`core-graphics` crate) or the `rdev` crate for cross-platform global hotkey capture.
- On Windows, use `SetWindowsHookEx` with `WH_KEYBOARD_LL` or the `rdev` crate.
- Both sub-pipelines run as independent async tasks communicating through Rust channels, not FIFO files (eliminating the filesystem-based IPC of the shell version).
- Deactivation stops both pipelines atomically.

**Acceptance Criteria**
- AC-2.5.a: Activating "Combined" starts both the voice pipeline and dictation daemon, with the tray icon turning green.
- AC-2.5.b: F18 press during recording stops the voice recording and sends it for transcription/Claude processing.
- AC-2.5.c: F19 press triggers dictation recording, transcription, and type-at-cursor, independent of the voice pipeline state.
- AC-2.5.d: Both pipelines operate without interfering with each other (e.g., simultaneous recording does not corrupt audio files).
- AC-2.5.e: Deactivating Combined mode stops both pipelines and all associated processes within 3 seconds.
- AC-2.5.f: Global hotkeys work regardless of which application has focus.

**Acceptance Tests**
- Test-2.5.a: Activate Mode 3, use F18 to interact with Claude, then use F19 to dictate into a text editor. Verify both functions work independently.
- Test-2.5.b: Rapidly alternate between F18 and F19 presses. Verify no crashes, race conditions, or process corruption.
- Test-2.5.c: Deactivate Mode 3, verify both voice pipeline and dictation processes are terminated and F18/F19 no longer trigger recording.
- Test-2.5.d: Activate Mode 3 while another application has focus, press F19. Verify dictation text is typed into the focused application.

---

## Phase 3: Preferences & Configuration
**Purpose:** Build the Preferences window GUI, enabling users to view and modify all pipeline settings without editing files. Implement audio device enumeration, Whisper model selection, hotkey customization, and launch-at-login.

### 3.1 Preferences Window Shell
Create the Preferences window as a Tauri webview window with a tabbed interface. The window is opened from the "Preferences..." tray menu item. It uses React components styled to match platform conventions (macOS: sidebar tabs, Windows: top tabs).

**Implementation Details**
- Systems affected: Tauri frontend (React), Tauri Rust backend (window management).
- Create a multi-tab Preferences window with tabs: "General", "Audio", "Pipeline", "Home Assistant".
- The window opens as a modal, non-resizable, centered window (600x450px).
- Use Tauri's `WebviewWindow::new()` to create the window on demand; close it when the user clicks "Save" or "Cancel".
- Settings are loaded from the `Config` struct via Tauri commands (IPC bridge) and saved back on "Save".
- Add a "Restore Defaults" button that resets all settings to their default values.

**Acceptance Criteria**
- AC-3.1.a: Clicking "Preferences..." in the tray menu opens a Preferences window with four tabs.
- AC-3.1.b: The window displays current config values loaded from `config.toml`.
- AC-3.1.c: Clicking "Save" persists changes to `config.toml` and closes the window.
- AC-3.1.d: Clicking "Cancel" discards changes and closes the window.
- AC-3.1.e: "Restore Defaults" resets all fields to default values (but does not save until "Save" is clicked).
- AC-3.1.f: Only one Preferences window can be open at a time; clicking "Preferences..." again brings the existing window to front.

**Acceptance Tests**
- Test-3.1.a: Click "Preferences...", verify the window opens with all four tabs visible.
- Test-3.1.b: Modify a value, click "Cancel", reopen Preferences. Verify the original value is still shown.
- Test-3.1.c: Modify a value, click "Save", reopen Preferences. Verify the new value is persisted.
- Test-3.1.d: Click "Preferences..." twice rapidly. Verify only one window opens.
- Test-3.1.e: Click "Restore Defaults", verify all fields reset. Click "Cancel", reopen. Verify original values are intact.

---

### 3.2 General Tab: Launch at Login and Hotkey Configuration
Implement the "General" tab with launch-at-login toggle and hotkey customization for the F18 (send) and F19 (dictate) keys.

**Implementation Details**
- Systems affected: Tauri frontend (General tab component), Tauri Rust backend (auto-launch, hotkey config).
- **Launch at login (macOS)**: Use `SMAppService.mainApp` (macOS 13+) or a LaunchAgent plist for older versions to register/unregister login items.
- **Launch at login (Windows)**: Add/remove a registry entry at `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
- **Hotkey configuration**: Display the current hotkey assignments in a "press to record" input field. Users click the field and press a new key combination to rebind. Store the key code in config. Validate that the selected key is not a common system shortcut.
- Default hotkeys: F18 (send/stop recording), F19 (dictate trigger).

**Acceptance Criteria**
- AC-3.2.a: Toggling "Launch at Login" on/off correctly registers/unregisters the app as a login item on both platforms.
- AC-3.2.b: The hotkey fields display the current key assignments (e.g., "F18", "F19").
- AC-3.2.c: Clicking a hotkey field and pressing a new key updates the field to show the new key.
- AC-3.2.d: Saving hotkey changes updates the global hotkey listener to use the new keys.
- AC-3.2.e: Assigning the same key to both send and dictate shows a validation error and prevents saving.

**Acceptance Tests**
- Test-3.2.a: Enable "Launch at Login", log out and log back in. Verify the app starts automatically with the tray icon visible.
- Test-3.2.b: Disable "Launch at Login", restart the machine. Verify the app does not start automatically.
- Test-3.2.c: Change the send hotkey from F18 to F17, save. Activate Mode 1, verify F17 now stops recording (F18 no longer does).
- Test-3.2.d: Attempt to set both hotkeys to F18. Verify a validation error is displayed.

---

### 3.3 Audio Tab: Device Enumeration and Selection
Implement the "Audio" tab with a dropdown listing available audio input devices. The selected device is used for all ffmpeg recording commands.

**Implementation Details**
- Systems affected: Tauri frontend (Audio tab component), Tauri Rust backend (device enumeration).
- **macOS device enumeration**: Use `ffmpeg -f avfoundation -list_devices true -i ""` and parse the output to extract audio input devices with their AVFoundation indices (matching the `AUDIO_DEVICE=":1"` format in `config.env`).
- **Windows device enumeration**: Use `ffmpeg -f dshow -list_devices true -i dummy` and parse the output to extract audio input devices.
- Display devices in a dropdown with human-readable names. Store the device identifier (AVFoundation index or DirectShow name) in config.
- Show the currently selected device. If the configured device is no longer available, highlight it in red and prompt the user to select a new one.
- Include a "Test Microphone" button that records 2 seconds and plays it back (or shows a volume meter).

**Acceptance Criteria**
- AC-3.3.a: The Audio tab dropdown lists all available microphone input devices by name.
- AC-3.3.b: The currently configured device is pre-selected in the dropdown.
- AC-3.3.c: Selecting a different device and saving updates the config and subsequent recordings use the new device.
- AC-3.3.d: If the configured device is unavailable (e.g., USB mic unplugged), the dropdown shows a warning.
- AC-3.3.e: "Test Microphone" records a short clip and provides audible or visual feedback.

**Acceptance Tests**
- Test-3.3.a: Open the Audio tab with a USB microphone connected. Verify it appears in the dropdown.
- Test-3.3.b: Select the USB mic, save, activate Mode 1, speak, verify recording uses the selected device.
- Test-3.3.c: Unplug the USB mic, open the Audio tab. Verify a warning is shown for the missing device.
- Test-3.3.d: Click "Test Microphone", speak briefly. Verify feedback is provided (playback or volume indicator).

---

### 3.4 Pipeline Tab: Whisper Model and Transcription Settings
Implement the "Pipeline" tab with Whisper model selection, language setting, and silence detection threshold configuration.

**Implementation Details**
- Systems affected: Tauri frontend (Pipeline tab component), Tauri Rust backend.
- **Whisper model dropdown**: Options: tiny, base, small, medium, large. Display estimated VRAM/RAM and speed for each model. Default: base.
- **Language dropdown**: Common languages (English, Spanish, French, German, etc.) plus "Auto-detect". Default: English.
- **Silence threshold slider**: Range -60dB to -10dB. Default: -30dB (matching `config.env` `SILENCE_THRESHOLD`).
- **Silence duration field**: Seconds of silence before auto-stop. Range 0.5 to 5.0. Default: 1.5 (matching `config.env` `SILENCE_DURATION`).
- Display the paths to the Whisper binary and Claude CLI with validation indicators (green check if found, red X if missing).

**Acceptance Criteria**
- AC-3.4.a: The Pipeline tab displays the current Whisper model, language, silence threshold, and silence duration values from config.
- AC-3.4.b: Changing the Whisper model and saving causes subsequent transcriptions to use the new model.
- AC-3.4.c: The Whisper binary path shows a green indicator if the binary exists at the configured path, red if not.
- AC-3.4.d: The Claude CLI path shows a green indicator if the binary exists, red if not.
- AC-3.4.e: Changing the silence threshold affects recording behavior (lower threshold = more sensitive to silence).

**Acceptance Tests**
- Test-3.4.a: Change Whisper model from "base" to "small", save. Activate Mode 1, dictate. Verify logs show `--model small` in the Whisper invocation.
- Test-3.4.b: Remove the Whisper binary from its configured path. Open Pipeline tab. Verify the path shows a red indicator.
- Test-3.4.c: Set silence threshold to -50dB. Record in a quiet room. Verify recording does not auto-stop from ambient noise.
- Test-3.4.d: Set language to "Spanish", save. Speak Spanish, verify transcription is in Spanish.

---

## Phase 4: Home Assistant Integration
**Purpose:** Implement the optional Home Assistant integration flow in the Preferences window, including Docker status detection, HA container management, entity configuration, and connection testing.

### 4.1 Home Assistant Tab: Connection Configuration
Implement the "Home Assistant" tab with fields for HA URL, Long-Lived Access Token, media player entity, and TTS entity. Include a connection test button.

**Implementation Details**
- Systems affected: Tauri frontend (Home Assistant tab component), Tauri Rust backend (HTTP client).
- The tab is gated by an "Enable Home Assistant Integration" toggle. When disabled, all HA fields are grayed out and TTS falls back to local (`say`/SAPI).
- Fields: HA URL (text input, default `http://localhost:8123`), Long-Lived Access Token (password input), Media Player Entity (text input, e.g., `media_player.office_speaker`), TTS Entity (text input, e.g., `tts.elevenlabs_text_to_speech`).
- **Connection Test**: Send `GET <ha_url>/api/` with `Authorization: Bearer <token>` header. Expect 200 response with `{"message": "API running."}`. Display success/failure with details.
- **Entity validation**: After successful connection, query `GET <ha_url>/api/states/<entity>` to verify the media player and TTS entities exist. Show green check or red X next to each entity field.
- Store `hass_token` securely using the platform keychain (macOS Keychain via `security` CLI, Windows Credential Manager via `wincred` crate) rather than plaintext in config.toml.

**Acceptance Criteria**
- AC-4.1.a: The HA toggle enables/disables all HA configuration fields.
- AC-4.1.b: "Test Connection" with valid credentials shows a green "Connected" status.
- AC-4.1.c: "Test Connection" with invalid URL or token shows a red error message with details (e.g., "Connection refused", "401 Unauthorized").
- AC-4.1.d: Entity validation shows per-entity status (valid/invalid) after a successful connection test.
- AC-4.1.e: The HA token is stored in the platform keychain, not in `config.toml`.
- AC-4.1.f: With HA enabled and configured, Mode 1 TTS uses the HA ElevenLabs endpoint. With HA disabled, TTS uses local `say`/SAPI.

**Acceptance Tests**
- Test-4.1.a: Enable HA, enter a valid URL and token, click "Test Connection". Verify green "Connected" status.
- Test-4.1.b: Enter an invalid token, click "Test Connection". Verify red "401 Unauthorized" error.
- Test-4.1.c: Enter a valid entity name, verify green check. Enter a nonexistent entity, verify red X.
- Test-4.1.d: Enable HA with valid config, activate Mode 1, speak a question. Verify TTS plays through the HA media player entity.
- Test-4.1.e: Disable HA, activate Mode 1, speak a question. Verify TTS uses local `say` (macOS) or SAPI (Windows).
- Test-4.1.f: Inspect `config.toml` after saving HA settings. Verify the token is not stored in the file. Verify it is in the system keychain.

---

### 4.2 Docker Status Detection and Installation Guidance
Detect whether Docker Desktop is installed and running. Provide installation guidance for users who want to run Home Assistant locally via Docker.

**Implementation Details**
- Systems affected: Tauri Rust backend (Docker detection), Tauri frontend (HA tab Docker section).
- **Docker detection**: Run `docker info` and parse the output. States: Not Installed, Installed but Not Running, Running.
- Display Docker status in the HA tab with appropriate messaging and action buttons:
  - Not Installed: Show "Docker is required for local Home Assistant" with a "Download Docker Desktop" button that opens `https://www.docker.com/products/docker-desktop/` in the default browser.
  - Installed but Not Running: Show "Docker is installed but not running" with a "Start Docker Desktop" button (launch the Docker Desktop app).
  - Running: Show a green "Docker is running" status.
- Do not attempt to automatically install Docker -- this requires elevated privileges and varies too much across environments. Instead, provide clear guidance and a link.
- After Docker is detected as running, enable the "Set Up Home Assistant" wizard button.

**Acceptance Criteria**
- AC-4.2.a: When Docker is not installed, the status shows "Not Installed" with a download link.
- AC-4.2.b: When Docker is installed but not running, the status shows "Not Running" with a start button.
- AC-4.2.c: When Docker is running, the status shows "Running" with a green indicator.
- AC-4.2.d: The "Download Docker Desktop" button opens the Docker website in the default browser.
- AC-4.2.e: Docker status refreshes automatically every 10 seconds while the HA tab is visible.

**Acceptance Tests**
- Test-4.2.a: On a machine without Docker, open the HA tab. Verify "Not Installed" status and download link.
- Test-4.2.b: Install Docker but do not start it. Open the HA tab. Verify "Not Running" status.
- Test-4.2.c: Start Docker Desktop, open the HA tab. Verify "Running" status with green indicator.
- Test-4.2.d: Click "Download Docker Desktop". Verify the browser opens to the Docker download page.

---

### 4.3 Home Assistant Container Setup Wizard
Provide a step-by-step wizard that provisions a Home Assistant Docker container, waits for it to start, and guides the user through initial HA configuration (creating an account, generating a Long-Lived Access Token, and configuring ElevenLabs TTS and media player entities).

**Implementation Details**
- Systems affected: Tauri Rust backend (Docker commands), Tauri frontend (wizard UI).
- The wizard is a multi-step modal flow (5 steps):
  1. **Container Check**: Check if a container named `homeassistant` already exists. If yes, offer to start it or recreate it.
  2. **Container Creation**: Run `docker run -d --name homeassistant --privileged --restart=unless-stopped -v <config_path>:/config -p 8123:8123 ghcr.io/home-assistant/home-assistant:stable`. Display progress.
  3. **Wait for HA**: Poll `http://localhost:8123/api/` every 5 seconds until HA responds (up to 120 seconds timeout). Show a spinner.
  4. **Account Setup**: Open `http://localhost:8123` in the browser and instruct the user to create their admin account and generate a Long-Lived Access Token at `/profile/security`. Provide a text field for them to paste the token.
  5. **Entity Configuration**: Instruct the user to install the ElevenLabs TTS integration in HA and configure a Google Cast media player. Provide fields for entity IDs and run validation.
- Each step has "Back" and "Next" buttons (where applicable). The wizard can be cancelled at any step.
- On completion, save all HA settings to config and enable HA integration.

**Acceptance Criteria**
- AC-4.3.a: The wizard detects an existing `homeassistant` container and offers appropriate options (start/recreate).
- AC-4.3.b: Container creation runs the correct `docker run` command and displays progress output.
- AC-4.3.c: The wizard waits for HA to become responsive, showing a spinner, and proceeds automatically when HA responds.
- AC-4.3.d: The wizard times out after 120 seconds if HA does not respond, showing an error with troubleshooting suggestions.
- AC-4.3.e: On completion, all HA settings are saved and a connection test passes automatically.
- AC-4.3.f: Cancelling the wizard at any step does not leave partial configuration in the saved settings.

**Acceptance Tests**
- Test-4.3.a: Run the wizard on a machine with Docker running and no existing HA container. Verify the container is created and HA becomes accessible at `http://localhost:8123`.
- Test-4.3.b: Run the wizard when an existing `homeassistant` container exists but is stopped. Verify the wizard offers to start it.
- Test-4.3.c: Complete the full wizard flow, paste a valid token, enter entity IDs. Verify HA settings are saved and the connection test passes.
- Test-4.3.d: Cancel the wizard at step 3. Verify no HA settings are saved to config.
- Test-4.3.e: Stop Docker during step 3. Verify the wizard shows a timeout error with guidance.

---

## Phase 5: App Store Preparation
**Purpose:** Prepare the application for distribution through the macOS App Store and Microsoft Store, including code signing, sandboxing, packaging, and submission.

### 5.1 macOS Code Signing and Entitlements
Configure the macOS build for code signing with an Apple Developer certificate and set up the required entitlements for App Store submission.

**Implementation Details**
- Systems affected: Tauri build configuration, macOS entitlements plist, CI pipeline.
- Obtain or configure an Apple Developer certificate (Developer ID Application for direct distribution, Apple Distribution for App Store).
- Create `entitlements.plist` with required entitlements:
  - `com.apple.security.app-sandbox` = `true`
  - `com.apple.security.device.audio-input` = `true` (microphone access)
  - `com.apple.security.automation.apple-events` = `true` (System Events for keystroke injection)
  - `com.apple.security.network.client` = `true` (outgoing network for HA, Clawdbot)
  - `com.apple.security.temporary-exception.mach-lookup.global-name` (if needed for accessibility)
  - `com.apple.security.files.user-selected.read-write` = `true` (temp file access)
- Configure Tauri to use the entitlements file during signing: `tauri.conf.json` > `bundle` > `macOS` > `entitlements`.
- Set up provisioning profiles and App Store Connect metadata.
- If the sandbox prevents subprocess execution (ffmpeg, whisper, claude CLI), implement an XPC Service helper that runs outside the sandbox and communicates with the main app via XPC.

**Acceptance Criteria**
- AC-5.1.a: The macOS build is signed with a valid Apple Developer certificate.
- AC-5.1.b: The app runs correctly with all entitlements applied (microphone, network, accessibility).
- AC-5.1.c: The app passes `codesign --verify --deep --strict` validation.
- AC-5.1.d: The app passes `spctl --assess --type execute` (Gatekeeper validation).
- AC-5.1.e: If an XPC Service is required, it is bundled correctly and communicates with the main app for subprocess execution.

**Acceptance Tests**
- Test-5.1.a: Build the app with signing, run `codesign --verify --deep --strict <app_path>`. Verify no errors.
- Test-5.1.b: Download the signed app on a different Mac, launch it. Verify Gatekeeper does not block it.
- Test-5.1.c: Launch the signed app, activate Mode 1. Verify microphone permission prompt appears and recording works after granting.
- Test-5.1.d: Launch the signed app, activate Mode 2 (dictation). Verify accessibility permission prompt appears and keystroke injection works after granting.

---

### 5.2 Windows Code Signing and MSIX Packaging
Configure the Windows build for code signing and MSIX packaging for Microsoft Store submission.

**Implementation Details**
- Systems affected: Tauri build configuration, Windows signing certificate, CI pipeline.
- Obtain an EV or standard code signing certificate from a certificate authority, or use the Microsoft Partner Center to sign MSIX packages.
- Configure Tauri to produce MSIX packages: `tauri.conf.json` > `bundle` > `windows` > `wix` or `nsis` settings, plus MSIX manifest.
- Create `AppxManifest.xml` with required capabilities:
  - `microphone` capability for audio recording.
  - `internetClient` capability for HA and Clawdbot network access.
  - `inputInjection` or `uiAccess` if needed for SendInput keystroke injection.
- Set up the Microsoft Partner Center app listing with screenshots, descriptions, and privacy policy.
- Test the MSIX package on clean Windows 10 and Windows 11 installations.

**Acceptance Criteria**
- AC-5.2.a: The Windows build produces a signed MSIX package.
- AC-5.2.b: The MSIX package installs correctly on Windows 10 and Windows 11 via double-click.
- AC-5.2.c: The installed app appears in the Start menu and system tray on launch.
- AC-5.2.d: All capabilities (microphone, network, keyboard input) work correctly after installation.
- AC-5.2.e: The MSIX package passes the Windows App Certification Kit (WACK) tests.

**Acceptance Tests**
- Test-5.2.a: Build the MSIX package, install it on a clean Windows 10 VM. Verify it installs without errors.
- Test-5.2.b: Launch the installed app, activate Mode 1. Verify microphone access works.
- Test-5.2.c: Activate Mode 2 (dictation), dictate into Notepad. Verify keystroke injection works.
- Test-5.2.d: Run WACK on the MSIX package. Verify all tests pass.

---

### 5.3 First-Run Setup Wizard and Dependency Verification
Implement a first-run wizard that verifies or installs required dependencies (Whisper CLI, ffmpeg, Clawdbot CLI, python3) and guides the user through initial configuration.

**Implementation Details**
- Systems affected: Tauri frontend (setup wizard UI), Tauri Rust backend (dependency detection).
- On first launch (no `config.toml` exists), display a setup wizard instead of going directly to idle state.
- **Step 1 - Welcome**: Brief explanation of the app and its three modes.
- **Step 2 - Dependency Check**: Scan for required binaries (`whisper`, `ffmpeg`, `claude`/`clawdbot`, `python3`). Display each with a status icon (found/not found). For missing dependencies, provide platform-specific installation instructions:
  - macOS: `brew install ffmpeg`, `pip install openai-whisper`, Clawdbot download link.
  - Windows: `winget install ffmpeg`, `pip install openai-whisper`, Clawdbot download link.
- **Step 3 - Audio Device**: Pre-select the default microphone or let the user choose (reuses Audio tab logic from 3.3).
- **Step 4 - Permissions (macOS only)**: Guide the user to grant Microphone and Accessibility permissions. Show real-time status of each permission.
- **Step 5 - Ready**: Summarize the configuration and offer to start in a selected mode or idle.
- The wizard can be re-run from the tray menu ("Run Setup Wizard...").

**Acceptance Criteria**
- AC-5.3.a: On first launch, the setup wizard opens automatically.
- AC-5.3.b: Dependency check correctly identifies installed and missing binaries.
- AC-5.3.c: Installation instructions are platform-specific and accurate.
- AC-5.3.d: On macOS, the permissions step shows real-time status of Microphone and Accessibility grants.
- AC-5.3.e: After completing the wizard, the app is fully configured and ready to use.
- AC-5.3.f: The wizard does not appear on subsequent launches (unless re-run from the menu).

**Acceptance Tests**
- Test-5.3.a: Delete `config.toml`, launch the app. Verify the setup wizard appears.
- Test-5.3.b: On a machine missing ffmpeg, run the setup wizard. Verify ffmpeg shows as "Not Found" with install instructions.
- Test-5.3.c: Complete the setup wizard, quit, relaunch. Verify the wizard does not appear again.
- Test-5.3.d: Click "Run Setup Wizard..." from the tray menu. Verify the wizard opens.
- Test-5.3.e: On macOS, deny microphone permission during the wizard. Verify the status shows "Denied" with instructions to grant it in System Settings.

---

### 5.4 App Store Submission and Review
Submit the application to the macOS App Store and Microsoft Store, addressing review feedback and ensuring compliance with store guidelines.

**Implementation Details**
- Systems affected: App Store Connect, Microsoft Partner Center, CI pipeline.
- **macOS App Store submission**:
  - Upload the signed `.app` bundle via Xcode or `altool`/`notarytool`.
  - Provide App Store metadata: app name, description, keywords, screenshots (tray menu, Preferences window, HA wizard), privacy policy URL, support URL.
  - Address potential review concerns: explain subprocess usage in the review notes, justify accessibility entitlement for global hotkeys, clarify Docker guidance is educational (not auto-installing).
  - Category: Productivity or Utilities.
- **Microsoft Store submission**:
  - Upload the signed MSIX package via Microsoft Partner Center.
  - Provide store listing metadata: description, screenshots, privacy policy.
  - Category: Productivity.
- **Privacy policy**: Create a privacy policy page covering: local-only audio processing (no cloud upload except to Clawdbot API), optional HA connection, no analytics or tracking.
- **Review response plan**: Pre-draft responses to anticipated rejections (sandbox subprocess concerns, accessibility justification).

**Acceptance Criteria**
- AC-5.4.a: The macOS App Store listing is submitted with complete metadata, screenshots, and privacy policy.
- AC-5.4.b: The Microsoft Store listing is submitted with complete metadata, screenshots, and privacy policy.
- AC-5.4.c: The app is accepted on both stores within 2 submission attempts.
- AC-5.4.d: The privacy policy accurately describes data handling practices.
- AC-5.4.e: Store listings include at least 3 screenshots showing the tray menu, Preferences window, and HA wizard.

**Acceptance Tests**
- Test-5.4.a: Download the app from the macOS App Store on a test device. Verify it installs and runs correctly.
- Test-5.4.b: Download the app from the Microsoft Store on a test device. Verify it installs and runs correctly.
- Test-5.4.c: Verify the privacy policy URL is accessible and accurate.
- Test-5.4.d: Verify all screenshots in the store listings match the current app version.

---

## 3. Completion Criteria

The project is considered complete when all of the following are satisfied:

1. **Functional completeness**: All three pipeline modes (Voice-to-Claude, Dictation, Combined) work correctly when toggled from the tray menu on both macOS and Windows.
2. **Preferences window**: All four Preferences tabs (General, Audio, Pipeline, Home Assistant) are functional with settings persistence.
3. **Home Assistant integration**: The optional HA setup wizard creates a Docker container, configures HA connection details, and validates entity availability. TTS routes through HA when enabled.
4. **Process lifecycle**: The app reliably starts, stops, and cleans up all child processes across mode toggles, app restarts, and crash recovery scenarios.
5. **First-run experience**: The setup wizard correctly identifies missing dependencies, guides permission grants, and produces a working configuration.
6. **Store distribution**: The app is accepted and published on both the macOS App Store and Microsoft Store.
7. **Logging**: All pipeline activity, errors, and state transitions are logged to rotating log files accessible from the tray menu.
8. **No regressions**: The tray app produces identical pipeline behavior to the existing shell scripts (`start.sh`, `voice-pipeline.sh`, `dictate.sh`) for all three modes.

---

## 4. Rollout & Validation

### Pre-Release Validation
1. **Internal alpha testing** (Phases 1-2 complete): Test on macOS (Intel + Apple Silicon) and Windows (10 + 11) with all three modes. Verify subprocess lifecycle, hotkey responsiveness, and TTS output.
2. **Beta testing** (Phases 1-3 complete): Distribute via TestFlight (macOS) and sideloaded MSIX (Windows) to 5-10 external testers. Collect feedback on Preferences UX, audio device compatibility, and first-run experience.
3. **HA integration testing** (Phase 4 complete): Test the Docker/HA wizard on fresh macOS and Windows installations with varying Docker states (not installed, installed but stopped, running). Verify ElevenLabs TTS routing end-to-end.
4. **Store review dry-run** (Phase 5): Submit to both stores and iterate on any review feedback.

### Rollout Plan
1. **Week 1-2**: macOS App Store and Microsoft Store submissions.
2. **Week 3**: Address any store review feedback and resubmit if needed.
3. **Week 4**: Public availability. Monitor crash reports and user feedback.
4. **Ongoing**: Bug fixes and minor updates via store update channels.

### Monitoring
- Monitor App Store crash reports (Xcode Organizer, Partner Center Health).
- Track store ratings and review content for common issues.
- Monitor download counts against adoption targets.
- Log collection (opt-in) for diagnosing reported issues.

---

## 5. Open Questions

1. **Framework confirmation**: This PRD assumes Tauri v2 as the cross-platform framework. If evaluation reveals Tauri limitations (e.g., system tray issues on Windows, XPC Service support), Electron or native per-platform builds should be reconsidered. Decision needed before Phase 1 implementation begins.

2. **App Store sandbox feasibility**: Can the macOS sandbox accommodate spawning ffmpeg, Whisper CLI, and Clawdbot CLI as child processes? If not, the XPC Service approach in work item 5.1 becomes critical-path. Early prototyping (during Phase 1) is recommended.

3. **Dependency bundling strategy**: Should Whisper, ffmpeg, and Clawdbot CLI be bundled inside the app package (simpler UX, larger download ~500MB+) or installed separately via the first-run wizard (smaller app, more friction)? This affects both store review and user experience. The current PRD assumes external installation with guided setup.

4. **Pricing model**: Free, freemium, or paid? This affects App Store listing configuration and should be decided before Phase 5 submission.

5. **Mode mutual exclusivity**: The current design enforces one mode at a time (matching `start.sh`). Should Voice-to-Claude and Dictation be allowed to run simultaneously as independent toggles in a future version?

6. **Windows keystroke injection reliability**: The `SendInput` API may not work in all applications (e.g., elevated/UWP apps). Should a clipboard-paste fallback be implemented for dictation mode on Windows?

7. **Auto-launch mode**: Should the app auto-start a default mode on login, or always start in idle state? The current PRD defaults to idle.

8. **Direct download distribution**: Should the app also be distributed as a direct download (notarized `.dmg` on macOS, standalone `.msi` on Windows) in addition to store distribution, as a fallback if store sandbox constraints prove too limiting?
