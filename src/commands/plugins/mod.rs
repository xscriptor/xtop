//! Plugin management subcommands (list, install, scaffold).
//!
//! Since the kernel became a monocrate, plugins live in their own repos
//! (`xtop-cli/plugins`) and are integrated through Cargo git dependencies +
//! feature flags, exactly like the built-in `xtop-plugin-samurai`. These
//! commands edit the kernel's own `Cargo.toml` accordingly.

use std::path::Path;

mod install;
mod list;
mod scaffold;

pub const PLUGINS_REPO: &str = "https://github.com/xtop-cli/plugins";
pub(crate) const LOCAL_PLUGINS_MARKER: &str = "# Local plugin installs (xtop plugin install)";

/// Kernel repo root: `Cargo.toml` lives right at `CARGO_MANIFEST_DIR`.
pub(crate) fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Handle `xtop plugin <sub>` from the parsed argument vector.
pub fn plugin_command(args: &[String]) -> anyhow::Result<()> {
    if args.len() < 3 {
        eprintln!("Usage: xtop plugin <list|install|scaffold>");
        return Ok(());
    }
    match args[2].as_str() {
        "list" => {
            list::cmd_plugin_list();
            Ok(())
        }
        "install" => {
            if args.len() < 4 {
                eprintln!("Usage: xtop plugin install <git-url|name>");
                return Ok(());
            }
            install::cmd_plugin_install(&args[3])
        }
        "scaffold" => {
            if args.len() < 4 {
                eprintln!("Usage: xtop plugin scaffold <name>");
                return Ok(());
            }
            scaffold::cmd_plugin_scaffold(&args[3])
        }
        _ => {
            eprintln!("Unknown plugin subcommand: {}", args[2]);
            Ok(())
        }
    }
}
