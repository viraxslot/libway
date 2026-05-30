//! Routing tray menu clicks to actions.

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

use super::{
    ID_ABOUT_GITHUB, ID_CHECK_NOW, ID_MARK_ALL, ID_QUIT, ID_SELF_UPDATE, ID_SETTINGS, REPO_PREFIX,
};
use crate::db::{self, Db};
use crate::events::Event;

/// Project repository, opened from the About submenu.
const REPO_URL: &str = "https://github.com/viraxslot/libway";

/// Route a menu click to the right action.
pub(super) fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref().to_string();

    match id.as_str() {
        ID_QUIT => app.exit(0),
        ID_SETTINGS => open_settings(app),
        ID_MARK_ALL => mark_all(app),
        ID_ABOUT_GITHUB => {
            if let Err(e) = app.opener().open_url(REPO_URL, None::<&str>) {
                eprintln!("libway: failed to open repo url: {e:#}");
            }
        }
        ID_SELF_UPDATE => {
            if let Some(update) = app.state::<crate::selfupdate::SelfUpdate>().get() {
                if let Err(e) = app.opener().open_url(update.url, None::<&str>) {
                    eprintln!("libway: failed to open update url: {e:#}");
                }
            }
        }
        ID_CHECK_NOW => {
            // Run the check off the UI thread; refresh happens inside check_all.
            // Notify the result so the manual action isn't silent.
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let db = app.state::<Db>();
                let client = app.state::<Box<dyn crate::github::GitHubApi>>();
                match crate::checker::check_all(&app, &db, client.inner().as_ref()).await {
                    Ok(updated) => crate::notify::notify_check_result(&app, &updated),
                    Err(e) => eprintln!("libway: manual check failed: {e:#}"),
                }
            });
        }
        other => {
            if let Some(rest) = other.strip_prefix(REPO_PREFIX) {
                if let Ok(repo_id) = rest.parse::<i64>() {
                    open_repo(app, repo_id);
                }
            }
        }
    }
}

/// Open a repo's release page and clear its unseen flag.
fn open_repo(app: &AppHandle, repo_id: i64) {
    let db = app.state::<Db>();
    let url = db
        .with(db::list_repos)
        .ok()
        .and_then(|repos| repos.into_iter().find(|r| r.id == repo_id))
        .and_then(|r| r.latest_url);

    if let Some(url) = url {
        if let Err(e) = app.opener().open_url(url, None::<&str>) {
            eprintln!("libway: failed to open url: {e:#}");
        }
    }

    let _ = db.with(|c| db::mark_seen(c, repo_id));
    // Notify an open settings window since this change came from the tray.
    // The tray itself refreshes via the `repos:updated` listener.
    let _ = app.emit(Event::ReposUpdated.as_str(), ());
}

/// Clear the unseen flag on all repos (the tray "Mark all as read" item).
fn mark_all(app: &AppHandle) {
    let db = app.state::<Db>();
    let _ = db.with(db::mark_all_seen);
    let _ = app.emit(Event::ReposUpdated.as_str(), ());
}

/// Show and focus the settings window.
fn open_settings(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}
