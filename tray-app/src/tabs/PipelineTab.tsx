import { Config } from "../types";

interface PipelineTabProps {
  config: Config;
  onChange: (updates: Partial<Config>) => void;
}

export default function PipelineTab({ config, onChange }: PipelineTabProps) {
  return (
    <div className="tab-content pipeline-tab">
      <h2>Pipeline</h2>

      {/* Whisper Settings */}
      <section className="settings-section">
        <h3>Whisper Transcription</h3>

        <div className="field">
          <label htmlFor="whisper-bin">Whisper Binary Path</label>
          <input
            id="whisper-bin"
            type="text"
            value={config.whisper_bin}
            onChange={(e) => onChange({ whisper_bin: e.target.value })}
            placeholder="e.g. ~/.voice-pipeline/venv/bin/whisper"
          />
        </div>

        <div className="field">
          <label htmlFor="whisper-model">Model</label>
          <select
            id="whisper-model"
            value={config.whisper_model}
            onChange={(e) => onChange({ whisper_model: e.target.value })}
          >
            <option value="tiny">tiny</option>
            <option value="base">base</option>
            <option value="small">small</option>
            <option value="medium">medium</option>
            <option value="large">large</option>
          </select>
        </div>

        <div className="field">
          <label htmlFor="whisper-language">Language</label>
          <input
            id="whisper-language"
            type="text"
            value={config.whisper_language}
            onChange={(e) => onChange({ whisper_language: e.target.value })}
            placeholder="en"
          />
        </div>
      </section>

      {/* Claude Settings */}
      <section className="settings-section">
        <h3>Claude</h3>

        <div className="field">
          <label htmlFor="claude-bin">Claude Binary Path</label>
          <input
            id="claude-bin"
            type="text"
            value={config.claude_bin}
            onChange={(e) => onChange({ claude_bin: e.target.value })}
            placeholder="e.g. ~/.local/bin/claude"
          />
        </div>
      </section>

      {/* Directories */}
      <section className="settings-section">
        <h3>Directories</h3>

        <div className="field">
          <label htmlFor="tmp-dir">Temp Directory</label>
          <input
            id="tmp-dir"
            type="text"
            value={config.tmp_dir}
            onChange={(e) => onChange({ tmp_dir: e.target.value })}
            placeholder="/tmp/voice-pipeline"
          />
        </div>
      </section>
    </div>
  );
}
