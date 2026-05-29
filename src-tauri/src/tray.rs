//! System tray icon and menu.
//!
//! The menu is rebuilt from the repository list on every change (Tauri menus
//! are immutable once built, so `refresh` constructs a fresh menu and assigns
//! it to the tray). Menu item ids encode the repo id so clicks can be routed
//! back to "open release + mark seen".

use std::sync::Mutex;

use anyhow::{Context, Result};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Emitter, Manager, Wry,
};
use tauri_plugin_opener::OpenerExt;

use crate::db::{self, Db};

/// Holds the tray icon handle so `refresh` can update it later.
pub struct TrayState(pub Mutex<Option<TrayIcon<Wry>>>);

// Stable ids for the fixed menu items.
const ID_CHECK_NOW: &str = "check_now";
const ID_MARK_ALL: &str = "mark_all_seen";
const ID_SETTINGS: &str = "settings";
const ID_QUIT: &str = "quit";
// Repo items use the prefix "repo:" followed by the numeric id.
const REPO_PREFIX: &str = "repo:";

/// Tray icon bytes embedded at compile time.
const ICON_IDLE: &[u8] = include_bytes!("../icons/tray.png");
const ICON_NEW: &[u8] = include_bytes!("../icons/tray-new.png");

/// Create the tray icon during app setup and store its handle in state.
pub fn create(app: &AppHandle) -> Result<()> {
    let icon = Image::from_bytes(ICON_IDLE).context("failed to load tray icon")?;

    let tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .icon_as_template(true) // monochrome menu-bar styling on macOS
        .on_menu_event(handle_menu_event)
        .build(app)
        .context("failed to build the tray icon")?;

    app.manage(TrayState(Mutex::new(Some(tray))));

    // Build the initial menu from current data.
    let db = app.state::<Db>();
    refresh(app, &db)
}

/// Rebuild the tray menu and swap the icon based on current state.
pub fn refresh(app: &AppHandle, db: &Db) -> Result<()> {
    let (repos, any_unseen) = {
        let conn = db.0.lock().unwrap();
        (db::list_repos(&conn)?, db::any_unseen(&conn)?)
    };

    let menu = build_menu(app, &repos, any_unseen)?;

    let state = app.state::<TrayState>();
    let guard = state.0.lock().unwrap();
    if let Some(tray) = guard.as_ref() {
        tray.set_menu(Some(menu))
            .context("failed to set tray menu")?;

        let bytes = if any_unseen { ICON_NEW } else { ICON_IDLE };
        let icon = Image::from_bytes(bytes).context("failed to load tray icon")?;
        tray.set_icon(Some(icon)).context("failed to set tray icon")?;
        tray.set_icon_as_template(true).ok();
    }
    Ok(())
}

/// Build the menu: one entry per repo, then Check now / Mark all as read /
/// Settings / Quit.
fn build_menu(app: &AppHandle, repos: &[db::Repo], any_unseen: bool) -> Result<Menu<Wry>> {
    let menu = Menu::new(app)?;

    if repos.is_empty() {
        let empty = MenuItem::with_id(app, "noop", "No repositories", false, None::<&str>)?;
        menu.append(&empty)?;
    } else {
        for repo in repos {
            let version = repo.latest_version.as_deref().unwrap_or("…");
            let mark = if repo.has_unseen { " ●" } else { "" };
            let label = format!("{}/{}  {}{}", repo.owner, repo.name, version, mark);
            let id = format!("{REPO_PREFIX}{}", repo.id);
            let item = MenuItem::with_id(app, id, label, true, None::<&str>)?;
            menu.append(&item)?;
        }
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        ID_CHECK_NOW,
        "Check now",
        true,
        None::<&str>,
    )?)?;
    // Enabled only when there is something to clear.
    menu.append(&MenuItem::with_id(
        app,
        ID_MARK_ALL,
        "Mark all as read",
        any_unseen,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        ID_SETTINGS,
        "Settings…",
        true,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        ID_QUIT,
        "Quit",
        true,
        None::<&str>,
    )?)?;

    Ok(menu)
}

/// Route a menu click to the right action.
fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref().to_string();

    match id.as_str() {
        ID_QUIT => app.exit(0),
        ID_SETTINGS => open_settings(app),
        ID_MARK_ALL => mark_all(app),
        ID_CHECK_NOW => {
            // Run the check off the UI thread; refresh happens inside check_all.
            // Notify the result so the manual action isn't silent.
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let db = app.state::<Db>();
                match crate::checker::check_all(&app, &db).await {
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
    let url = {
        let conn = db.0.lock().unwrap();
        db::list_repos(&conn)
            .ok()
            .and_then(|repos| repos.into_iter().find(|r| r.id == repo_id))
            .and_then(|r| r.latest_url)
    };

    if let Some(url) = url {
        if let Err(e) = app.opener().open_url(url, None::<&str>) {
            eprintln!("libway: failed to open url: {e:#}");
        }
    }

    {
        let conn = db.0.lock().unwrap();
        let _ = db::mark_seen(&conn, repo_id);
    }
    let _ = refresh(app, &db);
    // Notify an open settings window since this change came from the tray.
    let _ = app.emit("repos-updated", ());
}

/// Clear the unseen flag on all repos (the tray "Mark all as read" item).
fn mark_all(app: &AppHandle) {
    let db = app.state::<Db>();
    {
        let conn = db.0.lock().unwrap();
        let _ = db::mark_all_seen(&conn);
    }
    let _ = refresh(app, &db);
    let _ = app.emit("repos-updated", ());
}

/// Show and focus the settings window.
fn open_settings(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}
