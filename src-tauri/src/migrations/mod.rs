//! Schema migrations for the SQLite database.
//!
//! Migrations are an ordered list of SQL scripts, one per `.sql` file in this
//! directory and embedded at compile time. Each entry is applied once, in
//! order; the database's `PRAGMA user_version` tracks how many have run.
//!
//! To evolve the schema, ADD a new `NNN_name.sql` file and APPEND it to
//! `MIGRATIONS` — never edit or reorder existing ones (the index is the version
//! number). Multiple statements per file are fine (separated by `;`). DDL
//! auto-commits in SQLite, so no explicit transaction is needed.

use anyhow::{Context, Result};
use rusqlite::Connection;

/// Ordered schema migrations; the array index is the version number. Each entry
/// is the contents of the matching `.sql` file, embedded at compile time.
const MIGRATIONS: &[&str] = &[
    include_str!("001_initial.sql"),
    include_str!("002_repo_tags.sql"),
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
