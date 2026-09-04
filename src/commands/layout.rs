//! `xtop layout` subcommands: validate layout files and install community
//! layouts from `layouts/custom/` of the `xtop-cli/layouts` repo into the
//! user config dir.

use std::fs;
use std::path::Path;

const LAYOUTS_REPO: &str = "https://github.com/xtop-cli/layouts";

/// Handle `xtop layout <sub>` from the parsed argument vector.
pub fn layout_command(args: &[String]) -> anyhow::Result<()> {
    if args.len() < 3 {
        eprintln!("Usage: xtop layout <install|check>");
        return Ok(());
    }
    match args[2].as_str() {
        "install" => {
            if args.len() < 4 {
                eprintln!("Usage: xtop layout install <name>");
                return Ok(());
            }
            cmd_install(&args[3])
        }
        "check" => {
            if args.len() < 4 {
                eprintln!("Usage: xtop layout check <file.jsonc|file.json>");
                return Ok(());
            }
            cmd_check(Path::new(&args[3]))
        }
        other => {
            eprintln!("Unknown layout subcommand: {other}");
            Ok(())
        }
    }
}

/// Validate a layout file against the schema.
fn cmd_check(path: &Path) -> anyhow::Result<()> {
    let data = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", path.display()))?;
    match xtop_layout::parse_layout_err(&data) {
        Ok(def) => {
            println!("OK  {} -> layout \"{}\" is valid", path.display(), def.name);
            Ok(())
        }
        Err(e) => anyhow::bail!("INVALID {} -> {e}", path.display()),
    }
}

/// Install a community layout from the repo's `layouts/custom/` folder.
fn cmd_install(name: &str) -> anyhow::Result<()> {
    let tmp = std::env::temp_dir().join("xtop-layout-install");
    let _ = fs::remove_dir_all(&tmp);
    println!("Fetching community layouts from {LAYOUTS_REPO} ...");
    let status = std::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--filter=blob:none",
            "--sparse",
            LAYOUTS_REPO,
            tmp.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;
    if !status.success() {
        anyhow::bail!("git clone failed");
    }
    let checkout = std::process::Command::new("git")
        .args(["sparse-checkout", "set", "layouts/custom"])
        .current_dir(&tmp)
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;
    if !checkout.success() {
        anyhow::bail!("git sparse-checkout failed");
    }

    let custom_dir = tmp.join("layouts").join("custom");
    let needle = name.to_lowercase();
    let mut found: Option<(String, String)> = None; // (filename, content)
    if let Ok(entries) = fs::read_dir(&custom_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());
            if !matches!(ext, Some("json") | Some("jsonc")) {
                continue;
            }
            let content = fs::read_to_string(&path).unwrap_or_default();
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            let matches = stem == needle
                || xtop_layout::parse_layout(&content)
                    .map(|d| d.name.to_lowercase() == needle)
                    .unwrap_or(false);
            if matches {
                found = Some((entry.file_name().to_string_lossy().to_string(), content));
                break;
            }
        }
    }
    let _ = fs::remove_dir_all(&tmp);

    let Some((file_name, content)) = found else {
        anyhow::bail!("no community layout named '{name}' in layouts/custom/ of {LAYOUTS_REPO}");
    };

    let target_dir = crate::config::config_dir().join("layouts");
    fs::create_dir_all(&target_dir)?;
    let target = target_dir.join(&file_name);
    if target.exists() {
        anyhow::bail!(
            "{} already exists (edit it in place instead)",
            target.display()
        );
    }
    fs::write(&target, content)?;
    println!("Installed '{name}' -> {}", target.display());
    println!("Cycle layouts with 'l' (or restart xtop) to use it.");
    Ok(())
}
