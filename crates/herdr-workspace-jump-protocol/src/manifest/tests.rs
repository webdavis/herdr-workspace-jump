use super::*;

const GOLDEN_MANIFEST: &str = r#"id = "herdr-workspace-jump"
name = "Workspace Jump"
version = "1.2.3"
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
id = "jump_casually_concerned"
title = "Jump to the casually-concerned workspace"
command = ["./target/release/herdr-workspace-jump", "jump", "casually-concerned", "~/repos/casually-concerned"]

[[actions]]
id = "jump_homelab"
title = "Jump to the homelab workspace"
command = ["./target/release/herdr-workspace-jump", "jump", "homelab", "/srv/homelab"]

[[actions]]
id = "jump_justdavis_ansible"
title = "Jump to the justdavis-ansible workspace"
command = ["./target/release/herdr-workspace-jump", "jump", "justdavis-ansible", "~/repos/justdavis-ansible"]

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

fn target(label: &str, directory: &str) -> JumpTarget {
    JumpTarget {
        action_id: herdr_workspace_jump_domain::action_id(label),
        label: label.to_string(),
        directory: directory.to_string(),
        pick_key: 'x',
    }
}

fn five_targets() -> Vec<JumpTarget> {
    vec![
        target("Ivy", "~/workspaces/Ivy"),
        target("casually-concerned", "~/repos/casually-concerned"),
        target("homelab", "/srv/homelab"),
        target("justdavis-ansible", "~/repos/justdavis-ansible"),
        target("netpulse", "/opt/netpulse"),
    ]
}

#[test]
fn render_manifest_renders_one_action_per_target_in_the_order_given() {
    let rendered = render_manifest("1.2.3", &five_targets()).expect("a manifest");

    assert_eq!(rendered, GOLDEN_MANIFEST);
}

#[test]
fn render_manifest_carries_the_version_the_caller_passes() {
    let rendered = render_manifest("0.9.4", &five_targets()).expect("a manifest");

    assert!(rendered.contains("version = \"0.9.4\""), "{rendered}");
}

#[test]
fn render_manifest_escapes_a_label_that_would_otherwise_break_the_toml() {
    let awkward = target(r#"say "hi"\"#, "/opt/quoted");

    let rendered = render_manifest("1.2.3", &[awkward]).expect("a manifest");

    let parsed: toml::Value = toml::from_str(&rendered).expect("the manifest parses back");
    assert_eq!(
        parsed["actions"][0]["command"][2].as_str(),
        Some(r#"say "hi"\"#)
    );
    assert_eq!(parsed["actions"][0]["id"].as_str(), Some("jump_say__hi__"));
}

#[test]
fn the_rendered_manifest_round_trips_with_argv_arrays_for_every_action() {
    let rendered = render_manifest("1.2.3", &five_targets()).expect("a manifest");

    let parsed: toml::Value = toml::from_str(&rendered).expect("the manifest parses back");
    let actions = parsed["actions"].as_array().expect("an array of actions");
    let argv: Vec<Vec<&str>> = actions
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
        argv.first(),
        Some(&vec![
            "./target/release/herdr-workspace-jump",
            "jump",
            "Ivy",
            "~/workspaces/Ivy"
        ])
    );
    assert_eq!(
        argv.last(),
        Some(&vec![
            "./target/release/herdr-workspace-jump",
            "last-workspace"
        ])
    );
    assert_eq!(argv.len(), 6, "five jumps and the toggle");
    assert_eq!(
        parsed["events"][0]["on"].as_str(),
        Some("workspace.focused")
    );
    assert_eq!(
        parsed["build"][0]["command"][0].as_str(),
        Some("cargo"),
        "the build step survives the round trip"
    );
}
