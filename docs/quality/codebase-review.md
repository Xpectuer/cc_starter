---
doc_type: review
generated_by: mci-phase-4
review_date: 2026-03-15
---

# cct — Codebase Review

> Produced by the MCI Phase 4 (Reflections & Post-Analysis) workflow.

---

<!-- BEGIN:gap-analysis -->
## Gap Analysis

### Missing Files / Uncovered Areas

1. **`tests/helpers/claude` — undocumented fake binary**: This shell script is a critical test fixture (it is the mock `claude` binary used in `exec_full_profile_fake_binary` and `exec_env_injection` integration tests) but has no associated documentation. Developers unfamiliar with the project may not understand why it exists or how to maintain it.

2. **`tests/live.rs` — live test conditions**: The live test file exists but requires `CCT_LIVE_TESTS=1` and a real `claude` binary. Its exact test cases and expectations are unknown from static analysis alone. A companion doc or inline comments explaining the live test matrix would help CI contributors.

3. **`cli` module had no dedicated doc**: Resolved 2026-03-15 — `docs/modules/cli.md` created. The module index previously noted it as "(inline in src/cli.rs)" without a doc file.

4. **~~No `README.md`~~**: ✅ **Resolved** — `README.md` now exists at the repo root with installation instructions, config format, keybindings table, build/test commands, and architecture overview.

### Dead Code

No dead code was identified in the five source modules — each exported symbol is consumed by `main.rs` or the test suites. The codebase is small enough that dead code is unlikely to accumulate unnoticed.

### Structural Observations

- **All source files are flat** (`src/*.rs`) — no subdirectory structure. This is appropriate for the current size (~7 files, ~450 lines total) but will need reorganization if significant new features are added.
- **`src/lib.rs`** simply re-exports all five modules (`pub mod app; pub mod cli; pub mod config; pub mod launch; pub mod ui;`). It exists only to enable integration tests to import `cct::*` from outside the binary.
- **`--continue` flag** added to `build_args`/`exec_claude` as `with_continue: bool` — cleanly threaded from TUI `c` key through launch; no branching added to `config`.
<!-- END:gap-analysis -->

---

<!-- BEGIN:risk-assessment -->
## Risk Assessment

### Technical Debt

| Area | Severity | Description | Recommendation |
|------|----------|-------------|----------------|
| `exec_claude` return type | Low | Returns `anyhow::Error` (not `Result<!, E>`) because stable Rust doesn't stabilize `!` as a return type. The caller must treat the return as always indicating failure. | Add a doc comment on the call site in `main.rs` or wait for `!` stabilization. No action required now. |
| `env::set_var` thread safety | Low | `set_var` is not thread-safe. In the current single-threaded event loop this is fine. | Document the single-threaded assumption; add a `#[cfg(target_os = "...")]` note if Windows support is ever attempted. |
| `dirs::config_dir()` fallback | Low | Falls back to the literal string `"~/.config"` (not tilde-expanded). On exotic systems where XDG is unavailable this would produce a broken path. | Use `dirs::home_dir()` as a secondary fallback, or document the limitation explicitly. |
| ~~Missing `README.md`~~ | ~~Medium~~ | ✅ Resolved — `README.md` added with full installation, config, and keybinding docs. | — |
| Live tests require external binary | Medium | `CCT_LIVE_TESTS=1 cargo test --test live` silently skips when `claude` is absent rather than failing with a clear message. CI cannot easily enforce live test coverage. | Add a check at the start of the live test file that prints a clear skip message, or document a mock-only CI strategy. |

### Security Hotspots

1. **Plaintext credential storage** (`src/config.rs`): `profiles.toml` stores `ANTHROPIC_AUTH_TOKEN` and other secrets in plaintext with user-default file permissions (typically `0644`). The `ui` module masks them in the TUI display, but the file itself is unencrypted. Documented in `README.md` as a security consideration. **Risk**: Low for single-user machines; relevant if the file is backed up to shared storage.

2. **`env::set_var` before exec** (`src/launch.rs:34-37`): Profile env vars are injected into the current process environment before exec-replacing with `claude`. If exec fails, these vars remain in the process environment until exit (caller then calls `std::process::exit(1)`). Not exploitable in current design.

3. **No input validation on `extra_args`**: Profile `extra_args` are appended verbatim to the `claude` invocation. A malicious or misconfigured `profiles.toml` could inject arbitrary flags. Since the config file is user-controlled and the `claude` binary is also user-controlled, this is an acceptable trust boundary.
<!-- END:risk-assessment -->

---

<!-- BEGIN:ai-score -->
## AI Usability Score: 9.0 / 10

### Score Breakdown

| Criterion | Weight | Score | Notes |
|-----------|--------|-------|-------|
| Code clarity | 20% | 9/10 | Clear variable/function names; well-structured; idiomatic Rust. `with_continue: bool` cleanly threaded without branching. |
| Modularity | 20% | 9/10 | 5 focused modules, single responsibility each, no circular deps, no shared mutable globals. |
| Documentation completeness | 20% | 9/10 | README.md ✅, ARCHITECTURE.md ✅, all 5 module docs ✅ (cli.md added 2026-03-15). Live test coverage still partially conditional on external binary. |
| Type safety | 15% | 9/10 | Full Rust type system; serde-derived deserialization; `Option<T>` for all optional fields; no `unsafe` blocks in library code. |
| Test coverage | 15% | 8/10 | Good unit + integration test coverage for config parsing, arg building, masking, exec, and `--continue` flag. Live E2E partially conditional on external binary. |
| Code complexity | 10% | 9/10 | All functions are short (<50 lines), no deep nesting, cyclomatic complexity is low throughout. |

**Weighted total**: (9×0.20) + (9×0.20) + (9×0.20) + (9×0.15) + (8×0.15) + (9×0.10) = **9.0/10**

**Overall assessment**: Very high AI usability. The codebase is exceptionally clean and well-structured. All five modules have dedicated documentation, README.md exists, and the `--continue` feature was added cleanly without complicating any module's surface area. The primary remaining gap is incomplete live-test documentation.
<!-- END:ai-score -->

---

<!-- BEGIN:improvements -->
## Improvement Recommendations (Priority-Ordered)

### 1. ~~Add `README.md`~~ ✅ Resolved (2026-03-15)

README.md exists with installation, config format, keybindings, build commands, and architecture overview.

### 2. Document `tests/helpers/claude` fake binary (Impact: Medium, Effort: Low)

- **Current state**: The fake `claude` shell script at `tests/helpers/claude` is a critical test fixture but has no doc comment or companion documentation explaining what it does, what arguments it accepts, and what env vars it reads to simulate behavior.
- **Target**: Add a header comment block to `tests/helpers/claude` explaining its role, and add a brief section to `docs/modules/index.md` or a separate `docs/references/test-infrastructure.md`.
- **Effort**: < 1 hour.
- **AI impact**: Agents modifying integration tests will not accidentally break or misuse this fixture.

### 3. Document live test matrix (Impact: Medium, Effort: Low)

- **Current state**: `tests/live.rs` requires `CCT_LIVE_TESTS=1` and a real `claude` binary. Its test cases and expected behavior are unknown from static analysis.
- **Target**: Add inline comments or a `docs/references/live-tests.md` describing the test matrix, what each test validates, and how CI should be configured to run them.
- **Effort**: 1-2 hours.
<!-- END:improvements -->
