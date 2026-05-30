//! System tray icon and menu.
//!
//! The menu is rebuilt from the repository list on every change (Tauri menus
//! are immutable once built, so `refresh` constructs a fresh menu and assigns
//! it to the tray). Menu item ids encode the repo id so clicks can be routed
//! back to "open release + mark seen".
//!
//! Split by concern: `menu` builds the menu tree, `events` routes clicks, and
//! this module owns the icon lifecycle and the `repos:updated` wiring.

use std::sync::Mutex;

use anyhow::{Context, Result};
use tauri::{
    image::Image,
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Listener, Manager, Wry,
};

use crate::db::{self, Db};
use crate::events::Event;

mod events;
mod menu;

/// Holds the tray icon handle so `refresh` can update it later.
pub struct TrayState(pub Mutex<Option<TrayIcon<Wry>>>);

// Stable ids for the fixed menu items, shared between `menu` (which builds
// them) and `events` (which routes their clicks).
pub(super) const ID_CHECK_NOW: &str = "check_now";
pub(super) const ID_MARK_ALL: &str = "mark_all_seen";
pub(super) const ID_SETTINGS: &str = "settings";
pub(super) const ID_ABOUT_GITHUB: &str = "about_github";
pub(super) const ID_QUIT: &str = "quit";
/// Repo items use the prefix "repo:" followed by the numeric id.
pub(super) const REPO_PREFIX: &str = "repo:";

/// Tray icon bytes embedded at compile time.
const ICON_IDLE: &[u8] = include_bytes!("../../icons/tray.png");
const ICON_NEW: &[u8] = include_bytes!("../../icons/tray-new.png");

/// Create the tray icon during app setup and store its handle in state.
pub fn create(app: &AppHandle) -> Result<()> {
    let icon = Image::from_bytes(ICON_IDLE).context("failed to load tray icon")?;

    let tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .icon_as_template(true) // monochrome menu-bar styling on macOS
        .on_menu_event(events::handle_menu_event)
        .build(app)
        .context("failed to build the tray icon")?;

    app.manage(TrayState(Mutex::new(Some(tray))));

    // Rebuild the tray menu whenever the repository list changes. Commands
    // and the background checker emit `repos:updated`; we refresh from here so
    // the command layer does not depend on the tray (and stays testable on
    // MockRuntime). refresh() does not emit, so this does not recurse.
    let handle = app.clone();
    app.listen(Event::ReposUpdated.as_str(), move |_event| {
        let db = handle.state::<Db>();
        if let Err(e) = refresh(&handle, &db) {
            eprintln!("libway: tray refresh failed: {e:#}");
        }
    });

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

    let menu = menu::build_menu(app, &repos, any_unseen)?;

    let state = app.state::<TrayState>();
    let guard = state.0.lock().unwrap();
    if let Some(tray) = guard.as_ref() {
        tray.set_menu(Some(menu))
            .context("failed to set tray menu")?;

        let bytes = if any_unseen { ICON_NEW } else { ICON_IDLE };
        let icon = Image::from_bytes(bytes).context("failed to load tray icon")?;
        tray.set_icon(Some(icon))
            .context("failed to set tray icon")?;
        tray.set_icon_as_template(true).ok();
    }
    Ok(())
}
