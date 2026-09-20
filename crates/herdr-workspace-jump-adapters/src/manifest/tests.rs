use super::*;
use crate::config::read_jump_targets;
use std::fs;
use std::path::PathBuf;

const TWO_WORKSPACES: &str = r#"
[workspaces]
netpulse = "/opt/netpulse"
Ivy = "~/workspaces/Ivy"
"#;

const GOLDEN_MANIFEST: &str = r#"id = "herdr-workspace-jump"
name = "Workspace Jump"
version = "0.1.0"
min_herdr_version = "0.7.0"
description = "Workspace navigation herdr has no built-in for: create-or-focus by label, and a most-recently-used toggle."
platforms = ["macos", "linux"]

[[build]]
command = ["cargo", "build", "--release", "--locked"]

[[actions]]
id = "jump_ivy"
title = "Jump to the Ivy workspace"
command = ["./target/release/herdr-workspace-jump", "jump", "Ivy", "~/workspaces/Ivy"]

[[actions]]
id = "jump_netpulse"
title = "Jump to the netpulse workspace"
command = ["./target/release/herdr-workspace-jump", "jump", "netpulse", "/opt/netpulse"]

[[actions]]
id = "last_workspace"
title = "Last workspace (most-recently-used toggle)"
command = ["./target/release/herdr-workspace-jump", "last-workspace"]

[[events]]
on = "workspace.focused"
command = ["./target/release/herdr-workspace-jump", "record"]
"#;

struct TempDirectory(PathBuf);

impl TempDirectory {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-manifest-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }

    fn write(&self, name: &str, content: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, content).expect("a sandbox file");
        path
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn targets_from(sandbox: &TempDirectory, declarations: &str) -> Vec<JumpTarget> {
    read_jump_targets(&sandbox.write("config.toml", declarations)).expect("a readable config")
}

#[test]
fn render_manifest_renders_the_declared_workspaces_ordered_by_label() {
    let sandbox = TempDirectory::new("golden");

    let rendered = render_manifest(&targets_from(&sandbox, TWO_WORKSPACES)).expect("a manifest");

    assert_eq!(rendered, GOLDEN_MANIFEST);
}

#[test]
fn render_manifest_escapes_a_label_that_would_otherwise_break_the_toml() {
    let sandbox = TempDirectory::new("escaping");
    let declarations = r#"
[workspaces]
'say "hi"\' = "/opt/quoted"
"#;

    let rendered = render_manifest(&targets_from(&sandbox, declarations)).expect("a manifest");

    let parsed: toml::Value = toml::from_str(&rendered).expect("the manifest parses back");
    assert_eq!(
        parsed["actions"][0]["command"][2].as_str(),
        Some(r#"say "hi"\"#)
    );
    assert_eq!(parsed["actions"][0]["id"].as_str(), Some("jump_say__hi__"));
}

#[test]
fn write_manifest_writes_the_plugin_manifest_and_replaces_an_existing_one() {
    let sandbox = TempDirectory::new("write");
    let plugin_directory = sandbox.0.join("plugin");
    fs::create_dir_all(&plugin_directory).expect("a plugin directory");
    let manifest = plugin_directory.join("herdr-plugin.toml");
    fs::write(&manifest, "stale").expect("a stale manifest");

    write_manifest(&plugin_directory, &targets_from(&sandbox, TWO_WORKSPACES))
        .expect("the manifest is written");

    assert_eq!(fs::read_to_string(&manifest).unwrap(), GOLDEN_MANIFEST);
    let leftovers: Vec<PathBuf> = fs::read_dir(&plugin_directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path != &manifest)
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[test]
fn write_manifest_reports_a_directory_it_cannot_write() {
    let sandbox = TempDirectory::new("unwritable");
    let absent = sandbox.0.join("no-such-directory");

    let refusal = write_manifest(&absent, &targets_from(&sandbox, TWO_WORKSPACES))
        .expect_err("a missing directory is refused");

    assert!(refusal.contains(&absent.display().to_string()), "{refusal}");
}

#[test]
fn the_rendered_manifest_round_trips_with_argv_arrays_for_every_action() {
    let sandbox = TempDirectory::new("round-trip");

    let rendered = render_manifest(&targets_from(&sandbox, TWO_WORKSPACES)).expect("a manifest");

    let parsed: toml::Value = toml::from_str(&rendered).expect("the manifest parses back");
    let jumps: Vec<Vec<&str>> = parsed["actions"]
        .as_array()
        .expect("an array of actions")
        .iter()
        .map(|action| {
            action["command"]
                .as_array()
                .expect("an argv array")
                .iter()
                .map(|word| word.as_str().expect("an argv word"))
                .collect()
        })
        .collect();
    assert_eq!(
        jumps,
        vec![
            vec![
                "./target/release/herdr-workspace-jump",
                "jump",
                "Ivy",
                "~/workspaces/Ivy"
            ],
            vec![
                "./target/release/herdr-workspace-jump",
                "jump",
                "netpulse",
                "/opt/netpulse"
            ],
            vec!["./target/release/herdr-workspace-jump", "last-workspace"],
        ]
    );
    assert_eq!(
        parsed["events"][0]["on"].as_str(),
        Some("workspace.focused")
    );
}
