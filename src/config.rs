use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use hopbox_core::config::{AiConfig, ServerConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub ai: AiConfig,
    pub server: Option<ServerConfig>,
    pub keybindings: KeybindingsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    pub ai_trigger: String,
    pub quit: String,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            ai_trigger: "ctrl+\\".to_string(),
            quit: "ctrl+q".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Config::default())
        }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hopbox")
        .join("config.toml")
}
