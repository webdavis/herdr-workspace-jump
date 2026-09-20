use super::*;
use std::fs;

struct TempDirectory(PathBuf);

impl TempDirectory {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-manifest-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }

    fn manifest(&self) -> PathBuf {
        self.0.join(MANIFEST_NAME)
    }

    fn entries(&self) -> Vec<PathBuf> {
        fs::read_dir(&self.0)
            .expect("a readable sandbox")
            .map(|entry| entry.expect("a readable entry").path())
            .collect()
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn one_target() -> Vec<JumpTarget> {
    vec![JumpTarget {
        action_id: "jump_netpulse".to_string(),
        label: "netpulse".to_string(),
        directory: "/opt/netpulse".to_string(),
    }]
}

#[test]
fn write_manifest_writes_the_plugin_manifest_and_replaces_an_existing_one() {
    let sandbox = TempDirectory::new("write");
    fs::write(sandbox.manifest(), "stale").expect("a stale manifest");

    write_manifest(&sandbox.0, "1.2.3", &one_target()).expect("the manifest is written");

    let written = fs::read_to_string(sandbox.manifest()).expect("a manifest");
    assert!(written.contains("id = \"jump_netpulse\""), "{written}");
    assert!(!written.contains("stale"), "{written}");
    assert_eq!(
        sandbox.entries(),
        vec![sandbox.manifest()],
        "no temporary file survives a successful write"
    );
}

#[test]
fn write_manifest_reports_a_directory_it_cannot_write() {
    let sandbox = TempDirectory::new("unwritable");
    let absent = sandbox.0.join("no-such-directory");

    let refusal = write_manifest(&absent, "1.2.3", &one_target())
        .expect_err("a missing directory is refused");

    match &refusal {
        ManifestError::Unwritable { path, .. } => {
            assert_eq!(path, &absent.join(MANIFEST_NAME));
        }
        other => panic!("expected Unwritable, got {other:?}"),
    }
    assert!(
        refusal.to_string().contains(&absent.display().to_string()),
        "{refusal}"
    );
}

#[test]
fn write_manifest_removes_its_temporary_file_when_the_rename_fails() {
    // A directory standing in the manifest's place fails the rename after the
    // temporary file has been written, which is the path that leaks one.
    let sandbox = TempDirectory::new("failed-rename");
    fs::create_dir_all(sandbox.manifest()).expect("a blocking directory");

    let refusal = write_manifest(&sandbox.0, "1.2.3", &one_target())
        .expect_err("a directory in the manifest's place is refused");

    assert!(
        matches!(refusal, ManifestError::Unwritable { .. }),
        "{refusal}"
    );
    assert_eq!(
        sandbox.entries(),
        vec![sandbox.manifest()],
        "the temporary file is removed after the failed rename"
    );
}
