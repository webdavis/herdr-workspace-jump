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

```bash
herdr plugin install webdavis/herdr-workspace-jump --ref <commit-or-tag> -y
```

herdr clones the repository and runs the manifest's build step (`cargo build --release --locked`),
so a Rust toolchain has to be on the machine. `--ref` is optional but recommended: herdr v1 has no
`plugin update`, so an unpinned install is whatever tip was on the day it ran.

**The actions in `herdr-plugin.toml` are one per workspace, and the ones committed here are mine.**
herdr plugin v1 registers no actions at runtime and a `plugin_action` keybinding passes no arguments,
so each jump target's label and directory are baked into an action's argv. To use your own, fork this
repository, replace the `[[actions]]` blocks with yours, and install from the fork. The
`last_workspace` action and the `record` event hook need no editing.

Then bind the actions in `~/.config/herdr/config.toml`:

```toml
[[keys.command]]
key = "prefix+ctrl+\\"
type = "plugin_action"
command = "herdr-workspace-jump.last_workspace"
```

## License

MIT
