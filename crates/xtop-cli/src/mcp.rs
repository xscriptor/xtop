//! MCP server entry point.
//!
//! Initializes xtop state and plugins, then delegates to the Sentinel plugin's
//! MCP module (`xtop_plugin_sentinel::mcp::run_server`) which handles the
//! actual stdin/stdout MCP protocol loop.

use std::path::{Path, PathBuf};
use xtop_core::application::plugin_manager::PluginManager;
use xtop_core::application::state::AppState;
use xtop_core::infrastructure::composite_provider::CompositeProvider;
use xtop_core::infrastructure::config;
use xtop_core::infrastructure::layout_loader;
use xtop_core::infrastructure::sysinfo_provider::SysinfoProvider;
use xtop_core::infrastructure::theme_loader::load_all_themes;

#[cfg(feature = "plugin-sentinel")]
use xtop_plugin_sentinel::SentinelPlugin;

/// Run the MCP server.
///
/// Sets up AppState + PluginManager with Sentinel, then delegates to
/// the plugin's MCP module for the protocol loop.
pub fn run_mcp_server() -> anyhow::Result<()> {
    let cfg_dir = config_dir();
    let mut state = initialize_state(&cfg_dir)?;

    // Delegate to Sentinel's MCP module
    #[cfg(feature = "plugin-sentinel")]
    {
        xtop_plugin_sentinel::mcp::run_server(&mut state)
    }

    #[cfg(not(feature = "plugin-sentinel"))]
    {
        eprintln!("MCP server requires the 'plugin-sentinel' feature.");
        eprintln!("Rebuild with: cargo build --features plugin-sentinel");
        std::process::exit(1);
    }
}

fn config_dir() -> PathBuf {
    xtop_core::infrastructure::config::config_dir()
}

fn build_plugin_manager(state: &mut AppState, cfg_dir: &Path) -> PluginManager {
    let plugins_dir = cfg_dir.join("plugins");
    std::fs::create_dir_all(&plugins_dir).ok();
    let mut mgr = PluginManager::new(plugins_dir);

    #[cfg(feature = "plugin-sentinel")]
    {
        let plugin = Box::new(SentinelPlugin::new());
        if let Err(e) = mgr.register(plugin, state) {
            eprintln!("[xtop-mcp] failed to load sentinel plugin: {e}");
        }
    }

    mgr
}

fn initialize_state(cfg_dir: &Path) -> anyhow::Result<AppState> {
    let sysinfo_provider = SysinfoProvider::new();
    let composite = CompositeProvider::new(Box::new(sysinfo_provider));

    let themes = load_all_themes();
    let cfg = config::load_config();
    let mut builtin_layouts = layout_loader::builtin_layouts();
    let custom_layouts = layout_loader::load_custom_layouts();
    builtin_layouts.extend(custom_layouts);
    let mut state = AppState::new(Box::new(composite), themes, cfg, builtin_layouts);

    let plugin_mgr = build_plugin_manager(&mut state, cfg_dir);
    let extra_providers = plugin_mgr.collect_data_providers();
    state.init_plugins(plugin_mgr, extra_providers);

    Ok(state)
}
