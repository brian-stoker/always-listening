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
          <label htmlFor="hotkey-cycle">Agent Cycle (Off ... agents ... Off)</label>
          <input
            id="hotkey-cycle"
            type="text"
            value="F19"
            disabled
            title="Agent cycling hotkey — currently fixed to F19"
          />
          <span className="field-hint">Press F19 to cycle through configured agents</span>
        </div>
      </section>
    </div>
  );
}
