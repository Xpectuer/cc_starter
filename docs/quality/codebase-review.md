---
doc_type: review
generated_by: mci-phase-4
review_date: 2026-03-03
---

# cct — Codebase Review

> Produced by the MCI Phase 4 (Reflections & Post-Analysis) workflow.

---

<!-- BEGIN:gap-analysis -->
## Gap Analysis

### Missing Files / Uncovered Areas

1. **`tests/helpers/claude` — undocumented fake binary**: This shell script is a critical test fixture (it is the mock `claude` binary used in `exec_full_profile_fake_binary` and `exec_env_injection` integration tests) but has no associated documentation. Developers unfamiliar with the project may not understand why it exists or how to maintain it.

2. **`tests/live.rs` — live test conditions**: The live test file exists but was not analyzed in this workflow (it requires `CCT_LIVE_TESTS=1` and a real `claude` binary). Its exact test cases and expectations are unknown from static analysis alone. A companion doc or inline comments explaining the live test matrix would help CI contributors.

3. **`examples/` directory**: The integration tests invoke `cargo run --example exec_profile` but there is no `examples/` directory visible in the source tree from `Glob`. This example binary is invoked as a subprocess in tests but not documented. If it exists as an implicit or derived artifact, it needs a home. If it is missing from the checked-in source, that is a test infrastructure gap.

4. **No `README.md`**: The project has `CLAUDE.md` and `ARCHITECTURE.md` but no user-facing `README.md`. Casual GitHub visitors or package consumers have no installation instructions, usage guide, or config reference at the repo root.

### Dead Code

No dead code was identified in the four source modules — each exported symbol is consumed by `main.rs` or the test suites. The codebase is small enough that dead code is unlikely to accumulate unnoticed.

### Structural Observations

- **All source files are flat** (`src/*.rs`) — no subdirectory structure. This is appropriate for the current size (~6 files, ~350 lines total) but will need reorganization if significant new features are added.
- **`src/lib.rs`** simply re-exports all four modules (`pub mod app; pub mod config; pub mod launch; pub mod ui;`). It exists only to enable integration tests to import `cct::*` from outside the binary. This is idiomatic but means the crate has a somewhat unusual dual-crate layout (lib + bin sharing the same source modules).
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
| Missing `README.md` | Medium | No user-facing installation or usage documentation. Package consumers on crates.io or GitHub have no onboarding guide. | Add a `README.md` with installation instructions, config file example, and keybindings table. Estimated effort: 1-2 hours. |
| Live tests require external binary | Medium | `CCT_LIVE_TESTS=1 cargo test --test live` silently skips when `claude` is absent rather than failing with a clear message. CI cannot easily enforce live test coverage. | Add a check at the start of the live test file that prints a clear skip message, or document a mock-only CI strategy. |

### Security Hotspots

1. **Plaintext credential storage** (`src/config.rs`): `profiles.toml` stores `ANTHROPIC_AUTH_TOKEN` and other secrets in plaintext with user-default file permissions (typically `0644`). The `ui` module masks them in the TUI display, but the file itself is unencrypted. This is an intentional UX trade-off (simpler than a keychain) but should be documented explicitly in the README as a security consideration. **Risk**: Low for single-user machines; relevant if the file is backed up to shared storage.

2. **`env::set_var` before exec** (`src/launch.rs:31-35`): Profile env vars are injected into the current process environment before exec-replacing with `claude`. If exec fails, these vars remain in the process environment until exit (line 39: `std::process::exit(1)`). This is not exploitable in the current design but is worth noting for future architectures that might continue running after a failed exec.

3. **No input validation on `extra_args`**: Profile `extra_args` are appended verbatim to the `claude` invocation. A malicious or misconfigured `profiles.toml` could inject arbitrary flags. Since the config file is user-controlled and the `claude` binary is also user-controlled, this is an acceptable trust boundary. Document it explicitly.
<!-- END:risk-assessment -->

---

<!-- BEGIN:ai-score -->
## AI Usability Score: 8.5 / 10

### Score Breakdown

| Criterion | Weight | Score | Notes |
|-----------|--------|-------|-------|
| Code clarity | 20% | 9/10 | Clear variable/function names; well-structured; idiomatic Rust. Only minor: `exec_claude`'s unusual return type needs a comment. |
| Modularity | 20% | 9/10 | 4 focused modules, single responsibility each, no circular deps, no shared mutable globals. |
| Documentation completeness | 20% | 7/10 | Strong after MCI workflow; missing `README.md` and live test documentation. |
| Type safety | 15% | 9/10 | Full Rust type system; serde-derived deserialization; `Option<T>` for all optional fields; no `unsafe` blocks in library code. |
| Test coverage | 15% | 8/10 | Good unit + integration test coverage for config parsing, arg building, masking, and exec. Live E2E partially conditional on external binary. |
| Code complexity | 10% | 9/10 | All functions are short (<50 lines), no deep nesting, cyclomatic complexity is low throughout. |

**Weighted total**: (9×0.20) + (9×0.20) + (7×0.20) + (9×0.15) + (8×0.15) + (9×0.10) = **8.5/10**

**Overall assessment**: High AI usability. The codebase is exceptionally clean and well-structured for its size. An AI agent can navigate it with minimal context: four modules, one `Profile` struct, one data flow direction. The primary gaps are the missing user-facing README and incomplete live-test documentation.
<!-- END:ai-score -->

---

<!-- BEGIN:improvements -->
## Improvement Recommendations (Priority-Ordered)

### 1. Add `README.md` (Impact: High, Effort: Low)

- **Current state**: No user-facing installation or configuration documentation.
- **Target**: A `README.md` at the repository root covering: what `cct` is, installation (`cargo install` or binary download), config file format with a commented example, keybinding reference, and a security note about plaintext credential storage.
- **Effort**: 1-3 hours.
- **AI impact**: Enables AI agents to answer "how do I install/use this?" without reading source code.

### 2. Document and fix the `examples/exec_profile` binary (Impact: Medium, Effort: Low)

- **Current state**: The integration tests spawn `cargo run --example exec_profile` as a subprocess, but no `examples/exec_profile.rs` file was found in the glob results. This may indicate a missing file that tests rely on silently.
- **Target**: Confirm `examples/exec_profile.rs` exists and is committed; add a brief doc comment at the top explaining its role as a test harness.
- **Effort**: < 1 hour to verify and document.
- **AI impact**: AI agents modifying the integration test suite will understand why this binary exists and not accidentally delete it.

### 3. Add `--help` / `--version` CLI flags (Impact: Medium, Effort: Medium)

- **Current state**: `cct` launches directly into the TUI with no command-line help. `cct --help` would be passed as an `extra_arg` to `claude` (since it reaches `main` before any arg parsing).
- **Target**: Parse `--help` and `--version` before entering the TUI loop using a minimal arg parser (or even manual `std::env::args()` checking). Print usage and exit cleanly.
- **Effort**: 2-4 hours.
- **AI impact**: Makes the binary self-documenting; agents can test `cct --version` to verify installation.
<!-- END:improvements -->
