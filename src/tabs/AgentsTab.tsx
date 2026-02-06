import { useState } from "react";
import { AgentConfig, TtsConfig, TtsMethod, Config } from "../types";

interface AgentsTabProps {
  config: Config;
  onChange: (updates: Partial<Config>) => void;
}

const DEFAULT_TTS: TtsConfig = {
  method: "none",
  ha_tts_entity: null,
  ha_media_entity: null,
  local_voice: null,
};

function newAgent(): AgentConfig {
  return {
    id: `agent-${Date.now()}`,
    name: "",
    icon: null,
    command: null,
    response_json_path: null,
    tts: { ...DEFAULT_TTS },
    type_at_cursor: false,
    auto_submit: false,
    exit_phrases: [],
    prompt_prefix: null,
  };
}

export default function AgentsTab({ config, onChange }: AgentsTabProps) {
  const [editingIndex, setEditingIndex] = useState<number | null>(null);

  const agents = config.agents ?? [];

  function updateAgents(updated: AgentConfig[]) {
    onChange({ agents: updated });
  }

  function updateAgent(index: number, patch: Partial<AgentConfig>) {
    const updated = agents.map((a, i) => (i === index ? { ...a, ...patch } : a));
    updateAgents(updated);
  }

  function addAgent() {
    const agent = newAgent();
    updateAgents([...agents, agent]);
    setEditingIndex(agents.length);
  }

  function removeAgent(index: number) {
    updateAgents(agents.filter((_, i) => i !== index));
    setEditingIndex(null);
  }

  function moveAgent(index: number, dir: -1 | 1) {
    const target = index + dir;
    if (target < 0 || target >= agents.length) return;
    const copy = [...agents];
    [copy[index], copy[target]] = [copy[target], copy[index]];
    updateAgents(copy);
    setEditingIndex(target);
  }

  return (
    <div className="tab-content agents-tab">
      <h2>Agents</h2>
      <p className="field-hint" style={{ marginBottom: 12 }}>
        Configure voice agents. F19 cycles through them in order. Each agent
        can optionally run a command, speak a response, and/or type at the cursor.
      </p>

      <div className="agents-list">
        {agents.map((agent, idx) => (
          <div key={agent.id} className="agent-card">
            <div className="agent-card-header" onClick={() => setEditingIndex(editingIndex === idx ? null : idx)}>
              <span className="agent-icon">{agent.icon ?? "?"}</span>
              <span className="agent-name">{agent.name || "(unnamed)"}</span>
              <span className="agent-id">{agent.id}</span>
              <span className="agent-expand">{editingIndex === idx ? "▾" : "▸"}</span>
            </div>

            {editingIndex === idx && (
              <AgentForm
                agent={agent}
                onUpdate={(patch) => updateAgent(idx, patch)}
                onRemove={() => removeAgent(idx)}
                onMoveUp={() => moveAgent(idx, -1)}
                onMoveDown={() => moveAgent(idx, 1)}
                isFirst={idx === 0}
                isLast={idx === agents.length - 1}
              />
            )}
          </div>
        ))}
      </div>

      <button className="btn btn-secondary" onClick={addAgent} style={{ marginTop: 12 }}>
        + Add Agent
      </button>
    </div>
  );
}

interface AgentFormProps {
  agent: AgentConfig;
  onUpdate: (patch: Partial<AgentConfig>) => void;
  onRemove: () => void;
  onMoveUp: () => void;
  onMoveDown: () => void;
  isFirst: boolean;
  isLast: boolean;
}

function AgentForm({ agent, onUpdate, onRemove, onMoveUp, onMoveDown, isFirst, isLast }: AgentFormProps) {
  return (
    <div className="agent-form">
      <div className="field">
        <label>ID</label>
        <input type="text" value={agent.id} onChange={(e) => onUpdate({ id: e.target.value })} />
      </div>
      <div className="field">
        <label>Name</label>
        <input type="text" value={agent.name} onChange={(e) => onUpdate({ name: e.target.value })} />
      </div>
      <div className="field">
        <label>Icon (emoji)</label>
        <input type="text" value={agent.icon ?? ""} onChange={(e) => onUpdate({ icon: e.target.value || null })} style={{ width: 60 }} />
      </div>
      <div className="field">
        <label>Command template</label>
        <input
          type="text"
          value={agent.command ?? ""}
          onChange={(e) => onUpdate({ command: e.target.value || null })}
          placeholder='e.g. clawdbot agent --message "{text}" --json'
        />
        <span className="field-hint">Use {"{text}"} as placeholder for transcribed speech</span>
      </div>
      <div className="field">
        <label>Response JSON path</label>
        <input
          type="text"
          value={agent.response_json_path ?? ""}
          onChange={(e) => onUpdate({ response_json_path: e.target.value || null })}
          placeholder="e.g. result.payloads.0.text"
        />
      </div>
      <div className="field">
        <label>Prompt prefix</label>
        <input
          type="text"
          value={agent.prompt_prefix ?? ""}
          onChange={(e) => onUpdate({ prompt_prefix: e.target.value || null })}
          placeholder="Prepended to transcribed text before sending to command"
        />
      </div>

      {/* TTS */}
      <fieldset style={{ border: "1px solid var(--border)", padding: 8, borderRadius: 6, marginTop: 8 }}>
        <legend>TTS</legend>
        <div className="field">
          <label>Method</label>
          <select value={agent.tts.method} onChange={(e) => onUpdate({ tts: { ...agent.tts, method: e.target.value as TtsMethod } })}>
            <option value="none">None</option>
            <option value="ha">Home Assistant</option>
            <option value="local_say">Local (say)</option>
          </select>
        </div>
        {agent.tts.method === "ha" && (
          <>
            <div className="field">
              <label>HA TTS Entity</label>
              <input type="text" value={agent.tts.ha_tts_entity ?? ""} onChange={(e) => onUpdate({ tts: { ...agent.tts, ha_tts_entity: e.target.value || null } })} />
            </div>
            <div className="field">
              <label>HA Media Entity</label>
              <input type="text" value={agent.tts.ha_media_entity ?? ""} onChange={(e) => onUpdate({ tts: { ...agent.tts, ha_media_entity: e.target.value || null } })} />
            </div>
          </>
        )}
        {agent.tts.method === "local_say" && (
          <div className="field">
            <label>Voice</label>
            <input type="text" value={agent.tts.local_voice ?? ""} onChange={(e) => onUpdate({ tts: { ...agent.tts, local_voice: e.target.value || null } })} placeholder="e.g. Samantha" />
          </div>
        )}
      </fieldset>

      {/* Behavior */}
      <fieldset style={{ border: "1px solid var(--border)", padding: 8, borderRadius: 6, marginTop: 8 }}>
        <legend>Behavior</legend>
        <label className="checkbox-label">
          <input type="checkbox" checked={agent.type_at_cursor} onChange={(e) => onUpdate({ type_at_cursor: e.target.checked })} />
          Type at cursor
        </label>
        <label className="checkbox-label">
          <input type="checkbox" checked={agent.auto_submit} onChange={(e) => onUpdate({ auto_submit: e.target.checked })} />
          Auto-submit (press Enter)
        </label>
      </fieldset>

      {/* Exit phrases */}
      <div className="field" style={{ marginTop: 8 }}>
        <label>Exit phrases (comma-separated)</label>
        <input
          type="text"
          value={agent.exit_phrases.join(", ")}
          onChange={(e) =>
            onUpdate({ exit_phrases: e.target.value.split(",").map((s) => s.trim()).filter(Boolean) })
          }
          placeholder="goodbye claude, stop listening"
        />
      </div>

      {/* Actions */}
      <div style={{ display: "flex", gap: 8, marginTop: 12 }}>
        <button className="btn btn-secondary" onClick={onMoveUp} disabled={isFirst}>Up</button>
        <button className="btn btn-secondary" onClick={onMoveDown} disabled={isLast}>Down</button>
        <button className="btn btn-danger" onClick={onRemove} style={{ marginLeft: "auto" }}>Remove</button>
      </div>
    </div>
  );
}
