//! Native macOS notifications via tauri-plugin-notification.

use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::{NotificationExt, PermissionState};

use crate::db::Repo;

/// Request notification permission at startup if it has not been granted yet.
/// Without this, `show()` silently fails on macOS.
pub fn ensure_permission<R: Runtime>(app: &AppHandle<R>) {
    let notif = app.notification();
    let granted = matches!(notif.permission_state(), Ok(PermissionState::Granted));
    if !granted {
        if let Err(e) = notif.request_permission() {
            eprintln!("libway: failed to request notification permission: {e:#}");
        }
    }
}

/// Show a notification with the given title and body.
fn show<R: Runtime>(app: &AppHandle<R>, title: impl Into<String>, body: impl Into<String>) {
    if let Err(e) = app
        .notification()
        .builder()
        .title(title.into())
        .body(body.into())
        .show()
    {
        eprintln!("libway: failed to show notification: {e:#}");
    }
}

/// Notify the user that a tracked repository has a new version.
pub fn notify_new_version<R: Runtime>(app: &AppHandle<R>, repo: &Repo, version: &str) {
    show(
        app,
        format!("{}/{}", repo.owner, repo.name),
        format!("New version: {version}"),
    );
}

/// Notify the user of the result of a manual "Check now" run.
pub fn notify_check_result<R: Runtime>(app: &AppHandle<R>, updated: &[Repo]) {
    match updated.len() {
        0 => show(app, "libway", "All repositories are up to date."),
        1 => {
            let r = &updated[0];
            show(
                app,
                "libway",
                format!("Update found: {}/{}", r.owner, r.name),
            );
        }
        n => show(app, "libway", format!("{n} updates found.")),
    }
}
