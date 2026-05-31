//! GitHub API client: determines the latest version of a repository.
//!
//! Logic: first try `releases/latest` (the stable release; GitHub does not
//! return pre-releases there). If there are no releases (404), take the top
//! tag from `tags`. Version comparison uses semver, with a fallback to string
//! comparison for tags that do not parse as semver.

use std::sync::Mutex;
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
    /// Warm any cached credentials up front. Calling this before a concurrent
    /// batch keeps the (possibly blocking) Keychain prompt on a single thread
    /// instead of racing across the batch's tasks. Default no-op.
    fn prepare(&self) {}
    /// Drop any cached credentials, so the next call re-reads them. Called when
    /// the stored token changes. Default no-op for clients that don't cache.
    fn invalidate_token_cache(&self) {}
}

/// The production client.
///
/// The Keychain token is read once and cached, so a run that checks many
/// repositories doesn't trigger a macOS access prompt per request (and a
/// denied prompt isn't re-shown for every repo). The cache is dropped via
/// [`invalidate_token_cache`](GitHubApi::invalidate_token_cache) when the token
/// changes, so a token added or removed at runtime is still picked up without a
/// restart. The token only ever lives in memory here, never on disk.
pub struct RealGitHub {
    /// `None` until first read; `Some(maybe_token)` once cached.
    token: Mutex<Option<Option<String>>>,
}

impl RealGitHub {
    /// Retry policy for GitHub's rate limit: a few attempts, but never wait
    /// longer than this even when the reported reset window is far out.
    const MAX_RETRIES: u32 = 3;
    const MAX_RETRY_DELAY: Duration = Duration::from_secs(30);

    pub fn new() -> Self {
        RealGitHub {
            token: Mutex::new(None),
        }
    }

    /// The cached token, reading the Keychain once on first use. A read failure
    /// is treated (and cached) as "no token", matching the previous behaviour.
    fn token(&self) -> Option<String> {
        cached_or_load(&self.token, || keychain::get_token().unwrap_or(None))
    }
}

/// Return the cached value, computing and storing it with `load` on first use.
/// Split out so the cache-once / invalidate behaviour is testable without the
/// Keychain.
fn cached_or_load<T: Clone>(cache: &Mutex<Option<T>>, load: impl FnOnce() -> T) -> T {
    cache
        .lock()
        .expect("token cache mutex poisoned")
        .get_or_insert_with(load)
        .clone()
}

impl Default for RealGitHub {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GitHubApi for RealGitHub {
    async fn repo_exists(&self, owner: &str, name: &str) -> Result<bool> {
        repo_exists(owner, name, self.token().as_deref()).await
    }

    async fn fetch_latest(&self, owner: &str, name: &str) -> Result<LatestVersion> {
        fetch_latest(owner, name, self.token().as_deref()).await
    }

    fn prepare(&self) {
        self.token();
    }

    fn invalidate_token_cache(&self) {
        *self.token.lock().expect("token cache mutex poisoned") = None;
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn cached_or_load_reads_once_then_reuses() {
        let cache: Mutex<Option<u32>> = Mutex::new(None);
        let loads = Cell::new(0);
        let load = || {
            loads.set(loads.get() + 1);
            42
        };

        assert_eq!(cached_or_load(&cache, load), 42);
        assert_eq!(cached_or_load(&cache, load), 42);
        assert_eq!(loads.get(), 1, "loader must run only on the first call");
    }

    #[test]
    fn clearing_the_cache_forces_a_reload() {
        let cache: Mutex<Option<u32>> = Mutex::new(None);
        let loads = Cell::new(0);
        let load = || {
            loads.set(loads.get() + 1);
            loads.get()
        };

        assert_eq!(cached_or_load(&cache, load), 1);
        *cache.lock().unwrap() = None;
        assert_eq!(cached_or_load(&cache, load), 2, "reload after invalidation");
        assert_eq!(loads.get(), 2);
    }
}
