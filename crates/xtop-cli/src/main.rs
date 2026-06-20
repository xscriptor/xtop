use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use xtop_core::application::plugin_manager::PluginManager;
use xtop_core::application::state::{AppState, Config, InputMode, PalettePage};
use xtop_core::domain::keybinding::Action;
use xtop_core::infrastructure::composite_provider::CompositeProvider;
use xtop_core::infrastructure::config;
use xtop_core::infrastructure::layout_loader;
use xtop_core::infrastructure::sysinfo_provider::SysinfoProvider;
use xtop_core::infrastructure::theme_loader::load_all_themes;
use xtop_tui::render;
use xtop_tui::terminal;

mod mcp;

// ---------------------------------------------------------------------------
// Plugin imports (feature-gated)
// ---------------------------------------------------------------------------
#[cfg(feature = "plugin-sentinel")]
use xtop_plugin_sentinel::SentinelPlugin;

// ---------------------------------------------------------------------------
// Config dir helper (delegated to xtop-core to avoid duplication)
// ---------------------------------------------------------------------------
fn config_dir() -> PathBuf {
    xtop_core::infrastructure::config::config_dir()
}

fn key_event_to_str(key: &KeyEvent) -> String {
    let mut s = String::new();
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    if ctrl {
        s.push_str("ctrl+");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        s.push_str("alt+");
    }
    match key.code {
        KeyCode::Char(c) => {
            if ctrl {
                s.push(c.to_ascii_lowercase());
            } else {
                s.push(c);
            }
        }
        KeyCode::Esc => s.push_str("escape"),
        KeyCode::Enter => s.push_str("enter"),
        KeyCode::Backspace => s.push_str("backspace"),
        KeyCode::Tab => s.push_str("tab"),
        KeyCode::Up => s.push_str("up"),
        KeyCode::Down => s.push_str("down"),
        KeyCode::Left => s.push_str("left"),
        KeyCode::Right => s.push_str("right"),
        KeyCode::Delete => s.push_str("delete"),
        KeyCode::Home => s.push_str("home"),
        KeyCode::End => s.push_str("end"),
        KeyCode::PageUp => s.push_str("pageup"),
        KeyCode::PageDown => s.push_str("pagedown"),
        _ => return String::new(),
    }
    s
}

// Embedded default asset files (shipped with the binary)
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

fn ensure_default_assets() {
    let theme_assets: &[(&str, &str)] = DEFAULT_THEMES;
    let layout_assets: &[(&str, &str)] = DEFAULT_LAYOUTS;

    let dir = xtop_core::infrastructure::theme_loader::themes_dir();
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

    let dir = xtop_core::infrastructure::layout_loader::layouts_dir();
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

fn save_config(state: &AppState) {
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

fn build_plugin_manager(state: &mut AppState, cfg_dir: &PathBuf) -> PluginManager {
    let plugins_dir = cfg_dir.join("plugins");
    fs::create_dir_all(&plugins_dir).ok();
    let mut mgr = PluginManager::new(plugins_dir);

    // Register plugins behind feature flags
    #[cfg(feature = "plugin-sentinel")]
    {
        let plugin = Box::new(SentinelPlugin::new());
        if let Err(e) = mgr.register(plugin, state) {
            eprintln!("[xtop] failed to load sentinel plugin: {e}");
        }
    }

    mgr
}

// ---------------------------------------------------------------------------
// CLI subcommands
// ---------------------------------------------------------------------------
fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  xtop                        Start the TUI system monitor");
    eprintln!("  xtop mcp                    Start MCP server (stdio transport) for AI agents");
    eprintln!("  xtop plugin list            List installed plugins");
    eprintln!(
        "  xtop plugin install <name>  Install a plugin from github.com/xscriptor/xtop/plugins/"
    );
    eprintln!("  xtop plugin install <url>   Install a plugin from a git URL");
    eprintln!("  xtop plugin scaffold <name> Create a new plugin crate");
}

/// Check if a string looks like a git URL (not a simple name).
fn is_git_url(s: &str) -> bool {
    s.contains("://") || s.contains("github.com") || s.contains("git@")
}

fn cmd_plugin_list() {
    let workspace_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("Cargo.toml");

    let content = match fs::read_to_string(&workspace_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading workspace Cargo.toml: {e}");
            return;
        }
    };

    // Parse workspace members for plugin crates
    let mut in_members = false;
    let mut plugins: Vec<String> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("members") {
            in_members = true;
            continue;
        }
        if in_members {
            if trimmed == "]" {
                break;
            }
            let name = trimmed.trim_matches(',').trim().trim_matches('"');
            if name.starts_with("plugins/xtop-plugin-") || name.starts_with("crates/xtop-plugin-") {
                plugins.push(name.to_string());
            }
        }
    }

    if plugins.is_empty() {
        println!("No plugins installed.");
        return;
    }
    println!("Installed plugins:");
    for p in &plugins {
        println!("  {p}");
    }
}

fn cmd_plugin_install(name_or_url: &str) -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let workspace_toml = workspace_dir.join("Cargo.toml");
    let cli_toml = manifest_dir.join("Cargo.toml");
    let plugins_dir = workspace_dir.join("plugins");

    let tmp = std::env::temp_dir().join("xtop-plugin-install");
    let _ = fs::remove_dir_all(&tmp);

    let repo_url: &str;
    let mut plugin_subdir: String = String::new();

    if is_git_url(name_or_url) {
        // URL-based: clone the repo directly
        repo_url = name_or_url;
        println!("Cloning {repo_url} ...");
        let status = std::process::Command::new("git")
            .args(["clone", repo_url, tmp.to_str().unwrap()])
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run git: {e}"))?;
        if !status.success() {
            anyhow::bail!("git clone failed");
        }
    } else {
        // Name-based: look in xtop repo's plugins/ directory
        repo_url = "https://github.com/xscriptor/xtop.git";
        let candidate_names = [
            format!("plugins/xtop-plugin-{name_or_url}"),
            format!("plugins/{name_or_url}"),
        ];
        println!("Looking for plugin '{name_or_url}' in {repo_url} ...");
        let status = std::process::Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--filter=blob:none",
                "--sparse",
                repo_url,
                tmp.to_str().unwrap(),
            ])
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run git: {e}"))?;
        if !status.success() {
            anyhow::bail!("git clone failed");
        }

        // Try each candidate path
        let mut found = false;
        for candidate in &candidate_names {
            if tmp.join(candidate).join("Cargo.toml").exists() {
                plugin_subdir = candidate.clone();
                found = true;
                break;
            }
        }
        if !found {
            let _ = fs::remove_dir_all(&tmp);
            anyhow::bail!(
                "Plugin '{name_or_url}' not found in plugins/. \
                 Tried: {}",
                candidate_names.join(", ")
            );
        }
        println!("Found plugin at {plugin_subdir}");
    }

    // --- Determine the plugin source directory ---
    let plugin_src = if plugin_subdir.is_empty() {
        // URL-based: cloned repo root
        tmp.clone()
    } else {
        // Name-based: subdirectory within cloned xtop repo
        tmp.join(&plugin_subdir)
    };

    // --- Read the plugin's Cargo.toml to get the package name ---
    let plugin_toml_path = plugin_src.join("Cargo.toml");
    let plugin_toml_content = fs::read_to_string(&plugin_toml_path)
        .map_err(|e| anyhow::anyhow!("No Cargo.toml found: {e}"))?;
    let plugin_pkg: toml::Value = plugin_toml_content
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid Cargo.toml: {e}"))?;

    let pkg_name = plugin_pkg
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow::anyhow!("package.name not found in plugin Cargo.toml"))?;

    let feature_name = pkg_name.replace('-', "_");
    let plugin_dir_name = pkg_name.replace('-', "_");

    println!("Package name: {pkg_name}");

    // --- Copy into local plugins/ directory ---
    let target_dir = plugins_dir.join(&plugin_dir_name);
    if target_dir.exists() {
        anyhow::bail!(
            "Plugin '{}' already exists at plugins/{plugin_dir_name}",
            pkg_name
        );
    }
    fs::create_dir_all(&plugins_dir)?;
    cp_recursive(&plugin_src, &target_dir)?;

    // --- Add to workspace Cargo.toml ---
    let ws_content = fs::read_to_string(&workspace_toml)?;
    let member_entry = format!("    \"plugins/{plugin_dir_name}\"");
    if ws_content.contains(&member_entry) {
        anyhow::bail!("Already in workspace");
    }
    // Insert before the closing bracket of members
    let sentinel_entry = "    \"plugins/xtop-plugin-sentinel\",";
    let new_ws = if ws_content.contains(sentinel_entry) {
        ws_content.replace(
            sentinel_entry,
            &format!("{sentinel_entry}\n{member_entry},"),
        )
    } else {
        // Fallback: insert before the closing ] of members
        ws_content.replacen("]", &format!("  {member_entry},\n]"), 1)
    };
    fs::write(&workspace_toml, &new_ws)?;

    // --- Add to xtop-cli Cargo.toml ---
    let cli_content = fs::read_to_string(&cli_toml)?;

    // Build dependency path relative to crates/xtop-cli/
    let dep_path = format!("../../plugins/{plugin_dir_name}");
    let dep_line = format!("{pkg_name} = {{ path = \"{dep_path}\", optional = true }}");

    if !cli_content.contains(&dep_line) {
        // Find the last optional plugin dependency and insert after it
        let marker = "# Optional plugins (behind feature flags)";
        let new_cli = cli_content.replace(marker, &format!("{marker}\n{dep_line}"));
        fs::write(&cli_toml, &new_cli)?;
    }

    // Add feature flag
    let feature_line = format!("{feature_name} = [\"dep:{pkg_name}\"]");
    let cli_content2 = fs::read_to_string(&cli_toml)?;
    if !cli_content2.contains(&feature_line) {
        let sentinel_feature = "plugin-sentinel = [\"dep:xtop-plugin-sentinel\"]";
        let new_cli2 = if cli_content2.contains(sentinel_feature) {
            cli_content2.replace(
                sentinel_feature,
                &format!("{sentinel_feature}\n{feature_line}"),
            )
        } else {
            cli_content2.replacen("[features]", &format!("[features]\n{feature_line}"), 1)
        };
        fs::write(&cli_toml, &new_cli2)?;
    }

    // --- Rebuild ---
    println!("Building xtop with {pkg_name} ...");
    let build = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&workspace_dir)
        .status()
        .map_err(|e| anyhow::anyhow!("cargo build failed: {e}"))?;
    if !build.success() {
        anyhow::bail!("Build failed. Check the plugin's compatibility.");
    }

    // --- Cleanup ---
    let _ = fs::remove_dir_all(&tmp);

    println!();
    println!("Plugin '{pkg_name}' installed successfully.");
    println!("  Location: plugins/{plugin_dir_name}");
    println!("  Feature flag: {feature_name}");
    println!();
    println!("Note: '{feature_name}' is NOT enabled by default.");
    println!("To enable it, add '{feature_name}' to the 'default' feature list");
    println!("in crates/xtop-cli/Cargo.toml, then rebuild.");

    Ok(())
}

fn cmd_plugin_scaffold(name: &str) -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let plugins_dir = workspace_dir.join("plugins");
    let plugin_dir = plugins_dir.join(format!("xtop-plugin-{name}"));

    if plugin_dir.exists() {
        anyhow::bail!("Plugin crate already exists at {}", plugin_dir.display());
    }

    let src_dir = plugin_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Cargo.toml (path refs go up from plugins/ to workspace root, then into crates/)
    let cargo_toml = format!(
        r#"[package]
name = "xtop-plugin-{name}"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "xtop plugin: {name}"

[dependencies]
xtop-core = {{ path = "../../crates/xtop-core" }}
ratatui.workspace = true
"#
    );
    fs::write(plugin_dir.join("Cargo.toml"), &cargo_toml)?;

    // lib.rs
    let lib_rs = format!(
        r#"use xtop_core::domain::plugin::{{Plugin, PluginCapability, PluginContext, PluginError, PluginManifest}};

pub struct {name_cap}Plugin;

impl {name_cap}Plugin {{
    pub fn new() -> Self {{
        Self
    }}
}}

impl Plugin for {name_cap}Plugin {{
    fn manifest(&self) -> PluginManifest {{
        PluginManifest {{
            id: "{name}".to_string(),
            name: "{name_cap}".to_string(),
            version: "0.1.0".to_string(),
            description: "xtop plugin: {name}".to_string(),
            capabilities: vec![PluginCapability::ReadSystemInfo],
        }}
    }}

    fn on_tick(&mut self, _ctx: &mut PluginContext) -> Result<(), PluginError> {{
        Ok(())
    }}
}}
"#,
        name = name,
        name_cap = {
            let mut chars = name.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        }
    );
    fs::write(src_dir.join("lib.rs"), &lib_rs)?;

    println!("Plugin scaffold created at {}", plugin_dir.display());
    println!("To register it:");
    println!("  1. Add \"plugins/xtop-plugin-{name}\" to [workspace].members in Cargo.toml");
    println!("  2. Add dependency + feature flag in crates/xtop-cli/Cargo.toml");
    println!("  3. Add #[cfg(feature = \"plugin-{name}\")] import in main.rs");
    println!("  4. Implement Plugin trait methods");

    Ok(())
}

fn cp_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if file_type.is_dir() {
                // Skip .git directory
                if entry.file_name() != ".git" {
                    cp_recursive(&src_path, &dst_path)?;
                }
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    } else {
        fs::copy(src, dst)?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // CLI subcommands
    if args.len() > 1 {
        match args[1].as_str() {
            "mcp" => {
                ensure_default_assets();
                return mcp::run_mcp_server();
            }
            "plugin" => {
                if args.len() < 3 {
                    print_usage();
                    return Ok(());
                }
                match args[2].as_str() {
                    "list" => {
                        cmd_plugin_list();
                        return Ok(());
                    }
                    "install" => {
                        if args.len() < 4 {
                            eprintln!("Usage: xtop plugin install <git-url>");
                            return Ok(());
                        }
                        return cmd_plugin_install(&args[3]);
                    }
                    "scaffold" => {
                        if args.len() < 4 {
                            eprintln!("Usage: xtop plugin scaffold <name>");
                            return Ok(());
                        }
                        return cmd_plugin_scaffold(&args[3]);
                    }
                    _ => {
                        print_usage();
                        return Ok(());
                    }
                }
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            _ => {}
        }
    }

    ensure_default_assets();

    terminal::install_panic_hook();
    let mut terminal = terminal::init()?;

    let cfg_dir = config_dir();

    // Build the primary (composite) provider
    let sysinfo_provider = SysinfoProvider::new();
    let composite = CompositeProvider::new(Box::new(sysinfo_provider));

    let themes = load_all_themes();
    let cfg = config::load_config();
    let mut builtin_layouts = layout_loader::builtin_layouts();
    let custom_layouts = layout_loader::load_custom_layouts();
    builtin_layouts.extend(custom_layouts);
    let mut state = AppState::new(Box::new(composite), themes, cfg, builtin_layouts);

    // Build and register plugins
    let plugin_mgr = build_plugin_manager(&mut state, &cfg_dir);

    // Collect extra data providers from plugins and inject everything into state
    let extra_providers = plugin_mgr.collect_data_providers();
    state.init_plugins(plugin_mgr, extra_providers);

    let tick_rate = Duration::from_millis(state.update_interval_ms);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| render::render(f, &state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_default();

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                let key_str = key_event_to_str(&key);

                // Give plugins first chance to consume the key
                let key_str_clone = key_str.clone();
                let key_consumed =
                    state.with_plugin_manager_mut(|mgr, this| mgr.handle_key(this, &key_str_clone));
                if key_consumed {
                    continue;
                }

                // DEBUG: print key for diagnostics
                if cfg!(debug_assertions) && !key_str.is_empty() {
                    eprintln!("[key] '{key_str}'");
                }

                match state.input_mode {
                    InputMode::Normal => {
                        // Direct Ctrl+P check (works regardless of keybinding config, important on macOS)
                        if key_str == "ctrl+p" {
                            state.open_palette();
                            state.input_mode = InputMode::CommandPalette;
                        } else if let Some(action) = state.keybindings.resolve(&key_str) {
                            match action {
                                Action::Quit => {
                                    save_config(&state);
                                    state.quit();
                                }
                                Action::Cancel if state.show_help => {
                                    state.toggle_help();
                                }
                                Action::OpenCommandPalette => {
                                    state.open_palette();
                                    state.input_mode = InputMode::CommandPalette;
                                }
                                Action::KillProcess | Action::ProcessUp | Action::ProcessDown => {
                                    state.execute_action(&action);
                                }
                                _ => {
                                    state.execute_action(&action);
                                }
                            }
                        }
                    }
                    InputMode::Searching => match key.code {
                        KeyCode::Esc => {
                            state.search_query.clear();
                            state.end_search();
                        }
                        KeyCode::Enter => {
                            state.end_search();
                        }
                        KeyCode::Backspace => {
                            state.search_pop_char();
                        }
                        KeyCode::Char(c) => {
                            state.search_push_char(c);
                        }
                        _ => {}
                    },
                    InputMode::CommandPalette => {
                        let is_main = state.palette.page == PalettePage::Main;
                        match key.code {
                            KeyCode::Esc => {
                                state.close_palette();
                            }
                            KeyCode::Enter => {
                                if let Some(action) = state.palette_selected_action() {
                                    state.execute_action(&action);
                                    save_config(&state);
                                }
                            }
                            KeyCode::Down => {
                                state.palette_select_next();
                            }
                            KeyCode::Up => {
                                state.palette_select_prev();
                            }
                            KeyCode::Char(c) => {
                                state.palette.query.push(c);
                                state.palette_filter();
                            }
                            KeyCode::Backspace => {
                                if state.palette.query.is_empty() && !is_main {
                                    state.palette_navigate_to(PalettePage::Main);
                                } else {
                                    state.palette.query.pop();
                                    state.palette_filter();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            state.on_tick();
            last_tick = Instant::now();
        }

        if state.should_quit {
            break;
        }
    }

    // Disable plugins on shutdown
    state.with_plugin_manager_mut(|mgr, this| {
        mgr.disable_all(this);
    });

    terminal::restore()?;
    Ok(())
}
