# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`cct` is a terminal UI launcher for Claude Code. It reads named profiles from a TOML config file (`~/.config/cc-tui/profiles.toml`), displays them in a ratatui TUI, and exec-replaces the process with `claude <args>` when the user selects one.

## Build & Test Commands

```bash
cargo build           # debug build
cargo build --release # release build
cargo test            # run all tests
cargo test <name>     # run a single test by name (e.g. cargo test build_args_full)
cargo clippy          # lint
cargo run             # run the TUI locally

# E2E (mock — no real claude needed)
cargo test --test integration

# E2E (live — requires `claude` binary installed)
CCT_LIVE_TESTS=1 cargo test --test live
```

## Architecture

The app is four focused modules with no shared mutable state:

| Module | File | Responsibility |
|--------|------|----------------|
| `config` | `src/config.rs` | Deserialize `profiles.toml` via serde/toml; write default config on first run |
| `app` | `src/app.rs` | Cursor state (`selected` index) and navigation (`next`/`prev`) |
| `ui` | `src/ui.rs` | ratatui rendering — 35/65 split list+detail panel + footer; masks sensitive env vars |
| `launch` | `src/launch.rs` | Build `claude` CLI args from a profile; `exec()` the process (Unix replace); open `$EDITOR` |

**Data flow:** `main` loads profiles → creates `App` → draw loop → on Enter calls `launch::exec_claude` which injects env vars and exec-replaces via `std::os::unix::process::CommandExt::exec`.

**Key design choices:**
- `exec` (not `spawn`) is used so `claude` inherits the terminal cleanly; there is no return path on success.
- `ui::mask_value` redacts any env key containing `TOKEN`, `KEY`, or `SECRET`.
- Config hot-reload on `e`: editor opens, then profiles are re-parsed in-place without restart.

## Config File Format

Located at `~/.config/cc-tui/profiles.toml`. Each profile block:

```toml
[[profiles]]
name = "default"
description = "Default Claude Code"
model = "claude-sonnet-4-6"      # optional — maps to --model
skip_permissions = false          # optional — adds --dangerously-skip-permissions
extra_args = ["--verbose"]        # optional — appended verbatim

[profiles.env]
ANTHROPIC_BASE_URL = "https://..."
ANTHROPIC_AUTH_TOKEN = "sk-..."
```

## Docs Directory

| Directory | Purpose |
|-----------|---------|
| `docs/drafts/` | Design phase — intake, idea, interview, plan sessions |
| `docs/procs/` | Execution phase — tdd/progress tracking (activated work) |
| `docs/lessons/` | Lessons learned |
| `docs/rules/` | Coding rules and standards |
| `docs/sops/` | Standard operating procedures |
| `docs/issues/` | Issue tracking and bug reports |
| `docs/modules/` | Module/component documentation |
| `docs/references/` | Reference documents and external resources |
| `docs/quality/` | Quality reviews and audits |
