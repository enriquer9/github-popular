use clap::Parser;
use colored::Colorize;
use github_popular::{
    write_json_output, write_toml_output, Cli, GitHubClient, OutputPayload, Repository,
    TerminalFormatter,
};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Err(err) = cli.validate() {
        eprintln!("{}: {}", "Error".bright_red().bold(), err);
        return ExitCode::FAILURE;
    }

    let client = match GitHubClient::new() {
        Ok(c) => c,
        Err(err) => {
            eprintln!(
                "{}: Failed to initialize GitHub client: {}",
                "Error".bright_red().bold(),
                err
            );
            return ExitCode::FAILURE;
        }
    };

    let formatter = TerminalFormatter::new();
    let mut all_repositories: Vec<Repository> = Vec::new();
    let mut has_errors = false;

    for account in &cli.accounts {
        match client.fetch_popular_repos(account).await {
            Ok(repos) => {
                if !cli.has_file_output() {
                    formatter.render_account_repos(account, &repos);
                }
                all_repositories.extend(repos);
            }
            Err(err) => {
                eprintln!(
                    "{}: Failed to fetch repositories for account '{}': {}",
                    "Error".bright_red().bold(),
                    account,
                    err
                );
                has_errors = true;
            }
        }
    }

    // If file outputs are specified, serialize the combined or per-account list
    if cli.has_file_output() {
        // Sort all accumulated repositories by star count descending across all queried accounts
        all_repositories.sort_by(|a, b| {
            b.stars
                .cmp(&a.stars)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        let payload = OutputPayload::new(all_repositories);

        if let Some(json_path) = &cli.json {
            if let Err(err) = write_json_output(json_path, &payload) {
                eprintln!(
                    "{}: Failed to write JSON output to '{}': {}",
                    "Error".bright_red().bold(),
                    json_path.display(),
                    err
                );
                has_errors = true;
            }
        }

        if let Some(toml_path) = &cli.toml {
            if let Err(err) = write_toml_output(toml_path, &payload) {
                eprintln!(
                    "{}: Failed to write TOML output to '{}': {}",
                    "Error".bright_red().bold(),
                    toml_path.display(),
                    err
                );
                has_errors = true;
            }
        }
    }

    if has_errors {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
