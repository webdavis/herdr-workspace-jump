use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use herdr_workspace_jump_domain::JumpTarget;
use herdr_workspace_jump_protocol::render_manifest;

const MANIFEST_NAME: &str = "herdr-plugin.toml";

/// Why the plugin manifest could not be put in place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// The declared workspaces could not be encoded as TOML.
    Unrenderable(String),
    /// The manifest could not be written where it was asked for.
    Unwritable { path: PathBuf, reason: String },
}

impl fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unrenderable(reason) => {
                write!(formatter, "cannot render the plugin manifest: {reason}")
            }
            Self::Unwritable { path, reason } => {
                write!(formatter, "cannot write {}: {reason}", path.display())
            }
        }
    }
}

/// Write the manifest into the plugin directory, replacing any manifest already there.
pub fn write_manifest(
    directory: &Path,
    version: &str,
    targets: &[JumpTarget],
) -> Result<(), ManifestError> {
    let rendered = render_manifest(version, targets)
        .map_err(|failure| ManifestError::Unrenderable(failure.to_string()))?;
    let target = directory.join(MANIFEST_NAME);
    replace(directory, &target, rendered.as_bytes()).map_err(|failure| ManifestError::Unwritable {
        path: target,
        reason: failure.to_string(),
    })
}

fn replace(directory: &Path, target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let temporary = directory.join(format!(".{MANIFEST_NAME}.{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o644)
        .open(&temporary)?;
    let written = file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
        .and_then(|()| std::fs::rename(&temporary, target));
    if written.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    written
}

#[cfg(test)]
mod tests;
