//! Tauri commands — the bridge invoked from the React frontend.
//!
//! Commands return `Result<_, String>` because Tauri serializes the error
//! variant to the frontend; we stringify anyhow errors for display.
//!
//! The commands are split into submodules by topic (repos, settings, token,
//! system) and re-exported here so callers keep referring to them as
//! `commands::<name>` — this keeps the `generate_handler!` lists unchanged.

use std::time::{SystemTime, UNIX_EPOCH};

mod repos;
mod settings;
mod system;
mod token;

pub use repos::*;
pub use settings::*;
pub use system::*;
pub use token::*;

/// Map any error into a String for the frontend.
pub(crate) fn e<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}

pub(crate) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
