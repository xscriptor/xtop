//! Persisted configuration schema.
//!
//! The on-disk config of the app. Theme and layout names are plain strings
//! here; the schema types stay independent of the runtime state.

use crate::config::keybinding::Keybindings;
use crate::layout::LayoutMode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub cpu_high: f64,
    pub mem_high: f64,
    pub disk_high: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_high: 90.0,
            mem_high: 90.0,
            disk_high: 90.0,
        }
    }
}

fn default_layout_mode() -> LayoutMode {
    LayoutMode::Dashboard
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    #[serde(default = "default_layout_mode")]
    pub layout_mode: LayoutMode,
    /// Layout name for custom layouts beyond the 7 built-in LayoutMode variants.
    /// If non-empty, takes precedence over `layout_mode`.
    #[serde(default)]
    pub layout_name: String,
    pub update_interval_ms: u64,
    pub history_points: usize,
    pub alerts: AlertThresholds,
    #[serde(default)]
    pub keybindings: Keybindings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "x".to_string(),
            layout_mode: LayoutMode::Dashboard,
            layout_name: String::new(),
            update_interval_ms: 1000,
            history_points: 100,
            alerts: AlertThresholds::default(),
            keybindings: Keybindings::default(),
        }
    }
}
