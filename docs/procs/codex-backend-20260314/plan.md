---
title: "Plan: Codex Backend Support"
doc_type: proc
brief: "Implementation plan for adding OpenAI Codex CLI backend to cct"
confidence: verified
created: 2026-03-14
updated: 2026-03-14
revision: 1
---

# Plan: Codex Backend Support

## Files Changed

| File | Change Type |
|------|-------------|
| `src/config.rs` | Major edit |
| `src/app.rs` | Major edit |
| `src/ui.rs` | Major edit |
| `src/launch.rs` | Major edit |
| `src/main.rs` | Major edit |
| `src/cli.rs` | Minor edit |

---

## Step 1 — Add Backend enum and Profile fields to config.rs

**File**: `src/config.rs`
**What**: Add `Backend` enum, `backend` and `full_auto` fields to `Profile`, add `validate()` function, update `NewProfile` and `append_profile()`.

**Old**:
```
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub description: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub extra_args: Option<Vec<String>>,
    pub skip_permissions: Option<bool>,
    pub model: Option<String>,
}
```

**New**:
```rust
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Default, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    #[default]
    Claude,
    Codex,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub backend: Backend,
    pub env: Option<HashMap<String, String>>,
    pub extra_args: Option<Vec<String>>,
    pub skip_permissions: Option<bool>,
    pub model: Option<String>,
    pub full_auto: Option<bool>,
}

/// Validate field combinations per backend. Called after deserialization.
pub fn validate_profiles(profiles: &[Profile]) -> Result<()> {
    for p in profiles {
        match p.backend {
            Backend::Codex => {
                if p.skip_permissions.unwrap_or(false) {
                    anyhow::bail!(
                        "Profile '{}': skip_permissions is not supported for codex backend",
                        p.name
                    );
                }
            }
            Backend::Claude => {
                if p.full_auto.unwrap_or(false) {
                    anyhow::bail!(
                        "Profile '{}': full_auto is not supported for claude backend",
                        p.name
                    );
                }
            }
        }
    }
    Ok(())
}
```

Also update `load_profiles()`:

**Old**:
```
pub fn load_profiles() -> Result<Vec<Profile>> {
    let path = config_path();
    let content = fs::read_to_string(&path).with_context(|| format!("read config {path:?}"))?;
    let config: Config =
        toml::from_str(&content).with_context(|| format!("parse TOML in {path:?}"))?;
    Ok(config.profiles)
}
```

**New**:
```rust
pub fn load_profiles() -> Result<Vec<Profile>> {
    let path = config_path();
    let content = fs::read_to_string(&path).with_context(|| format!("read config {path:?}"))?;
    let config: Config =
        toml::from_str(&content).with_context(|| format!("parse TOML in {path:?}"))?;
    validate_profiles(&config.profiles)?;
    Ok(config.profiles)
}
```

Also update `NewProfile` and `append_profile()`:

**Old** (`NewProfile`):
```
pub struct NewProfile {
    pub name: String,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
}
```

**New**:
```rust
pub struct NewProfile {
    pub name: String,
    pub description: Option<String>,
    pub backend: Backend,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub full_auto: Option<bool>,
}
```

Update `append_profile()` to write `backend` and `full_auto` fields, and generate codex-specific env vars when backend is Codex.

Also update `DEFAULT_CONFIG` to include a commented codex example.

Add unit tests: codex profile deserialization, validate rejects codex+skip_perms, validate rejects claude+full_auto, backward compat (no backend field defaults to claude).

**Verify**: `cargo test config` — all new and existing config tests pass.

---

## Step 2 — Add active_backend and filtered navigation to app.rs

**File**: `src/app.rs`
**What**: Add `active_backend` to `App`, modify `next()`/`prev()` to navigate within backend, add backend-aware `FormState`.

**Old**:
```
pub struct App {
    pub profiles: Vec<Profile>,
    pub selected: usize,
    pub mode: AppMode,
}
```

**New**:
```rust
pub struct App {
    pub profiles: Vec<Profile>,
    pub selected: usize,
    pub mode: AppMode,
    pub active_backend: Backend,
}
```

Add helper methods:
- `filtered_indices(&self) -> Vec<usize>` — indices of profiles matching `active_backend`
- `switch_backend(&mut self, backend: Backend)` — set `active_backend`, reset `selected` to first matching profile
- `next()`/`prev()` navigate only within filtered indices

Update `FormState` to include `backend: Backend` field. Update `FIELD_LABELS` to be a function `field_labels(backend: &Backend) -> [&str; 5]` returning backend-specific labels:
- Claude: `["Name *", "Description", "Base URL", "API Key", "Model"]`
- Codex: `["Name *", "Description", "API Key", "Model", "Full Auto (y/n)"]`

Add unit tests: switch_backend resets selected, next/prev skip non-matching backend, filtered_indices correctness.

**Verify**: `cargo test app` — all tests pass.

---

## Step 3 — Update UI rendering for tabs and codex detail

**File**: `src/ui.rs`
**What**: Add tab indicator to profile list, filter displayed profiles by backend, show codex-specific fields in detail panel, update footer.

Key changes:
- Profile list: render `[Claude] [Codex]` tab bar above the list, highlight active tab
- List items: only show profiles where `backend == active_backend`
- `build_detail()`: show `full_auto: ✓` for codex profiles (instead of `skip_permissions`)
- `build_form_lines()`: use `field_labels(backend)` for form rendering
- Footer: add `[Tab/1/2] Backend` hint in Normal mode
- Codex profiles with `full_auto=true` use orange/yellow style (analogous to red for skip_perms)

**Verify**: `cargo test ui` — all tests pass.

---

## Step 4 — Add codex launch functions to launch.rs

**File**: `src/launch.rs`
**What**: Add `build_codex_args()` and `exec_codex()` functions.

**Old** (after `exec_claude`):
```
/// Check if `claude` (or override via CCT_CLAUDE_BIN) is available in PATH.
```

**New** (insert before that line):
```rust
/// Build the CLI argument list for `codex` from a profile. Pure — no side effects.
pub fn build_codex_args(profile: &Profile) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(model) = &profile.model {
        args.push("--model".to_string());
        args.push(model.clone());
    }
    if profile.full_auto.unwrap_or(false) {
        args.push("--full-auto".to_string());
    }
    if let Some(extra) = &profile.extra_args {
        args.extend(extra.iter().cloned());
    }
    args
}

/// Inject profile env vars and exec-replace the current process with `codex`.
pub fn exec_codex(profile: &Profile) -> anyhow::Error {
    if let Some(env_map) = &profile.env {
        for (k, v) in env_map {
            env::set_var(k, v);
        }
    }
    let args = build_codex_args(profile);
    let err = Command::new("codex").args(&args).exec();
    anyhow::anyhow!("exec codex: {err}")
}

```

Add unit tests: `build_codex_args` with empty, model only, full_auto, extra_args combinations.

**Verify**: `cargo test launch` — all tests pass.

---

## Step 5 — Update main.rs event handling

**File**: `src/main.rs`
**What**: Add Tab/1/2 key handlers, dispatch Enter/c/s by backend.

Key changes in `AppMode::Normal` match:
- `KeyCode::Tab` → `app.switch_backend(opposite)` (toggle)
- `KeyCode::Char('1')` → `app.switch_backend(Backend::Claude)`
- `KeyCode::Char('2')` → `app.switch_backend(Backend::Codex)`
- `KeyCode::Enter` → match `profile.backend`: Claude → `exec_claude`, Codex → `exec_codex`
- `KeyCode::Char('c')` → only works if selected profile is Claude
- `KeyCode::Char('s')` → only works if selected profile is Claude
- `KeyCode::Char('a')` → create `FormState` with `backend: app.active_backend`
- AddForm confirm: pass `backend` and `full_auto` to `NewProfile`

Update `App::new()` to accept profiles and set `active_backend: Backend::Claude`.

**Verify**: `cargo test main` — existing clap tests still pass; manual TUI verification.

---

## Step 6 — Update cli.rs for codex add flow

**File**: `src/cli.rs`
**What**: Minor — `NewProfile` now requires `backend` and `full_auto` fields. Update `run_add_with()` to pass `backend: Backend::Claude` and `full_auto: None` (CLI add always creates claude profiles; codex profiles added via TUI).

**Old**:
```
    let profile = NewProfile {
        name: name.clone(),
        description,
        base_url,
        api_key,
        model,
    };
```

**New**:
```rust
    let profile = NewProfile {
        name: name.clone(),
        description,
        backend: config::Backend::Claude,
        base_url,
        api_key,
        model,
        full_auto: None,
    };
```

**Verify**: `cargo test cli` — existing CLI tests pass.

---

## Step 7 — Proof-Read End-to-End

Read each changed file in full. Check: formatting, no leftover TODOs, spec intent preserved.

**Verify**: `cargo clippy` passes with no warnings; `cargo build` succeeds.

---

## Step 8 — Cross-Check Acceptance Criteria

| Criterion | Addressed in Step |
|-----------|------------------|
| AC1: Codex profile parsed correctly | Step 1 (Backend enum, serde default) |
| AC2: Invalid combos rejected at parse | Step 1 (validate_profiles) |
| AC3: TUI tab/group separation | Step 2 (filtered_indices), Step 3 (tab bar) |
| AC4: Codex Enter exec-replaces with codex | Step 4 (exec_codex), Step 5 (dispatch) |
| AC5: full_auto=true → --full-auto | Step 4 (build_codex_args) |
| AC6: Codex env vars injected | Step 4 (exec_codex env injection) |
| AC7: Unit tests for config + arg building | Step 1, Step 4 |
| AC8: Integration tests for flow | Step 2, Step 3, Step 5 |
| AC9: Existing claude profiles unchanged | Step 1 (serde default), Step 6 (backward compat) |

---

## Step 9 — Review

Follow Phase 3 self-review. Writes `review.md`.

---

## Step 10 — Commit

Use /commit. Suggested message:
feat: add codex backend support with tab switching UI
- Add Backend enum (claude/codex) with runtime field validation
- Add tab switching (Tab/1/2) and filtered profile navigation
- Add codex arg building and exec-replace launch
- Update TUI detail panel and add form for codex profiles

---

## Execution Order

Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 → Step 8 → Step 9 → Step 10

Steps 1 and 4 could run in parallel (config and launch are independent modules), but sequential is safer for this size of change.
