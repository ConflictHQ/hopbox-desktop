# CLAUDE.md — hopbox-desktop

Read [`bootstrap.md`](./bootstrap.md) first. It is the source of truth for what this crate is, how it's structured, and what v0 looks like.

## Quick orientation

- **What this is:** Rust TUI app. Terminal multiplexer + AI copilot. Runs inside an existing terminal — not a standalone emulator.
- **Rendering:** ratatui + crossterm. All UI goes through ratatui, including the embedded terminal pane.
- **AI logic lives in `src/ai/`.** Context building, agent loop, keystroke injection. Don't put AI logic in `src/tui/`.
- **Rendering lives in `src/tui/`.** Panel widgets, layout, the terminal-pane renderer. Don't put rendering in `src/ai/`.
- **`hopbox-core` is the shared library dep.** `AiContext`, `AiConfig`, `ServerConfig`, the provider registry — they all live there. Don't re-implement them here. If a type belongs to the shared domain, add it upstream in `hopbox-core` and bump the dep.

## Tests

PTY tests must run serialized:

```bash
cargo test -- --test-threads=1
```

Parallel PTY tests fight over PTY allocation and you get heisen-failures. Apply the flag project-wide.

## Git identity

- Use `ragelink` / `Leo Mata <lmata@weareconflict.com>` for commits in this repo.
- Set per-repo, never globally.
- This is a Conflict / personal-adjacent repo, not a work (steadymd) repo — don't cross identities.

## License

MIT.
