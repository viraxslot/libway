//! Native macOS notifications via tauri-plugin-notification.

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::db::Repo;

/// Show a notification with the given title and body.
fn show(app: &AppHandle, title: impl Into<String>, body: impl Into<String>) {
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
pub fn notify_new_version(app: &AppHandle, repo: &Repo, version: &str) {
    show(app, format!("{}/{}", repo.owner, repo.name), format!("New version: {version}"));
}

/// Notify the user of the result of a manual "Check now" run.
pub fn notify_check_result(app: &AppHandle, updated: &[Repo]) {
    match updated.len() {
        0 => show(app, "libway", "All repositories are up to date."),
        1 => {
            let r = &updated[0];
            show(app, "libway", format!("Update found: {}/{}", r.owner, r.name));
        }
        n => show(app, "libway", format!("{n} updates found.")),
    }
}
