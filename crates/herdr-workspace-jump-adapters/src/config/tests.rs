use super::*;

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

fn refusal_for(name: &str, content: &str) -> String {
    let sandbox = TempDirectory::new(name);
    let path = sandbox.write(content);
    let refusal = read_jump_targets(&path).expect_err("the config is refused");
    assert!(
        refusal.contains(&path.display().to_string()),
        "the refusal names the file: {refusal}"
    );
    refusal
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
fn read_jump_targets_reads_the_declared_workspaces_ordered_by_label() {
    let sandbox = TempDirectory::new("ordered");
    let path = sandbox.write(
        r#"
[workspaces]
netpulse = "/opt/netpulse"
"casually-concerned" = "~/workspaces/Ivy/casually-concerned"
"#,
    );

    let targets = read_jump_targets(&path).expect("a readable config");

    let labels: Vec<&str> = targets.iter().map(|target| target.label.as_str()).collect();
    assert_eq!(labels, vec!["casually-concerned", "netpulse"]);
    assert_eq!(targets[0].action_id, "jump_casually_concerned");
    assert_eq!(
        targets[0].directory, "~/workspaces/Ivy/casually-concerned",
        "the tilde stays for the jump verb to expand"
    );
}

#[test]
fn read_jump_targets_refuses_a_missing_file() {
    let sandbox = TempDirectory::new("missing");
    let path = sandbox.0.join("config.toml");

    let refusal = read_jump_targets(&path).expect_err("a missing file is refused");

    assert!(refusal.contains(&path.display().to_string()), "{refusal}");
}

#[test]
fn read_jump_targets_refuses_unparseable_toml() {
    refusal_for("unparseable", "[workspaces\nnetpulse = ");
}

#[test]
fn read_jump_targets_refuses_a_file_with_no_workspaces_table() {
    refusal_for("no-table", "[other]\nkey = \"value\"\n");
}

#[test]
fn read_jump_targets_refuses_an_empty_workspaces_table() {
    let refusal = refusal_for("empty-table", "[workspaces]\n");
    assert!(refusal.contains("no workspace"), "{refusal}");
}

#[test]
fn read_jump_targets_refuses_an_empty_label_or_an_empty_directory() {
    let empty_label = refusal_for("empty-label", "[workspaces]\n\"\" = \"/opt/project\"\n");
    assert!(empty_label.contains("label is empty"), "{empty_label}");
    let empty_directory = refusal_for("empty-directory", "[workspaces]\nnetpulse = \"\"\n");
    assert!(
        empty_directory.contains("empty directory"),
        "{empty_directory}"
    );
}

#[test]
fn read_jump_targets_refuses_two_labels_that_derive_the_same_action_id() {
    let refusal = refusal_for(
        "collision",
        "[workspaces]\n\"my-project\" = \"/opt/one\"\n\"my_project\" = \"/opt/two\"\n",
    );

    assert!(refusal.contains("jump_my_project"), "{refusal}");
}

#[test]
fn read_jump_targets_refuses_an_unknown_top_level_key() {
    let refusal = refusal_for(
        "unknown-key",
        "[workspace]\nnetpulse = \"/opt/netpulse\"\n\n[workspaces]\nivy = \"/opt/ivy\"\n",
    );

    assert!(refusal.contains("workspace"), "{refusal}");
}
