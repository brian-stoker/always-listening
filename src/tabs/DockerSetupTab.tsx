import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Config } from "../types";

interface DockerSetupTabProps {
  config: Config;
  onChange: (updates: Partial<Config>) => void;
}

interface DockerStatus {
  installed: boolean;
  version: string;
}

interface HaContainerStatus {
  exists: boolean;
  running: boolean;
  status: string;
}

function StatusBadge({ ok, warn }: { ok: boolean; warn?: boolean }) {
  const cls = ok ? "ok" : warn ? "warn" : "fail";
  return (
    <span className={`status-badge ${cls}`}>
      {ok ? "\u2713" : "\u2717"}
    </span>
  );
}

export default function DockerSetupTab({ config, onChange }: DockerSetupTabProps) {
  const [dockerStatus, setDockerStatus] = useState<DockerStatus | null>(null);
  const [haStatus, setHaStatus] = useState<HaContainerStatus | null>(null);
  const [configPath, setConfigPath] = useState("~/.homeassistant");
  const [loading, setLoading] = useState(false);
  const [actionMessage, setActionMessage] = useState("");
  const [actionError, setActionError] = useState("");

  const checkDocker = useCallback(async () => {
    try {
      const status = await invoke<DockerStatus>("check_docker");
      setDockerStatus(status);
      return status.installed;
    } catch {
      setDockerStatus({ installed: false, version: "" });
      return false;
    }
  }, []);

  const checkHaContainer = useCallback(async () => {
    try {
      const status = await invoke<HaContainerStatus>("check_ha_container_status");
      setHaStatus(status);
    } catch {
      setHaStatus({ exists: false, running: false, status: "Check failed" });
    }
  }, []);

  const refreshAll = useCallback(async () => {
    setActionMessage("");
    setActionError("");
    const dockerInstalled = await checkDocker();
    if (dockerInstalled) {
      await checkHaContainer();
    }
  }, [checkDocker, checkHaContainer]);

  useEffect(() => {
    refreshAll();
  }, [refreshAll]);

  const handleSetupHa = async () => {
    setLoading(true);
    setActionMessage("");
    setActionError("");
    try {
      const result = await invoke<string>("setup_ha", { configPath });
      setActionMessage(result);
      await checkHaContainer();
    } catch (err) {
      setActionError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleStartHa = async () => {
    setLoading(true);
    setActionMessage("");
    setActionError("");
    try {
      const result = await invoke<string>("start_ha");
      setActionMessage(result);
      await checkHaContainer();
    } catch (err) {
      setActionError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="tab-content docker-setup-tab">
      <h2>Docker & Home Assistant</h2>

      <section className="settings-section">
        <button className="btn btn-secondary" onClick={refreshAll} disabled={loading}>
          Refresh Status
        </button>
      </section>

      {/* Step 1: Docker */}
      <div className="step-card">
        <div className="step-header">
          <StatusBadge ok={dockerStatus?.installed ?? false} />
          Step 1: Docker Installation
        </div>
        <div className="step-body">
          {dockerStatus === null ? (
            <p className="status-text muted">Checking...</p>
          ) : dockerStatus.installed ? (
            <p className="status-text success">{dockerStatus.version}</p>
          ) : (
            <p className="status-text error">
              Docker is not installed.{" "}
              <a href="https://www.docker.com/products/docker-desktop/" target="_blank" rel="noreferrer">
                Download Docker Desktop
              </a>
            </p>
          )}
        </div>
      </div>

      {/* Step 2: Container */}
      <div className="step-card">
        <div className="step-header">
          <StatusBadge
            ok={haStatus?.running ?? false}
            warn={haStatus?.exists && !haStatus?.running}
          />
          Step 2: Home Assistant Container
        </div>
        <div className="step-body">
          {!dockerStatus?.installed ? (
            <p className="status-text muted">Install Docker first</p>
          ) : haStatus === null ? (
            <p className="status-text muted">Checking...</p>
          ) : haStatus.running ? (
            <p className="status-text success">Running: {haStatus.status}</p>
          ) : haStatus.exists ? (
            <p className="status-text warning">Stopped: {haStatus.status}</p>
          ) : (
            <p className="status-text error">No container found</p>
          )}
        </div>
      </div>

      {/* Step 3: Setup / Control */}
      <div className="step-card">
        <div className="step-header">
          <StatusBadge ok={haStatus?.running ?? false} />
          Step 3: Setup / Control
        </div>
        <div className="step-body">
          {dockerStatus?.installed && !haStatus?.exists && (
            <>
              <div className="field">
                <label htmlFor="ha-config-path">HA Config Directory</label>
                <input
                  id="ha-config-path"
                  type="text"
                  value={configPath}
                  onChange={(e) => setConfigPath(e.target.value)}
                  placeholder="~/.homeassistant"
                />
                <span className="field-hint">
                  Must be under your home directory for Docker Desktop file sharing
                </span>
              </div>
              <button className="btn btn-primary" onClick={handleSetupHa} disabled={loading}>
                {loading ? "Setting up..." : "Setup Home Assistant"}
              </button>
            </>
          )}

          {dockerStatus?.installed && haStatus?.exists && !haStatus?.running && (
            <button className="btn btn-primary" onClick={handleStartHa} disabled={loading}>
              {loading ? "Starting..." : "Start Home Assistant"}
            </button>
          )}

          {haStatus?.running && (
            <p className="status-text success">Home Assistant is running. No action needed.</p>
          )}
        </div>
      </div>

      {/* Feedback */}
      {actionMessage && <div className="alert-box success">{actionMessage}</div>}
      {actionError && <div className="alert-box error">{actionError}</div>}
    </div>
  );
}
