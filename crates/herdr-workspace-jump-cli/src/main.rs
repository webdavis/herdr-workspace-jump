//! Workspace navigation: `jump <label> <cwd>`, `pick`, which offers the declared
//! workspaces in a popup, `last-workspace`, the `record` event hook, and
//! `generate`, which renders the plugin manifest from the declared workspaces.

mod command;
mod generate;
mod last_workspace;
mod path;
mod pick;
mod run;

use std::env;
use std::process::ExitCode;

use command::{CommandError, USAGE};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match command::execute(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CommandError::Usage) => {
            eprintln!("herdr-workspace-jump: {USAGE}");
            ExitCode::from(2)
        }
        Err(CommandError::Failed(reason)) => {
            eprintln!("herdr-workspace-jump: {reason}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
#[path = "tests/socket_server.rs"]
mod socket_server;

#[cfg(test)]
#[path = "tests/cli_command.rs"]
mod cli_command;
