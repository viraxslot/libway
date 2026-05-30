//! Commands for managing the GitHub token stored in the system Keychain.

use super::e;
use crate::keychain;

#[tauri::command]
pub fn has_token() -> Result<bool, String> {
    keychain::has_token().map_err(e)
}

#[tauri::command]
pub fn set_token(token: String) -> Result<(), String> {
    let token = token.trim();
    if token.is_empty() {
        // Empty input means "clear the token".
        return keychain::delete_token().map_err(e);
    }
    keychain::set_token(token).map_err(e)
}

#[tauri::command]
pub fn clear_token() -> Result<(), String> {
    keychain::delete_token().map_err(e)
}
