//! Tag storage and the cross-repository tag operations.
//!
//! Tags are stored per repo as a comma-separated string. They are normalized
//! on write (trimmed, blanks dropped, sorted, de-duplicated case-insensitively
//! while preserving the first spelling seen).

use anyhow::Result;
use rusqlite::{params, Connection};

use super::repos::list_repos;

/// Serialize tags into the comma-separated form stored in the DB.
/// Tags are trimmed and sorted for stability; the original case is kept, but
/// de-duplication is case-insensitive (so "Build" and "build" don't coexist).
fn join_tags(tags: &[String]) -> String {
    let mut cleaned: Vec<String> = tags
        .iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();
    // Case-insensitive sort + dedup, keeping the first spelling of each tag.
    cleaned.sort_by_key(|t| t.to_lowercase());
    cleaned.dedup_by_key(|t| t.to_lowercase());
    cleaned.join(",")
}

/// Parse the comma-separated tag string from the DB into a list.
pub(super) fn split_tags(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

/// Replace the tags of a repository.
pub fn set_repo_tags(conn: &Connection, id: i64, tags: &[String]) -> Result<()> {
    conn.execute(
        "UPDATE repos SET tags = ?1 WHERE id = ?2",
        params![join_tags(tags), id],
    )?;
    Ok(())
}

/// Rename a tag across every repository (case-insensitive match on the source).
/// Renaming into a tag a repo already has merges them, because `join_tags`
/// de-duplicates case-insensitively. Returns how many repositories changed.
/// An empty `to`, or a `from` that no repo carries, is a no-op returning 0.
pub fn rename_tag(conn: &Connection, from: &str, to: &str) -> Result<usize> {
    let from_lc = from.trim().to_lowercase();
    let to = to.trim();
    if from_lc.is_empty() || to.is_empty() {
        return Ok(0);
    }
    let mut changed = 0;
    for repo in list_repos(conn)? {
        if !repo.tags.iter().any(|t| t.to_lowercase() == from_lc) {
            continue;
        }
        let next: Vec<String> = repo
            .tags
            .iter()
            .map(|t| {
                if t.to_lowercase() == from_lc {
                    to.to_string()
                } else {
                    t.clone()
                }
            })
            .collect();
        set_repo_tags(conn, repo.id, &next)?;
        changed += 1;
    }
    Ok(changed)
}

/// Remove a tag from every repository (case-insensitive match).
/// Returns how many repositories changed. Absent tag is a no-op returning 0.
pub fn delete_tag(conn: &Connection, tag: &str) -> Result<usize> {
    let tag_lc = tag.trim().to_lowercase();
    if tag_lc.is_empty() {
        return Ok(0);
    }
    let mut changed = 0;
    for repo in list_repos(conn)? {
        if !repo.tags.iter().any(|t| t.to_lowercase() == tag_lc) {
            continue;
        }
        let next: Vec<String> = repo
            .tags
            .iter()
            .filter(|t| t.to_lowercase() != tag_lc)
            .cloned()
            .collect();
        set_repo_tags(conn, repo.id, &next)?;
        changed += 1;
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::db::*;

    #[test]
    fn tags_are_normalized_and_roundtrip() {
        let db = Db::open_in_memory().unwrap();
        let c = db.0.lock().unwrap();
        let id = add_repo(&c, "cli", "cli", 1).unwrap();
        assert!(list_repos(&c).unwrap()[0].tags.is_empty());

        // Whitespace is trimmed, blanks dropped, and duplicates removed
        // case-insensitively — but the original case is preserved.
        set_repo_tags(
            &c,
            id,
            &[
                " Editors ".to_string(),
                "build".to_string(),
                "Build".to_string(),
                "".to_string(),
            ],
        )
        .unwrap();
        // "Build" is dropped as a case-insensitive dup of "build"; case kept.
        assert_eq!(list_repos(&c).unwrap()[0].tags, vec!["build", "Editors"]);

        // Clearing tags works.
        set_repo_tags(&c, id, &[]).unwrap();
        assert!(list_repos(&c).unwrap()[0].tags.is_empty());
    }

    #[test]
    fn rename_tag_simple() {
        let db = Db::open_in_memory().unwrap();
        let c = db.0.lock().unwrap();
        let a = add_repo(&c, "o", "a", 1).unwrap();
        let b = add_repo(&c, "o", "b", 2).unwrap();
        set_repo_tags(&c, a, &["build".to_string()]).unwrap();
        set_repo_tags(&c, b, &["editors".to_string()]).unwrap();

        let n = rename_tag(&c, "build", "ci").unwrap();
        assert_eq!(n, 1);
        // Most recently added first: b (created_at=2) precedes a (created_at=1).
        let repos = list_repos(&c).unwrap();
        assert_eq!(repos[0].tags, vec!["editors"]); // b untouched
        assert_eq!(repos[1].tags, vec!["ci"]); // a: build -> ci
    }

    #[test]
    fn rename_tag_merges_into_existing() {
        let db = Db::open_in_memory().unwrap();
        let c = db.0.lock().unwrap();
        let a = add_repo(&c, "o", "a", 1).unwrap();
        // One repo carries both the source and the target tag.
        set_repo_tags(&c, a, &["build".to_string(), "ci".to_string()]).unwrap();

        let n = rename_tag(&c, "build", "ci").unwrap();
        assert_eq!(n, 1);
        // "build" -> "ci" collides with the existing "ci"; dedup collapses them.
        assert_eq!(list_repos(&c).unwrap()[0].tags, vec!["ci"]);
    }

    #[test]
    fn rename_tag_is_case_insensitive_and_keeps_new_case() {
        let db = Db::open_in_memory().unwrap();
        let c = db.0.lock().unwrap();
        let a = add_repo(&c, "o", "a", 1).unwrap();
        set_repo_tags(&c, a, &["Build".to_string()]).unwrap();

        // Source matched case-insensitively; the new spelling is kept verbatim.
        let n = rename_tag(&c, "build", "CI").unwrap();
        assert_eq!(n, 1);
        assert_eq!(list_repos(&c).unwrap()[0].tags, vec!["CI"]);
    }

    #[test]
    fn rename_tag_absent_is_noop() {
        let db = Db::open_in_memory().unwrap();
        let c = db.0.lock().unwrap();
        let a = add_repo(&c, "o", "a", 1).unwrap();
        set_repo_tags(&c, a, &["build".to_string()]).unwrap();

        let n = rename_tag(&c, "missing", "ci").unwrap();
        assert_eq!(n, 0);
        assert_eq!(list_repos(&c).unwrap()[0].tags, vec!["build"]);
    }

    #[test]
    fn delete_tag_removes_from_all() {
        let db = Db::open_in_memory().unwrap();
        let c = db.0.lock().unwrap();
        let a = add_repo(&c, "o", "a", 1).unwrap();
        let b = add_repo(&c, "o", "b", 2).unwrap();
        set_repo_tags(&c, a, &["build".to_string(), "ci".to_string()]).unwrap();
        set_repo_tags(&c, b, &["Build".to_string()]).unwrap();

        let n = delete_tag(&c, "build").unwrap();
        assert_eq!(n, 2); // matched on both repos, case-insensitively
                          // Most recently added first: b (created_at=2) precedes a (created_at=1).
        let repos = list_repos(&c).unwrap();
        assert!(repos[0].tags.is_empty()); // b: Build deleted
        assert_eq!(repos[1].tags, vec!["ci"]); // a: build deleted, ci kept
    }

    #[test]
    fn delete_tag_absent_is_noop() {
        let db = Db::open_in_memory().unwrap();
        let c = db.0.lock().unwrap();
        let a = add_repo(&c, "o", "a", 1).unwrap();
        set_repo_tags(&c, a, &["build".to_string()]).unwrap();

        let n = delete_tag(&c, "missing").unwrap();
        assert_eq!(n, 0);
        assert_eq!(list_repos(&c).unwrap()[0].tags, vec!["build"]);
    }
}
