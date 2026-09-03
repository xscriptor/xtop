//! Asset bootstrapping and config persistence used by several commands.

use std::fs;

use crate::config;
use crate::config::Config;
use crate::state::AppState;

const DEFAULT_THEMES: &[(&str, &str)] = &[
    ("x", include_str!("../../../assets/themes/x.jsonc")),
    (
        "madrid",
        include_str!("../../../assets/themes/madrid.jsonc"),
    ),
    (
        "lahabana",
        include_str!("../../../assets/themes/lahabana.jsonc"),
    ),
    ("paris", include_str!("../../../assets/themes/paris.jsonc")),
    ("tokio", include_str!("../../../assets/themes/tokio.jsonc")),
    ("oslo", include_str!("../../../assets/themes/oslo.jsonc")),
    (
        "helsinki",
        include_str!("../../../assets/themes/helsinki.jsonc"),
    ),
    (
        "berlin",
        include_str!("../../../assets/themes/berlin.jsonc"),
    ),
    (
        "london",
        include_str!("../../../assets/themes/london.jsonc"),
    ),
    ("praha", include_str!("../../../assets/themes/praha.jsonc")),
    (
        "bogota",
        include_str!("../../../assets/themes/bogota.jsonc"),
    ),
];

const DEFAULT_LAYOUTS: &[(&str, &str)] = &[
    (
        "dashboard",
        include_str!("../../../assets/layouts/dashboard.jsonc"),
    ),
    (
        "vertical",
        include_str!("../../../assets/layouts/vertical.jsonc"),
    ),
    (
        "horizontal",
        include_str!("../../../assets/layouts/horizontal.jsonc"),
    ),
    (
        "cpu_focus",
        include_str!("../../../assets/layouts/cpu_focus.jsonc"),
    ),
    (
        "memory_focus",
        include_str!("../../../assets/layouts/memory_focus.jsonc"),
    ),
    (
        "network_focus",
        include_str!("../../../assets/layouts/network_focus.jsonc"),
    ),
    (
        "process_focus",
        include_str!("../../../assets/layouts/process_focus.jsonc"),
    ),
];

pub fn config_dir() -> std::path::PathBuf {
    crate::config::config_dir()
}

pub fn ensure_default_assets() {
    let theme_assets: &[(&str, &str)] = DEFAULT_THEMES;
    let layout_assets: &[(&str, &str)] = DEFAULT_LAYOUTS;

    let dir = crate::theme::themes_dir();
    if !dir.join(".xtop_initialized").exists() {
        fs::create_dir_all(&dir).ok();
        for (name, content) in theme_assets {
            let path = dir.join(format!("{name}.jsonc"));
            if !path.exists() {
                fs::write(&path, content).ok();
            }
        }
        fs::write(dir.join(".xtop_initialized"), "").ok();
    }

    let dir = crate::layout::layouts_dir();
    if !dir.join(".xtop_initialized").exists() {
        fs::create_dir_all(&dir).ok();
        for (name, content) in layout_assets {
            let path = dir.join(format!("{name}.jsonc"));
            if !path.exists() {
                fs::write(&path, content).ok();
            }
        }
        fs::write(dir.join(".xtop_initialized"), "").ok();
    }
}

pub fn save_config(state: &AppState) {
    let layout_name = if state.layout_index < state.layout_defs.len() {
        state.layout_defs[state.layout_index].name.clone()
    } else {
        String::new()
    };
    let cfg = Config {
        theme: state.current_theme.name.clone(),
        layout_mode: state.save_layout_mode(),
        layout_name,
        update_interval_ms: state.update_interval_ms,
        history_points: 100,
        alerts: state.alerts,
        keybindings: state.keybindings.clone(),
    };
    let _ = config::save_config(&cfg);
}
