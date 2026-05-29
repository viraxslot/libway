//! Core check logic shared by the manual `check_now` command and the
//! background scheduler.
//!
//! Walks every tracked repository, fetches its latest version from GitHub,
//! and on a newer version updates the database, fires a notification, and
//! refreshes the tray. Returns the list of repos that got a new version.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use tauri::{AppHandle, Emitter};

use crate::db::{self, Db, Repo};
use crate::{github, keychain, notify, tray};

/// Current unix time in seconds.
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Run a check over all tracked repositories.
///
/// Network calls happen outside the DB lock; the lock is only taken for short
/// reads and writes. After the run the tray is rebuilt to reflect new state.
pub async fn check_all(app: &AppHandle, db: &Db) -> Result<Vec<Repo>> {
    let repos = {
        let conn = db.0.lock().unwrap();
        db::list_repos(&conn)?
    };

    let token = keychain::get_token().unwrap_or(None);
    let mut updated = Vec::new();

    for repo in repos {
        match github::fetch_latest(&repo.owner, &repo.name, token.as_deref()).await {
            Ok(latest) => {
                let is_new = github::is_newer(&latest.version, repo.latest_version.as_deref());
                let ts = now();
                let conn = db.0.lock().unwrap();
                if is_new {
                    db::update_version(
                        &conn,
                        repo.id,
                        &latest.version,
                        &latest.url,
                        latest.source_kind,
                        ts,
                    )?;
                    // Only notify when this is not the very first discovery,
                    // i.e. we already had a known version before.
                    if repo.latest_version.is_some() {
                        notify::notify_new_version(app, &repo, &latest.version);
                    }
                    updated.push(repo);
                } else {
                    db::touch_checked(&conn, repo.id, ts)?;
                }
            }
            Err(e) => {
                // A single repo failing (typo, deleted, rate limit) must not
                // abort the whole run.
                eprintln!("libway: check failed for {}/{}: {e:#}", repo.owner, repo.name);
            }
        }
    }

    tray::refresh(app, db)?;
    // Let an open settings window reload its list.
    let _ = app.emit("repos-updated", ());
    Ok(updated)
}
