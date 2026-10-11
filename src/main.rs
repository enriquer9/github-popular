use clap::Parser;
use colored::Colorize;
use github_popular::file_exporter::{export_json, export_toml, ExportPayload};
use github_popular::github_client::GitHubResult;
use github_popular::github_client::GitHubResult::{Failure, Success};
use github_popular::github_models::stars_descending_then_name;
use github_popular::{AppError, Cli, GitHubClient, Repository, TerminalWriter};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};

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

    let has_file_export = cli.has_file_export();

    let stream = match stream_repos(client, cli.accounts) {
        Ok(s) => s,
        Err(err) => {
            eprintln!(
                "{}: Failed to open stream: {}",
                "Error".bright_red().bold(),
                err
            );
            return ExitCode::FAILURE;
        }
    };

    let (has_stream_errors, all_repositories) = handle_stream(has_file_export, stream).await;
    let has_export_errors = export(has_file_export, cli.json, cli.toml, all_repositories);

    if has_stream_errors || has_export_errors {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn stream_repos(
    client: GitHubClient,
    accounts: Vec<String>,
) -> Result<impl Stream<Item = Result<GitHubResult, AppError>>, AppError> {
    let client = Arc::new(client);
    let (tx, rx) = mpsc::channel::<github_popular::Result<GitHubResult>>(1);

    for account in accounts {
        let client = Arc::clone(&client);
        let tx = tx.clone();

        tokio::spawn(async move {
            let result = client.fetch_popular_repos(&account).await;
            let _ = tx.send(result).await;
        });
    }

    drop(tx);
    Ok(ReceiverStream::new(rx))
}

async fn handle_stream(
    has_file_export: bool,
    mut stream: impl Stream<Item = Result<GitHubResult, AppError>> + Unpin,
) -> (bool, Vec<Repository>) {
    let terminal = TerminalWriter::new();
    let mut all_repositories: Vec<Repository> = Vec::new();
    let mut has_errors = false;

    while let Some(result) = stream.next().await {
        match result {
            Ok(ghr) => match ghr {
                Success { account, repos } => {
                    if !has_file_export {
                        terminal.render_account_repos(&account, &repos);
                    }
                    all_repositories.extend(repos);
                }
                Failure { account, error } => {
                    eprintln!(
                        "{}: Failed to fetch repositories for account '{}': {}",
                        "Error".bright_red().bold(),
                        account,
                        error
                    );
                    has_errors = true;
                }
            },
            Err(err) => {
                eprintln!(
                    "{}: Failed to fetch repositories: {}",
                    "Error".bright_red().bold(),
                    err
                );
                has_errors = true;
            }
        }
    }

    (has_errors, all_repositories)
}

fn export(
    has_file_export: bool,
    json: Option<PathBuf>,
    toml: Option<PathBuf>,
    mut all_repositories: Vec<Repository>,
) -> bool {
    let mut has_errors = false;
    // If file exports are specified, serialize the combined or per-account list
    if has_file_export {
        all_repositories.sort_by(stars_descending_then_name());

        let payload = ExportPayload::new(all_repositories);

        if let Some(json_path) = &json {
            if let Err(err) = export_json(json_path, &payload) {
                eprintln!(
                    "{}: Failed to write JSON export to '{}': {}",
                    "Error".bright_red().bold(),
                    json_path.display(),
                    err
                );
                has_errors = true;
            }
        }

        if let Some(toml_path) = &toml {
            if let Err(err) = export_toml(toml_path, &payload) {
                eprintln!(
                    "{}: Failed to write TOML export to '{}': {}",
                    "Error".bright_red().bold(),
                    toml_path.display(),
                    err
                );
                has_errors = true;
            }
        }
    }

    has_errors
}
