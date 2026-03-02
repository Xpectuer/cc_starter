---
title: "Spec: cct — Claude Code TUI Launcher"
doc_type: proc
brief: "Flat-module Rust/ratatui TUI with list+detail layout, profile exec, editor suspend/resume"
confidence: verified
created: 2026-03-02
updated: 2026-03-02
revision: 1
source_skill: idea
---

# Spec: cct — Claude Code TUI Launcher

## Chosen Approach

**Architecture A — Flat modules**, no abstraction layers:

```
src/
├── main.rs      # terminal lifecycle + event loop
├── app.rs       # App struct (state)
├── config.rs    # Profile struct, TOML parse, first-run default
├── ui.rs        # ratatui rendering (draw fn + mask_value)
└── launch.rs    # terminal teardown + exec claude / spawn $EDITOR
```

## Key Design Decisions

### 1. Terminal Lifecycle

`main.rs` owns terminal init (`enable_raw_mode`, `EnterAlternateScreen`).
`launch::restore_terminal()` owns teardown (`disable_raw_mode`, `LeaveAlternateScreen`).
This function is called before both `exec` (process replace) and `$EDITOR` spawn.

For `$EDITOR` (suspend/resume): after editor exits, re-call `enable_raw_mode` +
`execute!(EnterAlternateScreen)` + `tui.clear()`. The existing
`Terminal<CrosstermBackend>` object is reused — no need to recreate it.

**Critical**: `exec()` never returns on success, so Drop destructors don't run.
`restore_terminal()` must be called explicitly before `exec_claude()`.

### 2. Layout

```
┌───────────────────────────────────────────────────────┐
│ ┌─── Profiles ──────┐ ┌─── Details ──────────────────┐│
│ │ > kclaude         │ │ Kimi AI via OpenAI-compat    ││
│ │   minimax         │ │ ──────────────────────────── ││
│ │   deepseek        │ │ model: kimi-k1.5             ││
│ │                   │ │ ENV:                         ││
│ │                   │ │   BASE_URL = https://api.k…  ││
│ │                   │ │   AUTH_TOK = ***             ││
│ └───────────────────┘ └──────────────────────────────┘│
│  [↑↓/jk] Navigate  [Enter] Launch  [e] Edit  [q] Quit │
└───────────────────────────────────────────────────────┘
```

Three-zone layout: `Layout::vertical` with `Constraint::Min(1)` for content +
`Constraint::Length(1)` for footer. Content uses `Layout::horizontal` with
`Percentage(35)` / `Percentage(65)`.

### 3. Token Masking

`ui::mask_value(key, val)` returns `"***"` if `key.to_uppercase()` contains any of
`["TOKEN", "KEY", "SECRET"]`. Applied in `build_detail()` for every ENV entry in
the detail panel.

### 4. Args Construction

`launch::build_args(profile)` — pure function, unit-testable:
1. `--model <val>` if `profile.model.is_some()`
2. `--dangerously-skip-permissions` if `profile.skip_permissions == Some(true)`
3. All `profile.extra_args` appended last

### 5. First Run

`config::ensure_default_config()` called in `main` before profile load. Writes a
commented template with one `[[profiles]]` block named "default" if no config file
exists. Uses `dirs::config_dir()` for XDG compliance.

### 6. Empty Profile List

If `profiles` is empty after load, the list shows a placeholder message and
`Enter` is a no-op. `e` still opens the editor.

### 7. Crates

```toml
ratatui    = "0.29"
crossterm  = "0.28"
serde      = { version = "1", features = ["derive"] }
toml       = "0.8"
dirs       = "5"
anyhow     = "1"
```

## Acceptance Criteria (from requirements.md §5)

- [ ] `cargo build --release` produces binary `cct`
- [ ] Renders profile list from `~/.config/cc-tui/profiles.toml`
- [ ] Arrow keys move selection; Enter launches claude with correct env+args
- [ ] `e` suspends TUI, opens `$EDITOR`, resumes on exit
- [ ] `q` / `Ctrl-C` exits cleanly with code 0
- [ ] Token/key values shown as `***` in TUI
- [ ] No config → default created with example profile
- [ ] `extra_args`, `skip_permissions`, `model` correctly translated to CLI flags

## Open Questions

None — all criteria are concrete and testable.
