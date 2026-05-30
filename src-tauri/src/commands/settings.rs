//! Commands for the check schedule and triggering checks on demand.

use tauri::{AppHandle, State};

use super::e;
use crate::checker;
use crate::db::{self, Db, Repo};
use crate::github;
use crate::scheduler;

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
pub async fn check_now<R: tauri::Runtime>(
    app: AppHandle<R>,
    db: State<'_, Db>,
    client: State<'_, Box<dyn github::GitHubApi>>,
) -> Result<Vec<Repo>, String> {
    checker::check_all(&app, &db, client.inner().as_ref())
        .await
        .map_err(e)?;
    let conn = db.0.lock().unwrap();
    db::list_repos(&conn).map_err(e)
}
