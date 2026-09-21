//! Where the plugin keeps what it writes between runs.

use std::path::PathBuf;

/// The directory herdr hands the plugin, else the documented path under the home.
fn plugin_state_dir(state_dir: Option<&str>, home: Option<&str>) -> PathBuf {
    let directory = state_dir
        .filter(|directory| !directory.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!(
                "{}/.local/state/herdr/plugins/herdr-workspace-jump",
                home.unwrap_or_default()
            )
        });
    PathBuf::from(directory)
}

/// The two most recently focused workspaces.
pub fn state_file(state_dir: Option<&str>, home: Option<&str>) -> PathBuf {
    plugin_state_dir(state_dir, home).join("mru")
}

/// Where a jump spawned by the pick popup reports what went wrong.
///
/// The popup pane is gone by the time that jump runs, so its stderr has nowhere
/// else to land.
pub fn log_file(state_dir: Option<&str>, home: Option<&str>) -> PathBuf {
    plugin_state_dir(state_dir, home).join("jump.log")
}

#[cfg(test)]
mod tests;
