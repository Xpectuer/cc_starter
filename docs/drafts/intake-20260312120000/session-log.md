---
title: "Intake Session Log"
doc_type: proc
status: activated
brief: "Q&A transcript for cct --continue one-shot launch key"
confidence: verified
created: 2026-03-12
updated: 2026-03-12
revision: 1
context_ref: "docs/references/context-20260312120000/"
---

# Intake Session Log — --continue Key Binding

## Context Snapshot (Phase 1a)

**Project**: `cct` — Terminal UI launcher for Claude Code (Rust/ratatui)
**Relevant source files read**:
- `src/main.rs` — event loop, Normal mode key handlers (line 56–101)
- `src/app.rs` — App, AppMode, FormState
- `src/launch.rs` — build_args, exec_claude
- `src/ui.rs` — draw, footer string (line 102)
- `CLAUDE.md` — project overview and architecture

**Existing key bindings (Normal mode)**:
```
q / Ctrl-C → quit
↑↓ / jk   → navigate
Enter       → launch (exec_claude)
e           → open editor + hot-reload
s           → toggle skip_permissions (persisted)
a           → open AddForm
```

**Existing `build_args` output**: `--model <m>`, `--dangerously-skip-permissions`, `<extra_args...>`

---

## Q&A Transcript (Phase 2)

### Round 1 — Goal (Behavior)

**Q**: How should `--continue` behave when triggered from the TUI?

Options:
1. One-shot launch key — press dedicated key, launch with --continue once. Nothing persisted.
2. Per-profile toggle — like skip_permissions, persisted to TOML.
3. Session-level toggle — runtime flag, shown in footer, resets on exit.

**A**: One-shot launch key (Recommended)

### Round 2 — Key binding

**Q**: Which key should launch the selected profile with `--continue`?

Options:
1. `c` — continue (bare, no modifier; Ctrl+C uses modifier combo)
2. `r` — resume
3. Ctrl+Enter
4. Other

**A**: `c — continue (Recommended)`

---

## Synthesis (Phase 3)

### Understood Requirements

The user wants a simple, zero-friction way to resume the most-recent Claude Code conversation from
within the cct TUI. The feature should behave exactly like pressing `Enter` (launch the selected
profile), but with `--continue` prepended to the claude CLI args.

Key decisions:
- **One-shot, not persistent**: pressing `c` launches once with `--continue`; no TOML change
- **Key**: bare `c` (currently unbound; Ctrl+C is a separate event with `KeyModifiers::CONTROL`)
- **Placement**: Normal mode only (same level as `Enter`)
- **Args order**: `--continue` prepended before profile args

Implementation touches:
1. `launch.rs` — new function `exec_claude_continue` (or `build_args` bool param)
2. `main.rs` — add `(KeyCode::Char('c'), _)` arm in Normal mode, guard `!app.profiles.is_empty()`
3. `ui.rs` — update footer constant to include `[c] Resume`
4. Tests — `build_args` with continue, integration coverage

No `[UNCERTAIN]` areas. All 5 categories answered with confidence.

### User Corrections

None — user accepted synthesis.
