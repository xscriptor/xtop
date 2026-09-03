//! Widgets area: one folder per widget.
//!
//! Every widget exposes a `render(f, state, area)` entry point. Folders let
//! widgets subdivide into their own modules (and `share/`) as they grow.

pub mod battery;
pub mod cpu;
pub mod disk_io;
pub mod gpu;
pub mod header;
pub mod help;
pub mod memory;
pub mod network;
pub mod palette;
pub mod processes;
pub mod storage;
