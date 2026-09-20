use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use herdr_workspace_jump_domain::{JumpTarget, TargetError, jump_targets};
use serde::Deserialize;

/// The declared workspaces, keyed by the label herdr shows.
///
/// A `BTreeMap` orders the manifest by label, so two runs over the same file
/// render the same bytes.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceDeclarations {
    workspaces: BTreeMap<String, String>,
}

/// Why the declared workspaces could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// The file could not be opened or read.
    Unreadable { path: PathBuf, reason: String },
    /// The file is not the TOML this plugin expects.
    Unparseable {
        path: PathBuf,
        reason: String,
        line: Option<usize>,
    },
    /// The file parsed, but the workspaces it declares cannot become actions.
    Invalid { path: PathBuf, cause: TargetError },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (path, reason) = match self {
            Self::Unreadable { path, reason } => (path, reason.clone()),
            Self::Unparseable { path, reason, line } => (
                path,
                match line {
                    Some(line) => format!("{reason} at line {line}"),
                    None => reason.clone(),
                },
            ),
            Self::Invalid { path, cause } => (path, cause.to_string()),
        };
        write!(
            formatter,
            "cannot read the workspace list from {}: {reason}",
            path.display()
        )
    }
}

/// herdr hands a plugin its own config directory; the documented path is the
/// fallback for a run from a shell.
pub fn config_file(
    plugin_config_dir: Option<&str>,
    xdg_config_home: Option<&str>,
    home: Option<&str>,
) -> PathBuf {
    let directory = match plugin_config_dir.filter(|directory| !directory.is_empty()) {
        Some(directory) => PathBuf::from(directory),
        None => {
            base_config_dir(xdg_config_home, home).join("herdr/plugins/config/herdr-workspace-jump")
        }
    };
    directory.join("config.toml")
}

fn base_config_dir(xdg_config_home: Option<&str>, home: Option<&str>) -> PathBuf {
    match xdg_config_home.filter(|directory| !directory.is_empty()) {
        Some(directory) => PathBuf::from(directory),
        None => PathBuf::from(home.unwrap_or_default()).join(".config"),
    }
}

/// Read the declared workspaces and turn them into jump targets, ordered by label.
pub fn read_jump_targets(path: &Path) -> Result<Vec<JumpTarget>, ConfigError> {
    let content = fs::read_to_string(path).map_err(|failure| ConfigError::Unreadable {
        path: path.to_owned(),
        reason: failure.to_string(),
    })?;
    let declarations: WorkspaceDeclarations =
        toml::from_str(&content).map_err(|failure| ConfigError::Unparseable {
            path: path.to_owned(),
            reason: failure.message().to_string(),
            line: line_of(&content, &failure),
        })?;
    jump_targets(declarations.workspaces).map_err(|cause| ConfigError::Invalid {
        path: path.to_owned(),
        cause,
    })
}

/// The one-based line a parse failure points at, when it carries a span.
fn line_of(content: &str, failure: &toml::de::Error) -> Option<usize> {
    let span = failure.span()?;
    let before = content.get(..span.start)?;
    Some(before.matches('\n').count() + 1)
}

#[cfg(test)]
mod tests;
