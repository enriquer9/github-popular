use serde::{Deserialize, Serialize};

/// Upstream GitHub repository structure returned by REST API v3.
#[derive(Debug, Clone, Deserialize)]
pub struct RawGitHubRepo {
    pub name: String,
    pub owner: RawGitHubOwner,
    pub html_url: String,
    pub description: Option<String>,
    pub fork: bool,
    pub stargazers_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawGitHubOwner {
    pub login: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawGitHubAccount {
    pub login: String,
    #[serde(rename = "type")]
    pub account_type: String, // "User" or "Organization"
}

/// Normalized repository record satisfying the required output specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repository {
    pub owner: String,
    pub name: String,
    pub url: String,
    pub description: String,
    pub stars: u64,
}

impl Repository {
    /// Constructs a clean `Repository` from upstream GitHub API data.
    /// Ensures sensible fallback for empty or missing descriptions.
    pub fn from_raw(raw: RawGitHubRepo) -> Self {
        let description = match raw.description {
            Some(desc) if !desc.trim().is_empty() => desc.trim().to_string(),
            _ => "(no description)".to_string(),
        };

        Self {
            owner: raw.owner.login,
            name: raw.name,
            url: raw.html_url,
            description,
            stars: raw.stargazers_count,
        }
    }
}

/// Container struct for serialized outputs (JSON and TOML).
/// Ensures identical fields and consistent formatting in both formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputPayload {
    pub repositories: Vec<Repository>,
}

impl OutputPayload {
    pub fn new(repositories: Vec<Repository>) -> Self {
        Self { repositories }
    }
}
