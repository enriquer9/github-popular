use crate::{AppError, Repository};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

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

/// Writes repository results to JSON format at the specified path.
pub fn export_json(path: &Path, payload: &ExportPayload) -> crate::Result<()> {
    create_parents(path)?;

    let json_str = serde_json::to_string_pretty(payload)?;
    let mut file = File::create(path).map_err(|e| AppError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    file.write_all(json_str.as_bytes())
        .map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

    Ok(())
}

/// Writes repository results to TOML format at the specified path.
pub fn export_toml(path: &Path, payload: &ExportPayload) -> crate::Result<()> {
    create_parents(path)?;

    let toml_str = toml::to_string_pretty(payload)?;
    let mut file = File::create(path).map_err(|e| AppError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    file.write_all(toml_str.as_bytes())
        .map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

    Ok(())
}

fn create_parents(path: &Path) -> crate::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }

    Ok(())
}
