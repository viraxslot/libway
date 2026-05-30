//! Commands for the check schedule and triggering checks on demand.

use tauri::{AppHandle, Emitter, Manager, State};

use super::e;
use crate::checker;
use crate::db::{self, Db, Repo};
use crate::events::Event;
use crate::github;
use crate::scheduler;
use crate::selfupdate::{self, SelfUpdate};

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
    db.with(|c| db::set_setting(c, scheduler::SETTING_INTERVAL, &minutes.to_string()))
        .map_err(e)
}

/// Whether a check runs immediately at startup.
#[tauri::command]
pub fn get_check_on_startup(db: State<'_, Db>) -> Result<bool, String> {
    Ok(scheduler::check_on_startup(&db))
}

/// Set whether a check runs immediately at startup.
#[tauri::command]
pub fn set_check_on_startup(db: State<'_, Db>, enabled: bool) -> Result<(), String> {
    let value = if enabled { "1" } else { "0" };
    db.with(|c| db::set_setting(c, scheduler::SETTING_CHECK_ON_STARTUP, value))
        .map_err(e)
}

/// Whether the self-update check is enabled.
#[tauri::command]
pub fn get_check_self_update(db: State<'_, Db>) -> Result<bool, String> {
    Ok(selfupdate::enabled(&db))
}

/// Enable or disable the self-update check. Disabling clears any pending
/// update and refreshes the tray so the item disappears immediately.
#[tauri::command]
pub fn set_check_self_update<R: tauri::Runtime>(
    app: AppHandle<R>,
    db: State<'_, Db>,
    enabled: bool,
) -> Result<(), String> {
    let value = if enabled { "1" } else { "0" };
    db.with(|c| db::set_setting(c, selfupdate::SETTING_CHECK_SELF_UPDATE, value))
        .map_err(e)?;

    if !enabled {
        app.state::<SelfUpdate>().set(None);
        let _ = app.emit(Event::ReposUpdated.as_str(), ());
    }
    Ok(())
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
    db.with(db::list_repos).map_err(e)
}
