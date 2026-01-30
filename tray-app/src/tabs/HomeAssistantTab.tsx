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

  return (
    <div className="tab-content homeassistant-tab">
      <h2>Home Assistant</h2>

      {/* Enable/Disable Toggle */}
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

      {/* Connection Settings */}
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
        </div>

        <div className="field">
          <button
            className="btn btn-secondary"
            onClick={handleTestConnection}
            disabled={disabled || testing}
          >
            {testing ? "Testing..." : "Test Connection"}
          </button>
          {testStatus && (
            <span className={`test-status ${testStatus.startsWith("Connection successful") ? "success" : "error"}`}>
              {testStatus}
            </span>
          )}
        </div>
      </section>

      {/* TTS Settings */}
      <section className="settings-section">
        <h3>Text-to-Speech</h3>

        <div className="field">
          <label htmlFor="ha-tts-entity">TTS Entity</label>
          <input
            id="ha-tts-entity"
            type="text"
            value={config.ha_tts_entity}
            onChange={(e) => onChange({ ha_tts_entity: e.target.value })}
            placeholder="tts.google_translate_say"
            disabled={disabled}
          />
        </div>

        <div className="field">
          <label htmlFor="ha-entity">Media Player Entity</label>
          <input
            id="ha-entity"
            type="text"
            value={config.ha_entity}
            onChange={(e) => onChange({ ha_entity: e.target.value })}
            placeholder="media_player.living_room"
            disabled={disabled}
          />
        </div>
      </section>
    </div>
  );
}
