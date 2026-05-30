//! SQLite storage layer (rusqlite).
//!
//! Holds the list of tracked repositories and key-value settings.
//! The GitHub token is NOT stored here — it lives in the Keychain (see
//! `keychain.rs`).
//!
//! The connection is wrapped in a `Mutex` and placed in `tauri::State` so that
//! commands and the background scheduler share a single database.
//!
//! The layer is split by topic — domain types (`model`), repository CRUD
//! (`repos`), tag operations (`tags`), and key-value settings (`settings`) —
//! and re-exported here so callers keep using `db::<item>` unchanged.

use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result};
use rusqlite::Connection;

mod model;
mod repos;
mod settings;
mod tags;

pub use model::{Repo, SourceKind};
pub use repos::*;
pub use settings::*;
pub use tags::*;

/// Wrapper around the SQLite connection for storage in `tauri::State`.
pub struct Db(pub Mutex<Connection>);

impl Db {
    /// Open the database at `path` and apply the schema.
    pub fn open(path: &PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {parent:?}"))?;
        }
        let conn =
            Connection::open(path).with_context(|| format!("failed to open database {path:?}"))?;
        init_schema(&conn)?;
        Ok(Db(Mutex::new(conn)))
    }

    /// Open an in-memory database. Used by unit tests and the integration
    /// test crate (which cannot see `#[cfg(test)]` items).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;
        Ok(Db(Mutex::new(conn)))
    }
}

/// Apply the schema to a fresh or existing connection.
/// Migration definitions live in `crate::migrations`.
fn init_schema(conn: &Connection) -> Result<()> {
    crate::migrations::run(conn)
}

/// Columns selected when mapping a row into a [`Repo`]; shared by the queries
/// in `repos` and `tags`.
pub(super) const REPO_COLUMNS: &str = "id, owner, name, latest_version, latest_url, source_kind, \
     has_unseen, last_checked_at, tags";

/// Map a result row into a `Repo`.
pub(super) fn row_to_repo(row: &rusqlite::Row) -> rusqlite::Result<Repo> {
    let source_kind: Option<String> = row.get("source_kind")?;
    Ok(Repo {
        id: row.get("id")?,
        owner: row.get("owner")?,
        name: row.get("name")?,
        latest_version: row.get("latest_version")?,
        latest_url: row.get("latest_url")?,
        source_kind: source_kind.as_deref().and_then(SourceKind::from_str),
        has_unseen: row.get::<_, i64>("has_unseen")? != 0,
        last_checked_at: row.get("last_checked_at")?,
        tags: tags::split_tags(&row.get::<_, String>("tags")?),
    })
}
