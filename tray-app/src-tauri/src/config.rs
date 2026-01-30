use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Audio settings
    pub audio_device: String,

    // Pipeline binaries
    pub whisper_bin: String,
    pub claude_bin: String,

    // Home Assistant settings
    pub ha_enabled: bool,
    pub ha_entity: String,
    pub ha_url: String,
    pub ha_tts_entity: String,
    pub hass_token: String,

    // Recording settings
    pub record_duration: u32,
    pub silence_threshold: String,
    pub silence_duration: f64,

    // Whisper settings
    pub whisper_model: String,
    pub whisper_language: String,

    // Directories
    pub tmp_dir: String,
    pub control_fifo: String,

    // App-specific settings
    pub hotkey_send: String,
    pub hotkey_dictate: String,
    pub launch_at_login: bool,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            audio_device: ":1".to_string(),
            whisper_bin: home.join(".voice-pipeline/venv/bin/whisper").to_string_lossy().to_string(),
            claude_bin: home.join(".local/bin/claude").to_string_lossy().to_string(),
            ha_enabled: false,
            ha_entity: "media_player.office_speaker".to_string(),
            ha_url: "http://192.168.1.208:8123".to_string(),
            ha_tts_entity: "tts.elevenlabs_text_to_speech".to_string(),
            hass_token: String::new(),
            record_duration: 5,
            silence_threshold: "-30dB".to_string(),
            silence_duration: 1.5,
            whisper_model: "base".to_string(),
            whisper_language: "en".to_string(),
            tmp_dir: "/tmp/voice-pipeline".to_string(),
            control_fifo: "/tmp/voice-pipeline/control.fifo".to_string(),
            hotkey_send: "F18".to_string(),
            hotkey_dictate: "F19".to_string(),
            launch_at_login: false,
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
                Ok(contents) => {
                    match toml::from_str::<Config>(&contents) {
                        Ok(config) => {
                            tracing::info!("Config loaded from {:?}", config_path);
                            return config;
                        }
                        Err(e) => {
                            tracing::warn!("Invalid config file, using defaults: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read config file, using defaults: {}", e);
                }
            }
        }

        // Try to migrate from legacy config.env
        let legacy_config = Self::migrate_from_legacy();
        let config = legacy_config.unwrap_or_default();

        // Save the config
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

    /// Attempt to migrate from legacy config.env
    fn migrate_from_legacy() -> Option<Self> {
        // Look for config.env in common locations
        let possible_paths = vec![
            // Same directory as the executable's parent (repo root)
            std::env::current_exe().ok()?.parent()?.parent()?.join("config.env"),
            // Home directory
            dirs::home_dir()?.join("config.env"),
        ];

        for path in possible_paths {
            if path.exists() {
                tracing::info!("Found legacy config.env at {:?}, migrating...", path);
                if let Ok(contents) = fs::read_to_string(&path) {
                    return Some(Self::parse_legacy_config(&contents));
                }
            }
        }
        None
    }

    /// Parse legacy config.env format
    fn parse_legacy_config(contents: &str) -> Self {
        let mut config = Self::default();

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');
                // Expand $HOME
                let home = dirs::home_dir().unwrap_or_default();
                let value = value.replace("$HOME", &home.to_string_lossy());

                match key {
                    "AUDIO_DEVICE" => config.audio_device = value,
                    "WHISPER_BIN" => config.whisper_bin = value,
                    "CLAUDE_BIN" => config.claude_bin = value,
                    "HA_ENTITY" => config.ha_entity = value,
                    "HA_URL" => config.ha_url = value,
                    "HA_TTS_ENTITY" => config.ha_tts_entity = value,
                    "RECORD_DURATION" => {
                        config.record_duration = value.parse().unwrap_or(5);
                    }
                    "SILENCE_THRESHOLD" => config.silence_threshold = value,
                    "SILENCE_DURATION" => {
                        config.silence_duration = value.parse().unwrap_or(1.5);
                    }
                    "WHISPER_MODEL" => config.whisper_model = value,
                    "WHISPER_LANGUAGE" => config.whisper_language = value,
                    "TMP_DIR" => config.tmp_dir = value,
                    "CONTROL_FIFO" => config.control_fifo = value,
                    _ => {
                        tracing::debug!("Unknown config key: {}", key);
                    }
                }
            }
        }

        tracing::info!("Legacy config migrated successfully");
        config
    }
}

/// Tauri commands for config access
#[tauri::command]
pub fn get_config(config: tauri::State<'_, std::sync::Mutex<Config>>) -> Config {
    config.lock().unwrap().clone()
}

#[tauri::command]
pub fn update_config(config: tauri::State<'_, std::sync::Mutex<Config>>, new_config: Config) {
    let mut cfg = config.lock().unwrap();
    *cfg = new_config;
    cfg.save();
    tracing::info!("Config updated and saved");
}
