use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process;
use std::time::Duration;

use herdr_workspace_jump_adapters::{
    read_jump_targets, read_one_key, spawn_detached, wait_for_process_exit,
};
use herdr_workspace_jump_domain::{JumpTarget, Pick, decide_pick, render_menu};

use crate::command::{CommandError, config_path, failed, log_path};

/// How long a jump waits for the popup that spawned it to close.
const SPAWNER_EXIT_CAP: Duration = Duration::from_secs(2);

/// The flag carrying the popup's own pid to the jump it spawns.
pub(crate) const AFTER_PID: &str = "--after-pid";

/// Offer the declared workspaces, read one key, and hand the jump on.
pub(crate) fn run() -> Result<(), CommandError> {
    let targets = read_jump_targets(&config_path()).map_err(failed)?;
    print!("{}", render_menu(&targets));
    io::stdout()
        .flush()
        .map_err(|failure| failed(format!("cannot print the menu: {failure}")))?;
    let key = read_one_key().map_err(|failure| failed(format!("cannot read a key: {failure}")))?;
    match decide_pick(&targets, key) {
        Pick::Selected(target) => spawn_jump(target, |binary, arguments| {
            spawn_detached(binary, arguments, Some(&log_path()))
        }),
        Pick::Cancelled | Pick::Unbound => Ok(()),
    }
}

/// Hand the jump to a child that outlives this popup.
///
/// herdr may restore focus to the pane that opened the popup once the popup's
/// command exits, which would undo a jump issued from inside it. The child
/// waits for this process to go before it asks herdr for anything, and reports
/// a refusal to the log, since the popup that would have shown it is gone.
fn spawn_jump(
    target: &JumpTarget,
    spawn: impl FnOnce(&Path, &[&str]) -> io::Result<()>,
) -> Result<(), CommandError> {
    let binary = env::current_exe()
        .map_err(|failure| failed(format!("cannot find this binary: {failure}")))?;
    let popup = process::id().to_string();
    spawn(&binary, &jump_arguments(target, &popup))
        .map_err(|failure| failed(format!("cannot start the jump: {failure}")))
}

/// The argv the popup hands its jump.
///
/// `--after-pid` carries the popup's own pid, which is what holds the child
/// back until herdr can no longer restore focus to the pane the popup covered.
fn jump_arguments<'a>(target: &'a JumpTarget, popup: &'a str) -> [&'a str; 5] {
    ["jump", &target.label, &target.directory, AFTER_PID, popup]
}

/// Wait for the popup that spawned this jump to close.
pub(crate) fn wait_for_spawner(pid: u32) {
    wait_for_process_exit(pid, SPAWNER_EXIT_CAP);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> JumpTarget {
        JumpTarget {
            action_id: "jump_ivy".to_string(),
            label: "Ivy".to_string(),
            directory: "~/workspaces/Ivy".to_string(),
            pick_key: 'i',
        }
    }

    #[test]
    fn the_spawned_jump_is_this_binary_with_the_label_the_directory_and_this_pid() {
        let mut spawned = Vec::new();
        let this_binary = env::current_exe().expect("a running binary has a path");

        let started = spawn_jump(&target(), |binary, arguments| {
            assert_eq!(binary, this_binary, "the popup spawned something else");
            spawned = arguments.iter().map(|word| word.to_string()).collect();
            Ok(())
        });

        assert!(started.is_ok(), "{started:?}");
        assert_eq!(
            spawned,
            vec![
                "jump",
                "Ivy",
                "~/workspaces/Ivy",
                "--after-pid",
                &process::id().to_string()
            ]
        );
    }

    #[test]
    fn a_spawn_that_fails_is_reported_rather_than_swallowed() {
        let refused = spawn_jump(&target(), |_, _| Err(io::Error::other("no such binary")));

        assert!(
            matches!(&refused, Err(CommandError::Failed(reason)) if reason.contains("no such binary")),
            "{refused:?}"
        );
    }
}
