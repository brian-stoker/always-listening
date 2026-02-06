import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export default function LogsTab() {
  const [lines, setLines] = useState<string[]>([]);
  const containerRef = useRef<HTMLDivElement>(null);
  const autoScrollRef = useRef(true);

  // Load recent logs on mount
  useEffect(() => {
    invoke<string[]>("get_recent_logs", { lines: 500 })
      .then((recent) => setLines(recent))
      .catch((err) => console.error("Failed to load logs:", err));
  }, []);

  // Subscribe to real-time log lines
  useEffect(() => {
    const unlisten = listen<string>("log-line", (event) => {
      setLines((prev) => {
        const next = [...prev, event.payload];
        // Keep buffer bounded
        if (next.length > 2000) {
          return next.slice(next.length - 1500);
        }
        return next;
      });
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Auto-scroll to bottom when new lines arrive
  useEffect(() => {
    if (autoScrollRef.current && containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [lines]);

  // Detect if user scrolled away from bottom
  function handleScroll() {
    if (!containerRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } = containerRef.current;
    autoScrollRef.current = scrollHeight - scrollTop - clientHeight < 40;
  }

  function handleClear() {
    setLines([]);
  }

  return (
    <div className="tab-content logs-tab">
      <div className="logs-header">
        <h2>Logs</h2>
        <button className="btn btn-secondary btn-small" onClick={handleClear}>
          Clear
        </button>
      </div>
      <div
        className="log-viewer"
        ref={containerRef}
        onScroll={handleScroll}
      >
        {lines.map((line, i) => (
          <div key={i} className="log-line">
            {line}
          </div>
        ))}
        {lines.length === 0 && (
          <div className="log-empty">No log output yet.</div>
        )}
      </div>
    </div>
  );
}
