//! Commands for managing the GitHub token stored in the system Keychain.

use tauri::State;

use super::e;
use crate::github::GitHubApi;
use crate::keychain;

#[tauri::command]
pub fn has_token() -> Result<bool, String> {
    keychain::has_token().map_err(e)
}

/// What `set_token` should do with a given raw input. Pure, so the
/// trim / "empty means clear" decision is testable without touching the
/// Keychain.
#[derive(Debug, PartialEq, Eq)]
enum TokenAction<'a> {
    Store(&'a str),
    Clear,
}

fn classify_token(input: &str) -> TokenAction<'_> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        TokenAction::Clear
    } else {
        TokenAction::Store(trimmed)
    }
}

#[tauri::command]
pub fn set_token(token: String, client: State<'_, Box<dyn GitHubApi>>) -> Result<(), String> {
    let result = match classify_token(&token) {
        // Empty input means "clear the token".
        TokenAction::Clear => keychain::delete_token(),
        TokenAction::Store(t) => keychain::set_token(t),
    };
    // The client caches the token; drop the cache so the change takes effect.
    client.invalidate_token_cache();
    result.map_err(e)
}

#[tauri::command]
pub fn clear_token(client: State<'_, Box<dyn GitHubApi>>) -> Result<(), String> {
    let result = keychain::delete_token();
    client.invalidate_token_cache();
    result.map_err(e)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{classify_token, TokenAction};

    #[test]
    fn non_empty_token_is_stored_trimmed() {
        assert_eq!(classify_token("ghp_abc"), TokenAction::Store("ghp_abc"));
        assert_eq!(classify_token("  ghp_abc  "), TokenAction::Store("ghp_abc"));
    }

    #[test]
    fn empty_or_whitespace_clears() {
        assert_eq!(classify_token(""), TokenAction::Clear);
        assert_eq!(classify_token("   "), TokenAction::Clear);
        assert_eq!(classify_token("\t\n"), TokenAction::Clear);
    }
}
