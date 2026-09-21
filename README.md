# herdr-workspace-jump

Two pieces of workspace navigation [herdr](https://herdr.dev) has no built-in for:

- **Create-or-focus by label.** `herdr workspace create` is not idempotent and `herdr workspace
  focus` takes an id rather than a label, so a quick-jump chord needs to resolve the label against
  the live workspace list first and create the workspace only when no match comes back.
- **A most-recently-used toggle.** herdr ships `last_pane` and no workspace equivalent. The
  `workspace.focused` event hook records every focus change, mouse and picker included, which is
  what makes the toggle correct where a key-bound script was not: a script only sees the switches
  routed through itself.
- **A one-letter pick popup.** A herdr binding is a single chord, so "prefix, then a letter" has
  to read its second key somewhere. `pick` reads it inside a herdr popup and jumps to whichever
  workspace that letter names.

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
key is the label herdr shows, which is case sensitive, and the value is either the working
directory on its own or a table naming that directory and the key the pick popup selects it with:

```toml
[workspaces]
homelab = "~/workspaces/homelab"
"casually-concerned" = "~/workspaces/casually-concerned"
Ivy = { dir = "~/workspaces/Ivy", key = "v" }
```

A leading `~` is expanded when the jump runs, so it can stay in the file.

`key` defaults to the label's first character lowercased, which is why `homelab` picks on `h` and
`Ivy` would pick on `i` without the `v` above. It has to be exactly one printable ASCII character,
and two workspaces resolving to the same key are refused when the config is read, in one sentence
naming both labels. `generate` reads the same file, so that refusal fails a manifest render as much
as a popup: a new workspace whose key is already taken means giving one of the two its own `key`.

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

## The pick popup

`pick` prints one line per workspace, `<key>  <label>`, reads a single keystroke, and jumps to the
workspace that keystroke names. `esc` and ctrl-c always close it without jumping, and so does `q`
unless a workspace claims that key, and so does any key no workspace claims at all. Bind it as a
popup rather than a plugin action:

```toml
[[keys.command]]
key = "prefix+ctrl+o"
type = "popup"
command = "~/.local/share/herdr-workspace-jump/target/release/herdr-workspace-jump pick"
description = "pick a workspace"
width = 40
height = 13
```

herdr runs a custom command keybinding through a shell, so the leading `~` above is expanded and an
absolute path is not required. `width` and `height` are terminal cells and include the border, so
thirteen rows hold ten workspaces, the cancel line and the border itself.

`pick` is a popup command rather than a plugin action, so `generate` does not render it into the
manifest and nothing needs regenerating when the binding changes. It reads the same config file the
manifest comes from and has no `--config` flag; `HERDR_PLUGIN_CONFIG_DIR` is what points it
elsewhere.

The jump does not happen inside the popup. herdr may restore focus to the pane that opened the
popup once the popup's command exits, which would undo it, so `pick` spawns a detached
`jump <label> <cwd> --after-pid <pid>` and exits at once. That child waits for the popup to be gone
before it asks herdr for anything, and gives up waiting after two seconds.

## License

MIT
