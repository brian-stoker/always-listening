/** TypeScript types matching the Rust backend structures */

/** TTS method — matches Rust TtsMethod enum */
export type TtsMethod = "ha" | "local_say" | "none";

/** Per-agent TTS configuration */
export interface TtsConfig {
  method: TtsMethod;
  ha_tts_entity: string | null;
  ha_media_entity: string | null;
  local_voice: string | null;
}

/** Agent configuration — matches Rust AgentConfig */
export interface AgentConfig {
  id: string;
  name: string;
  icon: string | null;
  command: string | null;
  response_json_path: string | null;
  tts: TtsConfig;
  type_at_cursor: boolean;
  auto_submit: boolean;
  exit_phrases: string[];
  prompt_prefix: string | null;
}

/** Matches Rust Config struct from config.rs */
export interface Config {
  // Audio settings
  audio_device: string;

  // Home Assistant settings
  ha_enabled: boolean;
  ha_url: string;
  hass_token: string;

  // Whisper settings
  whisper_model: string;
  whisper_model_path: string;
  whisper_language: string;

  // Directories
  tmp_dir: string;

  // App-specific settings
  hotkey_send: string;
  hotkey_dictate: string;
  launch_at_login: boolean;

  // Agent configs
  agents: AgentConfig[];
}

/** Matches Rust AppState enum from state.rs */
export type AppState =
  | "Idle"
  | { Active: string }
  | { Recording: string }
  | { AgentSpeaking: string };
