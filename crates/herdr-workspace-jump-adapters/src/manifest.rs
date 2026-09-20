use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use herdr_workspace_jump_domain::JumpTarget;
use serde::Serialize;

/// The path herdr runs a plugin's commands from is the plugin directory, so the
/// release binary is named relative to it.
const EXECUTABLE: &str = "./target/release/herdr-workspace-jump";
const MANIFEST_NAME: &str = "herdr-plugin.toml";

#[derive(Serialize)]
struct Manifest {
    id: &'static str,
    name: &'static str,
    version: &'static str,
    min_herdr_version: &'static str,
    description: &'static str,
    platforms: [&'static str; 2],
    build: [BuildStep; 1],
    actions: Vec<Action>,
    events: [Event; 1],
}

#[derive(Serialize)]
struct BuildStep {
    command: Vec<String>,
}

#[derive(Serialize)]
struct Action {
    id: String,
    title: String,
    command: Vec<String>,
}

#[derive(Serialize)]
struct Event {
    on: &'static str,
    command: Vec<String>,
}

/// Render the plugin manifest that registers one jump action per workspace.
pub fn render_manifest(targets: &[JumpTarget]) -> Result<String, String> {
    let mut actions: Vec<Action> = targets
        .iter()
        .map(|target| Action {
            id: target.action_id.clone(),
            title: format!("Jump to the {} workspace", target.label),
            command: vec![
                EXECUTABLE.to_string(),
                "jump".to_string(),
                target.label.clone(),
                target.directory.clone(),
            ],
        })
        .collect();
    actions.push(Action {
        id: "last_workspace".to_string(),
        title: "Last workspace (most-recently-used toggle)".to_string(),
        command: vec![EXECUTABLE.to_string(), "last-workspace".to_string()],
    });
    let manifest = Manifest {
        id: "herdr-workspace-jump",
        name: "Workspace Jump",
        version: env!("CARGO_PKG_VERSION"),
        min_herdr_version: "0.7.0",
        description: "Workspace navigation herdr has no built-in for: create-or-focus by label, and a most-recently-used toggle.",
        platforms: ["macos", "linux"],
        build: [BuildStep {
            command: vec![
                "cargo".to_string(),
                "build".to_string(),
                "--release".to_string(),
                "--locked".to_string(),
            ],
        }],
        actions,
        events: [Event {
            on: "workspace.focused",
            command: vec![EXECUTABLE.to_string(), "record".to_string()],
        }],
    };
    toml::to_string(&manifest).map_err(|failure| format!("cannot render the manifest: {failure}"))
}

/// Write the manifest into the plugin directory, replacing any manifest already there.
pub fn write_manifest(directory: &Path, targets: &[JumpTarget]) -> Result<(), String> {
    let rendered = render_manifest(targets)?;
    let target = directory.join(MANIFEST_NAME);
    replace(directory, &target, rendered.as_bytes())
        .map_err(|failure| format!("cannot write {}: {failure}", target.display()))
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
