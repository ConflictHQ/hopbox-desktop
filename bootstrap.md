# hopbox-desktop — bootstrap

## What this is

`hopbox-desktop` is the Rust TUI terminal multiplexer and AI copilot. It is the client surface humans actually interact with. It runs *inside* an existing terminal (iTerm2, Alacritty, Ghostty, gnome-terminal, the macOS default, whatever) — it is **not** a standalone terminal emulator. Owning the emulator is a future milestone; for now we slot in alongside one.

The binary name is `hopbox`. The crate name is `hopbox-desktop`.

The job:

1. Own the PTY for the user's shell.
2. Render the PTY output via ratatui so we have something we can layout-control (panes, splits, the AI sidebar).
3. Continuously read the terminal scroll buffer so we always have *context* — what the user just ran, what came back, what's on screen right now.
4. On a trigger keybind, build an `AiContext` from that buffer + session metadata, hand it to an AI provider (resolved via `hopbox-core`), and stream the response into the AI panel.
5. If the response is a runnable command and the user accepts, inject it into the PTY as keystrokes.
6. Do all of that *transparently across SSH sessions* — SSH is just a subprocess in the managed PTY, so we still see everything on the local terminal buffer regardless of where the shell actually lives.

## v0 scope

**Standalone mode only.** No connection to `hopbox-server` yet. The desktop client runs entirely locally:

- Local PTY session via `session::local`.
- AI provider called directly from the client (API key in config or env).
- No team sharing, no remote sessions, no auth.

The `session::remote` module exists as a stub so we don't have to reshape the codebase later, but it is not wired up. The `server` block in the config is optional and unused in v0.

This means a v0 user can: `cargo install hopbox-desktop`, write a config with one provider's API key, run `hopbox`, get a TUI shell with a working AI copilot. That's the bar.

## The AI copilot concept

The whole point of owning the PTY is that we get to see *both sides* of the terminal conversation — what the user typed and what the shell printed back. That stream is the substrate for the copilot.

The trigger flow, in detail:

1. **User presses `Ctrl+\`** (configurable in `keybindings.ai_trigger`).
2. **Context capture.** `ai::context` snapshots:
   - The last N lines of the scroll buffer (visible + recent scrollback).
   - The current command line buffer (what the user has typed but not yet submitted).
   - Cursor position.
   - Session metadata: shell ($SHELL), cwd (from PTY title escape or `OSC 7` if available), and whether we're currently in an SSH subprocess (heuristically — if a `ssh` was the last foregrounded process we know about, mark the session as remote so the AI knows the cwd it's seeing is on the remote host).
   - The user's pending question, if any (a prompt input modal appears on trigger; empty input means "look at what's on screen and suggest something").
3. **Provider dispatch.** `ai::agent` takes the `AiContext`, resolves the configured provider via `hopbox-core`'s provider registry, and starts a streaming request.
4. **Render.** The AI panel (`tui::ai_panel`) shows the streamed response token-by-token. The user's PTY pane remains live and interactive while this happens — the AI panel is a sidebar/overlay, not a modal.
5. **Accept / reject.** If the response contains a fenced command block, the panel highlights it and binds `Enter` to "inject into PTY" and `Esc` to "dismiss." Multi-command responses get a numbered chooser.
6. **Injection.** `ai::injector` writes the chosen command bytes to the PTY master. Crucially, the command is *not auto-executed* — we inject the keystrokes up to but not including the newline, so the user can edit the suggested command before pressing Enter themselves. There is no "auto-run" toggle in v0; this is a safety floor we revisit when we have an audit log.

The AI never executes commands directly. It only ever proposes keystrokes that the user explicitly accepts. This is the hard rule.

### SSH transparency

We don't do anything special to handle SSH. Because SSH is just a subprocess running in the PTY we own, its output flows through our rendering layer like any other shell output. The terminal buffer we read from is *always* the local-perspective buffer, which is exactly what the AI needs — it sees what the user sees.

The one nuance is that paths and process names in the context refer to the remote host when SSH'd in. The context builder marks the session as "likely-remote" when it detects a foregrounded `ssh` subprocess (by parsing the PTY title or by tracking child process start/exit events). The AI provider receives this flag so it can avoid making suggestions that assume local-filesystem access (e.g. "open the file in your editor" doesn't necessarily work).

This also means: the AI copilot works through SSH without any cooperation from the remote host. No agent install on the server, no protocol on the wire — the remote box just runs a shell, same as any SSH session. Everything happens client-side.

## Crate structure

This is a **single binary crate**, not a Cargo workspace. `hopbox-core` is pulled in as a git dependency — it is *not* part of this workspace. The shared types (`AiContext`, provider registry, config schema) live there.

```
hopbox-desktop/
├── Cargo.toml
├── Cargo.lock        — committed (this is a binary)
├── bootstrap.md
├── CLAUDE.md
├── AGENTS.md
├── .gitignore
└── src/
    ├── main.rs       — entry point, tokio runtime, config load, hands off to App
    ├── app.rs        — App state machine, top-level event loop
    ├── config.rs     — config loading from ~/.config/hopbox/config.toml
    ├── tui/
    │   ├── mod.rs
    │   ├── layout.rs     — pane layout management (splits, sidebar placement)
    │   ├── terminal.rs   — PTY output → ratatui rendering (the terminal pane widget)
    │   └── ai_panel.rs   — AI copilot overlay/sidebar widget
    ├── session/
    │   ├── mod.rs
    │   ├── local.rs      — local PTY session (spawn shell, manage PTY)
    │   └── remote.rs     — remote session (connects to hopbox-server; STUB in v0)
    └── ai/
        ├── mod.rs
        ├── agent.rs      — copilot agent loop (drives the trigger → context → provider → render flow)
        ├── context.rs    — terminal buffer → AiContext builder
        └── injector.rs   — keystroke injection into the PTY master
```

## Key dependencies

- **ratatui 0.29** — TUI rendering. We render *everything* through ratatui, including the embedded terminal pane.
- **crossterm 0.28** — terminal backend for ratatui, also our source for keyboard/mouse events.
- **tokio 1 (full)** — async runtime. PTY I/O, provider streaming, and TUI event loop all share it.
- **tokio-tungstenite 0.24** with `rustls-tls-webpki-roots` — websocket client for the future server connection. Vendored now so we don't churn deps later.
- **rustls 0.23** — TLS. We do not link OpenSSL.
- **hopbox-core (git)** — shared types and provider registry. `AiContext`, `ServerConfig`, `AiConfig` come from here. Don't redefine them.
- **serde + toml** — config file.
- **tracing + tracing-subscriber** — logs to stderr by default, controllable via `RUST_LOG`.
- **anyhow** — top-level error type. Library-style modules should still use real error types; `anyhow` is for the binary edges.
- **dirs 5** — XDG config directory resolution.

## Build / run / test

```bash
# debug build
cargo build

# release build (ship this)
cargo build --release

# run
cargo run
# or after install:
hopbox

# tests — PTY tests must serialize
cargo test -- --test-threads=1
```

`--test-threads=1` is non-negotiable for PTY tests. Two PTY tests running concurrently will fight over `/dev/ptmx` slots on Linux and TTY allocation on macOS, and we get phantom failures that don't reproduce locally. Apply it project-wide rather than per-module so nobody forgets.

## Config

Lives at `~/.config/hopbox/config.toml` (XDG; falls back to platform-specific via `dirs::config_dir`). If the file doesn't exist, defaults are used and the app still runs — useful for first-launch.

Example:

```toml
[ai]
provider = "anthropic"
model = "claude-sonnet-4-5"
api_key_env = "ANTHROPIC_API_KEY"

[keybindings]
ai_trigger = "ctrl+\\"
quit = "ctrl+q"

# [server] is optional, unused in v0
# [server]
# url = "wss://hopbox.example.com"
# token_env = "HOPBOX_TOKEN"
```

`AiConfig` and `ServerConfig` are imported from `hopbox-core` — see that crate for the full schema.

## Default keybindings

- `Ctrl+\` — trigger AI copilot (opens the prompt input, or refreshes context if panel is already open)
- `Ctrl+Q` — quit
- Any other unbound key — passed through to the PTY (this is the default behavior; we only intercept registered binds)

Keybinds parsed via crossterm's `KeyEvent` model — `ctrl+<char>`, `alt+<char>`, `ctrl+shift+<char>`, etc. Parsing lives in `app.rs` so the config layer stays string-typed.

## How to add a new AI provider

You don't add it here — providers live in `hopbox-core`. The desktop client only resolves them by name via the registry.

1. In `hopbox-core`: add a new module under `providers/`, implement the provider trait, register it in the registry's `init()`.
2. In `hopbox-desktop`: bump the `hopbox-core` git dep to the commit that has the new provider.
3. Users select it via `ai.provider = "your-provider-name"` in their config.

If a provider needs new config fields, add them to `AiConfig` in `hopbox-core` (preserving backward compat — `serde(default)` on new fields). Do not extend `AiConfig` from the desktop crate.

## License

MIT.
