use crate::{cli_parser, file_exporter, github_client};
use thiserror::Error;

/// Application errors for `github-popular`.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("CLI parser error: {0}")]
    CliParser(#[from] cli_parser::CliError),

    #[error("File exporter error: {0}")]
    FileExporter(#[from] file_exporter::ExportError),

    #[error("GitHub client error: {0}")]
    GitHubClient(#[from] github_client::GitHubError),
}

pub type AppResult<T> = Result<T, AppError>;
