//! GitHub API client: determines the latest version of a repository.
//!
//! Logic: first try `releases/latest` (the stable release; GitHub does not
//! return pre-releases there). If there are no releases (404), take the top
//! tag from `tags`. Version comparison uses semver, with a fallback to string
//! comparison for tags that do not parse as semver.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

use crate::db::SourceKind;
use crate::keychain;

const API_BASE: &str = "https://api.github.com";
const USER_AGENT: &str = "libway";

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

/// Build the HTTP client. The token (if any) goes into the Authorization header.
fn client(token: Option<&str>) -> Result<reqwest::Client> {
    use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT as UA};

    let mut headers = HeaderMap::new();
    headers.insert(UA, HeaderValue::from_static(USER_AGENT));
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );
    if let Some(t) = token {
        let mut v =
            HeaderValue::from_str(&format!("Bearer {t}")).context("invalid character in token")?;
        v.set_sensitive(true);
        headers.insert(AUTHORIZATION, v);
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .context("failed to build the HTTP client")
}

/// Check whether the public repository `owner/name` exists on GitHub.
/// Returns Ok(true) on 200, Ok(false) on 404, and Err on network/other errors
/// (so the caller can distinguish "definitely missing" from "couldn't check").
pub async fn repo_exists(owner: &str, name: &str, token: Option<&str>) -> Result<bool> {
    let client = client(token)?;
    let url = format!("{API_BASE}/repos/{owner}/{name}");
    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("could not reach GitHub to verify {owner}/{name}"))?;

    match resp.status() {
        s if s.is_success() => Ok(true),
        reqwest::StatusCode::NOT_FOUND => Ok(false),
        s => Err(anyhow!(
            "GitHub returned {s} while verifying {owner}/{name}"
        )),
    }
}

/// Fetch the latest version of repository `owner/name`.
/// `token` is an optional GitHub token (raises rate limits; less needed for
/// public repos, but we pass it along whenever it is present).
pub async fn fetch_latest(owner: &str, name: &str, token: Option<&str>) -> Result<LatestVersion> {
    let client = client(token)?;

    // 1) Try the stable release.
    let rel_url = format!("{API_BASE}/repos/{owner}/{name}/releases/latest");
    let resp = client
        .get(&rel_url)
        .send()
        .await
        .with_context(|| format!("release request for {owner}/{name} failed"))?;

    if resp.status().is_success() {
        let r: ReleaseResponse = resp
            .json()
            .await
            .context("failed to parse the release response")?;
        return Ok(LatestVersion {
            version: r.tag_name,
            url: r.html_url,
            source_kind: SourceKind::Release,
        });
    }

    // 404 — no releases, fall back to tags. Other codes are errors.
    if resp.status() != reqwest::StatusCode::NOT_FOUND {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "GitHub returned {status} for {owner}/{name}: {}",
            body.chars().take(200).collect::<String>()
        ));
    }

    // 2) Fall back to tags.
    let tags_url = format!("{API_BASE}/repos/{owner}/{name}/tags?per_page=1");
    let resp = client
        .get(&tags_url)
        .send()
        .await
        .with_context(|| format!("tags request for {owner}/{name} failed"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(anyhow!(
            "GitHub returned {status} for tags of {owner}/{name}"
        ));
    }

    let tags: Vec<TagResponse> = resp
        .json()
        .await
        .context("failed to parse the tags response")?;

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

/// Compare a discovered version against the already-known one.
/// Returns true if `fetched` is newer than `known`.
///
/// If `known` is None, any discovered version counts as new. We first try to
/// compare as semver (stripping a leading 'v'); if either side fails to parse,
/// we compare as strings (and treat it as new only on an actual difference).
pub fn is_newer(fetched: &str, known: Option<&str>) -> bool {
    let known = match known {
        None => return true,
        Some(k) => k,
    };
    if fetched == known {
        return false;
    }

    match (parse_semver(fetched), parse_semver(known)) {
        (Some(f), Some(k)) => f > k,
        // Not semver — since the strings differ, treat it as new.
        _ => true,
    }
}

/// Parse a tag as semver, stripping an optional leading 'v'.
fn parse_semver(tag: &str) -> Option<semver::Version> {
    let trimmed = tag.strip_prefix('v').unwrap_or(tag);
    semver::Version::parse(trimmed).ok()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_when_unknown() {
        assert!(is_newer("v1.0.0", None));
    }

    #[test]
    fn equal_is_not_newer() {
        assert!(!is_newer("v1.2.3", Some("v1.2.3")));
        assert!(!is_newer("1.2.3", Some("1.2.3")));
    }

    #[test]
    fn semver_comparison() {
        assert!(is_newer("v1.2.4", Some("v1.2.3")));
        assert!(is_newer("v2.0.0", Some("v1.9.9")));
        assert!(!is_newer("v1.2.3", Some("v1.2.4")));
        // with and without 'v' is equivalent
        assert!(is_newer("1.2.4", Some("v1.2.3")));
    }

    #[test]
    fn non_semver_falls_back_to_string_diff() {
        // dates / non-standard tags: any difference counts as new
        assert!(is_newer("2024-05-01", Some("2024-04-01")));
        assert!(!is_newer("nightly", Some("nightly")));
        assert!(is_newer("release-42", Some("release-41")));
    }
}
