use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::{Client, Response, StatusCode};
use std::time::{Duration, UNIX_EPOCH};

use crate::errors::{AppError, Result};
use crate::models::{RawGitHubAccount, RawGitHubRepo, Repository};

const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// GitHub REST API v3 Client
#[derive(Clone, Debug)]
pub struct GitHubClient {
    client: Client,
    token: Option<String>,
}

impl GitHubClient {
    /// Creates a new GitHub API client configured with proper headers and timeouts.
    /// Reads `GITHUB_TOKEN` from the environment if present.
    pub fn new() -> Result<Self> {
        let token = std::env::var("GITHUB_TOKEN").ok().and_then(|t| {
            let trimmed = t.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("github-popular/0.1.0 (Rust)"),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github.v3+json"),
        );

        if let Some(ref t) = token {
            let auth_val = format!("Bearer {}", t);
            if let Ok(mut val) = HeaderValue::from_str(&auth_val) {
                val.set_sensitive(true);
                headers.insert(AUTHORIZATION, val);
            }
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()?;

        Ok(Self { client, token })
    }

    /// Whether this client is operating with an authenticated GitHub token.
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    /// Fetches all public, non-forked repositories owned by the specified account,
    /// sorted by star count in descending order.
    pub async fn fetch_popular_repos(&self, account: &str) -> Result<Vec<Repository>> {
        let trimmed_account = account.trim();
        if trimmed_account.is_empty() {
            return Err(AppError::AccountNotFound(account.to_string()));
        }

        // 1. Verify account exists and determine if it's a User or Organization
        let account_info = self.get_account_info(trimmed_account).await?;

        // 2. Fetch all public repositories owned by the account across all pages
        let raw_repos = self.fetch_all_repos(trimmed_account, &account_info.account_type).await?;

        // 3. Exclude forked repositories
        let mut repos: Vec<Repository> = raw_repos
            .into_iter()
            .filter(|r| !r.fork)
            .map(Repository::from_raw)
            .collect();

        // 4. Sort by star count in descending order (with secondary name sort for deterministic output)
        repos.sort_by(|a, b| {
            b.stars
                .cmp(&a.stars)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        Ok(repos)
    }

    /// Queries the account endpoint `/users/{account}` to verify existence and retrieve account type.
    async fn get_account_info(&self, account: &str) -> Result<RawGitHubAccount> {
        let url = format!("https://api.github.com/users/{}", account);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return self.handle_error_response(resp, account).await;
        }

        let body_bytes = resp.bytes().await?;
        let account_info: RawGitHubAccount = serde_json::from_slice(&body_bytes)?;
        Ok(account_info)
    }

    /// Fetches all repositories owned by the account, handling pagination.
    async fn fetch_all_repos(&self, account: &str, account_type: &str) -> Result<Vec<RawGitHubRepo>> {
        let mut all_repos = Vec::new();
        let mut page = 1;

        let base_url = if account_type.eq_ignore_ascii_case("Organization") {
            format!("https://api.github.com/orgs/{}/repos?type=public", account)
        } else {
            format!("https://api.github.com/users/{}/repos?type=owner", account)
        };

        loop {
            let page_url = format!("{}&per_page=100&page={}", base_url, page);
            let resp = self.client.get(&page_url).send().await?;

            if !resp.status().is_success() {
                return self.handle_error_response(resp, account).await;
            }

            // Check pagination link header to see if there is a next page
            let has_next_link = resp
                .headers()
                .get("link")
                .and_then(|val| val.to_str().ok())
                .map(|link_str| link_str.contains("rel=\"next\""))
                .unwrap_or(false);

            let body_bytes = resp.bytes().await?;
            let page_repos: Vec<RawGitHubRepo> = serde_json::from_slice(&body_bytes)?;
            let count = page_repos.len();

            all_repos.extend(page_repos);

            // GitHub returns fewer items than per_page (100) or empty array when at end
            if count < 100 || !has_next_link {
                break;
            }

            page += 1;
        }

        Ok(all_repos)
    }

    /// Translates HTTP error responses into descriptive domain errors.
    async fn handle_error_response<T>(&self, resp: Response, account: &str) -> Result<T> {
        let status = resp.status();
        let headers = resp.headers().clone();

        // Extract rate limit headers if present
        let rate_remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let rate_limit = headers
            .get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("60");
        let rate_reset = headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());

        let body_text = resp.text().await.unwrap_or_default();
        let message = extract_github_message(&body_text).unwrap_or(body_text);

        if status == StatusCode::NOT_FOUND {
            return Err(AppError::AccountNotFound(account.to_string()));
        }

        if status == StatusCode::UNAUTHORIZED {
            return Err(AppError::AuthenticationFailed {
                status: status.as_u16(),
                message,
            });
        }

        if (status == StatusCode::FORBIDDEN || status == StatusCode::TOO_MANY_REQUESTS)
            && (rate_remaining == "0" || message.to_lowercase().contains("rate limit"))
        {
            let reset_time_str = if let Some(epoch_secs) = rate_reset {
                format_epoch_secs(epoch_secs)
            } else {
                "shortly".to_string()
            };

            return Err(AppError::RateLimitExceeded {
                limit: rate_limit.to_string(),
                reset_time: reset_time_str,
            });
        }

        if status == StatusCode::FORBIDDEN {
            return Err(AppError::AuthenticationFailed {
                status: status.as_u16(),
                message,
            });
        }

        Err(AppError::GitHubApi {
            status: status.as_u16(),
            message,
        })
    }
}

/// Helper to parse standard GitHub JSON error message {"message": "..."}
fn extract_github_message(body: &str) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct ErrObj {
        message: Option<String>,
    }

    serde_json::from_str::<ErrObj>(body)
        .ok()
        .and_then(|obj| obj.message)
}

/// Formats unix epoch seconds into a human-readable UTC timestamp
fn format_epoch_secs(epoch_secs: u64) -> String {
    let now_secs = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if epoch_secs > now_secs {
        let diff_secs = epoch_secs - now_secs;
        let mins = diff_secs / 60;
        let secs = diff_secs % 60;
        format!("in {}m {}s (epoch {})", mins, secs, epoch_secs)
    } else {
        format!("epoch {}", epoch_secs)
    }
}
