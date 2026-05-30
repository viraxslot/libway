//! Background scheduler: periodically checks all repositories.
//!
//! Runs as a detached tokio task. The interval comes from the `settings`
//! table (key `check_interval_minutes`), defaulting to 10 minutes. When the
//! `check_on_startup` setting is enabled (default) a first check fires shortly
//! after launch so the tray is populated without waiting a full interval;
//! otherwise the first check only happens after one interval has elapsed.
//!
//! The loop sleeps in short ticks rather than one long sleep, so a new
//! interval set from the UI is picked up quickly instead of only after the
//! previously scheduled (possibly long) sleep elapses.

use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::checker;
use crate::db::{self, Db};

/// Settings key holding the check interval in minutes.
pub const SETTING_INTERVAL: &str = "check_interval_minutes";
/// Settings key holding whether to check immediately on startup ("0"/"1").
pub const SETTING_CHECK_ON_STARTUP: &str = "check_on_startup";
/// Default interval when the setting is unset or invalid.
pub const DEFAULT_INTERVAL_MINUTES: u64 = 10;
/// Default for "check on startup" when the setting is unset.
pub const DEFAULT_CHECK_ON_STARTUP: bool = true;
/// Short delay before the first check, to let the app finish starting up.
const STARTUP_DELAY_SECS: u64 = 5;
/// How often the loop wakes to re-evaluate whether a check is due.
const TICK_SECS: u64 = 5;

/// Read the configured interval in minutes, falling back to the default.
pub fn interval_minutes(db: &Db) -> u64 {
    db.with(|c| db::get_setting(c, SETTING_INTERVAL))
        .ok()
        .flatten()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|m| *m > 0)
        .unwrap_or(DEFAULT_INTERVAL_MINUTES)
}

/// Read whether to run a check immediately at startup.
pub fn check_on_startup(db: &Db) -> bool {
    db.with(|c| db::get_setting(c, SETTING_CHECK_ON_STARTUP))
        .ok()
        .flatten()
        .map(|v| v == "1")
        .unwrap_or(DEFAULT_CHECK_ON_STARTUP)
}

/// Spawn the background checking loop.
pub fn spawn(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(STARTUP_DELAY_SECS)).await;

        // Seconds elapsed since the last completed check.
        let mut elapsed: u64 = 0;
        // Whether a check is due now (immediately on startup if configured).
        let mut due = {
            let db = app.state::<Db>();
            check_on_startup(&db)
        };

        loop {
            if due {
                let db = app.state::<Db>();
                let client = app.state::<Box<dyn crate::github::GitHubApi>>();
                if let Err(e) = checker::check_all(&app, &db, client.inner().as_ref()).await {
                    eprintln!("libway: scheduled check failed: {e:#}");
                }
                elapsed = 0;
                due = false;
            }

            tokio::time::sleep(Duration::from_secs(TICK_SECS)).await;
            elapsed += TICK_SECS;

            // Re-read the interval every tick so UI changes apply promptly.
            let interval_secs = {
                let db = app.state::<Db>();
                interval_minutes(&db) * 60
            };
            if elapsed >= interval_secs {
                due = true;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn set(db: &Db, key: &str, value: &str) {
        db.with(|c| db::set_setting(c, key, value)).unwrap();
    }

    #[test]
    fn interval_defaults_when_unset() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(interval_minutes(&db), DEFAULT_INTERVAL_MINUTES);
    }

    #[test]
    fn interval_reads_valid_value() {
        let db = Db::open_in_memory().unwrap();
        set(&db, SETTING_INTERVAL, "30");
        assert_eq!(interval_minutes(&db), 30);
    }

    #[test]
    fn interval_falls_back_on_invalid_value() {
        let db = Db::open_in_memory().unwrap();

        // Non-numeric: does not parse as u64.
        set(&db, SETTING_INTERVAL, "not-a-number");
        assert_eq!(interval_minutes(&db), DEFAULT_INTERVAL_MINUTES);

        // Zero is filtered out (would spin); a negative parses as invalid u64.
        set(&db, SETTING_INTERVAL, "0");
        assert_eq!(interval_minutes(&db), DEFAULT_INTERVAL_MINUTES);

        set(&db, SETTING_INTERVAL, "-5");
        assert_eq!(interval_minutes(&db), DEFAULT_INTERVAL_MINUTES);
    }

    #[test]
    fn check_on_startup_defaults_when_unset() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(check_on_startup(&db), DEFAULT_CHECK_ON_STARTUP);
    }

    #[test]
    fn check_on_startup_reads_flag() {
        let db = Db::open_in_memory().unwrap();

        set(&db, SETTING_CHECK_ON_STARTUP, "1");
        assert!(check_on_startup(&db));

        set(&db, SETTING_CHECK_ON_STARTUP, "0");
        assert!(!check_on_startup(&db));

        // Anything other than "1" is treated as false.
        set(&db, SETTING_CHECK_ON_STARTUP, "yes");
        assert!(!check_on_startup(&db));
    }
}
