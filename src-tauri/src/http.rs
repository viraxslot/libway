//! Reusable HTTP client over reqwest: configured once via a builder, then used
//! for typed GET requests. Source-agnostic — no API specifics live here.

use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use serde::de::DeserializeOwned;

/// How much of an error response body to include in error messages.
const MAX_ERROR_BODY: usize = 200;

/// A configured HTTP client. Build one with [`Client::builder`].
pub struct Client {
    inner: reqwest::Client,
    base_url: String,
}

/// Accumulates configuration for a [`Client`]. Header-construction errors are
/// deferred until [`ClientBuilder::build`] so the call chain stays fluent.
pub struct ClientBuilder {
    base_url: String,
    headers: HeaderMap,
    error: Option<anyhow::Error>,
}

impl Client {
    /// Start building a client whose requests are relative to `base_url`.
    pub fn builder(base_url: impl Into<String>) -> ClientBuilder {
        ClientBuilder {
            base_url: base_url.into(),
            headers: HeaderMap::new(),
            error: None,
        }
    }

    /// Issue a GET to `path` appended to the base URL and return the response.
    /// `path` is concatenated directly, so it must start with `/` (and the base
    /// URL must not end with one). Network failures carry a contextual message.
    async fn send(&self, path: &str) -> Result<reqwest::Response> {
        let url = format!("{}{path}", self.base_url);
        self.inner
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {url} failed"))
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
        })
    }
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
}
