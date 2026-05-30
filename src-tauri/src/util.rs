//! Small shared helpers used across modules.

use std::time::{SystemTime, UNIX_EPOCH};

/// Current unix time in seconds. Saturates to 0 if the clock is before the
/// epoch (which should never happen in practice).
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
