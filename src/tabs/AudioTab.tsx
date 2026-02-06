import { Config } from "../types";

interface AudioTabProps {
  config: Config;
  onChange: (updates: Partial<Config>) => void;
}

export default function AudioTab({ config, onChange }: AudioTabProps) {
  return (
    <div className="tab-content audio-tab">
      <h2>Audio</h2>

      {/* Audio Device Selection */}
      <section className="settings-section">
        <h3>Input Device</h3>

        <div className="field">
          <label htmlFor="audio-device">Audio Device</label>
          <input
            id="audio-device"
            type="text"
            value={config.audio_device}
            onChange={(e) => onChange({ audio_device: e.target.value })}
            placeholder="default"
          />
          <span className="field-hint">
            Device name or "default" for the system default input
          </span>
        </div>
      </section>

      {/* Whisper Settings (merged from Pipeline tab) */}
      <section className="settings-section">
        <h3>Whisper Transcription</h3>

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
          <label htmlFor="whisper-model-path">Model Path (GGML)</label>
          <input
            id="whisper-model-path"
            type="text"
            value={config.whisper_model_path}
            onChange={(e) => onChange({ whisper_model_path: e.target.value })}
            placeholder="~/.cache/whisper/ggml-base.bin"
          />
          <span className="field-hint">
            Path to the GGML whisper model file. Downloaded automatically if missing.
          </span>
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
