//! System-integration commands: autostart and window management.

use tauri::{AppHandle, Manager};

use super::e;

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
