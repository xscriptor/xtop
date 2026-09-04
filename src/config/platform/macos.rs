//! macOS config dir: `$HOME/Library/Application Support/xtop`.

use std::path::PathBuf;

use super::shared::env_path;

pub fn config_dir() -> PathBuf {
    env_path(&["HOME"])
        .unwrap_or_default()
        .join("Library")
        .join("Application Support")
        .join("xtop")
}
