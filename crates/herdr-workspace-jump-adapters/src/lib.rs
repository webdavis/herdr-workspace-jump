mod cli;
mod config;
mod history;
mod keyboard;
mod manifest;
mod process;
mod response;
mod socket;

pub use cli::CliWorkspaceDirectory;
pub use config::{ConfigError, config_file, read_jump_targets};
pub use history::{FileWorkspaceHistory, state_file};
pub use keyboard::read_one_key;
pub use manifest::{ManifestError, write_manifest};
pub use process::{spawn_detached, wait_for_process_exit};
pub use socket::{DEADLINE, SocketWorkspaceDirectory};

#[cfg(test)]
#[path = "tests/cli_command.rs"]
mod cli_command;
#[cfg(test)]
#[path = "tests/socket_server.rs"]
mod socket_server;
