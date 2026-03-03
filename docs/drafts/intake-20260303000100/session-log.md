---
title: "Intake Session Log"
doc_type: proc
status: activated
brief: "Q&A transcript for config add functionality for cct"
confidence: speculative
created: 2026-03-03
updated: 2026-03-03
revision: 1
context_ref: "docs/references/context-20260303000100/"
---

# Intake Session Log

**Session**: intake-20260303000100
**Status**: active

## Context Snapshot

- **Project root**: `/home/zhengjy/workspace/cc_starter`
- **Tech stack**: Rust (Cargo), ratatui 0.29, crossterm 0.28, serde/toml, dirs, anyhow
- **Modules**: config.rs, app.rs, ui.rs, launch.rs, main.rs, lib.rs
- **Config location**: `~/.config/cc-tui/profiles.toml`
- **Current config editing**: hot-reload via `$EDITOR` on `e` key
- **Prior intakes**: 2 sessions (both activated, skipped)

## Q&A Transcript

### Round 1
**Category**: Goal
**Question**: What's the primary way you want users to add a new profile? Should it be an in-TUI guided form, a CLI subcommand (e.g. `cct add`), or both?
**Answer**: Both
**Files Read**: src/config.rs

### Round 2
**Category**: Scope
**Question**: Which profile fields should the 'add' flow collect?
**Answer**: Core only — name, description, model
**Files Read**: None

### Round 3
**Category**: Constraints
**Question**: For the CLI subcommand, should `cct add` be interactive (prompt-by-prompt) or flag-based?
**Answer**: Interactive prompts
**Files Read**: None

### Round 4
**Category**: Goal
**Question**: Should the in-TUI 'add' flow be triggered by a keybinding that opens an inline form, $EDITOR template, or popup modal?
**Answer**: Keybind + inline form
**Files Read**: None

### Round 5
**Category**: Acceptance
**Question**: Should there be validation when adding a profile?
**Answer**: Duplicate name check only (no model validation)
**Files Read**: None

### Round 6
**Category**: Acceptance
**Question**: After a profile is added, should the config file be written immediately or with a confirm step?
**Answer**: Confirm before save
**Files Read**: None

### Round 7
**Category**: Constraints
**Question**: For the CLI argument parser, clap or manual parsing?
**Answer**: clap (add dependency)
**Files Read**: None

### Round 8
**Category**: Scope
**Question**: Should the new profile be auto-selected in the TUI after it's added?
**Answer**: Auto-select new profile
**Files Read**: None

## Summary
**Rounds**: 8
**Stop Reason**: All 5 question categories answered with confidence
**Gaps**: None — all requirements clear
