# cct — Claude Code TUI Launcher

A terminal UI for managing and launching [Claude Code](https://claude.ai/code) with named profiles. Define multiple configurations in a single TOML file, pick one from a TUI, and `cct` exec-replaces itself with `claude` — no wrapper process, clean terminal inheritance.

## Features

- **Profile management** — store model, env vars, and CLI flags per profile
- **TUI selector** — ratatui-based list+detail panel with keyboard navigation
- **Sensitive value masking** — env keys containing `TOKEN`, `KEY`, or `SECRET` are redacted in the UI
- **Hot-reload** — press `e` to open `$EDITOR`, config is re-parsed on return
- **Zero overhead** — `exec()` replaces the process; no parent lingers

## Install

```bash
cargo install --path .
```

Requires Rust 1.70+ and a Unix-like OS (uses `exec`).

## Quick Start

1. Run `cct` once to generate the default config at `~/.config/cc-tui/profiles.toml`.
2. Edit the config to add your profiles:

```toml
[[profiles]]
name = "default"
description = "Default Claude Code"
model = "claude-sonnet-4-6"      # optional
skip_permissions = false          # optional
extra_args = ["--verbose"]        # optional

[profiles.env]
ANTHROPIC_BASE_URL = "https://..."
ANTHROPIC_AUTH_TOKEN = "sk-..."
```

3. Run `cct` again, select a profile, and press Enter.

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `Down` | Next profile |
| `k` / `Up` | Previous profile |
| `Enter` | Launch selected profile |
| `e` | Edit config in `$EDITOR` |
| `q` / `Esc` | Quit |

## Build & Test

```bash
cargo build                    # debug build
cargo build --release          # release build
cargo test                     # all tests
cargo clippy                   # lint
cargo test --test integration  # E2E (mock)
CCT_LIVE_TESTS=1 cargo test --test live  # E2E (live, needs claude binary)
```

## Architecture

Four modules, no shared mutable state:

| Module | Responsibility |
|--------|----------------|
| `config` | TOML deserialization, default config bootstrap |
| `app` | Cursor state and circular navigation |
| `ui` | ratatui rendering, 35/65 split layout, value masking |
| `launch` | CLI arg building, exec-replace, editor open |

See [ARCHITECTURE.md](ARCHITECTURE.md) for details.

## License

MIT
