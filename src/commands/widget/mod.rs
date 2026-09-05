//! Widget-pack management subcommands (list, install, scaffold).
//!
//! Widget packs live in their own crates (`xtop-widget-<name>` — the
//! installable/designable unit) and are integrated into the kernel exactly
//! like plugins: optional Cargo git/path dependencies + feature flags, plus
//! a row in the compile-time pack catalog (`ui/layout/pack_table.rs`) so the
//! render engine and `xtop widget list` enumerate one source of truth.
//! These commands edit the kernel's own sources accordingly; none of them
//! touch the user config or the runtime.

use std::path::Path;

mod install;
mod list;
mod scaffold;

pub const WIDGETS_REPO: &str = "https://github.com/xtop-cli/widgets";
pub(crate) const LOCAL_WIDGETS_MARKER: &str = "# Local widget pack installs (xtop widget install)";

/// Kernel repo root: `Cargo.toml` lives right at `CARGO_MANIFEST_DIR`.
pub(crate) fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// The pack-catalog source file `xtop widget install` self-edits.
pub(crate) fn pack_table_path() -> std::path::PathBuf {
    repo_root().join("src/ui/layout/pack_table.rs")
}

/// Handle `xtop widget <sub>` from the parsed argument vector.
pub fn widget_command(args: &[String]) -> anyhow::Result<()> {
    if args.len() < 3 {
        eprintln!("Usage: xtop widget <list|install|scaffold>");
        return Ok(());
    }
    match args[2].as_str() {
        "list" => {
            list::cmd_widget_list();
            Ok(())
        }
        "install" => {
            if args.len() < 4 {
                eprintln!("Usage: xtop widget install <repo|path|name>");
                return Ok(());
            }
            install::cmd_widget_install(&args[3])
        }
        "scaffold" => {
            if args.len() < 4 {
                eprintln!("Usage: xtop widget scaffold <name>");
                return Ok(());
            }
            scaffold::cmd_widget_scaffold(&args[3])
        }
        _ => {
            eprintln!("Unknown widget subcommand: {}", args[2]);
            Ok(())
        }
    }
}
