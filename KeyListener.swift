// KeyListener.swift — Global hotkey listener (F18=send, F19=dictate)
// Compile: swiftc -o KeyListener KeyListener.swift -framework Cocoa
// Requires: Accessibility permission in System Settings → Privacy & Security

import Cocoa

let controlFifo = "/tmp/voice-pipeline/control.fifo"
let sendSignal = "/tmp/voice-pipeline/send.signal"

// CGEvent callback for global key events
func eventCallback(
    proxy: CGEventTapProxy,
    type: CGEventType,
    event: CGEvent,
    refcon: UnsafeMutableRawPointer?
) -> Unmanaged<CGEvent>? {
    if type == .keyDown {
        let keyCode = event.getIntegerValueField(.keyboardEventKeycode)
        // F18 = keycode 79 — stop recording and send
        if keyCode == 79 {
            FileManager.default.createFile(atPath: sendSignal, contents: nil)
            fputs("[KeyListener] F18 pressed — send signal\n", stdout)
            return nil
        }
        // F19 = keycode 80 — dictate at cursor
        if keyCode == 80 {
            writeToPipe("dictate")
            return nil // consume the event
        }
    }

    // Handle tap disabled events (re-enable tap)
    if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
        if let refcon = refcon {
            let tap = Unmanaged<CFMachPort>.fromOpaque(refcon).takeUnretainedValue()
            CGEvent.tapEnable(tap: tap, enable: true)
        }
    }

    return Unmanaged.passRetained(event)
}

func writeToPipe(_ message: String) {
    // Ensure FIFO exists
    let fm = FileManager.default
    if !fm.fileExists(atPath: controlFifo) {
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/usr/bin/mkfifo")
        task.arguments = [controlFifo]
        try? task.run()
        task.waitUntilExit()
    }

    // Write to FIFO (non-blocking via DispatchQueue to avoid blocking the event tap)
    DispatchQueue.global().async {
        if let fh = FileHandle(forWritingAtPath: controlFifo) {
            if let data = (message + "\n").data(using: .utf8) {
                fh.write(data)
            }
            fh.closeFile()
        }
    }
}

// Create event tap
let eventMask = (1 << CGEventType.keyDown.rawValue)

guard let tap = CGEvent.tapCreate(
    tap: .cgSessionEventTap,
    place: .headInsertEventTap,
    options: .defaultTap,
    eventsOfInterest: CGEventMask(eventMask),
    callback: eventCallback,
    userInfo: nil
) else {
    fputs("Error: Failed to create event tap.\n", stderr)
    fputs("Grant Accessibility permission in System Settings → Privacy & Security → Accessibility\n", stderr)
    exit(1)
}

// Pass tap reference for re-enable on timeout
let tapPtr = Unmanaged.passUnretained(tap).toOpaque()

let runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0)
CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource, .commonModes)
CGEvent.tapEnable(tap: tap, enable: true)

print("[KeyListener] Listening for F18 (send) and F19 (dictate)... Press Ctrl+C to stop.")
print("[KeyListener] Requires Accessibility permission in System Settings")

CFRunLoopRun()
