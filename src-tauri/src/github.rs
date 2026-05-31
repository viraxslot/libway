//! GitHub API client: determines the latest version of a repository.
//!
//! Logic: first try `releases/latest` (the stable release; GitHub does not
//! return pre-releases there). If there are no releases (404), take the top
//! tag from `tags`. Version comparison uses semver, with a fallback to string
//! comparison for tags that do not parse as semver.

use std::time::Duration;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::Deserialize;

use crate::db::SourceKind;
use crate::http;
use crate::keychain;

const API_BASE: &str = "https://api.github.com";

/// A discovered version of a tool.
#[derive(Debug, Clone)]
pub struct LatestVersion {
    /// The version tag as returned by GitHub (e.g. "v2.40.0").
    pub version: String,
    /// A user-facing link (release or tag page).
    pub url: String,
    pub source_kind: SourceKind,
}

#[derive(Debug, Deserialize)]
struct ReleaseResponse {
    tag_name: String,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct TagResponse {
    name: String,
}

/// Build the configured GitHub client. The token (if any) raises rate limits.
fn client(token: Option<&str>) -> Result<http::Client> {
    http::Client::builder(API_BASE)
        .header("User-Agent", http::DEFAULT_USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer(token)
        .max_retries(RealGitHub::MAX_RETRIES)
        .max_retry_delay(RealGitHub::MAX_RETRY_DELAY)
        .build()
}

/// Check whether the public repository `owner/name` exists on GitHub.
/// `Ok(true)` on 200, `Ok(false)` on 404, `Err` on network/other errors.
pub async fn repo_exists(owner: &str, name: &str, token: Option<&str>) -> Result<bool> {
    client(token)?
        .exists(&format!("/repos/{owner}/{name}"))
        .await
}

/// Fetch the latest version of repository `owner/name`. Prefer the stable
/// release; fall back to tags only when there are none.
pub async fn fetch_latest(owner: &str, name: &str, token: Option<&str>) -> Result<LatestVersion> {
    let client = client(token)?;
    match fetch_latest_release(&client, owner, name).await? {
        Some(release) => Ok(release),
        None => fetch_latest_tag(&client, owner, name).await,
    }
}

/// Fetch the stable release. `Ok(None)` means GitHub returned 404 (no
/// releases), so the caller should fall back to tags.
async fn fetch_latest_release(
    client: &http::Client,
    owner: &str,
    name: &str,
) -> Result<Option<LatestVersion>> {
    let release: Option<ReleaseResponse> = client
        .get_optional(&format!("/repos/{owner}/{name}/releases/latest"))
        .await?;
    Ok(release.map(|r| LatestVersion {
        version: r.tag_name,
        url: r.html_url,
        source_kind: SourceKind::Release,
    }))
}

/// Fetch the top tag, used when a repository publishes tags but no releases.
async fn fetch_latest_tag(client: &http::Client, owner: &str, name: &str) -> Result<LatestVersion> {
    let tags: Vec<TagResponse> = client
        .get_json(&format!("/repos/{owner}/{name}/tags?per_page=1"))
        .await?;

    let tag = tags
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("{owner}/{name} has neither releases nor tags"))?;

    let url = format!(
        "https://github.com/{owner}/{name}/releases/tag/{}",
        tag.name
    );
    Ok(LatestVersion {
        version: tag.name,
        url,
        source_kind: SourceKind::Tag,
    })
}

/// Abstraction over the GitHub calls the app makes, so the network layer can
/// be replaced with a fake in tests. Implementors handle their own auth.
#[async_trait]
pub trait GitHubApi: Send + Sync {
    /// Whether the public repository `owner/name` exists.
    async fn repo_exists(&self, owner: &str, name: &str) -> Result<bool>;
    /// The latest version of `owner/name`.
    async fn fetch_latest(&self, owner: &str, name: &str) -> Result<LatestVersion>;
}

/// The production client. Reads the GitHub token from the Keychain per call
/// (so a token added at runtime is picked up without a restart).
pub struct RealGitHub;

impl RealGitHub {
    /// Retry policy for GitHub's rate limit: a few attempts, but never wait
    /// longer than this even when the reported reset window is far out.
    const MAX_RETRIES: u32 = 3;
    const MAX_RETRY_DELAY: Duration = Duration::from_secs(30);

    pub fn new() -> Self {
        RealGitHub
    }
}

impl Default for RealGitHub {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GitHubApi for RealGitHub {
    async fn repo_exists(&self, owner: &str, name: &str) -> Result<bool> {
        let token = keychain::get_token().unwrap_or(None);
        repo_exists(owner, name, token.as_deref()).await
    }

    async fn fetch_latest(&self, owner: &str, name: &str) -> Result<LatestVersion> {
        let token = keychain::get_token().unwrap_or(None);
        fetch_latest(owner, name, token.as_deref()).await
    }
}
