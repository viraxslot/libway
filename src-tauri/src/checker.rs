//! Core check logic shared by the manual `check_now` command and the
//! background scheduler.
//!
//! Walks every tracked repository, fetches its latest version from GitHub,
//! and on a newer version updates the database, fires a notification, and
//! refreshes the tray. Returns the list of repos that got a new version.

use anyhow::Result;
use tauri::{AppHandle, Emitter, Runtime};

use crate::db::{self, Db, Repo};
use crate::events::Event;
use crate::util::now;
use crate::{github, notify, version};

/// Run a check over all tracked repositories.
///
/// Network calls happen outside the DB lock; the lock is only taken for short
/// reads and writes. After the run the tray is rebuilt to reflect new state.
pub async fn check_all<R: Runtime>(
    app: &AppHandle<R>,
    db: &Db,
    github_client: &dyn github::GitHubApi,
) -> Result<Vec<Repo>> {
    let repos = db.with(db::list_repos)?;

    let mut updated = Vec::new();

    for repo in repos {
        match github_client.fetch_latest(&repo.owner, &repo.name).await {
            Ok(latest) => {
                let is_new = version::is_newer(&latest.version, repo.latest_version.as_deref());
                let ts = now();
                if is_new {
                    db.with(|c| {
                        db::update_version(
                            c,
                            repo.id,
                            &latest.version,
                            &latest.url,
                            latest.source_kind,
                            ts,
                        )
                    })?;
                    // Only notify when this is not the very first discovery,
                    // i.e. we already had a known version before.
                    if repo.latest_version.is_some() {
                        notify::notify_new_version(app, &repo, &latest.version);
                    }
                    updated.push(repo);
                } else {
                    db.with(|c| db::touch_checked(c, repo.id, ts))?;
                }
            }
            Err(e) => {
                // A single repo failing (typo, deleted, rate limit) must not
                // abort the whole run.
                eprintln!(
                    "libway: check failed for {}/{}: {e:#}",
                    repo.owner, repo.name
                );
            }
        }
    }

    // Let an open settings window reload its list. The tray refreshes via the
    // `repos:updated` listener registered in `tray::create`.
    let _ = app.emit(Event::ReposUpdated.as_str(), ());
    Ok(updated)
}
