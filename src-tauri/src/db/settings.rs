//! Key-value application settings.

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

/// Read a setting value.
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let value = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |r| r.get::<_, String>(0),
        )
        .optional()?;
    Ok(value)
}

/// Write a setting value.
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::db::*;

    #[test]
    fn settings_roundtrip() {
        let db = Db::open_in_memory().unwrap();
        let c = db.0.lock().unwrap();
        assert!(get_setting(&c, "interval").unwrap().is_none());
        set_setting(&c, "interval", "10").unwrap();
        assert_eq!(get_setting(&c, "interval").unwrap().as_deref(), Some("10"));
        set_setting(&c, "interval", "5").unwrap(); // upsert
        assert_eq!(get_setting(&c, "interval").unwrap().as_deref(), Some("5"));
    }
}
