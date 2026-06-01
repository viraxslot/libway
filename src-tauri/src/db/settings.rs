//! Key-value application settings.

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Clone, Copy)]
pub enum SettingKey {
    CheckIntervalMinutes,
    CheckOnStartup,
    SelfUpdate,
}

impl SettingKey {
    pub fn as_str(&self) -> &str {
        match self {
            SettingKey::CheckIntervalMinutes => "check_interval_minutes",
            SettingKey::CheckOnStartup => "check_on_startup",
            SettingKey::SelfUpdate => "self_update",
        }
    }
}

/// Read a setting value.
pub fn get_setting(conn: &Connection, key: SettingKey) -> Result<Option<String>> {
    let value = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key.as_str()],
            |r| r.get::<_, String>(0),
        )
        .optional()?;
    Ok(value)
}

/// Write a setting value.
pub fn set_setting(conn: &Connection, key: SettingKey, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key.as_str(), value],
    )?;
    Ok(())
}
