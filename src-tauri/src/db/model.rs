//! Domain types stored in the database and shared with the frontend.

use serde::{Deserialize, Serialize};

/// Where a version was obtained from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Release,
    Tag,
}

impl SourceKind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            SourceKind::Release => "release",
            SourceKind::Tag => "tag",
        }
    }

    pub(super) fn from_str(s: &str) -> Option<Self> {
        match s {
            "release" => Some(SourceKind::Release),
            "tag" => Some(SourceKind::Tag),
            _ => None,
        }
    }
}

/// A tracked repository together with its current state.
/// Serialized as camelCase for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
    pub id: i64,
    pub owner: String,
    pub name: String,
    pub latest_version: Option<String>,
    pub latest_url: Option<String>,
    pub source_kind: Option<SourceKind>,
    pub has_unseen: bool,
    pub last_checked_at: Option<i64>,
    /// User-assigned tags for grouping (stored as a comma-separated string).
    pub tags: Vec<String>,
}
