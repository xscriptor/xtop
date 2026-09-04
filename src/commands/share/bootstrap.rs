//! Bootstrap: assemble the live application state for a command run.

use std::fs;
use std::path::Path;

use crate::config;
use crate::plugins::PluginManager;
use crate::providers::sysinfo::SysinfoProvider;
use crate::providers::CompositeProvider;
use crate::state::AppState;
use crate::theme::load_all_themes;
use xtop_layout::{default_layouts, load_layouts_from_dir, merge_layouts};

#[cfg(feature = "plugin-samurai")]
use xtop_plugin_samurai::SamuraiPlugin;

pub(crate) fn build_plugin_manager(state: &mut AppState, cfg_dir: &Path) -> PluginManager {
    let plugins_dir = cfg_dir.join("plugins");
    fs::create_dir_all(&plugins_dir).ok();
    let mut mgr = PluginManager::new(plugins_dir);

    // Register plugins behind feature flags.
    register_plugins(&mut mgr, state);

    mgr
}

/// Register the optional compile-time plugins.
#[cfg(feature = "plugin-samurai")]
pub(crate) fn register_plugins(mgr: &mut PluginManager, state: &mut AppState) {
    let plugin = Box::new(SamuraiPlugin::new());
    if let Err(e) = mgr.register(plugin, state) {
        eprintln!("[xtop] failed to load samurai plugin: {e}");
    }
}

/// No plugins selected at compile time.
#[cfg(not(feature = "plugin-samurai"))]
fn register_plugins(_mgr: &mut PluginManager, _state: &mut AppState) {}

// ---------------------------------------------------------------------------
// CLI subcommands
// ---------------------------------------------------------------------------

/// Assemble a fully initialized `AppState` for a command run.
pub fn initialize_state(cfg_dir: &Path) -> anyhow::Result<AppState> {
    let sysinfo_provider = SysinfoProvider::new();
    let composite = CompositeProvider::new(Box::new(sysinfo_provider));

    let themes = load_all_themes();
    let cfg = config::load_config();

    // Layouts: embedded defaults first; user files from the config dir then
    // override defaults by name (user wins) — see `xtop_layout::merge_layouts`.
    let layouts_dir = config::config_dir().join("layouts");
    let layout_defs = merge_layouts(default_layouts(), load_layouts_from_dir(&layouts_dir));

    let mut state = AppState::new(Box::new(composite), themes, cfg, layout_defs);

    // Build and register plugins, then wire their providers into the state.
    let plugin_mgr = build_plugin_manager(&mut state, cfg_dir);
    let extra_providers = plugin_mgr.collect_data_providers();
    state.init_plugins(plugin_mgr, extra_providers);

    Ok(state)
}
