use clap::Parser;
use std::path::{Path, PathBuf};
use crate::errors::{AppError, Result};

#[derive(Parser, Debug, Clone)]
#[command(
    name = "github-popular",
    author = "Rust Systems Engineer",
    version,
    about = "Find the most popular public repositories owned by GitHub accounts",
    long_about = "Fetches public repositories owned by one or more GitHub accounts (users or organizations),\nexcludes forked repositories, sorts them by star count in descending order, and displays formatted results or saves them to JSON/TOML."
)]
pub struct Cli {
    /// GitHub account logins to inspect (one or more user or organization names)
    #[arg(required = true, value_name = "ACCOUNT")]
    pub accounts: Vec<String>,

    /// Save results as JSON
    #[arg(long, value_name = "PATH")]
    pub json: Option<PathBuf>,

    /// Save results as TOML
    #[arg(long, value_name = "PATH")]
    pub toml: Option<PathBuf>,
}

impl Cli {
    /// Validates command line arguments according to specification.
    /// Returns an error if --json and --toml point to the same target path.
    pub fn validate(&self) -> Result<()> {
        if self.accounts.is_empty() {
            return Err(AppError::NoAccountsProvided);
        }

        if let (Some(json_path), Some(toml_path)) = (&self.json, &self.toml) {
            if paths_are_equivalent(json_path, toml_path) {
                return Err(AppError::DuplicateOutputPath(json_path.clone()));
            }
        }

        Ok(())
    }

    /// Returns true if either `--json` or `--toml` is specified.
    pub fn has_file_output(&self) -> bool {
        self.json.is_some() || self.toml.is_some()
    }
}

/// Checks if two path references resolve to the same filesystem location.
fn paths_are_equivalent(p1: &Path, p2: &Path) -> bool {
    if p1 == p2 {
        return true;
    }

    // Attempt canonicalization if paths exist; otherwise compare normalized absolute representations
    let c1 = p1.canonicalize();
    let c2 = p2.canonicalize();

    match (c1, c2) {
        (Ok(a), Ok(b)) => a == b,
        _ => {
            // Fall back to current_dir combined path comparison
            if let (Ok(cwd), Ok(cwd2)) = (std::env::current_dir(), std::env::current_dir()) {
                let abs1 = cwd.join(p1);
                let abs2 = cwd2.join(p2);
                abs1 == abs2
            } else {
                p1 == p2
            }
        }
    }
}
