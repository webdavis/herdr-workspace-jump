use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use herdr_workspace_jump_domain::{JumpTarget, jump_targets};
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

/// Read the declared workspaces and turn them into jump targets.
pub fn read_jump_targets(path: &Path) -> Result<Vec<JumpTarget>, String> {
    let content =
        fs::read_to_string(path).map_err(|failure| refusal(path, &failure.to_string()))?;
    let declarations: WorkspaceDeclarations =
        toml::from_str(&content).map_err(|failure| refusal(path, failure.message()))?;
    jump_targets(declarations.workspaces).map_err(|failure| refusal(path, &failure.to_string()))
}

fn refusal(path: &Path, reason: &str) -> String {
    format!(
        "cannot read the workspace list from {}: {reason}",
        path.display()
    )
}

#[cfg(test)]
mod tests;
