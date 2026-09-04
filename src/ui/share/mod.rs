//! UI-wide shared logic used by overlays and the screen.
//!
//! Data widgets draw with helpers from their own pack; the kernel keeps only
//! what its own chrome needs.

mod color;

pub use color::*;
