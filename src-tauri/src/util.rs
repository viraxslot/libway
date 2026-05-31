//! Small shared helpers used across modules.

use std::time::{SystemTime, UNIX_EPOCH};

/// Current unix time in seconds. Saturates to 0 if the clock is before the
/// epoch (which should never happen in practice).
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Current unix time in seconds as `i64`, for DB timestamps.
pub fn now() -> i64 {
    now_unix() as i64
}
