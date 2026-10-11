use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

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

/// Sort repositories by star count descending then by name to break ties
pub fn stars_descending_then_name() -> fn(&Repository, &Repository) -> Ordering {
    |left, right| {
        right
            .stars
            .cmp(&left.stars)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorting_order_stars_descending() {
        let mut repos = vec![
            Repository {
                owner: "rust-lang".to_string(),
                name: "cargo".to_string(),
                url: "https://github.com/rust-lang/cargo".to_string(),
                description: "The Rust package manager".to_string(),
                stars: 12000,
            },
            Repository {
                owner: "rust-lang".to_string(),
                name: "rust".to_string(),
                url: "https://github.com/rust-lang/rust".to_string(),
                description: "Empowering everyone...".to_string(),
                stars: 100000,
            },
            Repository {
                owner: "rust-lang".to_string(),
                name: "rustlings".to_string(),
                url: "https://github.com/rust-lang/rustlings".to_string(),
                description: "Small exercises".to_string(),
                stars: 50000,
            },
            Repository {
                owner: "rust-lang".to_string(),
                name: "a-equal-stars".to_string(),
                url: "https://github.com/rust-lang/a-equal-stars".to_string(),
                description: "Equal stars tiebreak".to_string(),
                stars: 12000,
            },
        ];

        repos.sort_by(stars_descending_then_name());

        assert_eq!(repos[0].name, "rust");
        assert_eq!(repos[0].stars, 100000);
        assert_eq!(repos[1].name, "rustlings");
        assert_eq!(repos[1].stars, 50000);
        // Tie-break alphabetically: "a-equal-stars" before "cargo"
        assert_eq!(repos[2].name, "a-equal-stars");
        assert_eq!(repos[2].stars, 12000);
        assert_eq!(repos[3].name, "cargo");
        assert_eq!(repos[3].stars, 12000);
    }

    #[test]
    fn test_repository_from_raw_sensible_description() {
        // Test with empty description
        let raw_empty = RawGitHubRepo {
            name: "test-repo".to_string(),
            owner: RawGitHubOwner {
                login: "alice".to_string(),
            },
            html_url: "https://github.com/alice/test-repo".to_string(),
            description: None,
            fork: false,
            stargazers_count: 42,
        };
        let repo = Repository::from_raw(raw_empty);
        assert_eq!(repo.description, "(no description)");
        assert_eq!(repo.owner, "alice");
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.stars, 42);

        // Test with whitespace-only description
        let raw_ws = RawGitHubRepo {
            name: "test-repo-2".to_string(),
            owner: RawGitHubOwner {
                login: "bob".to_string(),
            },
            html_url: "https://github.com/bob/test-repo-2".to_string(),
            description: Some("   \t\n  ".to_string()),
            fork: false,
            stargazers_count: 10,
        };
        let repo2 = Repository::from_raw(raw_ws);
        assert_eq!(repo2.description, "(no description)");

        // Test with valid description
        let raw_valid = RawGitHubRepo {
            name: "popular-repo".to_string(),
            owner: RawGitHubOwner {
                login: "charlie".to_string(),
            },
            html_url: "https://github.com/charlie/popular-repo".to_string(),
            description: Some("  Awesome Rust library  ".to_string()),
            fork: false,
            stargazers_count: 1200,
        };
        let repo3 = Repository::from_raw(raw_valid);
        assert_eq!(repo3.description, "Awesome Rust library");
        assert_eq!(repo3.stars, 1200);
    }
}
