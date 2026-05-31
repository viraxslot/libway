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

use tauri::{AppHandle, Emitter, Manager};

use crate::checker;
use crate::db::{self, Db, SettingKey};

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
    db.with(|c| db::get_setting(c, SettingKey::CheckIntervalMinutes))
        .ok()
        .flatten()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|m| *m > 0)
        .unwrap_or(DEFAULT_INTERVAL_MINUTES)
}

/// Read whether to run a check immediately at startup.
pub fn check_on_startup(db: &Db) -> bool {
    db.with(|c| db::get_setting(c, SettingKey::CheckOnStartup))
        .ok()
        .flatten()
        .map(|v| v == "1")
        .unwrap_or(DEFAULT_CHECK_ON_STARTUP)
}

/// Tracks when the next check is due, by accumulating elapsed seconds against
/// the interval. Pure timer state — no clock, no DB — so it is unit-testable.
struct Schedule {
    /// Seconds elapsed since the last completed check.
    elapsed: u64,
    /// Whether a check is due now.
    due: bool,
}

impl Schedule {
    /// `run_on_startup` makes the very first check due immediately.
    fn new(run_on_startup: bool) -> Self {
        Schedule {
            elapsed: 0,
            due: run_on_startup,
        }
    }

    fn is_due(&self) -> bool {
        self.due
    }

    /// Record that a check just ran: reset the timer.
    fn mark_ran(&mut self) {
        self.elapsed = 0;
        self.due = false;
    }

    /// Advance by `tick` seconds; a check becomes due once `interval_secs` has
    /// accumulated. The interval is passed each tick so changes from the UI
    /// apply promptly.
    fn advance(&mut self, tick: u64, interval_secs: u64) {
        self.elapsed += tick;
        if self.elapsed >= interval_secs {
            self.due = true;
        }
    }
}

/// Spawn the background checking loop.
pub fn spawn(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(STARTUP_DELAY_SECS)).await;

        let mut schedule = Schedule::new(check_on_startup(&app.state::<Db>()));

        loop {
            if schedule.is_due() {
                run_check(&app).await;
                schedule.mark_ran();
            }

            tokio::time::sleep(Duration::from_secs(TICK_SECS)).await;
            let interval_secs = interval_minutes(&app.state::<Db>()) * 60;
            schedule.advance(TICK_SECS, interval_secs);
        }
    });
}

/// Run one check over all repositories plus libway's own self-update, logging
/// (not propagating) failures so the loop keeps running.
async fn run_check(app: &AppHandle) {
    let db = app.state::<Db>();
    let client = app.state::<Box<dyn crate::github::GitHubApi>>();

    if let Err(e) = checker::check_all(app, &db, client.inner().as_ref()).await {
        eprintln!("libway: scheduled check failed: {e:#}");
    }

    // Self-update state changes outside the DB, so emit a refresh afterwards so
    // the tray reflects a newly-found (or cleared) update. `check_all` already
    // emitted `repos:updated` for repo changes; this final emit covers the
    // self-update specifically.
    crate::selfupdate::check(app, &db, client.inner().as_ref()).await;
    let _ = app.emit(crate::events::Event::SelfUpdateChanged.as_str(), ());
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn set(db: &Db, key: SettingKey, value: &str) {
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
        set(&db, SettingKey::CheckIntervalMinutes, "30");
        assert_eq!(interval_minutes(&db), 30);
    }

    #[test]
    fn interval_falls_back_on_invalid_value() {
        let db = Db::open_in_memory().unwrap();

        // Non-numeric: does not parse as u64.
        set(&db, SettingKey::CheckIntervalMinutes, "not-a-number");
        assert_eq!(interval_minutes(&db), DEFAULT_INTERVAL_MINUTES);

        // Zero is filtered out (would spin); a negative parses as invalid u64.
        set(&db, SettingKey::CheckIntervalMinutes, "0");
        assert_eq!(interval_minutes(&db), DEFAULT_INTERVAL_MINUTES);

        set(&db, SettingKey::CheckIntervalMinutes, "-5");
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

        set(&db, SettingKey::CheckOnStartup, "1");
        assert!(check_on_startup(&db));

        set(&db, SettingKey::CheckOnStartup, "0");
        assert!(!check_on_startup(&db));

        // Anything other than "1" is treated as false.
        set(&db, SettingKey::CheckOnStartup, "yes");
        assert!(!check_on_startup(&db));
    }

    #[test]
    fn schedule_due_on_startup_only_when_enabled() {
        assert!(Schedule::new(true).is_due());
        assert!(!Schedule::new(false).is_due());
    }

    #[test]
    fn schedule_becomes_due_after_interval_elapses() {
        let mut s = Schedule::new(false);
        // 120s interval, 5s ticks: not due until the interval is reached.
        s.advance(5, 120);
        assert!(!s.is_due());
        for _ in 0..22 {
            s.advance(5, 120); // 10s..115s
        }
        assert!(!s.is_due(), "still under 120s at 115s");
        s.advance(5, 120); // 120s
        assert!(s.is_due());
    }

    #[test]
    fn schedule_mark_ran_resets_timer() {
        let mut s = Schedule::new(true);
        s.advance(5, 10);
        s.mark_ran();
        assert!(!s.is_due());
        // After reset it takes another full interval to come due again.
        s.advance(5, 10);
        assert!(!s.is_due());
        s.advance(5, 10);
        assert!(s.is_due());
    }

    #[test]
    fn schedule_picks_up_shortened_interval() {
        let mut s = Schedule::new(false);
        s.advance(60, 600); // 60s elapsed against a 10-minute interval
        assert!(!s.is_due());
        // Interval shortened from the UI to 60s: the next tick sees it due.
        s.advance(5, 60);
        assert!(s.is_due());
    }
}
