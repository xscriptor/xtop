//! State area: live application state and metrics history.
//!
//! `app` holds the full `AppState`; `history` keeps historical snapshots for
//! charts; `view` groups the view-control types (fullscreen, input mode,
//! palette). The persisted config schema lives under `config`.

pub mod app;
pub mod history;
pub mod view;

pub use app::*;
pub use view::*;
