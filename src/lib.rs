pub use cli::Cli;
pub use errors::{AppError, Result};
pub use github::GitHubClient;
pub use models::{OutputPayload, Repository};
pub use output::{write_json_output, write_toml_output, TerminalFormatter};

pub mod cli;
pub mod errors;
pub mod github;
pub mod models;
pub mod output;

#[cfg(test)]
mod tests {
    use super::*;
    use models::{RawGitHubOwner, RawGitHubRepo};
    use std::path::PathBuf;

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

        repos.sort_by(|a, b| {
            b.stars
                .cmp(&a.stars)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

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
    fn test_json_and_toml_representation_equivalence() {
        let repos = vec![Repository {
            owner: "tokio-rs".to_string(),
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio".to_string(),
            description: "An event-driven, non-blocking I/O platform".to_string(),
            stars: 26000,
        }];
        let payload = OutputPayload::new(repos);

        let json_output =
            serde_json::to_string_pretty(&payload).expect("JSON serialization failed");
        let toml_output = toml::to_string_pretty(&payload).expect("TOML serialization failed");

        let deserialized_from_json: OutputPayload =
            serde_json::from_str(&json_output).expect("JSON deserialization failed");
        let deserialized_from_toml: OutputPayload =
            toml::from_str(&toml_output).expect("TOML deserialization failed");

        assert_eq!(deserialized_from_json, payload);
        assert_eq!(deserialized_from_toml, payload);
        assert_eq!(deserialized_from_json, deserialized_from_toml);
    }

    #[test]
    fn test_duplicate_path_validation() {
        let cli = Cli {
            accounts: vec!["rust-lang".to_string()],
            json: Some(PathBuf::from("/tmp/results.data")),
            toml: Some(PathBuf::from("/tmp/results.data")),
        };

        let result = cli.validate();
        assert!(result.is_err(), "Duplicate paths should return an error");
        match result.unwrap_err() {
            AppError::DuplicateOutputPath(p) => {
                assert_eq!(p, PathBuf::from("/tmp/results.data"));
            }
            other => panic!("Unexpected error type: {:?}", other),
        }

        // Distinct paths should succeed
        let cli_ok = Cli {
            accounts: vec!["rust-lang".to_string()],
            json: Some(PathBuf::from("/tmp/results.json")),
            toml: Some(PathBuf::from("/tmp/results.toml")),
        };
        assert!(cli_ok.validate().is_ok());
    }

    #[test]
    fn test_no_accounts_validation() {
        let cli = Cli {
            accounts: vec![],
            json: None,
            toml: None,
        };
        assert!(matches!(cli.validate(), Err(AppError::NoAccountsProvided)));
    }
}
