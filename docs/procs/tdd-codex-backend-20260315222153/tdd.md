---
title: "TDD: Codex Backend Support"
doc_type: proc
status: active
source: "docs/drafts/intake-20260314120000"
brief: "TDD session for adding OpenAI Codex CLI backend to cct"
test_cmd: "cargo test"
created: 2026-03-15
updated: 2026-03-15
revision: 1
---

# Codex Backend Support - TDD Session

**Started**: 2026-03-15 22:21
**Plan**: `./plan.md`

## Test Cases

| # | Test Case | Plan Section | Target File(s) | Red | Green | Refactor |
|---|-----------|--------------|----------------|-----|-------|----------|
| 1 | Backend enum deserialization (claude default, codex explicit) | Step 1 | `src/config.rs` | [x] | [x] | [x] |
| 2 | Profile with base_url field round-trips | Step 1 | `src/config.rs` | [x] | [x] | [x] |
| 3 | validate_profiles rejects codex+skip_permissions | Step 1 | `src/config.rs` | [x] | [x] | [x] |
| 4 | validate_profiles rejects claude+full_auto | Step 1 | `src/config.rs` | [x] | [x] | [x] |
| 5 | append_profile generates codex env (only OPENAI_API_KEY) | Step 1 | `src/config.rs` | [x] | [x] | [x] |
| 6 | filtered_indices returns correct backend subset | Step 2 | `src/app.rs` | [x] | [x] | [x] |
| 7 | switch_backend resets selected to first matching | Step 2 | `src/app.rs` | [x] | [x] | [x] |
| 8 | next/prev navigate within filtered backend | Step 2 | `src/app.rs` | [x] | [x] | [x] |
| 9 | field_labels returns backend-specific labels | Step 2 | `src/app.rs` | [x] | [x] | [x] |
| 10 | UI tab bar renders with active highlight | Step 3 | `src/ui.rs` | [x] | [x] | [x] |
| 11 | Detail panel shows full_auto for codex profile | Step 3 | `src/ui.rs` | [x] | [x] | [x] |
| 12 | build_codex_args with full_auto and extra_args | Step 4 | `src/launch.rs` | [x] | [x] | [x] |
| 13 | generate_codex_config writes correct toml from profile fields | Step 4 | `src/launch.rs` | [x] | [x] | [x] |
| 14 | exec dispatches by backend (claude vs codex) | Step 5 | `src/main.rs` | [x] | [x] | [x] |
| 15 | CLI add passes Backend::Claude and full_auto:None | Step 6 | `src/cli.rs` | [x] | [x] | [x] |

## Subagent Log

| # | Case | Outcome | Notes | Timestamp |
|---|------|---------|-------|-----------|
| 1-5 | Step 1 config.rs | ✅ | Backend enum, validate_profiles, append_profile codex env. 49 tests pass. | 2026-03-15 |
| 6-9 | Step 2 app.rs | ✅ | filtered_indices, switch_backend, next/prev filtered, field_labels. 53 tests pass. | 2026-03-15 |
| 10-11 | Step 3 ui.rs | ✅ | build_tab_bar, detail panel codex full_auto. 55 tests pass. | 2026-03-15 |
| 12-13 | Step 4 launch.rs | ✅ | build_codex_args, generate_codex_config, exec_codex. 61 tests pass. | 2026-03-15 |
| 14 | Step 5 main.rs | ✅ | build_launch_command dispatch, Tab/1/2 keys, backend-aware form. 63 tests pass. | 2026-03-15 |
| 15 | Step 6 cli.rs | ✅ | CLI add confirmed Backend::Claude + full_auto:None. 65 tests pass. | 2026-03-15 |

## Status

**Current case**: 15 / 15
**Progress**: 100% (15/15 complete)
**Blocked**: None

---
**Updated**: 2026-03-15 22:21
