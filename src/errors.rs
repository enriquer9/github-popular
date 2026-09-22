use std::path::PathBuf;
use thiserror::Error;

/// Application errors for `github-popular`.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("GitHub account '{0}' was not found (HTTP 404)")]
    AccountNotFound(String),

    #[error("Authentication failed (HTTP {status}): {message}. Please verify your GITHUB_TOKEN.")]
    AuthenticationFailed { status: u16, message: String },

    #[error("GitHub API rate limit exceeded ({limit} requests/hr). Limit resets at {reset_time}. Consider providing a GITHUB_TOKEN environment variable.")]
    RateLimitExceeded {
        limit: String,
        reset_time: String,
    },

    #[error("GitHub API error (HTTP {status}): {message}")]
    GitHubApi { status: u16, message: String },

    #[error("Network transport failure: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("Malformed JSON response from GitHub API: {0}")]
    MalformedResponse(#[from] serde_json::Error),

    #[error("Failed to serialize TOML output: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("File I/O error at '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Conflicting output paths: --json and --toml cannot write to the same destination path: {0}")]
    DuplicateOutputPath(PathBuf),

    #[error("No accounts were provided. Specify at least one GitHub account login.")]
    NoAccountsProvided,
}

pub type Result<T> = std::result::Result<T, AppError>;
