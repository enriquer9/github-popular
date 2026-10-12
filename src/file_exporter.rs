use crate::file_exporter::ExportError::FileIo;
use crate::Repository;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Container struct for serialized exports (JSON and TOML).
/// Ensures identical fields and consistent formatting in both formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportPayload {
    pub repositories: Vec<Repository>,
}

impl ExportPayload {
    pub fn new(repositories: Vec<Repository>) -> Self {
        Self { repositories }
    }
}

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("File I/O error at '{path}': {source}")]
    FileIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to serialize JSON export: {0}")]
    JsonSerialize(#[from] serde_json::Error),

    #[error("Failed to serialize TOML export: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, ExportError>;

/// Writes repository results to JSON format at the specified path.
pub fn export_json(path: &Path, payload: &ExportPayload) -> Result<()> {
    create_parents(path)?;

    let json_str = serde_json::to_string_pretty(payload)?;
    let mut file = File::create(path).map_err(|e| FileIo {
        path: path.to_path_buf(),
        source: e,
    })?;

    file.write_all(json_str.as_bytes()).map_err(|e| FileIo {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

/// Writes repository results to TOML format at the specified path.
pub fn export_toml(path: &Path, payload: &ExportPayload) -> Result<()> {
    create_parents(path)?;

    let toml_str = toml::to_string_pretty(payload)?;
    let mut file = File::create(path).map_err(|e| FileIo {
        path: path.to_path_buf(),
        source: e,
    })?;

    file.write_all(toml_str.as_bytes()).map_err(|e| FileIo {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

fn create_parents(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| FileIo {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_and_toml_representation_equivalence() {
        let repos = vec![Repository {
            owner: "tokio-rs".to_string(),
            name: "tokio".to_string(),
            url: "https://github.com/tokio-rs/tokio".to_string(),
            description: "An event-driven, non-blocking I/O platform".to_string(),
            stars: 26000,
        }];
        let payload = ExportPayload::new(repos);

        let json_export =
            serde_json::to_string_pretty(&payload).expect("JSON serialization failed");
        let toml_export = toml::to_string_pretty(&payload).expect("TOML serialization failed");

        let deserialized_from_json: ExportPayload =
            serde_json::from_str(&json_export).expect("JSON deserialization failed");
        let deserialized_from_toml: ExportPayload =
            toml::from_str(&toml_export).expect("TOML deserialization failed");

        assert_eq!(deserialized_from_json, payload);
        assert_eq!(deserialized_from_toml, payload);
        assert_eq!(deserialized_from_json, deserialized_from_toml);
    }
}
