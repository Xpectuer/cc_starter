---
title: "Plan: Codex Backend Support"
doc_type: proc
brief: "Implementation plan for adding OpenAI Codex CLI backend to cct"
confidence: verified
created: 2026-03-14
updated: 2026-03-15
revision: 3
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
    pub base_url: Option<String>,
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

Update `append_profile()` to:
- Write `backend` field (if not Claude) and `full_auto` field (if Some)
- Write `base_url` as a profile-level field (not in env)
- For **Claude** backend: generate `ANTHROPIC_BASE_URL`, `ANTHROPIC_API_KEY`, `ANTHROPIC_MODEL` etc. in `[profiles.env]` (existing logic, but `base_url` now read from profile field)
- For **Codex** backend: only generate `OPENAI_API_KEY` in `[profiles.env]` (base_url goes through codex config.toml, not env)

Also update `DEFAULT_CONFIG` to include a commented codex example.

Add unit tests: codex profile deserialization, validate rejects codex+skip_perms, validate rejects claude+full_auto, backward compat (no backend field defaults to claude), base_url field round-trips.

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
- Codex: `["Name *", "Base URL", "API Key", "Model", "Full Auto (y/n)"]` — drops Description since Base URL is required for codex config.toml generation

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
**What**: Add `generate_codex_config()`, `build_codex_args()` and `exec_codex()` functions.

**Old** (after `exec_claude`):
```
/// Check if `claude` (or override via CCT_CLAUDE_BIN) is available in PATH.
```

**New** (insert before that line):
```rust
/// Check if `codex` is available in PATH.
pub fn check_codex_installed() -> bool {
    Command::new("which")
        .arg("codex")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Generate codex config.toml at ~/.config/cct-tui/codex/config.toml
/// Content is derived from the profile's name, model, and base_url fields.
/// This file is rewritten before every codex launch (multiple profiles share one file).
pub fn generate_codex_config(profile: &Profile) -> anyhow::Result<()> {
    let codex_home = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("cct-tui")
        .join("codex");
    fs::create_dir_all(&codex_home)?;

    let model = profile.model.as_deref().unwrap_or("gpt-4.1");
    let name = &profile.name;
    let base_url = profile.base_url.as_deref().unwrap_or("");

    let config_content = format!(
        "model_provider = \"custom\"\nmodel = \"{model}\"\n\n[model_providers.custom]\nname = \"{name}\"\nbase_url = \"{base_url}\"\n"
    );
    fs::write(codex_home.join("config.toml"), config_content)?;
    Ok(())
}

/// Build the CLI argument list for `codex` from a profile. Pure — no side effects.
pub fn build_codex_args(profile: &Profile) -> Vec<String> {
    let mut args = Vec::new();
    if profile.full_auto.unwrap_or(false) {
        args.push("--full-auto".to_string());
    }
    if let Some(extra) = &profile.extra_args {
        args.extend(extra.iter().cloned());
    }
    args
}

/// Generate codex config, inject profile env vars, set CODEX_HOME, and exec-replace with `codex`.
pub fn exec_codex(profile: &Profile) -> anyhow::Error {
    if !check_codex_installed() {
        return anyhow::anyhow!(
            "codex CLI not found in PATH. Install it first: npm install -g @openai/codex"
        );
    }

    if let Err(e) = generate_codex_config(profile) {
        return anyhow::anyhow!("failed to generate codex config: {e}");
    }

    let codex_home = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("cct-tui")
        .join("codex");
    env::set_var("CODEX_HOME", &codex_home);

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

Note: `build_codex_args` does NOT include `--model` because codex reads model from config.toml (via `CODEX_HOME`).

Add unit tests: `build_codex_args` with empty, full_auto, extra_args combinations; `generate_codex_config` writes correct content to temp dir.

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
- AddForm confirm: map fields based on `form.backend`:
  - Claude: fields[0]=name, [1]=description, [2]=base_url, [3]=api_key, [4]=model
  - Codex: fields[0]=name, [1]=base_url, [2]=api_key, [3]=model, [4]=full_auto (parse "y"/"yes" → true)

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
| AC4: Codex Enter generates config.toml then exec-replaces | Step 4 (generate_codex_config, exec_codex), Step 5 (dispatch) |
| AC5: full_auto=true → --full-auto | Step 4 (build_codex_args) |
| AC6: CODEX_HOME auto-set, OPENAI_API_KEY injected | Step 4 (exec_codex env setup) |
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
