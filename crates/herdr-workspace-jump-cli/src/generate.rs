use std::env;
use std::path::PathBuf;

use herdr_workspace_jump_adapters::{config_file, read_jump_targets, write_manifest};

use crate::command::CommandError;

struct Options {
    output: PathBuf,
    config: Option<PathBuf>,
}

/// Render the plugin manifest from the declared workspaces.
pub(crate) fn run(arguments: &[&str]) -> Result<(), CommandError> {
    let options = parse(arguments)?;
    let config = options.config.unwrap_or_else(default_config);
    let targets = read_jump_targets(&config).map_err(failed)?;
    write_manifest(&options.output, env!("CARGO_PKG_VERSION"), &targets).map_err(failed)
}

fn failed(refusal: impl std::fmt::Display) -> CommandError {
    CommandError::Failed(refusal.to_string())
}

fn parse(arguments: &[&str]) -> Result<Options, CommandError> {
    let mut output = None;
    let mut config = None;
    let mut remaining = arguments;
    while let [flag, value, rest @ ..] = remaining {
        let slot = match *flag {
            "--output" => &mut output,
            "--config" => &mut config,
            _ => return Err(CommandError::Usage),
        };
        // A flag in a value's place is a missing value, not a directory named `--config`.
        if slot.is_some() || value.starts_with("--") {
            return Err(CommandError::Usage);
        }
        *slot = Some(PathBuf::from(value));
        remaining = rest;
    }
    if !remaining.is_empty() {
        return Err(CommandError::Usage);
    }
    Ok(Options {
        output: output.ok_or(CommandError::Usage)?,
        config,
    })
}

fn default_config() -> PathBuf {
    config_file(
        env::var("HERDR_PLUGIN_CONFIG_DIR").ok().as_deref(),
        env::var("XDG_CONFIG_HOME").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
    )
}

#[cfg(test)]
mod tests;
