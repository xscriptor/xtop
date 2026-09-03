//! xtop CLI entry point.
//!
//! The binary is a thin dispatcher; the app is organized in areas:
//! `commands`, `config`, `layout`, `plugins`, `providers`, `state`, `theme`
//! and `ui`.

mod commands;
mod config;
mod layout;
mod plugins;
mod providers;
mod state;
mod theme;
mod ui;

use commands::{ensure_default_assets, mcp, run};

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  xtop                        Start the TUI system monitor");
    eprintln!("  xtop mcp                    Start MCP server (stdio transport) for AI agents");
    eprintln!("  xtop plugin list            List installed plugins");
    eprintln!(
        "  xtop plugin install <name>  Install a plugin from github.com/xtop-cli/xtop/plugins/"
    );
    eprintln!("  xtop plugin install <url>   Install a plugin from a git URL");
    eprintln!("  xtop plugin scaffold <name> Create a new plugin crate");
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // CLI subcommands.
    if args.len() > 1 {
        match args[1].as_str() {
            "mcp" => {
                ensure_default_assets();
                return mcp::run_mcp_server();
            }
            "plugin" => {
                return commands::plugins::plugin_command(&args);
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            _ => {}
        }
    }

    ensure_default_assets();
    run::run()
}
