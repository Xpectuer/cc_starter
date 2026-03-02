---
title: "Intake Session Log"
doc_type: proc
status: activated
brief: "Q&A transcript for cct — Claude Code TUI Launcher"
confidence: verified
created: 2026-03-02
updated: 2026-03-02
revision: 1
---

# Intake Session Log

**Session**: intake-20260302120000
**Status**: active

## Context Snapshot

- **Project root**: `/home/zhengjy/workspace/cc_starter` (lb-dev starter scaffold, no source yet)
- **Context file read**: `~/claude_sub.sh`
  - Contains 5 shell wrapper functions: `kclaude`, `clauddy`, `clauddy_1`, `minimax`, `deepseek`, `ccr_claude`
  - Each function sets env vars (ANTHROPIC_BASE_URL, ANTHROPIC_AUTH_TOKEN, ANTHROPIC_MODEL, etc.) then calls `claude $*`
  - Also defines `ccs`/`ccs1` aliases that add `--dangerously-skip-permissions`
- **Tech stack detected**: none yet (empty project)
- **Docs structure**: lb-dev standard scaffold (`docs/drafts/`, `docs/procs/`, etc.)

## Q&A Transcript

### Round 1
**Category**: Constraints
**Question**: What language/runtime should the TUI be built in?
**Answer**: Rust (ratatui)

### Round 2
**Category**: Constraints
**Question**: How should profiles be stored and edited?
**Answer**: TOML file (`~/.config/cc-tui/profiles.toml`)

### Round 3
**Category**: Scope
**Question**: What should the TUI's main workflow look like?
**Answer**: Profile list → select → launch (minimal, fast). Arrow keys + Enter to launch, `e` to edit.

### Round 4
**Category**: Scope
**Question**: When pressing `e` to edit, what should happen?
**Answer**: Open TOML in `$EDITOR`

### Round 5
**Category**: Scope
**Question**: Which claude CLI args should profiles also be able to set?
**Answer**: `--dangerously-skip-permissions`, `--model` override, arbitrary extra_args

### Round 6
**Category**: Acceptance
**Question**: What should the binary be named?
**Answer**: `cct` (cc-tui shorthand)

## Summary

**Rounds**: 6
**Stop Reason**: All 5 categories answered with confidence
**Gaps**: None — all acceptance criteria are concrete and testable
