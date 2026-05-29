//! SQLite storage layer (rusqlite).
//!
//! Holds the list of tracked repositories and key-value settings.
//! The GitHub token is NOT stored here — it lives in the Keychain (see
//! `keychain.rs`).
//!
//! The connection is wrapped in a `Mutex` and placed in `tauri::State` so that
//! commands and the background scheduler share a single database.

use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

/// Where a version was obtained from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Release,
    Tag,
}

impl SourceKind {
    fn as_str(self) -> &'static str {
        match self {
            SourceKind::Release => "release",
            SourceKind::Tag => "tag",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "release" => Some(SourceKind::Release),
            "tag" => Some(SourceKind::Tag),
            _ => None,
        }
    }
}

/// A tracked repository together with its current state.
/// Serialized as camelCase for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
    pub id: i64,
    pub owner: String,
    pub name: String,
    pub latest_version: Option<String>,
    pub latest_url: Option<String>,
    pub source_kind: Option<SourceKind>,
    pub has_unseen: bool,
    pub last_checked_at: Option<i64>,
}

/// Wrapper around the SQLite connection for storage in `tauri::State`.
pub struct Db(pub Mutex<Connection>);

impl Db {
    /// Open the database at `path` and apply the schema.
    pub fn open(path: &PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {parent:?}"))?;
        }
        let conn = Connection::open(path)
            .with_context(|| format!("failed to open database {path:?}"))?;
        init_schema(&conn)?;
        Ok(Db(Mutex::new(conn)))
    }

    /// Open an in-memory database — for tests.
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;
        Ok(Db(Mutex::new(conn)))
    }
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS repos (
            id              INTEGER PRIMARY KEY,
            owner           TEXT NOT NULL,
            name            TEXT NOT NULL,
            latest_version  TEXT,
            latest_url      TEXT,
            source_kind     TEXT,
            has_unseen      INTEGER NOT NULL DEFAULT 0,
            last_checked_at INTEGER,
            created_at      INTEGER NOT NULL,
            UNIQUE(owner, name)
        );

        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT
        );
        "#,
    )
    .context("failed to apply database schema")?;
    Ok(())
}

/// Map a result row into a `Repo`.
fn row_to_repo(row: &rusqlite::Row) -> rusqlite::Result<Repo> {
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
    })
}

const REPO_COLUMNS: &str =
    "id, owner, name, latest_version, latest_url, source_kind, has_unseen, last_checked_at";

/// All repositories in insertion order.
pub fn list_repos(conn: &Connection) -> Result<Vec<Repo>> {
    let sql = format!("SELECT {REPO_COLUMNS} FROM repos ORDER BY created_at, id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_repo)?;
    let mut repos = Vec::new();
    for r in rows {
        repos.push(r?);
    }
    Ok(repos)
}

/// Add a repository. `now` is the current unix time (passed in by the caller).
/// Returns the new row id. Fails with a clear error on duplicates.
pub fn add_repo(conn: &Connection, owner: &str, name: &str, now: i64) -> Result<i64> {
    conn.execute(
        "INSERT INTO repos (owner, name, created_at) VALUES (?1, ?2, ?3)",
        params![owner, name, now],
    )
    .with_context(|| format!("repository {owner}/{name} is already tracked or invalid"))?;
    Ok(conn.last_insert_rowid())
}

/// Remove a repository by id.
pub fn remove_repo(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM repos WHERE id = ?1", params![id])?;
    Ok(())
}

/// Update the discovered version of a repository after a check.
/// `has_unseen` is set to true only when the version actually changed.
pub fn update_version(
    conn: &Connection,
    id: i64,
    version: &str,
    url: &str,
    kind: SourceKind,
    now: i64,
) -> Result<()> {
    conn.execute(
        "UPDATE repos
            SET latest_version = ?1,
                latest_url = ?2,
                source_kind = ?3,
                has_unseen = 1,
                last_checked_at = ?4
          WHERE id = ?5",
        params![version, url, kind.as_str(), now, id],
    )?;
    Ok(())
}

/// Record a check time without changing the version (when there is no new one).
pub fn touch_checked(conn: &Connection, id: i64, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE repos SET last_checked_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    Ok(())
}

/// Clear the "unseen" flag on a single repository.
pub fn mark_seen(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE repos SET has_unseen = 0 WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

/// Clear the "unseen" flag on all repositories.
pub fn mark_all_seen(conn: &Connection) -> Result<()> {
    conn.execute("UPDATE repos SET has_unseen = 0", [])?;
    Ok(())
}

/// Whether any repository has an unseen update.
pub fn any_unseen(conn: &Connection) -> Result<bool> {
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM repos WHERE has_unseen = 1", [], |r| {
            r.get(0)
        })?;
    Ok(count > 0)
}

/// Read a setting value.
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let value = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |r| r.get::<_, String>(0),
        )
        .optional()?;
    Ok(value)
}

/// Write a setting value.
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn(db: &Db) -> std::sync::MutexGuard<'_, Connection> {
        db.0.lock().unwrap()
    }

    #[test]
    fn add_list_remove() {
        let db = Db::open_in_memory().unwrap();
        let c = conn(&db);

        let id = add_repo(&c, "cli", "cli", 100).unwrap();
        add_repo(&c, "BurntSushi", "ripgrep", 200).unwrap();

        let repos = list_repos(&c).unwrap();
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].owner, "cli"); // ordered by created_at
        assert_eq!(repos[1].name, "ripgrep");
        assert!(!repos[0].has_unseen);
        assert!(repos[0].latest_version.is_none());

        remove_repo(&c, id).unwrap();
        assert_eq!(list_repos(&c).unwrap().len(), 1);
    }

    #[test]
    fn duplicate_repo_fails() {
        let db = Db::open_in_memory().unwrap();
        let c = conn(&db);
        add_repo(&c, "cli", "cli", 1).unwrap();
        assert!(add_repo(&c, "cli", "cli", 2).is_err());
    }

    #[test]
    fn version_update_and_seen() {
        let db = Db::open_in_memory().unwrap();
        let c = conn(&db);
        let id = add_repo(&c, "cli", "cli", 1).unwrap();

        update_version(&c, id, "v2.40.0", "https://example/r", SourceKind::Release, 10).unwrap();
        let repos = list_repos(&c).unwrap();
        assert_eq!(repos[0].latest_version.as_deref(), Some("v2.40.0"));
        assert_eq!(repos[0].source_kind, Some(SourceKind::Release));
        assert!(repos[0].has_unseen);
        assert!(any_unseen(&c).unwrap());

        mark_seen(&c, id).unwrap();
        assert!(!any_unseen(&c).unwrap());
        assert!(!list_repos(&c).unwrap()[0].has_unseen);

        // touch_checked does not change the version
        touch_checked(&c, id, 99).unwrap();
        assert_eq!(list_repos(&c).unwrap()[0].last_checked_at, Some(99));
    }

    #[test]
    fn mark_all_seen_clears_everything() {
        let db = Db::open_in_memory().unwrap();
        let c = conn(&db);
        let a = add_repo(&c, "a", "a", 1).unwrap();
        let b = add_repo(&c, "b", "b", 2).unwrap();
        update_version(&c, a, "1", "u", SourceKind::Tag, 5).unwrap();
        update_version(&c, b, "1", "u", SourceKind::Tag, 5).unwrap();
        assert!(any_unseen(&c).unwrap());
        mark_all_seen(&c).unwrap();
        assert!(!any_unseen(&c).unwrap());
    }

    #[test]
    fn settings_roundtrip() {
        let db = Db::open_in_memory().unwrap();
        let c = conn(&db);
        assert!(get_setting(&c, "interval").unwrap().is_none());
        set_setting(&c, "interval", "10").unwrap();
        assert_eq!(get_setting(&c, "interval").unwrap().as_deref(), Some("10"));
        set_setting(&c, "interval", "5").unwrap(); // upsert
        assert_eq!(get_setting(&c, "interval").unwrap().as_deref(), Some("5"));
    }
}
