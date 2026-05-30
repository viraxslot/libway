//! Tauri commands — the bridge invoked from the React frontend.
//!
//! Commands return `Result<_, String>` because Tauri serializes the error
//! variant to the frontend; we stringify anyhow errors for display.

use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager, State};

use crate::checker;
use crate::db::{self, Db, Repo};
use crate::scheduler;
use crate::{github, keychain, tray};

/// Map any error into a String for the frontend.
fn e<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

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
pub async fn add_repo(
    app: AppHandle,
    db: State<'_, Db>,
    full_name: String,
) -> Result<Vec<Repo>, String> {
    let (owner, name) = parse_full_name(&full_name)?;

    // Verify the repository exists on GitHub before storing it, so typos and
    // non-existent repos don't end up in the list.
    let token = keychain::get_token().unwrap_or(None);
    match github::repo_exists(&owner, &name, token.as_deref()).await {
        Ok(true) => {}
        Ok(false) => return Err(format!("repository {owner}/{name} was not found on GitHub")),
        Err(err) => return Err(format!("could not verify {owner}/{name}: {err}")),
    }

    {
        let conn = db.0.lock().unwrap();
        db::add_repo(&conn, &owner, &name, now()).map_err(e)?;
    }
    tray::refresh(&app, &db).map_err(e)?;
    let conn = db.0.lock().unwrap();
    db::list_repos(&conn).map_err(e)
}

#[tauri::command]
pub fn remove_repo(app: AppHandle, db: State<'_, Db>, id: i64) -> Result<Vec<Repo>, String> {
    {
        let conn = db.0.lock().unwrap();
        db::remove_repo(&conn, id).map_err(e)?;
    }
    tray::refresh(&app, &db).map_err(e)?;
    let conn = db.0.lock().unwrap();
    db::list_repos(&conn).map_err(e)
}

#[tauri::command]
pub fn set_repo_tags(
    app: AppHandle,
    db: State<'_, Db>,
    id: i64,
    tags: Vec<String>,
) -> Result<Vec<Repo>, String> {
    {
        let conn = db.0.lock().unwrap();
        db::set_repo_tags(&conn, id, &tags).map_err(e)?;
    }
    tray::refresh(&app, &db).map_err(e)?;
    let conn = db.0.lock().unwrap();
    db::list_repos(&conn).map_err(e)
}

#[tauri::command]
pub fn mark_seen(app: AppHandle, db: State<'_, Db>, id: i64) -> Result<(), String> {
    {
        let conn = db.0.lock().unwrap();
        db::mark_seen(&conn, id).map_err(e)?;
    }
    tray::refresh(&app, &db).map_err(e)
}

#[tauri::command]
pub fn mark_all_seen(app: AppHandle, db: State<'_, Db>) -> Result<(), String> {
    {
        let conn = db.0.lock().unwrap();
        db::mark_all_seen(&conn).map_err(e)?;
    }
    tray::refresh(&app, &db).map_err(e)
}

// --- Check interval ---

/// Current check interval in minutes (falls back to the default).
#[tauri::command]
pub fn get_check_interval(db: State<'_, Db>) -> Result<u64, String> {
    Ok(scheduler::interval_minutes(&db))
}

/// Set the check interval in minutes. Must be >= 1.
#[tauri::command]
pub fn set_check_interval(db: State<'_, Db>, minutes: u64) -> Result<(), String> {
    if minutes < 1 {
        return Err("interval must be at least 1 minute".to_string());
    }
    let conn = db.0.lock().unwrap();
    db::set_setting(&conn, scheduler::SETTING_INTERVAL, &minutes.to_string()).map_err(e)
}

/// Whether a check runs immediately at startup.
#[tauri::command]
pub fn get_check_on_startup(db: State<'_, Db>) -> Result<bool, String> {
    Ok(scheduler::check_on_startup(&db))
}

/// Set whether a check runs immediately at startup.
#[tauri::command]
pub fn set_check_on_startup(db: State<'_, Db>, enabled: bool) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    let value = if enabled { "1" } else { "0" };
    db::set_setting(&conn, scheduler::SETTING_CHECK_ON_STARTUP, value).map_err(e)
}

/// Trigger an immediate check of all repositories. Returns the refreshed list.
#[tauri::command]
pub async fn check_now(app: AppHandle, db: State<'_, Db>) -> Result<Vec<Repo>, String> {
    checker::check_all(&app, &db).await.map_err(e)?;
    let conn = db.0.lock().unwrap();
    db::list_repos(&conn).map_err(e)
}

// --- GitHub token (Keychain) ---

#[tauri::command]
pub fn has_token() -> Result<bool, String> {
    keychain::has_token().map_err(e)
}

#[tauri::command]
pub fn set_token(token: String) -> Result<(), String> {
    let token = token.trim();
    if token.is_empty() {
        // Empty input means "clear the token".
        return keychain::delete_token().map_err(e);
    }
    keychain::set_token(token).map_err(e)
}

#[tauri::command]
pub fn clear_token() -> Result<(), String> {
    keychain::delete_token().map_err(e)
}

// --- Autostart (filled in during step 8) ---

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().map_err(e)
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(e)
    } else {
        manager.disable().map_err(e)
    }
}

/// Open the settings window (used from the tray menu).
#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("settings") {
        win.show().map_err(e)?;
        win.set_focus().map_err(e)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_full_name;

    #[test]
    fn parses_plain() {
        assert_eq!(parse_full_name("cli/cli").unwrap(), ("cli".into(), "cli".into()));
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
        assert!(parse_full_name("nope").is_err());
        assert!(parse_full_name("a/b/c").is_err());
        assert!(parse_full_name("/x").is_err());
    }
}
