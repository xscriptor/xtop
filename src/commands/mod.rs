//! CLI commands: interactive run, MCP server and plugin management.
//!
//! Shared assembly and asset helpers live under [`share`].

pub mod mcp;
pub mod plugins;
pub mod run;
pub(crate) mod share;

pub(crate) use share::*;
