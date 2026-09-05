//! State area: live application state and metrics history.
//!
//! `app` holds the full `AppState`; `history` keeps historical snapshots for
//! charts; `view` groups the view-control types (fullscreen, input mode,
//! palette). The persisted config schema lives under `config`.

pub mod app;
#[cfg(test)]
mod app_tests;
pub mod history;
mod palette;
mod proc_history;
mod users;
pub mod view;
mod widget_state;

pub use app::*;
pub use view::*;
