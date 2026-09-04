//! Asset bootstrapping and config persistence used by several commands.

use std::fs;

use crate::config;
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

/// Version of the seeded asset templates. Bumped when the shipped defaults
/// change so existing installs receive the new templates (without ever
/// clobbering files the user has edited).
const ASSETS_VERSION: &str = "1";

/// Write shipped defaults (themes and layouts) into the user config dir.
///
/// Files are only created when missing: user edits are never clobbered. The
/// defaults are the examples users copy to customize; the actual fallbacks
/// always live compiled into the binary/crates.
pub fn ensure_default_assets() {
    let dir = crate::theme::themes_dir();
    seed_assets(&dir, DEFAULT_THEMES);
    let dir = crate::config::config_dir().join("layouts");
    seed_assets(&dir, xtop_layout::default_layout_sources);
}

fn seed_assets(dir: &std::path::Path, sources: &[(&str, &str)]) {
    let marker = dir.join(".xtop_initialized");
    let up_to_date = fs::read_to_string(&marker)
        .map(|v| v.trim() == ASSETS_VERSION)
        .unwrap_or(false);
    if up_to_date {
        return;
    }
    fs::create_dir_all(dir).ok();
    for (name, content) in sources {
        let path = dir.join(format!("{name}.jsonc"));
        if !path.exists() {
            fs::write(&path, content).ok();
        }
    }
    fs::write(marker, ASSETS_VERSION).ok();
}

/// Persist the runtime state into the user config file.
///
/// Existing user values (`history_points`, update interval when untouched,
/// keybindings, alerts) are preserved: only the fields the runtime owns are
/// overwritten.
pub fn save_config(state: &AppState) {
    let mut cfg = config::load_config();
    cfg.theme = state.current_theme.name.clone();
    if let Some(def) = state.layout_defs.get(state.layout_index) {
        cfg.layout_name = def.name.clone();
    }
    cfg.layout_mode = state.layout_mode;
    cfg.update_interval_ms = state.update_interval_ms;
    cfg.alerts = state.alerts;
    cfg.keybindings = state.keybindings.clone();
    let _ = config::save_config(&cfg);
}
