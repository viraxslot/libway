//! Repository CRUD and "unseen update" bookkeeping.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use super::{row_to_repo, Repo, SourceKind, REPO_COLUMNS};

/// All repositories, most recently added first.
pub fn list_repos(conn: &Connection) -> Result<Vec<Repo>> {
    let sql = format!("SELECT {REPO_COLUMNS} FROM repos ORDER BY created_at DESC, id DESC");
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
    conn.execute("UPDATE repos SET has_unseen = 0 WHERE id = ?1", params![id])?;
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::db::*;

    #[test]
    fn add_list_remove() {
        let db = Db::open_in_memory().unwrap();
        let c = db.lock();

        let id = add_repo(&c, "cli", "cli", 100).unwrap();
        add_repo(&c, "BurntSushi", "ripgrep", 200).unwrap();

        let repos = list_repos(&c).unwrap();
        assert_eq!(repos.len(), 2);
        // Most recently added first (ripgrep has the larger created_at).
        assert_eq!(repos[0].name, "ripgrep");
        assert_eq!(repos[1].owner, "cli");
        assert!(!repos[0].has_unseen);
        assert!(repos[0].latest_version.is_none());

        remove_repo(&c, id).unwrap();
        assert_eq!(list_repos(&c).unwrap().len(), 1);
    }

    #[test]
    fn duplicate_repo_fails() {
        let db = Db::open_in_memory().unwrap();
        let c = db.lock();
        add_repo(&c, "cli", "cli", 1).unwrap();
        assert!(add_repo(&c, "cli", "cli", 2).is_err());
    }

    #[test]
    fn version_update_and_seen() {
        let db = Db::open_in_memory().unwrap();
        let c = db.lock();
        let id = add_repo(&c, "cli", "cli", 1).unwrap();

        update_version(
            &c,
            id,
            "v2.40.0",
            "https://example/r",
            SourceKind::Release,
            10,
        )
        .unwrap();
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
        let c = db.lock();
        let a = add_repo(&c, "a", "a", 1).unwrap();
        let b = add_repo(&c, "b", "b", 2).unwrap();
        update_version(&c, a, "1", "u", SourceKind::Tag, 5).unwrap();
        update_version(&c, b, "1", "u", SourceKind::Tag, 5).unwrap();
        assert!(any_unseen(&c).unwrap());
        mark_all_seen(&c).unwrap();
        assert!(!any_unseen(&c).unwrap());
    }
}
