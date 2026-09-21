use super::*;

#[test]
fn state_file_prefers_the_injected_directory_over_the_home_default() {
    assert_eq!(
        state_file(Some("/tmp/state-override"), Some("/home/ignored")),
        PathBuf::from("/tmp/state-override/mru")
    );
    assert_eq!(
        state_file(None, Some("/home/me")),
        PathBuf::from("/home/me/.local/state/herdr/plugins/herdr-workspace-jump/mru")
    );
    assert_eq!(
        state_file(Some(""), Some("/home/me")),
        PathBuf::from("/home/me/.local/state/herdr/plugins/herdr-workspace-jump/mru")
    );
}

#[test]
fn the_jump_log_sits_beside_the_history_in_the_same_directory() {
    assert_eq!(
        log_file(Some("/tmp/state-override"), None),
        PathBuf::from("/tmp/state-override/jump.log")
    );
    assert_eq!(
        log_file(None, Some("/home/me")),
        PathBuf::from("/home/me/.local/state/herdr/plugins/herdr-workspace-jump/jump.log")
    );
}
