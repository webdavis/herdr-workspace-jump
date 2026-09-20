# herdr-workspace-jump

Two pieces of workspace navigation [herdr](https://herdr.dev) has no built-in for:

- **Create-or-focus by label.** `herdr workspace create` is not idempotent and `herdr workspace
  focus` takes an id rather than a label, so a quick-jump chord needs to resolve the label against
  the live workspace list first and create the workspace only when no match comes back.
- **A most-recently-used toggle.** herdr ships `last_pane` and no workspace equivalent. The
  `workspace.focused` event hook records every focus change, mouse and picker included, which is
  what makes the toggle correct where a key-bound script was not: a script only sees the switches
  routed through itself.

It talks to herdr over `HERDR_SOCKET_PATH` with newline-delimited JSON, and falls back to the
`herdr` CLI at `HERDR_BIN_PATH` whenever the socket fails.

## Install

**This plugin ships no `herdr-plugin.toml`, so `herdr plugin install` cannot install it.** The jump
actions are one per workspace you declare, so the manifest is a function of your configuration
rather than a file that can be committed once for everybody. It is built, generated and linked
instead. A Rust toolchain has to be on the machine.

```bash
git clone https://github.com/webdavis/herdr-workspace-jump.git ~/.local/share/herdr-workspace-jump
cd ~/.local/share/herdr-workspace-jump
cargo build --release --locked
```

Declare your workspaces in `~/.config/herdr/plugins/config/herdr-workspace-jump/config.toml`. The
key is the label herdr shows, which is case sensitive, and the value is the working directory:

```toml
[workspaces]
homelab = "~/workspaces/homelab"
"casually-concerned" = "~/workspaces/casually-concerned"
Ivy = "~/workspaces/Ivy"
```

A leading `~` is expanded when the jump runs, so it can stay in the file.

Then render the manifest and link the directory:

```bash
./target/release/herdr-workspace-jump generate --output .
herdr plugin link ~/.local/share/herdr-workspace-jump
```

Re-run `generate`, and re-link, whenever the config changes. The manifest is build output rather
than source, and is deliberately not committed.

`generate` reads the config path above unless you point it elsewhere: `--config <file>` names one
file, and `HERDR_PLUGIN_CONFIG_DIR`, which herdr itself sets when it runs a plugin, names the
directory the `config.toml` sits in.

Each workspace gets the action id `jump_` plus its label, lowercased with every character outside
`a-z0-9` replaced by an underscore, so `Ivy` is `jump_ivy` and `casually-concerned` is
`jump_casually_concerned`. Two labels deriving the same id is refused rather than dropping one
silently. The `last_workspace` action and the `record` event hook are always rendered and need no
configuration.

Then bind the actions in `~/.config/herdr/config.toml`:

```toml
[[keys.command]]
key = "prefix+ctrl+\\"
type = "plugin_action"
command = "herdr-workspace-jump.last_workspace"
```

## License

MIT
