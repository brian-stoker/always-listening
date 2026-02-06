import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Config } from "../types";

interface HomeAssistantTabProps {
  config: Config;
  onChange: (updates: Partial<Config>) => void;
}

export default function HomeAssistantTab({ config, onChange }: HomeAssistantTabProps) {
  const [testStatus, setTestStatus] = useState<string | null>(null);
  const [testing, setTesting] = useState(false);

  const disabled = !config.ha_enabled;

  async function handleTestConnection() {
    setTesting(true);
    setTestStatus(null);
    try {
      await invoke("test_ha_connection", {
        url: config.ha_url,
        token: config.hass_token,
      });
      setTestStatus("Connection successful!");
    } catch (err) {
      setTestStatus(`Connection failed: ${err}`);
    } finally {
      setTesting(false);
    }
  }

  const testOk = testStatus?.startsWith("Connection successful");

  return (
    <div className="tab-content homeassistant-tab">
      <h2>Home Assistant</h2>

      <section className="settings-section">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={config.ha_enabled}
            onChange={(e) => onChange({ ha_enabled: e.target.checked })}
          />
          Enable Home Assistant integration
        </label>
      </section>

      <section className="settings-section">
        <h3>Connection</h3>

        <div className="field">
          <label htmlFor="ha-url">Home Assistant URL</label>
          <input
            id="ha-url"
            type="text"
            value={config.ha_url}
            onChange={(e) => onChange({ ha_url: e.target.value })}
            placeholder="http://homeassistant.local:8123"
            disabled={disabled}
          />
        </div>

        <div className="field">
          <label htmlFor="ha-token">Long-Lived Access Token</label>
          <input
            id="ha-token"
            type="password"
            value={config.hass_token}
            onChange={(e) => onChange({ hass_token: e.target.value })}
            placeholder="Enter your HA access token"
            disabled={disabled}
          />
          <span className="field-hint">
            Generate in HA: Profile &rarr; Long-Lived Access Tokens &rarr; Create Token
          </span>
        </div>

        <div className="field">
          <div className="inline-action">
            <button
              className="btn btn-secondary"
              onClick={handleTestConnection}
              disabled={disabled || testing}
            >
              {testing ? "Testing..." : "Test Connection"}
            </button>
            {testStatus && (
              <span className={`test-status ${testOk ? "success" : "error"}`}>
                {testStatus}
              </span>
            )}
          </div>
        </div>
      </section>

      <section className="settings-section">
        <p className="field-hint">
          TTS entities (TTS service + media player) are now configured per-agent in the Agents tab.
        </p>
      </section>
    </div>
  );
}
