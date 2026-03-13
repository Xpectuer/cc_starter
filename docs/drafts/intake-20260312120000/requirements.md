---
title: "Requirements: --continue key binding for cct TUI"
doc_type: proc
brief: "Add 'c' key in TUI Normal mode to launch selected profile with --continue flag (one-shot, no TOML persist)"
confidence: verified
created: 2026-03-12
updated: 2026-03-12
revision: 1
source_skill: intake
---

# Requirements: --continue Key Binding for cct TUI

## 1. Goal

Allow the user to launch the selected profile with `--continue` (resume most-recent Claude Code conversation) by pressing a single key in the TUI, without modifying the profile or its TOML config.

## 2. Constraints

- Language/stack: Rust, ratatui, crossterm — no new dependencies
- Key must not conflict with existing bindings (`q`, `j`, `k`, `↑`, `↓`, `Enter`, `e`, `s`, `a`, `Ctrl-C`)
- Key chosen: **`c`** (bare, no modifier — distinct from `Ctrl-C` which uses `KeyModifiers::CONTROL`)
- Must follow existing `exec` pattern (restore terminal → exec-replace → no return on success)

## 3. Existing State

- `main.rs` Normal mode handler at line 56: pattern-matches `(KeyCode, KeyModifiers)` pairs
- `launch::build_args(profile)` builds `Vec<String>` of claude CLI args; does not include `--continue`
- `launch::exec_claude(profile)` calls `build_args` then `Command::new("claude").args(&args).exec()`
- `ui.rs` footer at line 102: string literal `" [↑↓/jk] Navigate  [Enter] Launch  [s] Skip-perms  [a] Add  [e] Edit config  [q/Ctrl-C] Quit"`

## 4. Scope

### In scope (v1)
- Add `(KeyCode::Char('c'), _)` handler in Normal mode → launch with `--continue` prepended
- Add `exec_claude_continue(profile)` function (or extend `build_args`) in `launch.rs`
- Update footer string to include `[c] Resume`
- Unit test: `build_args_with_continue` / args include `--continue`
- Test: `build_args_continue_and_skip_perms` (both flags coexist)

### Out of scope
- Persisting `continue` flag to TOML
- Per-profile toggle (like `skip_permissions`)
- Session-level toggle with footer indicator
- `--resume <conversation-id>` variant

## 5. Acceptance Criteria

1. In TUI Normal mode, pressing `c` on a selected profile:
   - Restores terminal
   - Exec-replaces the process with `claude --continue [<profile-args>...]`
2. Normal `Enter` launch is unaffected (no `--continue` added)
3. Footer shows `[c] Resume` hint in Normal mode
4. `cargo test` passes including new unit tests
5. `cargo clippy` passes with no new warnings
