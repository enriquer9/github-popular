use colored::Colorize;
use std::fs::File;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

use crate::errors::{AppError, Result};
use crate::models::{OutputPayload, Repository};

/// Controls colorization and formatting for terminal output.
pub struct TerminalFormatter {
    is_tty: bool,
}

impl TerminalFormatter {
    pub fn new() -> Self {
        // Detect if stdout is attached to a real interactive terminal and respect NO_COLOR
        let is_tty = io::stdout().is_terminal() && std::env::var("NO_COLOR").is_err();
        if !is_tty {
            colored::control::set_override(false);
        } else {
            colored::control::set_override(true);
        }
        Self { is_tty }
    }

    /// Renders a list of repositories for an account with aligned fields and color.
    pub fn render_account_repos(&self, account: &str, repos: &[Repository]) {
        if repos.is_empty() {
            println!(
                "{}",
                format!("No public, non-forked repositories found for account '{}'.", account)
                    .yellow()
            );
            return;
        }

        // Header for the account
        let header_title = format!("Account: {}", account);
        let count_str = format!("({} public non-forked repo{})", repos.len(), if repos.len() == 1 { "" } else { "s" });
        println!();
        println!("{} {}", header_title.bold().bright_magenta(), count_str.dimmed());
        println!("{}", "=".repeat(60).dimmed());

        for (idx, repo) in repos.iter().enumerate() {
            self.render_single_repo(repo, idx + 1);
            if idx + 1 < repos.len() {
                println!("{}", "-".repeat(50).dimmed());
            }
        }
        println!();
    }

    /// Renders a single repository block with aligned separators and distinct colors.
    fn render_single_repo(&self, repo: &Repository, rank: usize) {
        // Aligned field labels (11 characters to align with 'Description')
        let label_owner = format!("{:11}", "Owner");
        let label_name = format!("{:11}", "Name");
        let label_url = format!("{:11}", "URL");
        let label_desc = format!("{:11}", "Description");
        let label_stars = format!("{:11}", "Stars");

        let star_symbol = if self.is_tty { "★ " } else { "" };
        let stars_display = format!("{}{}", star_symbol, repo.stars);

        println!(
            "{}{} {}",
            label_owner.cyan().bold(),
            ":".dimmed(),
            repo.owner.bright_white()
        );
        println!(
            "{}{} #{} {}",
            label_name.cyan().bold(),
            ":".dimmed(),
            rank.to_string().dimmed(),
            repo.name.bright_green().bold()
        );
        println!(
            "{}{} {}",
            label_url.cyan().bold(),
            ":".dimmed(),
            repo.url.bright_blue().underline()
        );
        println!(
            "{}{} {}",
            label_desc.cyan().bold(),
            ":".dimmed(),
            repo.description.normal()
        );
        println!(
            "{}{} {}",
            label_stars.cyan().bold(),
            ":".dimmed(),
            stars_display.bright_yellow().bold()
        );
    }
}

impl Default for TerminalFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Writes repository results to JSON format at the specified path.
pub fn write_json_output(path: &Path, payload: &OutputPayload) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }

    let json_str = serde_json::to_string_pretty(payload)?;
    let mut file = File::create(path).map_err(|e| AppError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    file.write_all(json_str.as_bytes()).map_err(|e| AppError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

/// Writes repository results to TOML format at the specified path.
pub fn write_toml_output(path: &Path, payload: &OutputPayload) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }

    let toml_str = toml::to_string_pretty(payload)?;
    let mut file = File::create(path).map_err(|e| AppError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    file.write_all(toml_str.as_bytes()).map_err(|e| AppError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}
