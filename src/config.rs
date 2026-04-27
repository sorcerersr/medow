use std::fs;
use std::path::PathBuf;

use dirs::config_dir;
use serde::{Deserialize, Serialize};

pub const CONFIG_DIR: &str = "medow";
pub const CONFIG_FILE: &str = "config.toml";
pub const STATE_FILE: &str = "state.json";
pub const DEFAULT_DOWNLOAD_DIR: &str = "Downloads/medow";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub default_download_dir: String,
    pub quality_preference: String,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_downloads: u32,
}

fn default_max_concurrent() -> u32 {
    3
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_download_dir: String::new(),
            quality_preference: String::from("SD"),
            max_concurrent_downloads: 3,
        }
    }
}

/// Returns the resolved download directory (uses default if not configured)
pub fn get_download_dir() -> PathBuf {
    let config = load_config();
    if !config.default_download_dir.is_empty() {
        PathBuf::from(&config.default_download_dir)
    } else if let Some(home) = dirs::home_dir() {
        home.join(DEFAULT_DOWNLOAD_DIR)
    } else {
        PathBuf::from(DEFAULT_DOWNLOAD_DIR)
    }
}

/// Returns the config file path following XDG spec
fn config_path() -> PathBuf {
    let dir = config_dir().map(|p| p.join(CONFIG_DIR)).unwrap_or_else(|| {
        // Fallback to $HOME/.config/medow
        if let Some(home) = dirs::home_dir() {
            home.join(".config").join(CONFIG_DIR)
        } else {
            PathBuf::from(CONFIG_DIR)
        }
    });
    dir.join(CONFIG_FILE)
}

/// Load configuration from disk
pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => {
                    println!("Config loaded from {:?}", path);
                    config
                }
                Err(e) => {
                    eprintln!("Failed to parse config: {e}, using defaults");
                    AppConfig::default()
                }
            },
            Err(e) => {
                eprintln!("Failed to read config: {e}, using defaults");
                AppConfig::default()
            }
        }
    } else {
        println!("No config file found, using defaults");
        AppConfig::default()
    }
}

/// Save configuration to disk
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {e}"))?;
    }

    // Serialize to TOML
    let content =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {e}"))?;

    // Write to file
    fs::write(&path, content).map_err(|e| format!("Failed to write config: {e}"))?;

    println!("Config saved to {:?}", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.default_download_dir, "");
        assert_eq!(config.quality_preference, "SD");
    }

    #[test]
    fn test_config_path_exists() {
        let path = config_path();
        assert!(path.ends_with(CONFIG_FILE));
        assert!(path.to_string_lossy().contains(CONFIG_DIR));
    }
}
