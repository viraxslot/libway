//! Self-update check: detects when a newer libway release is available.
//!
//! Unlike the per-repo checker, this compares libway's own latest GitHub
//! release tag against the compiled-in `CARGO_PKG_VERSION` and, when newer,
//! records it in the `SelfUpdate` managed state so the tray can surface it.
//! No download or in-place replacement — the tray item just opens the release
//! page (the app is ad-hoc signed, not notarized).

use std::sync::Mutex;

use tauri::{Manager, Runtime};

use crate::db::{self, Db};
use crate::github::{GitHubApi, LatestVersion};
use crate::version;

/// Settings key holding whether to check for libway's own updates ("0"/"1").
pub const SETTING_CHECK_SELF_UPDATE: &str = "check_self_update";
/// Default for the self-update check when the setting is unset.
pub const DEFAULT_CHECK_SELF_UPDATE: bool = true;
/// libway's own repository, checked for new releases of the app itself.
pub const SELF_OWNER: &str = "viraxslot";
pub const SELF_NAME: &str = "libway";

/// A newer libway release that the user can go download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableUpdate {
    /// The release tag as published on GitHub (e.g. "v0.4.0").
    pub version: String,
    /// The release page URL to open in the browser.
    pub url: String,
}

/// Managed state holding the currently-known available self-update, if any.
/// Ephemeral and version-scoped, so it lives here rather than in SQLite.
pub struct SelfUpdate(pub Mutex<Option<AvailableUpdate>>);

impl SelfUpdate {
    pub fn empty() -> Self {
        SelfUpdate(Mutex::new(None))
    }

    /// Replace the stored update (or clear it with `None`).
    pub fn set(&self, update: Option<AvailableUpdate>) {
        *self.0.lock().unwrap() = update;
    }

    /// A clone of the current value, for the tray to render.
    pub fn get(&self) -> Option<AvailableUpdate> {
        self.0.lock().unwrap().clone()
    }
}

/// Decide whether `latest` is newer than the currently-running `current`
/// version. Pure: no network, no state. Returns `Some` only when newer.
fn evaluate(latest: LatestVersion, current: &str) -> Option<AvailableUpdate> {
    if version::is_newer(&latest.version, Some(current)) {
        Some(AvailableUpdate {
            version: latest.version,
            url: latest.url,
        })
    } else {
        None
    }
}

/// Whether the self-update check is enabled (defaults to true when unset).
/// Mirrors `scheduler::check_on_startup`.
pub fn enabled(db: &Db) -> bool {
    db.with(|c| db::get_setting(c, SETTING_CHECK_SELF_UPDATE))
        .ok()
        .flatten()
        .map(|v| v == "1")
        .unwrap_or(DEFAULT_CHECK_SELF_UPDATE)
}

/// Check libway's own repository for a newer release and update the
/// `SelfUpdate` state. When the setting is disabled this is a no-op (existing
/// state is left untouched — callers clear it explicitly when toggling off).
///
/// Errors (network, parse, rate limit) are logged and swallowed so the
/// scheduler loop keeps running; a transient failure does not erase a
/// previously-found update.
pub async fn check<R: Runtime>(app: &tauri::AppHandle<R>, db: &Db, client: &dyn GitHubApi) {
    if !enabled(db) {
        return;
    }
    match client.fetch_latest(SELF_OWNER, SELF_NAME).await {
        Ok(latest) => {
            let update = evaluate(latest, env!("CARGO_PKG_VERSION"));
            app.state::<SelfUpdate>().set(update);
        }
        Err(e) => {
            eprintln!("libway: self-update check failed: {e:#}");
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::db::SourceKind;

    fn latest(version: &str) -> LatestVersion {
        LatestVersion {
            version: version.to_string(),
            url: format!("https://github.com/viraxslot/libway/releases/tag/{version}"),
            source_kind: SourceKind::Release,
        }
    }

    fn set(db: &Db, key: &str, value: &str) {
        db.with(|c| db::set_setting(c, key, value)).unwrap();
    }

    #[test]
    fn newer_release_is_an_update() {
        let got = evaluate(latest("v0.4.0"), "0.3.0");
        assert_eq!(
            got,
            Some(AvailableUpdate {
                version: "v0.4.0".into(),
                url: "https://github.com/viraxslot/libway/releases/tag/v0.4.0".into(),
            })
        );
    }

    #[test]
    fn equal_version_is_not_an_update() {
        assert_eq!(evaluate(latest("v0.3.0"), "0.3.0"), None);
    }

    #[test]
    fn older_release_is_not_an_update() {
        assert_eq!(evaluate(latest("v0.2.0"), "0.3.0"), None);
    }

    #[test]
    fn enabled_defaults_to_true_when_unset() {
        let db = Db::open_in_memory().unwrap();
        assert!(enabled(&db));
    }

    #[test]
    fn enabled_reads_flag() {
        let db = Db::open_in_memory().unwrap();

        set(&db, SETTING_CHECK_SELF_UPDATE, "1");
        assert!(enabled(&db));

        set(&db, SETTING_CHECK_SELF_UPDATE, "0");
        assert!(!enabled(&db));

        set(&db, SETTING_CHECK_SELF_UPDATE, "yes");
        assert!(!enabled(&db));
    }
}
