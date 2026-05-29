//! Storing the GitHub token in the macOS Keychain (the `keyring` crate).
//!
//! The token never goes into SQLite — only here. The service/account are
//! fixed, so the app always addresses a single entry.

use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE: &str = "libway";
const ACCOUNT: &str = "github-token";

fn entry() -> Result<Entry> {
    Entry::new(SERVICE, ACCOUNT).context("failed to access the Keychain")
}

/// Store (or overwrite) the token.
pub fn set_token(token: &str) -> Result<()> {
    entry()?
        .set_password(token)
        .context("failed to write the token to the Keychain")
}

/// Read the token. `None` if it has not been stored yet.
pub fn get_token() -> Result<Option<String>> {
    match entry()?.get_password() {
        Ok(t) => Ok(Some(t)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(anyhow::Error::new(e).context("failed to read the token from the Keychain")),
    }
}

/// Delete the token.
pub fn delete_token() -> Result<()> {
    match entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // already gone — not an error
        Err(e) => {
            Err(anyhow::Error::new(e).context("failed to delete the token from the Keychain"))
        }
    }
}

/// Whether a token is stored.
pub fn has_token() -> Result<bool> {
    Ok(get_token()?.is_some())
}
