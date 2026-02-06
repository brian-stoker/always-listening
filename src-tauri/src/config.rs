use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Agent configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TtsMethod {
    Ha,
    LocalSay,
    None,
}

impl Default for TtsMethod {
    fn default() -> Self {
        TtsMethod::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    #[serde(default)]
    pub method: TtsMethod,
    #[serde(default)]
    pub ha_tts_entity: Option<String>,
    #[serde(default)]
    pub ha_media_entity: Option<String>,
    #[serde(default)]
    pub local_voice: Option<String>,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            method: TtsMethod::None,
            ha_tts_entity: None,
            ha_media_entity: None,
            local_voice: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>,
    /// CLI command template. Use `{text}` as placeholder for the transcribed text.
    #[serde(default)]
    pub command: Option<String>,
    /// Dot-path into the JSON response (e.g. "result.payloads.0.text")
    #[serde(default)]
    pub response_json_path: Option<String>,
    #[serde(default)]
    pub tts: TtsConfig,
    #[serde(default)]
    pub type_at_cursor: bool,
    #[serde(default)]
    pub auto_submit: bool,
    #[serde(default)]
    pub exit_phrases: Vec<String>,
    #[serde(default)]
    pub prompt_prefix: Option<String>,
}

/// Build the three default agents that ship with the app.
fn default_agents() -> Vec<AgentConfig> {
    vec![
        AgentConfig {
            id: "combined".to_string(),
            name: "Combined".to_string(),
            icon: Some("🔄".to_string()),
            command: Some("clawdbot agent --agent main --session-id voice-pipeline --message \"{text}\" --json".to_string()),
            response_json_path: Some("result.payloads.0.text".to_string()),
            tts: TtsConfig {
                method: TtsMethod::Ha,
                ha_tts_entity: Some("tts.elevenlabs_text_to_speech".to_string()),
                ha_media_entity: Some("media_player.office_speaker".to_string()),
                local_voice: None,
            },
            type_at_cursor: true,
            auto_submit: true,
            exit_phrases: vec![
                "goodbye clawdbot".to_string(),
                "goodbye claudbot".to_string(),
                "goodbye claude bot".to_string(),
                "goodbye omar".to_string(),
                "goodbye claude".to_string(),
            ],
            prompt_prefix: Some("[Voice input — respond in plain conversational speech. No markdown, no bullet points, no headers, no emojis, no special characters. Keep it concise and natural as if speaking aloud.] ".to_string()),
        },
        AgentConfig {
            id: "dictation".to_string(),
            name: "Dictation".to_string(),
            icon: Some("📝".to_string()),
            command: None,
            response_json_path: None,
            tts: TtsConfig::default(),
            type_at_cursor: true,
            auto_submit: true,
            exit_phrases: vec![],
            prompt_prefix: None,
        },
        AgentConfig {
            id: "voice-to-claude".to_string(),
            name: "Voice-to-Claude".to_string(),
            icon: Some("🤖".to_string()),
            command: Some("clawdbot agent --agent main --session-id voice-pipeline --message \"{text}\" --json".to_string()),
            response_json_path: Some("result.payloads.0.text".to_string()),
            tts: TtsConfig {
                method: TtsMethod::Ha,
                ha_tts_entity: Some("tts.elevenlabs_text_to_speech".to_string()),
                ha_media_entity: Some("media_player.office_speaker".to_string()),
                local_voice: None,
            },
            type_at_cursor: false,
            auto_submit: false,
            exit_phrases: vec![
                "goodbye clawdbot".to_string(),
                "goodbye claudbot".to_string(),
                "goodbye claude bot".to_string(),
                "goodbye omar".to_string(),
                "goodbye claude".to_string(),
            ],
            prompt_prefix: Some("[Voice input — respond in plain conversational speech. No markdown, no bullet points, no headers, no emojis, no special characters. Keep it concise and natural as if speaking aloud.] ".to_string()),
        },
    ]
}

// ---------------------------------------------------------------------------
// Top-level Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Audio settings
    pub audio_device: String,

    // Home Assistant (global — agents reference these via their TTS config)
    pub ha_enabled: bool,
    pub ha_url: String,
    pub hass_token: String,

    // Whisper settings
    pub whisper_model: String,
    pub whisper_model_path: String,
    pub whisper_language: String,

    // Directories
    pub tmp_dir: String,

    // App-specific settings
    pub hotkey_send: String,
    pub hotkey_dictate: String,
    pub launch_at_login: bool,

    // Agent configs
    #[serde(default = "default_agents")]
    pub agents: Vec<AgentConfig>,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            audio_device: "default".to_string(),
            ha_enabled: false,
            ha_url: "http://192.168.1.208:8123".to_string(),
            hass_token: String::new(),
            whisper_model: "base".to_string(),
            whisper_model_path: home
                .join(".cache/whisper/ggml-base.bin")
                .to_string_lossy()
                .to_string(),
            whisper_language: "en".to_string(),
            tmp_dir: "/tmp/voice-pipeline".to_string(),
            hotkey_send: "F18".to_string(),
            hotkey_dictate: "F19".to_string(),
            launch_at_login: false,
            agents: default_agents(),
        }
    }
}

impl Config {
    /// Get the platform-specific config directory
    pub fn config_dir() -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            dirs::home_dir()
                .unwrap_or_default()
                .join("Library/Application Support/AlwaysListening")
        }
        #[cfg(target_os = "windows")]
        {
            dirs::data_dir()
                .unwrap_or_default()
                .join("AlwaysListening")
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            dirs::config_dir()
                .unwrap_or_default()
                .join("always-listening")
        }
    }

    /// Get the config file path
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Load config from disk, falling back to defaults
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(contents) => match toml::from_str::<Config>(&contents) {
                    Ok(config) => {
                        tracing::info!("Config loaded from {:?}", config_path);
                        return config;
                    }
                    Err(e) => {
                        tracing::warn!("Invalid config file, using defaults: {}", e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read config file, using defaults: {}", e);
                }
            }
        }

        let config = Self::default();
        config.save();
        config
    }

    /// Save config to disk
    pub fn save(&self) {
        let config_dir = Self::config_dir();
        if let Err(e) = fs::create_dir_all(&config_dir) {
            tracing::error!("Failed to create config directory: {}", e);
            return;
        }

        let config_path = Self::config_path();
        match toml::to_string_pretty(self) {
            Ok(contents) => {
                if let Err(e) = fs::write(&config_path, contents) {
                    tracing::error!("Failed to write config: {}", e);
                } else {
                    tracing::info!("Config saved to {:?}", config_path);
                }
            }
            Err(e) => {
                tracing::error!("Failed to serialize config: {}", e);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_config(
    config: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<Config>>>,
) -> Result<Config, String> {
    Ok(config.lock().await.clone())
}

#[tauri::command]
pub async fn update_config(
    config: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<Config>>>,
    new_config: Config,
) -> Result<(), String> {
    let mut cfg = config.lock().await;
    *cfg = new_config;
    cfg.save();
    tracing::info!("Config updated and saved");
    Ok(())
}

#[tauri::command]
pub async fn get_agents(
    config: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<Config>>>,
) -> Result<Vec<AgentConfig>, String> {
    Ok(config.lock().await.agents.clone())
}

#[tauri::command]
pub async fn update_agents(
    config: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<Config>>>,
    agents: Vec<AgentConfig>,
) -> Result<(), String> {
    let mut cfg = config.lock().await;
    cfg.agents = agents;
    cfg.save();
    tracing::info!("Agents updated and saved");
    Ok(())
}
