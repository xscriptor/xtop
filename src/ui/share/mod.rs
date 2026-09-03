//! UI-wide shared logic used by multiple widgets.
//!
//! Widgets never reach outside `share` for rendering helpers; screen-level
//! error handling also belongs here when it grows.

mod color;
mod format;

pub use color::*;
pub use format::*;
