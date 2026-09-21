use super::*;

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
    let started = Instant::now();

    wait_for_process_exit(a_pid_that_is_gone(), Duration::from_secs(2));

    assert!(
        started.elapsed() < Duration::from_secs(1),
        "the wait did not poll out the cap: {:?}",
        started.elapsed()
    );
}

#[test]
fn the_wait_gives_up_at_the_cap_for_a_process_that_stays() {
    let cap = Duration::from_millis(50);
    let started = Instant::now();

    wait_for_process_exit(std::process::id(), cap);

    let elapsed = started.elapsed();
    assert!(elapsed >= cap, "it returned before the cap: {elapsed:?}");
    assert!(elapsed < Duration::from_secs(2), "it overran: {elapsed:?}");
}

#[test]
fn a_detached_child_runs_in_a_process_group_of_its_own() {
    let sandbox = TempDirectory::new("detached");
    let marker = sandbox.0.join("marker");

    spawn_detached(
        Path::new("/bin/sh"),
        &["-c", &format!("ps -o pgid= -p $$ > {}", marker.display())],
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
    assert!(spawn_detached(Path::new("/nonexistent/binary"), &[]).is_err());
}
