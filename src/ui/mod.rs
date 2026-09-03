//! UI area of xtop: terminal setup, screen composition and widgets.
//!
//! - [`terminal`]      terminal backend lifecycle (raw mode, alternate screen)
//! - [`screen`]        top-level composition (fullscreen, minimal, layout)
//! - [`layout`]        layout engine + built-in widget registry
//! - [`widgets`]       one folder per widget
//! - [`share`]         UI-wide shared logic (colors, formatting, errors)

pub mod layout;
pub mod screen;
pub mod share;
pub mod terminal;
pub mod widgets;

pub use screen::*;
pub use terminal::*;
