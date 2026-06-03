use crate::application::state::Config;
use std::fs;
use std::path::PathBuf;

fn config_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg).join("xtop")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config").join("xtop")
    } else {
        PathBuf::from(".").join(".config").join("xtop")
    }
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn themes_dir() -> PathBuf {
    config_dir().join("themes")
}

pub fn load_config() -> Config {
    let path = config_path();
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<Config>(&data) {
            return cfg;
        }
    }
    Config::default()
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_custom_themes() -> Vec<crate::domain::theme::Theme> {
    let dir = themes_dir();
    if !dir.exists() {
        return vec![];
    }
    let mut themes = vec![];
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(theme) = serde_json::from_str::<crate::domain::theme::Theme>(&data) {
                        themes.push(theme);
                    }
                }
            }
        }
    }
    themes
}
