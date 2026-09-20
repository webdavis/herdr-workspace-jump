use herdr_workspace_jump_domain::JumpTarget;
use serde::Serialize;

/// herdr runs a plugin's commands from the plugin directory, so the release
/// binary is named relative to it.
const EXECUTABLE: &str = "./target/release/herdr-workspace-jump";

#[derive(Serialize)]
struct Manifest<'a> {
    id: &'static str,
    name: &'static str,
    version: &'a str,
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
///
/// `version` is the version of the package that owns the binary, which the
/// caller knows and this crate does not.
pub fn render_manifest(version: &str, targets: &[JumpTarget]) -> Result<String, toml::ser::Error> {
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
        version,
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
    toml::to_string(&manifest)
}

#[cfg(test)]
mod tests;
