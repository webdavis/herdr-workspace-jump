use super::*;
use herdr_workspace_jump_application::WorkspaceHistory;
use herdr_workspace_jump_domain::Mru;
use std::fs;
fn mru(current: &str, previous: &str) -> Mru {
    Mru {
        current: current.to_string(),
        previous: previous.to_string(),
    }
}

struct TempState(PathBuf);

impl TempState {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("hwj-mru-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        Self(dir)
    }

    fn path(&self) -> PathBuf {
        self.0.join("mru")
    }
}

impl Drop for TempState {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn record_writes_the_shift_that_next_mru_decided() {
    let state = TempState::new("record");
    record_at(&state.path(), r#"{"data":{"workspace_id":"wA"}}"#);
    record_at(&state.path(), r#"{"data":{"workspace_id":"wB"}}"#);
    assert_eq!(
        FileWorkspaceHistory::at(&state.path()).read(),
        mru("wB", "wA")
    );
}

#[test]
fn record_leaves_the_state_alone_for_a_refocus_or_a_garbage_event() {
    let state = TempState::new("record-noop");
    record_at(&state.path(), r#"{"data":{"workspace_id":"wA"}}"#);
    record_at(&state.path(), r#"{"data":{"workspace_id":"wA"}}"#);
    record_at(&state.path(), "not json");
    assert_eq!(
        FileWorkspaceHistory::at(&state.path()).read(),
        mru("wA", "")
    );
}

#[test]
fn generate_without_an_output_directory_is_a_usage_error() {
    let refusal = execute(&["generate".to_string()]);

    assert!(matches!(refusal, Err(CommandError::Usage)), "{refusal:?}");
}

/// These verbs reach herdr, so every case below is rejected before anything is sent.
fn refusal(words: &[&str]) -> Result<(), CommandError> {
    execute(
        &words
            .iter()
            .map(|word| word.to_string())
            .collect::<Vec<_>>(),
    )
}

#[test]
fn a_jump_whose_after_pid_is_not_a_number_is_a_usage_error() {
    for rejected in ["abc", "", "-1", "12.5"] {
        let refused = refusal(&["jump", "Ivy", "/opt/ivy", "--after-pid", rejected]);
        assert!(matches!(refused, Err(CommandError::Usage)), "{rejected:?}");
    }
}

#[test]
fn a_jump_with_an_unknown_trailing_flag_or_a_missing_pid_is_a_usage_error() {
    for rejected in [
        vec!["jump", "Ivy", "/opt/ivy", "--after", "1"],
        vec!["jump", "Ivy", "/opt/ivy", "--after-pid"],
        vec!["jump", "Ivy", "/opt/ivy", "--after-pid", "1", "extra"],
    ] {
        let refused = refusal(&rejected);
        assert!(matches!(refused, Err(CommandError::Usage)), "{rejected:?}");
    }
}

#[test]
fn pick_takes_no_arguments() {
    assert!(matches!(
        refusal(&["pick", "Ivy"]),
        Err(CommandError::Usage)
    ));
}

#[test]
fn the_usage_line_names_every_verb() {
    for verb in ["jump", "pick", "last-workspace", "record", "generate"] {
        assert!(USAGE.contains(verb), "the usage line omits {verb}: {USAGE}");
    }
}
