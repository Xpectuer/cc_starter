---
doc_type: architecture
generated_by: mci-phase-1
---

# cct — Architecture Document

<!-- BEGIN:architecture -->

## Project Overview

- **Problem Domain**: `cct` is a terminal UI launcher for Claude Code. It lets users define named "profiles" (different Claude model configs, API keys, extra flags) in a single TOML file and select them via an interactive ratatui TUI, replacing the need to manually construct long `claude ...` command lines.
- **Primary Users**: Individual developers who run Claude Code locally across multiple configurations (e.g., different models, API providers, permission modes).

## Tech Stack

| Item | Value |
|------|-------|
| Language | Rust (edition 2021) |
| TUI Framework | ratatui 0.29 + crossterm 0.28 |
| Config Serialization | serde + toml 0.8 |
| Path Utilities | dirs 5 |
| Error Handling | anyhow 1 |
| Build Tool | Cargo |
| Runtime | Native Unix binary (no runtime, exec-replacement) |
| CI | GitHub Actions (lint + test on push/PR; release on tag) |

## Architecture Pattern

**Flat single-binary CLI with 4 focused modules** — no shared mutable global state. The architecture is closer to a classic Unix filter than to a server application:

```
Config (TOML) → App (cursor state) → UI (ratatui draw loop) → Launch (exec-replace)
```

Each module has no circular dependency; `launch` and `ui` both depend on `config::Profile` but not on each other.

## Core Modules (First-level)

| Module | File | Responsibility |
|--------|------|----------------|
| `config` | `src/config.rs` | Deserialize `profiles.toml` via serde/toml; write default config on first run |
| `app` | `src/app.rs` | Cursor state (`selected` index) and list navigation (`next`/`prev`) |
| `ui` | `src/ui.rs` | ratatui rendering — 35/65 split list+detail panel + footer; masks sensitive env vars |
| `launch` | `src/launch.rs` | Build `claude` CLI args from a profile; `exec()` the process (Unix process replace); open `$EDITOR` |

## Critical Path

### Startup Flow
```
src/main.rs (entry point)
  → config::ensure_default_config()   # create ~/.config/cc-tui/profiles.toml if absent
  → config::load_profiles()           # parse TOML → Vec<Profile>
  → crossterm: enable_raw_mode + EnterAlternateScreen
  → App::new(profiles)                # initialize cursor at index 0
  → loop:
      tui.draw(|f| ui::draw(&app, f)) # render list + detail + footer
      event::read()                    # block on keypress
```

### Main Use Case — Launch Profile
```
User presses [Enter]
  → launch::restore_terminal()        # disable raw mode, LeaveAlternateScreen
  → launch::exec_claude(&profile)
      → env::set_var(k, v) for each profile.env entry
      → launch::build_args(profile)   # --model, --dangerously-skip-permissions, extra_args
      → Command::new("claude").args(...).exec()  # Unix exec — process replaced, no return
```

### Hot-reload Config (key `e`)
```
User presses [e]
  → launch::restore_terminal()
  → launch::open_editor(&config_path())   # blocks until $EDITOR exits
  → crossterm: re-enable raw mode
  → config::load_profiles()              # re-parse TOML in-place
  → update app.profiles + clamp cursor
```

## Configuration-Driven Logic

| Config Source | Effect |
|--------------|--------|
| `~/.config/cc-tui/profiles.toml` (default) | Main profile store; location overridable via `CCT_CONFIG` env var |
| `CCT_CONFIG` env var | Override config file path (used by integration tests) |
| `$EDITOR` env var | Editor opened on `e` key; falls back to `vi` |
| `profiles[].model` | Adds `--model <value>` to `claude` invocation |
| `profiles[].skip_permissions = true` | Adds `--dangerously-skip-permissions` to `claude` invocation |
| `profiles[].extra_args = [...]` | Appended verbatim after other flags |
| `profiles[].env.*` | Injected as process environment variables before exec |
| `CCT_LIVE_TESTS=1` | Enables the live E2E test suite (requires real `claude` binary) |
| `CCT_TEST_TOML` | Integration test: override config path for subprocess exec test |
| `CCT_TEST_ARGS_FILE` | Integration test: fake `claude` stub writes captured args here |

## System Context Diagram

```mermaid
graph TB
    User["Developer"]
    CCT["cct TUI (ratatui terminal)"]
    ProfilesFile["~/.config/cc-tui/profiles.toml (TOML config)"]
    Claude["claude binary (Claude Code CLI)"]
    Editor["$EDITOR (vi / nvim / etc.)"]
    AnthropicAPI["Anthropic API (or custom base URL)"]

    User -->|"navigate / launch / edit"| CCT
    CCT -->|"read on startup + hot-reload"| ProfilesFile
    CCT -->|"exec-replace (Unix exec)"| Claude
    CCT -->|"spawn on e key"| Editor
    Editor -->|"writes"| ProfilesFile
    Claude -->|"HTTPS"| AnthropicAPI
    Claude -->|"inherits env vars (ANTHROPIC_AUTH_TOKEN, etc.)"| AnthropicAPI
```

## Test Infrastructure

| Suite | Location | Description |
|-------|----------|-------------|
| Unit tests | `src/config.rs`, `src/ui.rs`, `src/launch.rs` (inline `#[cfg(test)]`) | Config parsing, arg building, masking |
| Integration (mock) | `tests/integration.rs` | Uses a fake `claude` shell script at `tests/helpers/claude` |
| Integration (live) | `tests/live.rs` | Requires `CCT_LIVE_TESTS=1` and real `claude` binary |

## Key Design Decisions

- **`exec` not `spawn`**: `launch::exec_claude` uses Unix `exec` so `claude` inherits the terminal cleanly; there is no return path on success.
- **`ui::mask_value`**: Redacts any env key containing `TOKEN`, `KEY`, or `SECRET` in the detail panel.
- **Config hot-reload on `e`**: Editor opens, then profiles are re-parsed in-place without process restart.
- **No shared mutable state**: Each module is self-contained; `App` owns `Vec<Profile>` and is the single source of truth for cursor position.

<!-- END:architecture -->
