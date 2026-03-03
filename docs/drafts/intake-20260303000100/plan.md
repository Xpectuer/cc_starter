---
title: "Plan: Config Add Functionality for cct"
doc_type: proc
brief: "Implementation plan for CLI and TUI profile add flows"
confidence: verified
created: 2026-03-03
updated: 2026-03-03
revision: 2
---

# Plan: Config Add Functionality for cct (Revision 2)

Spec: [./spec.md](./spec.md)
Requirements: [./requirements.md](./requirements.md)

> **Note**: Revision 1 (steps 1–13) is complete. This revision extends the add flow
> with `base_url`, `api_key` fields and auto-populated `[profiles.env]` env vars.
> All steps below are **delta edits** on top of the existing implementation.

## Files Changed

| File | Change Type |
|------|-------------|
| `src/config.rs` | Edit — extend `NewProfile`, rewrite `append_profile()` to emit `[profiles.env]` |
| `src/app.rs` | Edit — expand `FormState` from 3→5 fields, update `FIELD_LABELS`, adjust nav bounds |
| `src/cli.rs` | Edit — add base_url/api_key prompts, update summary, update `NewProfile` construction |
| `src/ui.rs` | Edit — render 5 form fields, update confirmation summary with base_url/api_key |
| `src/main.rs` | Edit — pass 5 fields to `NewProfile` in TUI AddForm save logic |

## Step 1 — Extend NewProfile and append_profile in config.rs

**File**: `src/config.rs`
**What**: Add `base_url` and `api_key` to `NewProfile`. Rewrite `append_profile()` to generate `[profiles.env]` section with auto-populated env vars when model/base_url/api_key are provided.

**Old**:
```rust
pub struct NewProfile {
    pub name: String,
    pub description: Option<String>,
    pub model: Option<String>,
}
```

**New**:
```rust
pub struct NewProfile {
    pub name: String,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
}
```

**Old** (`append_profile`):
```rust
pub fn append_profile(profile: &NewProfile) -> Result<()> {
    let path = config_path();
    let mut block = String::from("\n[[profiles]]\n");
    block.push_str(&format!("name = {:?}\n", profile.name));
    if let Some(desc) = &profile.description {
        if !desc.is_empty() {
            block.push_str(&format!("description = {:?}\n", desc));
        }
    }
    if let Some(model) = &profile.model {
        if !model.is_empty() {
            block.push_str(&format!("model = {:?}\n", model));
        }
    }
    let mut content = fs::read_to_string(&path)
        .with_context(|| format!("read config {path:?}"))?;
    content.push_str(&block);
    fs::write(&path, content)
        .with_context(|| format!("write config {path:?}"))?;
    Ok(())
}
```

**New** (`append_profile`):
```rust
pub fn append_profile(profile: &NewProfile) -> Result<()> {
    let path = config_path();
    let mut block = String::from("\n[[profiles]]\n");
    block.push_str(&format!("name = {:?}\n", profile.name));
    if let Some(desc) = &profile.description {
        if !desc.is_empty() {
            block.push_str(&format!("description = {:?}\n", desc));
        }
    }

    // Build [profiles.env] entries
    let mut env_lines: Vec<String> = Vec::new();
    if let Some(url) = &profile.base_url {
        if !url.is_empty() {
            env_lines.push(format!("ANTHROPIC_BASE_URL = {:?}", url));
        }
    }
    if let Some(key) = &profile.api_key {
        if !key.is_empty() {
            env_lines.push(format!("ANTHROPIC_API_KEY = {:?}", key));
        }
    }
    if let Some(model) = &profile.model {
        if !model.is_empty() {
            env_lines.push(format!("ANTHROPIC_MODEL = {:?}", model));
            env_lines.push(format!("ANTHROPIC_SMALL_FAST_MODEL = {:?}", model));
            env_lines.push(format!("ANTHROPIC_DEFAULT_SONNET_MODEL = {:?}", model));
            env_lines.push(format!("ANTHROPIC_DEFAULT_OPUS_MODEL = {:?}", model));
            env_lines.push(format!("ANTHROPIC_DEFAULT_HAIKU_MODEL = {:?}", model));
            env_lines.push("API_TIMEOUT_MS = \"600000\"".to_string());
            env_lines.push("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = \"1\"".to_string());
        }
    }

    if !env_lines.is_empty() {
        block.push_str("\n[profiles.env]\n");
        for line in &env_lines {
            block.push_str(line);
            block.push('\n');
        }
    }

    let mut content = fs::read_to_string(&path)
        .with_context(|| format!("read config {path:?}"))?;
    content.push_str(&block);
    fs::write(&path, content)
        .with_context(|| format!("write config {path:?}"))?;
    Ok(())
}
```

**Verify**: `cargo test --lib config 2>&1 | tail -10` — all existing tests still pass (after updating `NewProfile` construction sites in tests to add `base_url: None, api_key: None`).

## Step 2 — Update existing config tests + add new env var tests

**File**: `src/config.rs`
**What**: Update all existing `NewProfile` construction in tests to include `base_url: None, api_key: None`. Add new tests for env var generation.

All existing test `NewProfile { ... }` constructions gain two new fields:
```rust
// Before:
NewProfile { name: "x".into(), description: None, model: None }
// After:
NewProfile { name: "x".into(), description: None, base_url: None, api_key: None, model: None }
```

**New tests** to add:

```rust
    #[test]
    #[serial]
    fn append_profile_with_env_vars() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, DEFAULT_CONFIG).unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let new = NewProfile {
            name: "third-party".into(),
            description: Some("Third-party backend".into()),
            base_url: Some("https://api.example.com/v1".into()),
            api_key: Some("sk-test-key".into()),
            model: Some("MiniMax-M2.1".into()),
        };
        append_profile(&new).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("[profiles.env]"));
        assert!(content.contains("ANTHROPIC_BASE_URL = \"https://api.example.com/v1\""));
        assert!(content.contains("ANTHROPIC_API_KEY = \"sk-test-key\""));
        assert!(content.contains("ANTHROPIC_MODEL = \"MiniMax-M2.1\""));
        assert!(content.contains("ANTHROPIC_SMALL_FAST_MODEL = \"MiniMax-M2.1\""));
        assert!(content.contains("ANTHROPIC_DEFAULT_SONNET_MODEL = \"MiniMax-M2.1\""));
        assert!(content.contains("ANTHROPIC_DEFAULT_OPUS_MODEL = \"MiniMax-M2.1\""));
        assert!(content.contains("ANTHROPIC_DEFAULT_HAIKU_MODEL = \"MiniMax-M2.1\""));
        assert!(content.contains("API_TIMEOUT_MS = \"600000\""));
        assert!(content.contains("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = \"1\""));

        // Verify it round-trips through load_profiles
        let profiles = load_profiles().unwrap();
        assert_eq!(profiles.len(), 2);
        let p = profiles.iter().find(|p| p.name == "third-party").unwrap();
        let env = p.env.as_ref().unwrap();
        assert_eq!(env.get("ANTHROPIC_MODEL").map(String::as_str), Some("MiniMax-M2.1"));
        assert_eq!(env.get("ANTHROPIC_BASE_URL").map(String::as_str), Some("https://api.example.com/v1"));

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    #[serial]
    fn append_profile_base_url_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, DEFAULT_CONFIG).unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let new = NewProfile {
            name: "url-only".into(),
            description: None,
            base_url: Some("https://api.example.com".into()),
            api_key: None,
            model: None,
        };
        append_profile(&new).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("[profiles.env]"));
        assert!(content.contains("ANTHROPIC_BASE_URL"));
        // No model env vars when model not provided
        assert!(!content.contains("ANTHROPIC_MODEL"));

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    #[serial]
    fn append_minimal_no_env_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, DEFAULT_CONFIG).unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let new = NewProfile {
            name: "bare".into(),
            description: None,
            base_url: None,
            api_key: None,
            model: None,
        };
        append_profile(&new).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        // No [profiles.env] when nothing to populate
        assert!(!content.contains("[profiles.env]\nANTHROPIC"));

        std::env::remove_var("CCT_CONFIG");
    }
```

**Verify**: `cargo test --lib config 2>&1 | tail -10` — all tests pass.

## Step 3 — Expand FormState in app.rs

**File**: `src/app.rs`
**What**: Expand `FIELD_LABELS` from 3→5, `FormState.fields` from `[String; 3]` to `[String; 5]`, navigation bounds from `2` to `4`.

**Old**:
```rust
pub const FIELD_LABELS: [&str; 3] = ["Name *", "Description", "Model"];
```

**New**:
```rust
pub const FIELD_LABELS: [&str; 5] = ["Name *", "Description", "Base URL", "API Key", "Model"];
```

**Old** (`FormState`):
```rust
pub struct FormState {
    pub fields: [String; 3],
    pub active_field: usize,
    pub confirming: bool,
    pub error: Option<String>,
}
```

**New**:
```rust
pub struct FormState {
    pub fields: [String; 5],
    pub active_field: usize,
    pub confirming: bool,
    pub error: Option<String>,
}
```

**Old** (`FormState::new()`):
```rust
    pub fn new() -> Self {
        Self {
            fields: [String::new(), String::new(), String::new()],
            active_field: 0,
            confirming: false,
            error: None,
        }
    }
```

**New**:
```rust
    pub fn new() -> Self {
        Self {
            fields: [String::new(), String::new(), String::new(), String::new(), String::new()],
            active_field: 0,
            confirming: false,
            error: None,
        }
    }
```

**Old** (`next_field` / `prev_field`):
```rust
    pub fn next_field(&mut self) {
        self.active_field = (self.active_field + 1).min(2);
    }
```

**New**:
```rust
    pub fn next_field(&mut self) {
        self.active_field = (self.active_field + 1).min(4);
    }
```

**Verify**: `cargo test --lib app 2>&1 | tail -5` — update test assertions for 5 fields and max=4, then pass.

## Step 4 — Update cli.rs prompts

**File**: `src/cli.rs`
**What**: Add `Base URL` and `API Key` prompts between Description and Model. Update summary. Update `NewProfile` construction.

**Old** (after Description prompt, before Model prompt):
```rust
    // Model (optional)
    write!(writer, "Model (optional): ")?;
```

**New** (insert between Description and Model):
```rust
    // Base URL (optional)
    write!(writer, "Base URL (optional): ")?;
    writer.flush()?;
    let mut url_line = String::new();
    reader.read_line(&mut url_line)?;
    let base_url = {
        let t = url_line.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    };

    // API Key (optional)
    write!(writer, "API Key (optional): ")?;
    writer.flush()?;
    let mut key_line = String::new();
    reader.read_line(&mut key_line)?;
    let api_key = {
        let t = key_line.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    };

    // Model (optional)
    write!(writer, "Model (optional): ")?;
```

**Old** (summary):
```rust
    writeln!(writer, "  Name:        {}", name)?;
    writeln!(writer, "  Description: {}", description.as_deref().unwrap_or("(none)"))?;
    writeln!(writer, "  Model:       {}", model.as_deref().unwrap_or("(none)"))?;
```

**New** (summary — mask API key):
```rust
    writeln!(writer, "  Name:        {}", name)?;
    writeln!(writer, "  Description: {}", description.as_deref().unwrap_or("(none)"))?;
    writeln!(writer, "  Base URL:    {}", base_url.as_deref().unwrap_or("(none)"))?;
    writeln!(writer, "  API Key:     {}", api_key.as_ref().map(|k| mask_key(k)).unwrap_or_else(|| "(none)".into()))?;
    writeln!(writer, "  Model:       {}", model.as_deref().unwrap_or("(none)"))?;
```

Add helper at top of file (after imports):
```rust
fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        "*".repeat(key.len())
    } else {
        format!("{}...{}", &key[..4], &key[key.len()-4..])
    }
}
```

**Old** (`NewProfile` construction):
```rust
    let profile = NewProfile {
        name: name.clone(),
        description,
        model,
    };
```

**New**:
```rust
    let profile = NewProfile {
        name: name.clone(),
        description,
        base_url,
        api_key,
        model,
    };
```

**Verify**: `cargo test --lib cli 2>&1 | tail -5` — update test input to include 2 new fields, then pass.

## Step 5 — Update ui.rs form rendering

**File**: `src/ui.rs`
**What**: Update `build_form_lines` confirmation summary to show 5 fields. Mask API key in confirmation view.

**Old** (confirmation section in `build_form_lines`):
```rust
    if form.confirming {
        lines.push(Line::from("Save this profile?").style(Style::default().add_modifier(Modifier::BOLD)));
        lines.push(Line::from(""));
        lines.push(Line::from(format!("  Name:        {}", form.fields[0])));
        lines.push(Line::from(format!(
            "  Description: {}",
            if form.fields[1].is_empty() { "(none)" } else { &form.fields[1] }
        )));
        lines.push(Line::from(format!(
            "  Model:       {}",
            if form.fields[2].is_empty() { "(none)" } else { &form.fields[2] }
        )));
```

**New**:
```rust
    if form.confirming {
        lines.push(Line::from("Save this profile?").style(Style::default().add_modifier(Modifier::BOLD)));
        lines.push(Line::from(""));
        lines.push(Line::from(format!("  Name:        {}", form.fields[0])));
        lines.push(Line::from(format!(
            "  Description: {}",
            if form.fields[1].is_empty() { "(none)" } else { &form.fields[1] }
        )));
        lines.push(Line::from(format!(
            "  Base URL:    {}",
            if form.fields[2].is_empty() { "(none)" } else { &form.fields[2] }
        )));
        lines.push(Line::from(format!(
            "  API Key:     {}",
            if form.fields[3].is_empty() { "(none)".to_string() } else { mask_value("API_KEY", &form.fields[3]) }
        )));
        lines.push(Line::from(format!(
            "  Model:       {}",
            if form.fields[4].is_empty() { "(none)" } else { &form.fields[4] }
        )));
```

The existing `mask_value` function in `ui.rs` already handles masking for keys containing "KEY" — reuse it here.

**Verify**: `cargo check 2>&1 | tail -3` — compiles.

## Step 6 — Update main.rs TUI AddForm save logic

**File**: `src/main.rs`
**What**: Update the AddForm confirmation handler to read 5 fields and pass them to `NewProfile`.

**Old** (inside `KeyCode::Char('y')` handler):
```rust
                                let desc = form.fields[1].trim().to_string();
                                let model = form.fields[2].trim().to_string();
                                let new_profile = config::NewProfile {
                                    name,
                                    description: if desc.is_empty() { None } else { Some(desc) },
                                    model: if model.is_empty() { None } else { Some(model) },
                                };
```

**New**:
```rust
                                let desc = form.fields[1].trim().to_string();
                                let base_url = form.fields[2].trim().to_string();
                                let api_key = form.fields[3].trim().to_string();
                                let model = form.fields[4].trim().to_string();
                                let new_profile = config::NewProfile {
                                    name,
                                    description: if desc.is_empty() { None } else { Some(desc) },
                                    base_url: if base_url.is_empty() { None } else { Some(base_url) },
                                    api_key: if api_key.is_empty() { None } else { Some(api_key) },
                                    model: if model.is_empty() { None } else { Some(model) },
                                };
```

Also update the Enter-to-advance threshold:

**Old**:
```rust
                            KeyCode::Enter => {
                                if form.active_field < 2 {
                                    form.next_field();
                                } else {
                                    form.confirming = true;
                                }
                            }
```

**New**:
```rust
                            KeyCode::Enter => {
                                if form.active_field < 4 {
                                    form.next_field();
                                } else {
                                    form.confirming = true;
                                }
                            }
```

**Verify**: `cargo check 2>&1 | tail -3` — compiles.

## Step 7 — Update cli.rs test

**File**: `src/cli.rs`
**What**: Update existing `cli_run_add_rejects_duplicate` test to provide 5 fields in piped stdin.

**Old**:
```rust
        let input = b"newprofile\nmy desc\nsonnet\ny\n";
```

**New**:
```rust
        let input = b"newprofile\nmy desc\nhttps://api.example.com\nsk-test\nMiniMax-M2.1\ny\n";
```

Also verify the generated env vars:
```rust
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("[profiles.env]"));
        assert!(content.contains("ANTHROPIC_BASE_URL"));
        assert!(content.contains("ANTHROPIC_MODEL = \"MiniMax-M2.1\""));
```

**Verify**: `cargo test --lib cli 2>&1 | tail -5` — pass.

## Step 8 — Update app.rs tests

**File**: `src/app.rs`
**What**: Update `form_state_field_navigation` test for 5 fields (max = 4). Update `app_mode_transitions` test.

**Old**:
```rust
        form.next_field();
        assert_eq!(form.active_field, 2);

        // Should clamp at max (2)
        form.next_field();
        assert_eq!(form.active_field, 2);
```

**New**:
```rust
        form.next_field();
        assert_eq!(form.active_field, 2);

        form.next_field();
        assert_eq!(form.active_field, 3);

        form.next_field();
        assert_eq!(form.active_field, 4);

        // Should clamp at max (4)
        form.next_field();
        assert_eq!(form.active_field, 4);
```

**Verify**: `cargo test --lib app 2>&1 | tail -5` — pass.

## Step 9 — Proof-Read End-to-End

Read each changed file in full. Check: formatting, no leftover TODOs, spec intent preserved, all `NewProfile` construction sites updated.

## Step 10 — Cross-Check Acceptance Criteria

| Criterion | Addressed in Step |
|-----------|------------------|
| `cct add` prompts for name, description, base_url, api_key, model interactively | Step 4 |
| `cct add` rejects duplicate profile names | Existing (unchanged) |
| `cct add` shows summary with masked API key | Step 4 |
| `cct add` appends `[[profiles]]` + `[profiles.env]` | Step 1 |
| Pressing `a` opens inline form for 5 fields | Steps 3, 5, 6 |
| In-TUI form validates duplicate names | Existing (unchanged) |
| In-TUI form shows confirmation with 5 fields | Step 5 |
| After saving, cursor auto-selects new profile | Existing (unchanged) |
| Existing profiles not corrupted | Step 1 (preserves append strategy) |
| Model provided → 5 model env vars + 2 static defaults auto-populated | Step 1 |
| base_url → `ANTHROPIC_BASE_URL` in env | Step 1 |
| api_key → `ANTHROPIC_API_KEY` in env | Step 1 |
| No `[profiles.env]` when no env fields provided | Step 1, Step 2 test |

All criteria mapped.

## Step 11 — Review & Commit

Review changes, then commit with:
```
feat: add base_url, api_key fields and auto-populate env vars in profile add flow

- Extend add flow (CLI + TUI) with base_url, api_key input fields
- Auto-populate model env vars when model is specified
- Generate [profiles.env] section with ANTHROPIC_BASE_URL, ANTHROPIC_API_KEY,
  ANTHROPIC_MODEL, ANTHROPIC_SMALL_FAST_MODEL, ANTHROPIC_DEFAULT_SONNET_MODEL,
  ANTHROPIC_DEFAULT_OPUS_MODEL, ANTHROPIC_DEFAULT_HAIKU_MODEL, API_TIMEOUT_MS,
  CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC
- Mask API key in CLI summary and TUI confirmation
```

## Execution Order

Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 → Step 8 → Step 9 → Step 10 → Step 11

Steps 1-2 (config) and Step 3 (app) are parallel-safe.
Steps 4 (cli) and Step 5 (ui) are parallel-safe after Step 3.
Step 6 (main) depends on Steps 1 + 3.
