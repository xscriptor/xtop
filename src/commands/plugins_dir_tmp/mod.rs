//! Plugin management subcommands (list, install, scaffold).

use std::fs;

/// Handle `xtop plugin <sub>` from the parsed argument vector.
pub fn plugin_command(args: &[String]) -> anyhow::Result<()> {
    if args.len() < 3 {
        eprintln!("Usage: xtop plugin <list|install|scaffold>");
        return Ok(());
    }
    match args[2].as_str() {
        "list" => {
            cmd_plugin_list();
            Ok(())
        }
        "install" => {
            if args.len() < 4 {
                eprintln!("Usage: xtop plugin install <git-url|name>");
                return Ok(());
            }
            cmd_plugin_install(&args[3])
        }
        "scaffold" => {
            if args.len() < 4 {
                eprintln!("Usage: xtop plugin scaffold <name>");
                return Ok(());
            }
            cmd_plugin_scaffold(&args[3])
        }
        _ => {
            eprintln!("Unknown plugin subcommand: {}", args[2]);
            Ok(())
        }
    }
}

use std::path::PathBuf;

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
        repo_url = "https://github.com/xtop-cli/xtop.git";
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
    let samurai_entry = "    \"plugins/xtop-plugin-samurai\",";
    let new_ws = if ws_content.contains(samurai_entry) {
        ws_content.replace(samurai_entry, &format!("{samurai_entry}\n{member_entry},"))
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
        let samurai_feature = "plugin-samurai = [\"dep:xtop-plugin-samurai\"]";
        let new_cli2 = if cli_content2.contains(samurai_feature) {
            cli_content2.replace(
                samurai_feature,
                &format!("{samurai_feature}\n{feature_line}"),
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
        .current_dir(workspace_dir)
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
        r#"use xtop_plugin_api::{{Plugin, PluginCapability, PluginContext, PluginError, PluginManifest}};

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
