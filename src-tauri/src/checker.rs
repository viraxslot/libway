//! Core check logic shared by the manual `check_now` command and the
//! background scheduler.
//!
//! Walks every tracked repository, fetches its latest version from GitHub,
//! and on a newer version updates the database, fires a notification, and
//! refreshes the tray. Returns the list of repos that got a new version.

use anyhow::Result;
use futures::stream::{self, StreamExt};
use tauri::{AppHandle, Emitter, Runtime};

use crate::db::{self, Db, Repo};
use crate::events::Event;
use crate::github::LatestVersion;
use crate::util::now;
use crate::{github, notify, version};

/// Max GitHub requests in flight at once, to avoid flooding the API. Transient
/// 429s within this window are handled by the retry in `http`.
const MAX_CONCURRENT_CHECKS: usize = 10;

/// Run a check over all tracked repositories.
///
/// GitHub is queried concurrently, at most [`MAX_CONCURRENT_CHECKS`] requests
/// in flight. Network calls happen outside the DB lock; results are applied
/// sequentially afterwards, so the lock is only taken for short reads and
/// writes. After the run the tray is rebuilt to reflect new state.
pub async fn check_all<R: Runtime>(
    app: &AppHandle<R>,
    db: &Db,
    github_client: &dyn github::GitHubApi,
) -> Result<Vec<Repo>> {
    let repos = db.with(db::list_repos)?;

    // Warm the auth cache before fanning out, so a one-time credential prompt
    // happens once here rather than racing across the concurrent requests.
    github_client.prepare();

    let results: Vec<(Repo, Result<LatestVersion>)> = stream::iter(repos)
        .map(|repo| async move {
            let latest = github_client.fetch_latest(&repo.owner, &repo.name).await;
            (repo, latest)
        })
        .buffer_unordered(MAX_CONCURRENT_CHECKS)
        .collect()
        .await;

    let mut updated = Vec::new();
    for (repo, latest) in results {
        if let Some(repo) = apply_result(app, db, repo, latest)? {
            updated.push(repo);
        }
    }

    // Let an open settings window reload its list. The tray refreshes via the
    // `repos:updated` listener registered in `tray::create`.
    let _ = app.emit(Event::ReposUpdated.as_str(), ());
    Ok(updated)
}

/// Apply one repo's check result to the DB. Returns the repo if it got a newer
/// version (for the caller's "updated" list), `None` otherwise. A failed fetch
/// is logged and swallowed so one bad repo can't abort the whole run.
fn apply_result<R: Runtime>(
    app: &AppHandle<R>,
    db: &Db,
    repo: Repo,
    latest: Result<LatestVersion>,
) -> Result<Option<Repo>> {
    let latest = match latest {
        Ok(latest) => latest,
        Err(e) => {
            eprintln!(
                "libway: check failed for {}/{}: {e:#}",
                repo.owner, repo.name
            );
            return Ok(None);
        }
    };

    let ts = now();
    if !version::is_newer(&latest.version, repo.latest_version.as_deref()) {
        db.with(|c| db::touch_checked(c, repo.id, ts))?;
        return Ok(None);
    }

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
    // Only notify when this is not the very first discovery, i.e. we already
    // had a known version before.
    if repo.latest_version.is_some() {
        notify::notify_new_version(app, &repo, &latest.version);
    }
    Ok(Some(repo))
}
