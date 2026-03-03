---
title: "TDD: Config Add Env Vars"
doc_type: proc
status: completed
source: "docs/drafts/intake-20260303000100"
brief: "TDD session for extending profile add flow with base_url, api_key, and auto-populated env vars"
test_cmd: "cargo test"
created: 2026-03-03
updated: 2026-03-03
revision: 1
---

# Config Add Env Vars - TDD Session

**Started**: 2026-03-03 16:48
**Plan**: `./tdd-config-add-env-20260303164823_plan.md`

## Test Cases

| # | Test Case | Plan Section | Target File(s) | Red | Green | Refactor |
|---|-----------|--------------|----------------|-----|-------|----------|
| 1 | NewProfile struct extension | Step 1 — Extend NewProfile and append_profile | `src/config.rs` | [x] | [x] | [x] |
| 2 | Env var generation tests | Step 2 — Update existing config tests + add new env var tests | `src/config.rs` | [x] | [x] | [x] |
| 3 | FormState 5-field expansion | Step 3 — Expand FormState in app.rs | `src/app.rs` | [x] | [x] | [x] |
| 4 | CLI add prompts | Steps 4, 7 — Update cli.rs prompts + test | `src/cli.rs` | [x] | [x] | [x] |
| 5 | UI form rendering | Step 5 — Update ui.rs form rendering | `src/ui.rs` | [x] | [x] | [x] |
| 6 | Main TUI save logic | Step 6 — Update main.rs TUI AddForm save logic | `src/main.rs` | [x] | [x] | [x] |
| 7 | App test updates | Step 8 — Update app.rs tests | `src/app.rs` | [x] | [x] | [x] |

## Subagent Log

| # | Case | Outcome | Notes | Timestamp |
|---|------|---------|-------|-----------|
| 1 | NewProfile struct extension | ✅ | Extended struct, rewrote append_profile, updated all construction sites, extracted non_empty() helper | 2026-03-03 |
| 2 | Env var generation tests | ✅ | Added append_profile_base_url_only and append_minimal_no_env_section tests (33 total) | 2026-03-03 |
| 3 | FormState 5-field expansion | ✅ | FIELD_LABELS 3→5, fields [String;3]→[String;5], nav bounds 2→4, updated form_state tests (34 total) | 2026-03-03 |
| 4 | CLI add prompts | ✅ | Added base_url/api_key prompts, mask_key helper, 5-field summary, updated test input (34 total) | 2026-03-03 |
| 5 | UI form rendering | ✅ | Updated confirmation to 5 fields, masked API key via mask_value, added ui_confirmation test (35 total) | 2026-03-03 |
| 6 | Main TUI save logic | ✅ | Fixed field index mismatch, updated Enter threshold 2→4, proper base_url/api_key passing (35 total) | 2026-03-03 |
| 7 | App test updates | ✅ | Already done in Case 3 — no additional changes needed (35 total) | 2026-03-03 |

## Status

**Current case**: 7 / 7
**Progress**: 100% (7/7 complete)
**Blocked**: None

---
**Updated**: 2026-03-03 16:48
