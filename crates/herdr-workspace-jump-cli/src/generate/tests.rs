use super::*;
use std::fs;

struct TempDirectory(PathBuf);

impl TempDirectory {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-generate-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }

    fn write(&self, name: &str, content: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, content).expect("a sandbox file");
        path
    }

    fn manifest(&self) -> PathBuf {
        self.0.join("herdr-plugin.toml")
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const ONE_WORKSPACE: &str = "[workspaces]\nnetpulse = \"/opt/netpulse\"\n";

#[test]
fn generate_writes_the_manifest_and_a_second_run_replaces_it() {
    let sandbox = TempDirectory::new("writes");
    let config = sandbox.write("config.toml", ONE_WORKSPACE);
    let output = sandbox.0.to_string_lossy().to_string();
    let config_argument = config.to_string_lossy().to_string();

    let first = run(&["--output", &output, "--config", &config_argument]);

    assert!(first.is_ok(), "the first run writes the manifest");
    let written = fs::read_to_string(sandbox.manifest()).expect("a manifest");
    assert!(written.contains("id = \"jump_netpulse\""), "{written}");

    sandbox.write("config.toml", "[workspaces]\nivy = \"/opt/ivy\"\n");
    let second = run(&["--config", &config_argument, "--output", &output]);

    assert!(second.is_ok(), "the second run replaces the manifest");
    let replaced = fs::read_to_string(sandbox.manifest()).expect("a manifest");
    assert!(replaced.contains("id = \"jump_ivy\""), "{replaced}");
    assert!(!replaced.contains("jump_netpulse"), "{replaced}");
}

#[test]
fn generate_reports_a_config_it_cannot_read() {
    let sandbox = TempDirectory::new("unreadable");
    let absent = sandbox.0.join("absent.toml");

    let refusal = run(&[
        "--output",
        &sandbox.0.to_string_lossy(),
        "--config",
        &absent.to_string_lossy(),
    ]);

    match refusal {
        Err(CommandError::Failed(reason)) => {
            assert!(reason.contains(&absent.display().to_string()), "{reason}")
        }
        other => panic!("expected a failure naming the config, got {other:?}"),
    }
    assert!(!sandbox.manifest().exists(), "no manifest is left behind");
}

#[test]
fn generate_refuses_a_missing_output_an_unknown_flag_and_a_flag_without_a_value() {
    for arguments in [
        vec![],
        vec!["--config", "/tmp/config.toml"],
        vec!["--output", "/tmp/plugin", "--unknown", "value"],
        vec!["--output"],
        vec!["--output", "/tmp/plugin", "--config"],
        vec!["--output", "/tmp/one", "--output", "/tmp/two"],
    ] {
        assert!(
            matches!(parse(&arguments), Err(CommandError::Usage)),
            "{arguments:?}"
        );
    }
}
