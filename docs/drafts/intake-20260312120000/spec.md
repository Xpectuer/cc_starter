---
title: "Spec: --continue one-shot launch key for cct TUI"
doc_type: proc
brief: "Add 'c' key to launch selected profile with --continue via bool param on build_args/exec_claude"
confidence: verified
created: 2026-03-12
updated: 2026-03-12
revision: 1
---

# Spec: --continue One-Shot Launch Key

## Chosen Approach

Approach A — boolean flag threaded through `build_args` and `exec_claude`. Single arg-building
path, no duplication of env-injection logic, clean unit-test surface.

## Architecture

Three files change; no new files, no new dependencies.

```
main.rs  ──(KeyCode::Char('c'), _)──▶  launch::exec_claude(profile, with_continue=true)
main.rs  ──(KeyCode::Enter)──────────▶  launch::exec_claude(profile, with_continue=false)
                                              │
                                    build_args(profile, with_continue)
                                              │
                               prepend "--continue" if flag is true
                                              │
                                    Command::new("claude").args(&args).exec()
```

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| API shape | Bool param on both `build_args` and `exec_claude` | Single arg-building path; testable with `build_args(&p, true/false)` |
| `--continue` position | First arg (before `--model`, `--dangerously-skip-permissions`, extra_args) | Consistent positional intent |
| Key binding | bare `c` (no modifier) | `Ctrl+C` uses `KeyModifiers::CONTROL`; bare `c` is currently unbound |
| Persistence | None | One-shot modifier; profile TOML unchanged |

## Acceptance Criteria

1. Pressing `c` in TUI Normal mode exec-replaces with `claude --continue [<profile-args>...]`
2. Normal `Enter` launch is unaffected (passes `false` to `exec_claude`)
3. Footer shows `[c] Resume` hint in Normal mode
4. `cargo test` passes including `build_args_continue_only` and `build_args_continue_with_flags`
5. `cargo clippy` passes with no new warnings
