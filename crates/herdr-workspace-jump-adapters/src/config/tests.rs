use super::*;

/// Five workspaces whose declared order is not their sorted order, so a reader
/// that stopped sorting could not pass by chance.
const FIVE_WORKSPACES: &str = r#"
[workspaces]
netpulse = "/opt/netpulse"
Ivy = "~/workspaces/Ivy"
"casually-concerned" = "~/repos/casually-concerned"
homelab = "/srv/homelab"
"justdavis-ansible" = "~/repos/justdavis-ansible"
"#;

struct TempDirectory(PathBuf);

impl TempDirectory {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-config-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }

    fn write(&self, content: &str) -> PathBuf {
        let path = self.0.join("config.toml");
        fs::write(&path, content).expect("a sandbox file");
        path
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn refusal_for(name: &str, content: &str) -> (ConfigError, String) {
    let sandbox = TempDirectory::new(name);
    let path = sandbox.write(content);
    let refusal = read_jump_targets(&path).expect_err("the config is refused");
    let rendered = refusal.to_string();
    assert!(
        rendered.contains(&path.display().to_string()),
        "the refusal names the file: {rendered}"
    );
    assert!(!rendered.contains('\n'), "one sentence: {rendered}");
    (refusal, rendered)
}

#[test]
fn config_file_prefers_the_directory_herdr_hands_the_plugin() {
    assert_eq!(
        config_file(
            Some("/run/herdr/jump"),
            Some("/home/me/xdg"),
            Some("/home/me")
        ),
        PathBuf::from("/run/herdr/jump/config.toml")
    );
}

#[test]
fn config_file_falls_back_to_the_documented_path_under_xdg_then_home() {
    assert_eq!(
        config_file(None, Some("/home/me/xdg"), Some("/home/me")),
        PathBuf::from("/home/me/xdg/herdr/plugins/config/herdr-workspace-jump/config.toml")
    );
    assert_eq!(
        config_file(Some(""), None, Some("/home/me")),
        PathBuf::from("/home/me/.config/herdr/plugins/config/herdr-workspace-jump/config.toml")
    );
    assert_eq!(
        config_file(None, Some(""), Some("/home/me")),
        PathBuf::from("/home/me/.config/herdr/plugins/config/herdr-workspace-jump/config.toml")
    );
}

#[test]
fn read_jump_targets_orders_five_declared_workspaces_by_label() {
    let sandbox = TempDirectory::new("ordered");
    let path = sandbox.write(FIVE_WORKSPACES);

    let targets = read_jump_targets(&path).expect("a readable config");

    let labels: Vec<&str> = targets.iter().map(|target| target.label.as_str()).collect();
    assert_eq!(
        labels,
        vec![
            "Ivy",
            "casually-concerned",
            "homelab",
            "justdavis-ansible",
            "netpulse"
        ]
    );
    let ids: Vec<&str> = targets
        .iter()
        .map(|target| target.action_id.as_str())
        .collect();
    assert_eq!(
        ids,
        vec![
            "jump_ivy",
            "jump_casually_concerned",
            "jump_homelab",
            "jump_justdavis_ansible",
            "jump_netpulse"
        ]
    );
    assert_eq!(
        targets[0].directory, "~/workspaces/Ivy",
        "the tilde stays for the jump verb to expand"
    );
}

#[test]
fn read_jump_targets_refuses_a_missing_file() {
    let sandbox = TempDirectory::new("missing");
    let path = sandbox.0.join("config.toml");

    let refusal = read_jump_targets(&path).expect_err("a missing file is refused");

    assert!(
        matches!(refusal, ConfigError::Unreadable { .. }),
        "{refusal}"
    );
    assert!(refusal.to_string().contains(&path.display().to_string()));
}

#[test]
fn read_jump_targets_refuses_unparseable_toml_and_names_the_line() {
    let (refusal, rendered) = refusal_for(
        "unparseable",
        "[workspaces]\nnetpulse = \"/opt/netpulse\"\nhomelab = unquoted\n",
    );

    assert!(matches!(
        refusal,
        ConfigError::Unparseable { line: Some(3), .. }
    ));
    assert!(rendered.contains("at line 3"), "{rendered}");
}

#[test]
fn read_jump_targets_refuses_a_file_with_no_workspaces_table() {
    let (refusal, rendered) = refusal_for("no-table", "[other]\nkey = \"value\"\n");

    assert!(
        matches!(refusal, ConfigError::Unparseable { .. }),
        "{rendered}"
    );
    assert!(rendered.contains("unknown field"), "{rendered}");
}

#[test]
fn read_jump_targets_refuses_an_unknown_top_level_key() {
    let (_, rendered) = refusal_for(
        "unknown-key",
        "[workspace]\nnetpulse = \"/opt/netpulse\"\n\n[workspaces]\nivy = \"/opt/ivy\"\n",
    );

    assert!(rendered.contains("unknown field `workspace`"), "{rendered}");
}

#[test]
fn read_jump_targets_refuses_an_empty_workspaces_table() {
    let (refusal, rendered) = refusal_for("empty-table", "[workspaces]\n");

    assert_eq!(
        refusal_cause(refusal),
        TargetError::NoWorkspaces,
        "{rendered}"
    );
    assert!(rendered.contains("declares no workspace"), "{rendered}");
}

#[test]
fn read_jump_targets_refuses_an_empty_label_or_an_empty_directory() {
    let (empty_label, rendered) =
        refusal_for("empty-label", "[workspaces]\n\"\" = \"/opt/project\"\n");
    assert_eq!(refusal_cause(empty_label), TargetError::EmptyLabel);
    assert!(rendered.contains("label is empty"), "{rendered}");

    let (empty_directory, rendered) =
        refusal_for("empty-directory", "[workspaces]\nnetpulse = \"\"\n");
    assert_eq!(
        refusal_cause(empty_directory),
        TargetError::EmptyDirectory {
            label: "netpulse".to_string()
        }
    );
    assert!(rendered.contains("empty directory"), "{rendered}");
}

#[test]
fn read_jump_targets_refuses_two_labels_that_derive_the_same_action_id() {
    let (refusal, rendered) = refusal_for(
        "collision",
        "[workspaces]\n\"my-project\" = \"/opt/one\"\n\"my_project\" = \"/opt/two\"\n",
    );

    assert_eq!(
        refusal_cause(refusal),
        TargetError::CollidingActionId {
            first: "my-project".to_string(),
            second: "my_project".to_string(),
            action_id: "jump_my_project".to_string(),
        }
    );
    assert!(rendered.contains("jump_my_project"), "{rendered}");
}

fn refusal_cause(refusal: ConfigError) -> TargetError {
    match refusal {
        ConfigError::Invalid { cause, .. } => cause,
        other => panic!("expected a declared-workspace refusal, got {other:?}"),
    }
}

#[test]
fn read_jump_targets_accepts_a_table_declaring_a_directory_and_a_pick_key() {
    let sandbox = TempDirectory::new("keyed");
    let path = sandbox.write(
        "[workspaces]\nhomelab = \"/srv/homelab\"\nIvy = { dir = \"~/workspaces/Ivy\", key = \"v\" }\n",
    );

    let targets = read_jump_targets(&path).expect("a readable config");

    assert_eq!(targets[0].label, "Ivy");
    assert_eq!(targets[0].directory, "~/workspaces/Ivy");
    assert_eq!(targets[0].pick_key, 'v');
    assert_eq!(
        targets[1].pick_key, 'h',
        "a bare string still defaults its key"
    );
}

#[test]
fn read_jump_targets_refuses_a_table_without_a_directory_or_with_an_unknown_field() {
    let (refusal, rendered) = refusal_for("no-dir", "[workspaces]\nIvy = { key = \"v\" }\n");
    assert!(
        matches!(refusal, ConfigError::Unparseable { .. }),
        "{rendered}"
    );
    assert!(
        rendered.contains("a directory, or a table of a dir and an optional key"),
        "the refusal says what a declaration may be: {rendered}"
    );

    let (refusal, rendered) = refusal_for(
        "unknown-field",
        "[workspaces]\nIvy = { dir = \"/opt/ivy\", keys = \"v\" }\n",
    );
    assert!(
        matches!(refusal, ConfigError::Unparseable { .. }),
        "{rendered}"
    );
}

#[test]
fn read_jump_targets_refuses_two_workspaces_that_pick_on_the_same_key() {
    let (refusal, rendered) = refusal_for(
        "key-collision",
        "[workspaces]\ndamnit = \"/opt/damnit\"\ndotfiles = \"/opt/dotfiles\"\n",
    );

    assert_eq!(
        refusal_cause(refusal),
        TargetError::CollidingPickKey {
            first: "damnit".to_string(),
            second: "dotfiles".to_string(),
            key: 'd',
        }
    );
    assert!(rendered.contains("damnit"), "{rendered}");
    assert!(rendered.contains("dotfiles"), "{rendered}");
}

#[test]
fn read_jump_targets_refuses_a_pick_key_that_is_not_one_printable_character() {
    let (refusal, rendered) = refusal_for(
        "long-key",
        "[workspaces]\nIvy = { dir = \"/opt/ivy\", key = \"vv\" }\n",
    );

    assert_eq!(
        refusal_cause(refusal),
        TargetError::UnusablePickKey {
            label: "Ivy".to_string(),
            key: "vv".to_string(),
        },
        "{rendered}"
    );
}
