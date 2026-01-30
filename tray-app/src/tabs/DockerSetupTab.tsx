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

export default function DockerSetupTab({ config, onChange }: DockerSetupTabProps) {
  const [dockerStatus, setDockerStatus] = useState<DockerStatus | null>(null);
  const [haStatus, setHaStatus] = useState<HaContainerStatus | null>(null);
  const [configPath, setConfigPath] = useState("/opt/homeassistant/config");
  const [loading, setLoading] = useState(false);
  const [actionMessage, setActionMessage] = useState("");
  const [actionError, setActionError] = useState("");

  const checkDocker = useCallback(async () => {
    try {
      const status = await invoke<DockerStatus>("check_docker");
      setDockerStatus(status);
      return status.installed;
    } catch (err) {
      setDockerStatus({ installed: false, version: "" });
      return false;
    }
  }, []);

  const checkHaContainer = useCallback(async () => {
    try {
      const status = await invoke<HaContainerStatus>("check_ha_container_status");
      setHaStatus(status);
    } catch (err) {
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

  const StatusIcon = ({ ok }: { ok: boolean }) => (
    <span
      style={{
        display: "inline-block",
        width: 18,
        height: 18,
        lineHeight: "18px",
        textAlign: "center",
        borderRadius: "50%",
        fontSize: 13,
        fontWeight: "bold",
        marginRight: 8,
        color: "#fff",
        backgroundColor: ok ? "#22c55e" : "#ef4444",
      }}
    >
      {ok ? "\u2713" : "\u2717"}
    </span>
  );

  return (
    <div className="tab-content docker-setup-tab">
      <h2>Docker &amp; Home Assistant</h2>

      {/* Refresh button */}
      <section className="settings-section">
        <button onClick={refreshAll} disabled={loading}>
          Refresh Status
        </button>
      </section>

      {/* Step 1: Docker Installation */}
      <section className="settings-section">
        <h3>
          <StatusIcon ok={dockerStatus?.installed ?? false} />
          Step 1: Docker Installation
        </h3>
        {dockerStatus === null ? (
          <p>Checking Docker installation...</p>
        ) : dockerStatus.installed ? (
          <p style={{ color: "#22c55e" }}>{dockerStatus.version}</p>
        ) : (
          <p style={{ color: "#ef4444" }}>
            Docker is not installed. Please install Docker Desktop from{" "}
            <a href="https://www.docker.com/products/docker-desktop/" target="_blank" rel="noreferrer">
              docker.com
            </a>
          </p>
        )}
      </section>

      {/* Step 2: HA Container Status */}
      <section className="settings-section">
        <h3>
          <StatusIcon ok={haStatus?.running ?? false} />
          Step 2: Home Assistant Container
        </h3>
        {!dockerStatus?.installed ? (
          <p style={{ color: "#888" }}>Install Docker first</p>
        ) : haStatus === null ? (
          <p>Checking container status...</p>
        ) : haStatus.running ? (
          <p style={{ color: "#22c55e" }}>Running: {haStatus.status}</p>
        ) : haStatus.exists ? (
          <p style={{ color: "#eab308" }}>Stopped: {haStatus.status}</p>
        ) : (
          <p style={{ color: "#ef4444" }}>No Home Assistant container found</p>
        )}
      </section>

      {/* Step 3: Setup or Start */}
      <section className="settings-section">
        <h3>
          <StatusIcon ok={haStatus?.running ?? false} />
          Step 3: Setup / Control
        </h3>

        {dockerStatus?.installed && !haStatus?.exists && (
          <div>
            <div className="field">
              <label htmlFor="ha-config-path">HA Config Directory</label>
              <input
                id="ha-config-path"
                type="text"
                value={configPath}
                onChange={(e) => setConfigPath(e.target.value)}
                placeholder="/opt/homeassistant/config"
              />
            </div>
            <button onClick={handleSetupHa} disabled={loading}>
              {loading ? "Setting up..." : "Setup Home Assistant"}
            </button>
          </div>
        )}

        {dockerStatus?.installed && haStatus?.exists && !haStatus?.running && (
          <button onClick={handleStartHa} disabled={loading}>
            {loading ? "Starting..." : "Start Home Assistant"}
          </button>
        )}

        {haStatus?.running && (
          <p style={{ color: "#22c55e" }}>Home Assistant is running. No action needed.</p>
        )}
      </section>

      {/* Action feedback */}
      {actionMessage && (
        <section className="settings-section">
          <p style={{ color: "#22c55e" }}>{actionMessage}</p>
        </section>
      )}
      {actionError && (
        <section className="settings-section">
          <p style={{ color: "#ef4444" }}>{actionError}</p>
        </section>
      )}
    </div>
  );
}
