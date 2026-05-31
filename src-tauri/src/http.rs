//! Reusable HTTP client over reqwest: configured once via a builder, then used
//! for typed GET requests. Source-agnostic — no API specifics live here.

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;

/// How much of an error response body to include in error messages.
const MAX_ERROR_BODY: usize = 200;

/// User-Agent identifying this app on every outgoing request.
pub const DEFAULT_USER_AGENT: &str = "libway";

/// A configured HTTP client. Build one with [`Client::builder`].
pub struct Client {
    inner: reqwest::Client,
    base_url: String,
    max_retries: u32,
    max_retry_delay: Duration,
}

/// Accumulates configuration for a [`Client`]. Header-construction errors are
/// deferred until [`ClientBuilder::build`] so the call chain stays fluent.
pub struct ClientBuilder {
    base_url: String,
    headers: HeaderMap,
    max_retries: u32,
    max_retry_delay: Duration,
    error: Option<anyhow::Error>,
}

impl Client {
    /// Start building a client whose requests are relative to `base_url`.
    pub fn builder(base_url: impl Into<String>) -> ClientBuilder {
        ClientBuilder {
            base_url: base_url.into(),
            headers: HeaderMap::new(),
            max_retries: ClientBuilder::DEFAULT_MAX_RETRIES,
            max_retry_delay: ClientBuilder::DEFAULT_MAX_RETRY_DELAY,
            error: None,
        }
    }

    /// Issue a GET to `path` appended to the base URL and return the response.
    /// `path` is concatenated directly, so it must start with `/` (and the base
    /// URL must not end with one). Network failures carry a contextual message.
    /// Rate-limited responses are retried up to `max_retries` times.
    async fn send(&self, path: &str) -> Result<reqwest::Response> {
        let url = format!("{}{path}", self.base_url);
        let mut attempt = 0;
        loop {
            let resp = self
                .inner
                .get(&url)
                .send()
                .await
                .with_context(|| format!("GET {url} failed"))?;

            if attempt >= self.max_retries || !is_rate_limited(&resp) {
                return Ok(resp);
            }
            let delay =
                retry_delay(resp.headers(), crate::util::now_unix()).min(self.max_retry_delay);
            tokio::time::sleep(delay).await;
            attempt += 1;
        }
    }

    /// Turn a non-success response into an error that includes the status and a
    /// truncated body. Callers invoke this only for statuses they treat as
    /// errors.
    async fn status_error(resp: reqwest::Response) -> anyhow::Error {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow!(
            "request failed with {status}: {}",
            body.chars().take(MAX_ERROR_BODY).collect::<String>()
        )
    }

    /// GET `path` and decode the JSON body. Any non-2xx status is an error.
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self.send(path).await?;
        if !resp.status().is_success() {
            return Err(Self::status_error(resp).await);
        }
        resp.json()
            .await
            .with_context(|| format!("failed to parse the response from {path}"))
    }

    /// GET `path` and decode the JSON body, treating 404 as `Ok(None)`.
    /// Other non-2xx statuses are errors.
    pub async fn get_optional<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>> {
        let resp = self.send(path).await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !resp.status().is_success() {
            return Err(Self::status_error(resp).await);
        }
        let value = resp
            .json()
            .await
            .with_context(|| format!("failed to parse the response from {path}"))?;
        Ok(Some(value))
    }

    /// GET `path`; `Ok(true)` on 2xx, `Ok(false)` on 404, error otherwise.
    /// The body is ignored.
    pub async fn exists(&self, path: &str) -> Result<bool> {
        let resp = self.send(path).await?;
        match resp.status() {
            s if s.is_success() => Ok(true),
            reqwest::StatusCode::NOT_FOUND => Ok(false),
            _ => Err(Self::status_error(resp).await),
        }
    }
}

impl ClientBuilder {
    /// Default retry budget for rate-limited responses.
    const DEFAULT_MAX_RETRIES: u32 = 3;
    /// Default cap on a single retry wait, so a long server-reported reset
    /// window can't stall a request.
    const DEFAULT_MAX_RETRY_DELAY: Duration = Duration::from_secs(30);

    /// Add a default header sent on every request. An invalid name/value is
    /// remembered and surfaced from [`build`].
    pub fn header(mut self, name: &str, value: &str) -> Self {
        if self.error.is_some() {
            return self;
        }
        match (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            (Ok(n), Ok(v)) => {
                self.headers.insert(n, v);
            }
            _ => self.error = Some(anyhow!("invalid header {name}")),
        }
        self
    }

    /// Add an `Authorization: Bearer <token>` header when a token is present.
    /// No-op when `token` is `None`. The value is marked sensitive.
    pub fn bearer(mut self, token: Option<&str>) -> Self {
        if self.error.is_some() {
            return self;
        }
        if let Some(t) = token {
            match HeaderValue::from_str(&format!("Bearer {t}")) {
                Ok(mut v) => {
                    v.set_sensitive(true);
                    self.headers.insert(AUTHORIZATION, v);
                }
                Err(_) => self.error = Some(anyhow!("invalid character in token")),
            }
        }
        self
    }

    /// How many times to retry a rate-limited response before returning it.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Cap on a single retry wait, regardless of the server's reset window.
    pub fn max_retry_delay(mut self, delay: Duration) -> Self {
        self.max_retry_delay = delay;
        self
    }

    /// Finish building. Fails if any header (or the token) was invalid, or if
    /// the underlying reqwest client cannot be constructed.
    pub fn build(self) -> Result<Client> {
        if let Some(e) = self.error {
            return Err(e);
        }
        let inner = reqwest::Client::builder()
            .default_headers(self.headers)
            .build()
            .context("failed to build the HTTP client")?;
        Ok(Client {
            inner,
            base_url: self.base_url,
            max_retries: self.max_retries,
            max_retry_delay: self.max_retry_delay,
        })
    }
}

/// Whether a response indicates a GitHub rate limit. GitHub returns 429, or
/// 403 with `X-RateLimit-Remaining: 0`, when the limit is exhausted.
fn is_rate_limited(resp: &reqwest::Response) -> bool {
    if resp.status() == StatusCode::TOO_MANY_REQUESTS {
        return true;
    }
    resp.status() == StatusCode::FORBIDDEN
        && resp
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.trim() == "0")
            .unwrap_or(false)
}

/// Retry wait from the response headers: `Retry-After` (delta seconds) wins
/// over `X-RateLimit-Reset` (absolute unix time); 1s if neither parses. The
/// caller caps the result.
fn retry_delay(headers: &HeaderMap, now_unix: u64) -> Duration {
    let header_secs = |name: &str| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse::<u64>().ok())
    };

    if let Some(secs) = header_secs("retry-after") {
        return Duration::from_secs(secs);
    }
    if let Some(reset) = header_secs("x-ratelimit-reset") {
        return Duration::from_secs(reset.saturating_sub(now_unix));
    }
    Duration::from_secs(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The builder's accumulated headers, for inspection. Goes through the real
    /// builder methods so header construction is exercised; reads the private
    /// `headers` field before `build()` consumes it.
    fn headers(b: ClientBuilder) -> HeaderMap {
        b.headers.clone()
    }

    #[test]
    fn bearer_none_adds_no_authorization() {
        let b = Client::builder("https://x").bearer(None);
        assert!(!headers(b).contains_key(AUTHORIZATION));
    }

    #[test]
    fn bearer_some_sets_sensitive_authorization() {
        let b = Client::builder("https://x").bearer(Some("tok"));
        let h = headers(b);
        let v = h.get(AUTHORIZATION).expect("authorization header present");
        assert_eq!(v.to_str().unwrap(), "Bearer tok");
        assert!(v.is_sensitive());
    }

    #[test]
    fn header_lands_in_map() {
        let b = Client::builder("https://x").header("X-Test", "yes");
        let h = headers(b);
        assert_eq!(h.get("X-Test").unwrap().to_str().unwrap(), "yes");
    }

    #[test]
    fn invalid_token_fails_build() {
        // A newline is not a valid header-value character.
        let err = Client::builder("https://x")
            .bearer(Some("bad\ntoken"))
            .build();
        assert!(err.is_err());
    }

    #[test]
    fn valid_config_builds() {
        let ok = Client::builder("https://x")
            .header("User-Agent", "libway")
            .bearer(Some("tok"))
            .build();
        assert!(ok.is_ok());
    }

    fn map(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.insert(
                HeaderName::from_bytes(k.as_bytes()).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
        }
        h
    }

    #[test]
    fn retry_after_takes_precedence() {
        let h = map(&[("retry-after", "12"), ("x-ratelimit-reset", "9999999999")]);
        assert_eq!(retry_delay(&h, 1000), Duration::from_secs(12));
    }

    #[test]
    fn reset_is_relative_to_now() {
        let h = map(&[("x-ratelimit-reset", "1050")]);
        assert_eq!(retry_delay(&h, 1000), Duration::from_secs(50));
    }

    #[test]
    fn reset_in_the_past_is_zero() {
        let h = map(&[("x-ratelimit-reset", "900")]);
        assert_eq!(retry_delay(&h, 1000), Duration::from_secs(0));
    }

    #[test]
    fn no_headers_defaults_to_one_second() {
        assert_eq!(retry_delay(&HeaderMap::new(), 1000), Duration::from_secs(1));
    }

    #[test]
    fn unparseable_header_falls_back() {
        let h = map(&[("retry-after", "soon")]);
        assert_eq!(retry_delay(&h, 1000), Duration::from_secs(1));
    }
}
