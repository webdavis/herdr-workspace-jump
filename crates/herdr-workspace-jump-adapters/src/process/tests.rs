use std::sync::mpsc;
use std::thread;

use super::*;

/// How long a test waits for the wait itself, which is longer than any cap a
/// test passes and short enough to fail rather than hang the run.
const BOUND: Duration = Duration::from_secs(5);

/// Run the wait off the test thread, so one that never gives up fails here
/// instead of parking the suite until an external timeout kills it.
fn elapsed_waiting(pid: u32, cap: Duration) -> Duration {
    let (finished, waited) = mpsc::channel();
    thread::spawn(move || {
        let started = Instant::now();
        wait_for_process_exit(pid, cap);
        let _ = finished.send(started.elapsed());
    });
    waited
        .recv_timeout(BOUND)
        .expect("the wait never returned, so its cap is gone")
}

/// A pid that has run and been reaped, so nothing holds it.
fn a_pid_that_is_gone() -> u32 {
    let mut finished = Command::new("true").spawn().expect("a child that exits");
    let pid = finished.id();
    finished.wait().expect("the child is reaped");
    pid
}

struct TempDirectory(std::path::PathBuf);

impl TempDirectory {
    fn new(name: &str) -> Self {
        let directory =
            std::env::temp_dir().join(format!("hwj-process-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("a sandbox directory");
        Self(directory)
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_wait_ends_at_once_for_a_process_that_is_already_gone() {
    let elapsed = elapsed_waiting(a_pid_that_is_gone(), Duration::from_secs(2));

    assert!(
        elapsed < Duration::from_secs(1),
        "the wait polled out the cap instead: {elapsed:?}"
    );
}

#[test]
fn the_wait_gives_up_at_the_cap_for_a_process_that_stays() {
    let cap = Duration::from_millis(50);

    let elapsed = elapsed_waiting(std::process::id(), cap);

    assert!(elapsed >= cap, "it returned before the cap: {elapsed:?}");
    assert!(elapsed < BOUND, "it overran its cap: {elapsed:?}");
}

#[test]
fn a_detached_child_runs_in_a_process_group_of_its_own() {
    let sandbox = TempDirectory::new("detached");
    let marker = sandbox.0.join("marker");

    spawn_detached(
        Path::new("/bin/sh"),
        &["-c", &format!("ps -o pgid= -p $$ > {}", marker.display())],
        None,
    )
    .expect("the child starts");

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut child_group = String::new();
    while Instant::now() < deadline && child_group.trim().is_empty() {
        sleep(POLL_INTERVAL);
        child_group = std::fs::read_to_string(&marker).unwrap_or_default();
    }
    assert_ne!(
        child_group
            .trim()
            .parse::<i32>()
            .expect("the detached child never reported its process group"),
        rustix::process::getpgrp().as_raw_nonzero().get(),
        "the child shares this process group, so whoever tears the popup down takes it too"
    );
}

#[test]
fn a_program_that_cannot_be_started_is_reported() {
    assert!(spawn_detached(Path::new("/nonexistent/binary"), &[], None).is_err());
}

#[test]
fn what_a_detached_child_writes_to_stderr_is_appended_to_the_log() {
    let sandbox = TempDirectory::new("errors");
    // The directory the log names does not exist yet, the way a first run finds it.
    let log = sandbox.0.join("state").join("jump.log");

    for refusal in ["could not jump to Ivy: herdr unreachable", "and again"] {
        spawn_detached(
            Path::new("/bin/sh"),
            &["-c", &format!("echo '{refusal}' >&2")],
            Some(&log),
        )
        .expect("the child starts");
    }

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut written = String::new();
    while Instant::now() < deadline && written.lines().count() < 2 {
        sleep(POLL_INTERVAL);
        written = std::fs::read_to_string(&log).unwrap_or_default();
    }
    assert!(
        written.contains("could not jump to Ivy: herdr unreachable"),
        "the first refusal is missing: {written:?}"
    );
    assert!(
        written.contains("and again"),
        "the log was truncated rather than appended to: {written:?}"
    );
}
