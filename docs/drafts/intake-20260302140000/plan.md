---
title: "Plan: cct E2E Verification (Mock + Live Tests)"
doc_type: proc
brief: "Add lib target, fake claude stub, integration tests, live e2e tests, update CLAUDE.md"
confidence: verified
created: 2026-03-02
updated: 2026-03-02
revision: 1
---

## Files Changed

| File | Change Type |
|------|-------------|
| `Cargo.toml` | Edit — add `[lib]` target |
| `src/lib.rs` | New file — re-export pub modules |
| `src/main.rs` | Edit — replace `mod x;` with `use cct::x;` |
| `src/config.rs` | Edit — add `CCT_CONFIG` env var override to `config_path()` |
| `tests/helpers/claude` | New file — fake binary stub (shell script) |
| `examples/exec_profile.rs` | New file — subprocess helper that calls `exec_claude` |
| `tests/integration.rs` | New file — Tier 1 mock e2e tests |
| `tests/live.rs` | New file — Tier 2 live e2e tests (CCT_LIVE_TESTS=1) |
| `CLAUDE.md` | Edit — add e2e test commands to Build & Test section |

---

## Step 1 — Add lib target to Cargo.toml

**File**: `Cargo.toml`
**What**: Add `[lib]` section so integration tests can import `cct::config`, `cct::launch`, `cct::app`.

**Edit** (after `[package]` block, before `[[bin]]`):
```toml
[lib]
name = "cct"
path = "src/lib.rs"
```

---

## Step 2 — Create src/lib.rs

**File**: `src/lib.rs`
**What**: Re-export all modules as public so integration tests can access them.

**New**:
```rust
pub mod app;
pub mod config;
pub mod launch;
pub mod ui;
```

---

## Step 3 — Update src/main.rs

**File**: `src/main.rs`
**What**: Replace `mod x;` declarations with `use cct::x;` imports (modules now owned by lib).

**Replace**:
```rust
mod app;
mod config;
mod launch;
mod ui;

use app::App;
```
**With**:
```rust
use cct::{app, config, launch, ui};
use app::App;
```

---

## Step 4 — Add CCT_CONFIG override to config_path()

**File**: `src/config.rs`
**What**: Check `CCT_CONFIG` env var first; use it as override path if set (test isolation hook).

**Edit** `config_path()`:
```rust
pub fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("CCT_CONFIG") {
        return PathBuf::from(p);
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("cc-tui")
        .join("profiles.toml")
}
```

---

## Step 5 — Create fake claude stub

**File**: `tests/helpers/claude`
**What**: Shell script that writes received args to `$CCT_TEST_ARGS_FILE`. Must be executable (chmod 755).

**New**:
```bash
#!/bin/sh
# Fake claude stub for testing.
# Writes received args to $CCT_TEST_ARGS_FILE (required).
printf '%s' "$*" > "${CCT_TEST_ARGS_FILE}"
exit 0
```

After writing, run: `chmod 755 tests/helpers/claude`

---

## Step 6 — Create examples/exec_profile.rs

**File**: `examples/exec_profile.rs`
**What**: Minimal binary that reads `CCT_TEST_TOML` env var → loads first profile → calls `exec_claude`.
Used as a subprocess in integration tests to safely test the exec replacement path.

**New**:
```rust
use cct::{config, launch};
use std::env;

fn main() {
    let toml_path = env::var("CCT_TEST_TOML")
        .expect("CCT_TEST_TOML must be set");
    env::set_var("CCT_CONFIG", &toml_path);
    let profiles = config::load_profiles().expect("load profiles");
    let profile = profiles.into_iter().next().expect("at least one profile");
    launch::restore_terminal();
    let err = launch::exec_claude(&profile);
    eprintln!("exec_profile: {err:#}");
    std::process::exit(1);
}
```

---

## Step 7 — Create tests/integration.rs (Tier 1 mock e2e)

**File**: `tests/integration.rs`
**What**: Integration tests using the lib API. Five test cases covering config round-trip,
build_args ordering, full exec through fake binary, and env injection.

**Tests**:
1. `config_round_trip` — write TOML to temp, set `CCT_CONFIG`, call `load_profiles`, verify fields
2. `build_args_ordering` — full profile (model + skip + extra_args) → assert exact arg order
3. `build_args_empty_profile` — minimal profile → assert empty vec
4. `exec_full_profile_fake_binary` — spawn `exec_profile` example with fake PATH + temp TOML, read args file, assert correct args
5. `exec_env_injection` — same subprocess approach, verify env var set before exec (fake claude prints env)

**Key setup for subprocess tests**:
```rust
fn helpers_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/helpers")
}

fn prepend_path(dir: &std::path::Path) -> String {
    let orig = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), orig)
}
```

---

## Step 8 — Create tests/live.rs (Tier 2 live e2e)

**File**: `tests/live.rs`
**What**: Live tests gated on `CCT_LIVE_TESTS=1`. Four test cases.

**Gate macro** (used at top of each test):
```rust
macro_rules! require_live {
    () => {
        if std::env::var("CCT_LIVE_TESTS").is_err() {
            eprintln!("Skipped: set CCT_LIVE_TESTS=1 to run");
            return;
        }
    };
}
```

**Tests**:
1. `release_binary_builds` — run `cargo build --release`, assert `target/release/cct` exists
2. `real_config_loads` — call `cct::config::load_profiles()` against real config (skip if file absent), assert ≥1 profile
3. `binary_spawns_cleanly` — spawn `target/release/cct` as child, write 'q\n' to stdin, assert exit code 0
4. `arg_passthrough_via_fake` — spawn release binary with test config + fake PATH + simulated Enter, assert args file matches profile

---

## Step 9 — Update CLAUDE.md

**File**: `CLAUDE.md`
**What**: Extend the Build & Test Commands section with e2e commands and a commands table.

**Add** after existing commands block:
```bash
# E2E (mock — no real claude needed)
cargo test --test integration

# E2E (live — requires `claude` binary installed)
CCT_LIVE_TESTS=1 cargo test --test live
```
