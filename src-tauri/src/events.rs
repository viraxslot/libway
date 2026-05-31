//! Names of the Tauri events the backend emits and the frontend/tray listen
//! for. Kept in one enum so every event in the app is visible in a single
//! place and referenced by a typed name instead of a stringly-typed literal.

/// An application event. Use [`Event::as_str`] when calling `emit`/`listen`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The repository list or any repo's state changed; listeners refresh.
    ReposUpdated,
    /// The UI language changed; the tray rebuilds its menu in the new language.
    LanguageChanged,
    /// The self-update state changed (a new update was found or cleared); the
    /// tray rebuilds so its "Update available" item appears/disappears.
    SelfUpdateChanged,
}

impl Event {
    /// The wire name used with Tauri's `emit`/`listen`.
    pub fn as_str(self) -> &'static str {
        match self {
            Event::ReposUpdated => "repos:updated",
            Event::LanguageChanged => "language:changed",
            Event::SelfUpdateChanged => "self_update:changed",
        }
    }
}

impl AsRef<str> for Event {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
