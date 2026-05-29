//! Background scheduler: periodically checks all repositories.
//!
//! Runs as a detached tokio task. The interval comes from the `settings`
//! table (key `check_interval_minutes`), defaulting to 10 minutes. A first
//! check fires shortly after launch so the tray is populated without waiting
//! a full interval.
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
/// Default interval when the setting is unset or invalid.
pub const DEFAULT_INTERVAL_MINUTES: u64 = 10;
/// Short delay before the first check, to let the app finish starting up.
const STARTUP_DELAY_SECS: u64 = 5;
/// How often the loop wakes to re-evaluate whether a check is due.
const TICK_SECS: u64 = 5;

/// Read the configured interval in minutes, falling back to the default.
pub fn interval_minutes(db: &Db) -> u64 {
    let conn = db.0.lock().unwrap();
    db::get_setting(&conn, SETTING_INTERVAL)
        .ok()
        .flatten()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|m| *m > 0)
        .unwrap_or(DEFAULT_INTERVAL_MINUTES)
}

/// Spawn the background checking loop.
pub fn spawn(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(STARTUP_DELAY_SECS)).await;

        // Seconds elapsed since the last completed check.
        let mut elapsed: u64 = 0;
        // Run a check immediately on the first iteration.
        let mut due = true;

        loop {
            if due {
                let db = app.state::<Db>();
                if let Err(e) = checker::check_all(&app, &db).await {
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
