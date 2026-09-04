//! UI area of xtop: terminal setup, screen composition, overlays and the
//! widget engine.
//!
//! - [`terminal`]      terminal backend lifecycle (raw mode, alternate screen)
//! - [`screen`]        top-level composition (fullscreen, minimal, layout)
//! - [`layout`]        layout engine: rect split + widget pack resolution
//! - [`overlay`]       kernel-owned UI chrome (help, command palette)
//! - [`share`]         UI-wide shared logic (colors, formatting)

pub mod layout;
pub mod overlay;
pub mod screen;
pub mod share;
pub mod terminal;

pub use screen::*;
pub use terminal::*;
