//! Starting a jump that outlives this process, and waiting for the one that started it.

use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use rustix::io::Errno;
use rustix::process::{Pid, test_kill_process};
use std::os::unix::process::CommandExt;

const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Start a child in its own process group, with no terminal of its own.
///
/// The caller does not wait for it: whoever reaps this process reaps the child.
pub fn spawn_detached(program: &Path, arguments: &[&str]) -> io::Result<()> {
    Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map(|_| ())
}

/// Poll until the process is gone, and proceed anyway once the cap expires.
///
/// A process that outlives the cap is not waited for further: the caller's work
/// matters more than the reason the wait did not finish.
pub fn wait_for_process_exit(pid: u32, cap: Duration) {
    let deadline = Instant::now() + cap;
    while Instant::now() < deadline && process_exists(pid) {
        sleep(POLL_INTERVAL);
    }
}

fn process_exists(pid: u32) -> bool {
    let Some(pid) = i32::try_from(pid).ok().and_then(Pid::from_raw) else {
        return false;
    };
    // A process owned by somebody else exists; it is just not ours to signal.
    !matches!(test_kill_process(pid), Err(Errno::SRCH))
}

#[cfg(test)]
mod tests;
