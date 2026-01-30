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
            placeholder=":0 (macOS avfoundation default)"
          />
          <span className="field-hint">
            macOS avfoundation device index (e.g. ":0" for default input)
          </span>
        </div>
      </section>

      {/* Recording Settings */}
      <section className="settings-section">
        <h3>Recording</h3>

        <div className="field">
          <label htmlFor="record-duration">Recording Duration (seconds)</label>
          <input
            id="record-duration"
            type="number"
            min={1}
            value={config.record_duration}
            onChange={(e) =>
              onChange({ record_duration: parseInt(e.target.value, 10) || 1 })
            }
          />
          <span className="field-hint">
            Maximum recording length in seconds before auto-stop
          </span>
        </div>
      </section>

      {/* Silence Detection */}
      <section className="settings-section">
        <h3>Silence Detection</h3>

        <div className="field">
          <label htmlFor="silence-threshold">Silence Threshold</label>
          <input
            id="silence-threshold"
            type="text"
            value={config.silence_threshold}
            onChange={(e) => onChange({ silence_threshold: e.target.value })}
            placeholder="-30dB"
          />
          <span className="field-hint">
            Audio level below which input is considered silence (e.g. "-30dB")
          </span>
        </div>

        <div className="field">
          <label htmlFor="silence-duration">
            Silence Duration (seconds)
          </label>
          <input
            id="silence-duration"
            type="number"
            min={0}
            step={0.1}
            value={config.silence_duration}
            onChange={(e) =>
              onChange({
                silence_duration: parseFloat(e.target.value) || 0,
              })
            }
          />
          <span className="field-hint">
            Seconds of silence before recording automatically stops
          </span>
        </div>
      </section>
    </div>
  );
}
