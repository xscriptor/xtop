//! Config file read/write against the persisted schema.

use std::fs;
use std::path::PathBuf;

use super::schema::Config;
use crate::config::config_dir;

/// Path of the persisted config file.
pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// Load the config, falling back to defaults when missing or invalid.
pub fn load_config() -> Config {
    let path = config_path();
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<Config>(&data) {
            return cfg;
        }
    }
    Config::default()
}

/// Save the config to disk.
pub fn save_config(config: &Config) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}
