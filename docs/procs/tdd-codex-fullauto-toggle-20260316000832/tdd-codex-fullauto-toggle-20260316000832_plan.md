---
title: "Plan: Codex full_auto toggle via [s] key"
doc_type: proc
brief: "Add toggle_full_auto to config, extend s key handler for Codex, dynamic footer"
confidence: verified
created: 2026-03-16
updated: 2026-03-16
revision: 1
---

# Plan: Codex full_auto toggle via [s] key

## Files Changed

| File | Change Type |
|------|-------------|
| `src/config.rs` | Minor edit — add `toggle_full_auto()` function + 3 unit tests |
| `src/main.rs` | Minor edit — extend `s` key handler to dispatch by backend |
| `src/ui.rs` | Minor edit — make footer `s` hint backend-aware |

## Step 1 — Add `toggle_full_auto()` to config.rs

**File**: `src/config.rs`
**What**: Add `toggle_full_auto` function mirroring `toggle_skip_permissions`.

**Old** (insert after `toggle_skip_permissions` function, before `#[cfg(test)]`):
```
#[cfg(test)]
mod tests {
```

**New**:
```rust
/// Toggle `full_auto` for a named Codex profile in the config file.
/// Uses toml_edit for surgical edits that preserve comments and formatting.
pub fn toggle_full_auto(profile_name: &str, new_value: bool) -> Result<()> {
    let path = config_path();
    let content = fs::read_to_string(&path).with_context(|| format!("read config {path:?}"))?;
    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("parse TOML in {path:?}"))?;

    let profiles = doc
        .get_mut("profiles")
        .and_then(|v| v.as_array_of_tables_mut())
        .with_context(|| "no [[profiles]] array in config")?;

    let entry = profiles
        .iter_mut()
        .find(|t| t.get("name").and_then(|v| v.as_str()) == Some(profile_name))
        .with_context(|| format!("profile {profile_name:?} not found in config"))?;

    entry["full_auto"] = toml_edit::value(new_value);
    fs::write(&path, doc.to_string()).with_context(|| format!("write config {path:?}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
```

**Verify**: `grep -c "fn toggle_full_auto" src/config.rs` should return `1`.

## Step 2 — Add unit tests for `toggle_full_auto` in config.rs

**File**: `src/config.rs`
**What**: Add 3 tests mirroring the `toggle_skip_permissions` test pattern.

**Old** (insert before closing `}` of `mod tests`):
```
    #[test]
    #[serial]
    fn append_codex_profile_generates_openai_env() {
```

**New**:
```rust
    // --- toggle_full_auto tests ---

    #[test]
    #[serial]
    fn toggle_full_auto_insert() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, "# comment\n[[profiles]]\nname = \"codex-test\"\nbackend = \"codex\"\n").unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        toggle_full_auto("codex-test", true).unwrap();
        let profiles = load_profiles().unwrap();
        assert_eq!(profiles[0].full_auto, Some(true));

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# comment"), "comment should be preserved");

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    #[serial]
    fn toggle_full_auto_flip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(
            &path,
            "[[profiles]]\nname = \"codex-test\"\nbackend = \"codex\"\nfull_auto = true\n",
        )
        .unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        toggle_full_auto("codex-test", false).unwrap();
        let profiles = load_profiles().unwrap();
        assert_eq!(profiles[0].full_auto, Some(false));

        toggle_full_auto("codex-test", true).unwrap();
        let profiles = load_profiles().unwrap();
        assert_eq!(profiles[0].full_auto, Some(true));

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    #[serial]
    fn toggle_full_auto_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, "[[profiles]]\nname = \"other\"\nbackend = \"codex\"\n").unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let result = toggle_full_auto("missing", true);
        assert!(result.is_err());

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    #[serial]
    fn append_codex_profile_generates_openai_env() {
```

**Verify**: `cargo test toggle_full_auto -- --test-threads=1` should pass 3 tests.

## Step 3 — Extend `s` key handler in main.rs

**File**: `src/main.rs`
**What**: Dispatch `s` key by backend — Claude toggles `skip_permissions`, Codex toggles `full_auto`.

**Old**:
```rust
                    (KeyCode::Char('s'), _) if !app.profiles.is_empty() => {
                        let profile = &app.profiles[app.selected];
                        if profile.backend == config::Backend::Claude {
                            let profile = &mut app.profiles[app.selected];
                            let old_val = profile.skip_permissions.unwrap_or(false);
                            let new_val = !old_val;
                            match config::toggle_skip_permissions(&profile.name, new_val) {
                                Ok(()) => {
                                    profile.skip_permissions = Some(new_val);
                                }
                                Err(e) => {
                                    eprintln!("Warning: toggle failed: {e:#}");
                                }
                            }
                        }
                    }
```

**New**:
```rust
                    (KeyCode::Char('s'), _) if !app.profiles.is_empty() => {
                        let profile = &mut app.profiles[app.selected];
                        match profile.backend {
                            config::Backend::Claude => {
                                let old_val = profile.skip_permissions.unwrap_or(false);
                                let new_val = !old_val;
                                match config::toggle_skip_permissions(&profile.name, new_val) {
                                    Ok(()) => {
                                        profile.skip_permissions = Some(new_val);
                                    }
                                    Err(e) => {
                                        eprintln!("Warning: toggle failed: {e:#}");
                                    }
                                }
                            }
                            config::Backend::Codex => {
                                let old_val = profile.full_auto.unwrap_or(false);
                                let new_val = !old_val;
                                match config::toggle_full_auto(&profile.name, new_val) {
                                    Ok(()) => {
                                        profile.full_auto = Some(new_val);
                                    }
                                    Err(e) => {
                                        eprintln!("Warning: toggle failed: {e:#}");
                                    }
                                }
                            }
                        }
                    }
```

**Verify**: `grep -A5 "Char('s')" src/main.rs | grep -c "toggle_full_auto"` should return `1`.

## Step 4 — Make footer backend-aware in ui.rs

**File**: `src/ui.rs`
**What**: Change the Normal-mode footer to show backend-specific `s` key hint.

**Old**:
```rust
    let footer_text = match &app.mode {
        AppMode::Normal => {
            " [Tab/1/2] Backend  [↑↓/jk] Navigate  [Enter] Launch  [c] Resume  [s] Skip-perms  [a] Add  [e] Edit config  [q] Quit"
        }
```

**New**:
```rust
    let footer_text = match &app.mode {
        AppMode::Normal => match app.active_backend {
            Backend::Claude => {
                " [Tab/1/2] Backend  [↑↓/jk] Navigate  [Enter] Launch  [c] Resume  [s] Skip-perms  [a] Add  [e] Edit config  [q] Quit"
            }
            Backend::Codex => {
                " [Tab/1/2] Backend  [↑↓/jk] Navigate  [Enter] Launch  [s] Full-auto  [a] Add  [e] Edit config  [q] Quit"
            }
        },
```

**Verify**: `grep "Full-auto" src/ui.rs | wc -l` should return at least `1`.

## Step 5 — Update footer test in ui.rs

**File**: `src/ui.rs`
**What**: Update the existing `ui_footer_shows_add_hint` test to cover both backends.

**Old**:
```rust
    #[test]
    fn ui_footer_shows_add_hint() {
        let normal_footer =
            " [Tab/1/2] Backend  [↑↓/jk] Navigate  [Enter] Launch  [c] Resume  [s] Skip-perms  [a] Add  [e] Edit config  [q] Quit";
        assert!(normal_footer.contains("[a] Add"));
        assert!(normal_footer.contains("[s] Skip-perms"));
        assert!(normal_footer.contains("[c] Resume"));
        assert!(normal_footer.contains("[Tab/1/2] Backend"));
    }
```

**New**:
```rust
    #[test]
    fn ui_footer_shows_add_hint() {
        // Claude footer
        let claude_footer =
            " [Tab/1/2] Backend  [↑↓/jk] Navigate  [Enter] Launch  [c] Resume  [s] Skip-perms  [a] Add  [e] Edit config  [q] Quit";
        assert!(claude_footer.contains("[a] Add"));
        assert!(claude_footer.contains("[s] Skip-perms"));
        assert!(claude_footer.contains("[c] Resume"));
        assert!(claude_footer.contains("[Tab/1/2] Backend"));

        // Codex footer
        let codex_footer =
            " [Tab/1/2] Backend  [↑↓/jk] Navigate  [Enter] Launch  [s] Full-auto  [a] Add  [e] Edit config  [q] Quit";
        assert!(codex_footer.contains("[a] Add"));
        assert!(codex_footer.contains("[s] Full-auto"));
        assert!(!codex_footer.contains("[c] Resume"), "Codex footer should not show Resume");
    }
```

**Verify**: `cargo test ui_footer_shows_add_hint` should pass.

## Step 6 — Proof-Read End-to-End

Read each changed file in full. Check: formatting, no leftover TODOs, spec intent preserved.

## Step 7 — Cross-Check Acceptance Criteria

| Criterion | Addressed in Step |
|-----------|------------------|
| `toggle_full_auto()` persists to TOML | Step 1 |
| Codex tab `s` key toggles full_auto, detail panel refreshes | Step 3 |
| Claude tab `s` key unchanged | Step 3 |
| Footer shows backend-specific hint | Step 4 |
| `toggle_full_auto` uses `toml_edit`, preserves comments | Step 1 |
| All existing tests pass | Step 2, 5 |

## Step 8 — Review

Follow Phase 3 self-review. Writes `review.md`.

## Step 9 — Commit

Use /commit. Suggested message:
feat: press [s] to toggle full_auto on Codex profiles
- Add config::toggle_full_auto() mirroring toggle_skip_permissions
- Extend s key handler in TUI to dispatch by backend
- Make footer hint backend-aware (skip_perms vs full_auto)

## Execution Order

Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 → Step 8 → Step 9
(Steps 1-2 are config.rs changes; Steps 3 is main.rs; Steps 4-5 are ui.rs — these groups could run in parallel but sequential is simpler)
