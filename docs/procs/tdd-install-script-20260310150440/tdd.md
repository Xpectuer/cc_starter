---
title: "TDD: cct install script"
doc_type: proc
status: completed
source: "docs/drafts/intake-20260310120000"
brief: "TDD session for cct curl|bash install script"
test_cmd: "bats tests/install.bats"
created: 2026-03-10
updated: 2026-03-10
revision: 1
---

# cct install script - TDD Session

**Started**: 2026-03-10 15:04
**Plan**: `./plan.md`

## Test Cases

| # | Test Case | Plan Section | Target File(s) | Red | Green | Refactor |
|---|-----------|--------------|----------------|-----|-------|----------|
| 1 | detect_linux_x86_64 | Step 1 — `detect()` | `install.sh`, `tests/install.bats` | [x] | [x] | [x] |
| 2 | detect_macos_arm64 | Step 1 — `detect()` | `install.sh`, `tests/install.bats` | [x] | [x] | [x] |
| 3 | detect_unsupported_os | Step 1 — `detect()` | `install.sh`, `tests/install.bats` | [x] | [x] | [x] |
| 4 | fetch_latest_parses_version :hand: MANUAL | Step 1 — `fetch_latest()` | `install.sh`, `tests/install.bats` | [x] | [x] | [x] |
| 5 | fetch_latest_fails_on_bad_response :hand: MANUAL | Step 1 — `fetch_latest()` | `install.sh`, `tests/install.bats` | [x] | [x] | [x] |
| 6 | download_retries_on_failure :hand: MANUAL | Step 1 — `download()` | `install.sh`, `tests/install.bats` | [x] | [x] | [x] |
| 7 | install_binary_creates_dir_and_copies | Step 1 — `install_binary()` | `install.sh`, `tests/install.bats` | [x] | [x] | [x] |
| 8 | path_hint_shown_when_not_in_path | Step 1 — `path_hint()` | `install.sh`, `tests/install.bats` | [x] | [x] | [x] |
| 9 | path_hint_silent_when_in_path | Step 1 — `path_hint()` | `install.sh`, `tests/install.bats` | [x] | [x] | [x] |

## Subagent Log

| # | Case | Outcome | Notes | Timestamp |
|---|------|---------|-------|-----------|
| 1 | detect_linux_x86_64 | ✅ | Subagent: created install.sh + bats test | 2026-03-10 |
| 2 | detect_macos_arm64 | ✅ | detect() already green from case 1 | 2026-03-10 |
| 3 | detect_unsupported_os | ✅ | Error path already green from case 1 | 2026-03-10 |
| 4 | fetch_latest_parses_version | ✅ | MANUAL→stub: curl mock | 2026-03-10 |
| 5 | fetch_latest_fails_on_bad_response | ✅ | MANUAL→stub: bad JSON | 2026-03-10 |
| 6 | download_retries_on_failure | ✅ | MANUAL→stub: curl fails, retry verified | 2026-03-10 |
| 7 | install_binary_creates_dir_and_copies | ✅ | Fake tarball, mkdir+install verified | 2026-03-10 |
| 8 | path_hint_shown_when_not_in_path | ✅ | PATH hint output verified | 2026-03-10 |
| 9 | path_hint_silent_when_in_path | ✅ | No output when in PATH | 2026-03-10 |

## Status

**Current case**: 9 / 9
**Progress**: 100% (9/9 complete)
**Blocked**: None

---
**Updated**: 2026-03-10 15:30
