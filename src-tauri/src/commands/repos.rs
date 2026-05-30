//! Commands for managing tracked repositories and their tags.

use tauri::{AppHandle, Emitter, State};

use super::{e, now};
use crate::db::{self, Db, Repo};
use crate::github;

/// Parse an "owner/name" string into its two parts.
fn parse_full_name(input: &str) -> Result<(String, String), String> {
    let trimmed = input.trim().trim_start_matches("https://github.com/");
    let trimmed = trimmed.trim_end_matches('/');
    let mut parts = trimmed.splitn(2, '/');
    let owner = parts.next().unwrap_or("").trim();
    let name = parts.next().unwrap_or("").trim();
    if owner.is_empty() || name.is_empty() || name.contains('/') {
        return Err("expected the format owner/name".to_string());
    }
    Ok((owner.to_string(), name.to_string()))
}

#[tauri::command]
pub fn list_repos(db: State<'_, Db>) -> Result<Vec<Repo>, String> {
    let conn = db.0.lock().unwrap();
    db::list_repos(&conn).map_err(e)
}

#[tauri::command]
pub async fn add_repo<R: tauri::Runtime>(
    app: AppHandle<R>,
    db: State<'_, Db>,
    client: State<'_, Box<dyn github::GitHubApi>>,
    full_name: String,
) -> Result<Vec<Repo>, String> {
    let (owner, name) = parse_full_name(&full_name)?;

    // Verify the repository exists on GitHub before storing it, so typos and
    // non-existent repos don't end up in the list.
    match client.repo_exists(&owner, &name).await {
        Ok(true) => {}
        Ok(false) => return Err(format!("repository {owner}/{name} was not found on GitHub")),
        Err(err) => return Err(format!("could not verify {owner}/{name}: {err}")),
    }

    {
        let conn = db.0.lock().unwrap();
        db::add_repo(&conn, &owner, &name, now()).map_err(e)?;
    }
    app.emit("repos-updated", ()).map_err(e)?;
    let conn = db.0.lock().unwrap();
    db::list_repos(&conn).map_err(e)
}

#[tauri::command]
pub fn remove_repo<R: tauri::Runtime>(
    app: AppHandle<R>,
    db: State<'_, Db>,
    id: i64,
) -> Result<Vec<Repo>, String> {
    {
        let conn = db.0.lock().unwrap();
        db::remove_repo(&conn, id).map_err(e)?;
    }
    app.emit("repos-updated", ()).map_err(e)?;
    let conn = db.0.lock().unwrap();
    db::list_repos(&conn).map_err(e)
}

#[tauri::command]
pub fn set_repo_tags<R: tauri::Runtime>(
    app: AppHandle<R>,
    db: State<'_, Db>,
    id: i64,
    tags: Vec<String>,
) -> Result<Vec<Repo>, String> {
    {
        let conn = db.0.lock().unwrap();
        db::set_repo_tags(&conn, id, &tags).map_err(e)?;
    }
    app.emit("repos-updated", ()).map_err(e)?;
    let conn = db.0.lock().unwrap();
    db::list_repos(&conn).map_err(e)
}

#[tauri::command]
pub fn rename_tag<R: tauri::Runtime>(
    app: AppHandle<R>,
    db: State<'_, Db>,
    from: String,
    to: String,
) -> Result<Vec<Repo>, String> {
    {
        let conn = db.0.lock().unwrap();
        db::rename_tag(&conn, &from, &to).map_err(e)?;
    }
    app.emit("repos-updated", ()).map_err(e)?;
    let conn = db.0.lock().unwrap();
    db::list_repos(&conn).map_err(e)
}

#[tauri::command]
pub fn delete_tag<R: tauri::Runtime>(
    app: AppHandle<R>,
    db: State<'_, Db>,
    tag: String,
) -> Result<Vec<Repo>, String> {
    {
        let conn = db.0.lock().unwrap();
        db::delete_tag(&conn, &tag).map_err(e)?;
    }
    app.emit("repos-updated", ()).map_err(e)?;
    let conn = db.0.lock().unwrap();
    db::list_repos(&conn).map_err(e)
}

#[tauri::command]
pub fn mark_seen<R: tauri::Runtime>(
    app: AppHandle<R>,
    db: State<'_, Db>,
    id: i64,
) -> Result<(), String> {
    {
        let conn = db.0.lock().unwrap();
        db::mark_seen(&conn, id).map_err(e)?;
    }
    app.emit("repos-updated", ()).map_err(e)
}

#[tauri::command]
pub fn mark_all_seen<R: tauri::Runtime>(
    app: AppHandle<R>,
    db: State<'_, Db>,
) -> Result<(), String> {
    {
        let conn = db.0.lock().unwrap();
        db::mark_all_seen(&conn).map_err(e)?;
    }
    app.emit("repos-updated", ()).map_err(e)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::parse_full_name;

    #[test]
    fn parses_plain() {
        assert_eq!(
            parse_full_name("cli/cli").unwrap(),
            ("cli".into(), "cli".into())
        );
    }

    #[test]
    fn trims_url_and_slashes() {
        assert_eq!(
            parse_full_name("https://github.com/BurntSushi/ripgrep/").unwrap(),
            ("BurntSushi".into(), "ripgrep".into())
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse_full_name("nope").is_err()); // no slash
        assert!(parse_full_name("a/b/c").is_err()); // too many parts
        assert!(parse_full_name("/x").is_err()); // empty owner
        assert!(parse_full_name("owner/").is_err()); // empty name
    }
}
