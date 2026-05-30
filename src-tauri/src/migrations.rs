//! Schema migrations for the SQLite database.
//!
//! Migrations are an ordered list of SQL strings. Each entry is applied once,
//! in order; the database's `PRAGMA user_version` tracks how many have run.
//!
//! To evolve the schema, APPEND a new SQL string to `MIGRATIONS` — never edit
//! or reorder existing ones (the index is the version number). One logical
//! migration = one entry; multiple statements per entry are fine (separated by
//! `;`). DDL auto-commits in SQLite, so no explicit transaction is needed.

use anyhow::{Context, Result};
use rusqlite::Connection;

/// Ordered schema migrations; the array index is the version number.
const MIGRATIONS: &[&str] = &[
    // v1 — initial schema. Uses IF NOT EXISTS so databases created before
    // this migration system (which used CREATE TABLE IF NOT EXISTS and left
    // user_version at 0) migrate cleanly instead of erroring on existing tables.
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
    // v2 — per-repo tags for grouping. Guarded against pre-migration databases
    // that may already have the column from the earlier ad-hoc migration.
    r#"
    ALTER TABLE repos ADD COLUMN tags TEXT NOT NULL DEFAULT '';
    "#,
];

/// Whether an error is SQLite's "duplicate column name" (already-applied
/// `ADD COLUMN` from a pre-migration database).
fn is_duplicate_column(err: &rusqlite::Error) -> bool {
    err.to_string().contains("duplicate column name")
}

/// Apply any migrations the database hasn't seen yet, tracked via
/// `PRAGMA user_version`.
pub fn run(conn: &Connection) -> Result<()> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let target = MIGRATIONS.len() as i64;

    for version in current..target {
        let sql = MIGRATIONS[version as usize];
        match conn.execute_batch(sql) {
            Ok(()) => {}
            // Tolerate a column that an earlier ad-hoc migration already added,
            // so databases from before this system converge cleanly.
            Err(ref err) if is_duplicate_column(err) => {}
            Err(err) => {
                return Err(
                    anyhow::Error::new(err).context(format!("migration {} failed", version + 1))
                );
            }
        }
        // user_version doesn't accept bound params; the value is our own i64.
        conn.execute_batch(&format!("PRAGMA user_version = {};", version + 1))
            .with_context(|| format!("failed to record migration {}", version + 1))?;
    }
    Ok(())
}

/// Number of migrations defined — exposed for tests.
#[cfg(test)]
pub fn count() -> i64 {
    MIGRATIONS.len() as i64
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn sets_user_version() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();
        let v: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, count());
    }

    #[test]
    fn is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();
        // Running again on an up-to-date DB is a no-op and the schema works.
        run(&conn).unwrap();
        conn.execute(
            "INSERT INTO repos (owner, name, created_at) VALUES ('a', 'b', 1)",
            [],
        )
        .unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM repos", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }
}
