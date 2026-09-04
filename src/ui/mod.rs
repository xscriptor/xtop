//! UI area of xtop: terminal setup, screen composition, overlays and the
//! widget engine.
//!
//! - [`terminal`]      terminal backend lifecycle (raw mode, alternate screen)
//! - [`screen`]        top-level composition (fullscreen, minimal, layout)
//! - [`layout`]        layout engine: rect split + widget pack resolution
//! - [`effects`]       optional frame-effect host (feature `effects`)
//! - [`overlay`]       kernel-owned UI chrome (help, command palette)

pub mod layout;
pub mod overlay;
pub mod screen;
pub mod terminal;

pub mod effects;

pub use screen::*;
pub use terminal::*;
