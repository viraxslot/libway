//! System tray icon and menu.
//!
//! The menu is rebuilt from the repository list on every change (Tauri menus
//! are immutable once built, so `refresh` constructs a fresh menu and assigns
//! it to the tray). Menu item ids encode the repo id so clicks can be routed
//! back to "open release + mark seen".

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Emitter, Manager, Wry,
};
use tauri_plugin_opener::OpenerExt;

use crate::db::{self, Db, Repo};

/// Tag bucket name for repositories without any tags.
const UNGROUPED: &str = "Ungrouped";

/// Holds the tray icon handle so `refresh` can update it later.
pub struct TrayState(pub Mutex<Option<TrayIcon<Wry>>>);

// Stable ids for the fixed menu items.
const ID_CHECK_NOW: &str = "check_now";
const ID_MARK_ALL: &str = "mark_all_seen";
const ID_SETTINGS: &str = "settings";
const ID_ABOUT_GITHUB: &str = "about_github";
const ID_QUIT: &str = "quit";
// Repo items use the prefix "repo:" followed by the numeric id.
const REPO_PREFIX: &str = "repo:";

/// Project repository, opened from the About submenu.
const REPO_URL: &str = "https://github.com/viraxslot/libway";

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

/// Current unix time in seconds.
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// A human "N minutes ago" string for a unix timestamp.
fn relative_time(ts: i64) -> String {
    let secs = (now() - ts).max(0);
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

/// The non-clickable status line shown at the top of the menu.
fn status_label(repos: &[Repo]) -> String {
    let unseen = repos.iter().filter(|r| r.has_unseen).count();
    let head = match unseen {
        0 => "All up to date".to_string(),
        1 => "1 update".to_string(),
        n => format!("{n} updates"),
    };
    // Oldest successful check across repos, if any.
    let last = repos.iter().filter_map(|r| r.last_checked_at).min();
    match last {
        Some(ts) => format!("{head} · checked {}", relative_time(ts)),
        None => format!("{head} · not checked yet"),
    }
}

/// Label for a single repository entry.
fn repo_label(repo: &Repo) -> String {
    let version = repo.latest_version.as_deref().unwrap_or("…");
    let mark = if repo.has_unseen { " ●" } else { "" };
    format!("{}/{} — {}{}", repo.owner, repo.name, version, mark)
}

/// Append one repository as a clickable item to a menu or submenu.
fn append_repo(app: &AppHandle, menu: &Submenu<Wry>, repo: &Repo) -> Result<()> {
    let id = format!("{REPO_PREFIX}{}", repo.id);
    let item = MenuItem::with_id(app, id, repo_label(repo), true, None::<&str>)?;
    menu.append(&item)?;
    Ok(())
}

/// Collect the sorted set of distinct tags across all repos.
fn distinct_tags(repos: &[Repo]) -> Vec<String> {
    let mut tags: Vec<String> = repos.iter().flat_map(|r| r.tags.clone()).collect();
    tags.sort();
    tags.dedup();
    tags
}

/// Build the menu: status line, the repositories (grouped by tag into
/// submenus when any tags exist, otherwise a flat list), then the actions.
fn build_menu(app: &AppHandle, repos: &[Repo], any_unseen: bool) -> Result<Menu<Wry>> {
    let menu = Menu::new(app)?;

    // Status line (disabled = non-clickable).
    let status = MenuItem::with_id(app, "status", status_label(repos), false, None::<&str>)?;
    menu.append(&status)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    if repos.is_empty() {
        let empty = MenuItem::with_id(app, "noop", "No repositories", false, None::<&str>)?;
        menu.append(&empty)?;
    } else if distinct_tags(repos).is_empty() {
        // No tags anywhere — keep a simple flat list.
        for repo in repos {
            let id = format!("{REPO_PREFIX}{}", repo.id);
            let item = MenuItem::with_id(app, id, repo_label(repo), true, None::<&str>)?;
            menu.append(&item)?;
        }
    } else {
        // One submenu per tag, plus an "Ungrouped" submenu for untagged repos.
        for tag in distinct_tags(repos) {
            let members: Vec<&Repo> =
                repos.iter().filter(|r| r.tags.contains(&tag)).collect();
            append_group(app, &menu, &tag, &members)?;
        }
        let untagged: Vec<&Repo> = repos.iter().filter(|r| r.tags.is_empty()).collect();
        if !untagged.is_empty() {
            append_group(app, &menu, UNGROUPED, &untagged)?;
        }
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, ID_CHECK_NOW, "Check now", true, None::<&str>)?)?;
    // Enabled only when there is something to clear.
    menu.append(&MenuItem::with_id(
        app,
        ID_MARK_ALL,
        "Mark all as read",
        any_unseen,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(app, ID_SETTINGS, "Settings…", true, None::<&str>)?)?;
    menu.append(&about_submenu(app)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, ID_QUIT, "Quit", true, None::<&str>)?)?;

    Ok(menu)
}

/// "About" submenu: version, authors and a link to the repository.
fn about_submenu(app: &AppHandle) -> Result<Submenu<Wry>> {
    let about = Submenu::with_id(app, "about", "About", true)?;

    let version = format!("libway v{}", env!("CARGO_PKG_VERSION"));
    about.append(&MenuItem::with_id(app, "about_version", version, false, None::<&str>)?)?;
    about.append(&MenuItem::with_id(
        app,
        "about_authors",
        "By Alexander Vershinin & Claude",
        false,
        None::<&str>,
    )?)?;
    about.append(&PredefinedMenuItem::separator(app)?)?;
    about.append(&MenuItem::with_id(
        app,
        ID_ABOUT_GITHUB,
        "View on GitHub",
        true,
        None::<&str>,
    )?)?;
    Ok(about)
}

/// Append a tag group as a submenu: "tag (count) ●", containing its repos.
fn append_group(app: &AppHandle, menu: &Menu<Wry>, tag: &str, members: &[&Repo]) -> Result<()> {
    let unseen = members.iter().any(|r| r.has_unseen);
    let mark = if unseen { " ●" } else { "" };
    let label = format!("{tag} ({}){mark}", members.len());
    let submenu = Submenu::with_id(app, format!("group:{tag}"), label, true)?;
    for repo in members {
        append_repo(app, &submenu, repo)?;
    }
    menu.append(&submenu)?;
    Ok(())
}

/// Route a menu click to the right action.
fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
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
