import { Config } from "../types";

interface GeneralTabProps {
  config: Config;
  onChange: (updates: Partial<Config>) => void;
}

export default function GeneralTab({ config, onChange }: GeneralTabProps) {
  return (
    <div className="tab-content general-tab">
      <h2>General</h2>

      {/* Launch at Login */}
      <section className="settings-section">
        <h3>Startup</h3>
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={config.launch_at_login}
            onChange={(e) => onChange({ launch_at_login: e.target.checked })}
          />
          Launch at login
        </label>
      </section>

      {/* Global Hotkey Configuration */}
      <section className="settings-section">
        <h3>Global Hotkeys</h3>

        <div className="field">
          <label htmlFor="hotkey-voice-to-claude">Voice-to-Claude</label>
          <input
            id="hotkey-voice-to-claude"
            type="text"
            value={config.hotkey_send}
            onChange={(e) => onChange({ hotkey_send: e.target.value })}
            placeholder="e.g. F18"
          />
        </div>

        <div className="field">
          <label htmlFor="hotkey-dictation">Dictation</label>
          <input
            id="hotkey-dictation"
            type="text"
            value={config.hotkey_dictate}
            onChange={(e) => onChange({ hotkey_dictate: e.target.value })}
            placeholder="e.g. F19"
          />
        </div>

        <div className="field">
          <label htmlFor="hotkey-combined">Combined Mode</label>
          <input
            id="hotkey-combined"
            type="text"
            value=""
            placeholder="Not yet configured"
            disabled
            title="Combined mode hotkey is not yet supported in the backend"
          />
        </div>
      </section>
    </div>
  );
}
