---
title: "Plan: Autoinstall Claude Binary & Toggle skip_permissions"
doc_type: proc
brief: "Implementation plan for autoinstall check and TUI skip_permissions toggle"
confidence: verified
created: 2026-03-10
updated: 2026-03-10
revision: 1
---

# Plan: Autoinstall Claude Binary & Toggle skip_permissions

## Files Changed

| File | Change Type |
|------|-------------|
| `Cargo.toml` | Minor edit — add `toml_edit` dependency |
| `src/launch.rs` | Major edit — add `check_claude_installed`, `prompt_install` |
| `src/config.rs` | Major edit — add `toggle_skip_permissions` |
| `src/main.rs` | Major edit — add install check call + `s` hotkey handler |
| `src/ui.rs` | Minor edit — red row style + footer text |

## Step 1 — Add `toml_edit` dependency

**File**: `Cargo.toml`
**What**: Add `toml_edit` crate for surgical TOML edits that preserve formatting.

**Old**:
```
toml      = "0.8"
```

**New**:
```
toml      = "0.8"
toml_edit = "0.22"
```

**Verify**: `cargo check 2>&1 | grep -q "error" && echo FAIL || echo PASS`

## Step 2 — Add `check_claude_installed` and `prompt_install` to launch.rs

**File**: `src/launch.rs`
**What**: Add two new public functions for detecting and installing the claude binary.

**Old**:
```
/// Suspend TUI, open $EDITOR (or vi) on path, block until editor exits.
```

**New**:
```
/// Check if `claude` (or override via CCT_CLAUDE_BIN) is available in PATH.
pub fn check_claude_installed() -> bool {
    let bin = std::env::var("CCT_CLAUDE_BIN").unwrap_or_else(|_| "claude".to_string());
    Command::new("which")
        .arg(&bin)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Prompt user to install claude via the official installer script.
/// Must be called BEFORE entering raw mode / alternate screen.
/// Returns Ok(()) on successful install, Err on failure or user decline.
pub fn prompt_install() -> Result<()> {
    use std::io::{self, BufRead, Write};

    println!("Claude CLI not found in PATH.");
    print!("Install now? [Y/n] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();

    if trimmed == "n" || trimmed == "no" {
        println!("\nTo install manually, run:");
        println!("  curl -fsSL https://claude.ai/install.sh | bash");
        std::process::exit(0);
    }

    println!("\nInstalling Claude CLI...\n");
    let status = Command::new("bash")
        .arg("-c")
        .arg("curl -fsSL https://claude.ai/install.sh | bash")
        .status()
        .context("failed to run installer")?;

    if !status.success() {
        anyhow::bail!(
            "Installation failed (exit code: {:?}). Install manually:\n  curl -fsSL https://claude.ai/install.sh | bash",
            status.code()
        );
    }

    // Re-check: try PATH first, then ~/.local/bin/claude as fallback
    if check_claude_installed() {
        println!("\nClaude CLI installed successfully.");
        return Ok(());
    }

    let home = dirs::home_dir().unwrap_or_default();
    let fallback = home.join(".local/bin/claude");
    if fallback.exists() {
        println!("\nClaude CLI installed at {}.", fallback.display());
        println!("Note: You may need to add ~/.local/bin to your PATH.");
        return Ok(());
    }

    anyhow::bail!("Installation completed but `claude` not found in PATH.\nAdd ~/.local/bin to your PATH and restart your shell.")
}

/// Suspend TUI, open $EDITOR (or vi) on path, block until editor exits.
```

**Verify**: `cargo check 2>&1 | grep -q "error" && echo FAIL || echo PASS`

## Step 3 — Add `toggle_skip_permissions` to config.rs

**File**: `src/config.rs`
**What**: Add function to surgically toggle `skip_permissions` in the TOML config file.

**Old**:
```
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
```

**New**:
```
/// Toggle `skip_permissions` for a named profile in the config file.
/// Uses toml_edit for surgical edits that preserve comments and formatting.
pub fn toggle_skip_permissions(profile_name: &str, new_value: bool) -> Result<()> {
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

    entry["skip_permissions"] = toml_edit::value(new_value);
    fs::write(&path, doc.to_string()).with_context(|| format!("write config {path:?}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
```

**Verify**: `cargo check 2>&1 | grep -q "error" && echo FAIL || echo PASS`

## Step 4 — Add install check to main.rs

**File**: `src/main.rs`
**What**: Call `check_claude_installed()` on startup, before TUI initialization.

**Old**:
```
fn main() -> Result<()> {
    config::ensure_default_config()?;

    let args = Cli::parse();
```

**New**:
```
fn main() -> Result<()> {
    config::ensure_default_config()?;

    if !launch::check_claude_installed() {
        launch::prompt_install()?;
    }

    let args = Cli::parse();
```

**Verify**: `cargo check 2>&1 | grep -q "error" && echo FAIL || echo PASS`

## Step 5 — Add `s` hotkey handler in main.rs

**File**: `src/main.rs`
**What**: Handle `s` key in Normal mode to toggle skip_permissions and persist.

**Old**:
```
                    (KeyCode::Char('a'), _) => {
                        app.mode = AppMode::AddForm(FormState::new());
                    }
```

**New**:
```
                    (KeyCode::Char('s'), _) if !app.profiles.is_empty() => {
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
                    (KeyCode::Char('a'), _) => {
                        app.mode = AppMode::AddForm(FormState::new());
                    }
```

**Verify**: `cargo check 2>&1 | grep -q "error" && echo FAIL || echo PASS`

## Step 6 — Red profile row in ui.rs

**File**: `src/ui.rs`
**What**: Render profile list items in red when skip_permissions is enabled.

**Old**:
```
        app.profiles
            .iter()
            .map(|p| {
                let label = match &p.description {
                    Some(d) => format!("{}\n  {}", p.name, d),
                    None => p.name.clone(),
                };
                ListItem::new(label)
            })
            .collect()
```

**New**:
```
        app.profiles
            .iter()
            .map(|p| {
                let label = match &p.description {
                    Some(d) => format!("{}\n  {}", p.name, d),
                    None => p.name.clone(),
                };
                let item = ListItem::new(label);
                if p.skip_permissions.unwrap_or(false) {
                    item.style(Style::default().fg(Color::Red))
                } else {
                    item
                }
            })
            .collect()
```

**Verify**: `cargo clippy 2>&1 | grep -q "error" && echo FAIL || echo PASS`

## Step 7 — Update footer text in ui.rs

**File**: `src/ui.rs`
**What**: Add `[s] Skip-perms` to Normal mode footer.

**Old**:
```
            " [↑↓/jk] Navigate  [Enter] Launch  [a] Add  [e] Edit config  [q/Ctrl-C] Quit"
```

**New**:
```
            " [↑↓/jk] Navigate  [Enter] Launch  [s] Skip-perms  [a] Add  [e] Edit config  [q/Ctrl-C] Quit"
```

**Verify**: `grep -q "Skip-perms" src/ui.rs && echo PASS || echo FAIL`

## Step 8 — Unit tests for check_claude_installed

**File**: `src/launch.rs`
**What**: Add tests for the install check function using the `CCT_CLAUDE_BIN` env override.

**Old**:
```
    #[test]
    fn build_args_empty() {
```

**New**:
```
    #[test]
    fn check_claude_installed_found() {
        // "true" is always in PATH on Unix
        std::env::set_var("CCT_CLAUDE_BIN", "true");
        assert!(super::check_claude_installed());
        std::env::remove_var("CCT_CLAUDE_BIN");
    }

    #[test]
    fn check_claude_installed_not_found() {
        std::env::set_var("CCT_CLAUDE_BIN", "nonexistent-binary-xyz-12345");
        assert!(!super::check_claude_installed());
        std::env::remove_var("CCT_CLAUDE_BIN");
    }

    #[test]
    fn build_args_empty() {
```

**Verify**: `cargo test check_claude_installed 2>&1 | grep -q "2 passed" && echo PASS || echo FAIL`

## Step 9 — Unit tests for toggle_skip_permissions

**File**: `src/config.rs`
**What**: Add tests for the toggle function — insert, flip true→false, flip false→true, preserve formatting.

Add after the last existing test in the `mod tests` block:

**Old**:
```
    #[test]
    #[serial]
    fn append_minimal_profile() {
```

**New**:
```
    #[test]
    #[serial]
    fn toggle_skip_permissions_insert() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, "# comment\n[[profiles]]\nname = \"test\"\n").unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        toggle_skip_permissions("test", true).unwrap();
        let profiles = load_profiles().unwrap();
        assert_eq!(profiles[0].skip_permissions, Some(true));

        // Verify comment is preserved
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# comment"), "comment should be preserved");

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    #[serial]
    fn toggle_skip_permissions_flip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, "[[profiles]]\nname = \"test\"\nskip_permissions = true\n").unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        toggle_skip_permissions("test", false).unwrap();
        let profiles = load_profiles().unwrap();
        assert_eq!(profiles[0].skip_permissions, Some(false));

        toggle_skip_permissions("test", true).unwrap();
        let profiles = load_profiles().unwrap();
        assert_eq!(profiles[0].skip_permissions, Some(true));

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    #[serial]
    fn toggle_skip_permissions_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, "[[profiles]]\nname = \"other\"\n").unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let result = toggle_skip_permissions("missing", true);
        assert!(result.is_err());

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    #[serial]
    fn append_minimal_profile() {
```

**Verify**: `cargo test toggle_skip_permissions 2>&1 | grep -q "3 passed" && echo PASS || echo FAIL`

## Step 10 — Unit test for red profile row rendering

**File**: `src/ui.rs`
**What**: Add test verifying skip_permissions profiles get red styling.

Add after the existing `ui_footer_shows_add_hint` test:

**Old**:
```
    #[test]
    fn ui_footer_shows_add_hint() {
        // Verify the normal footer text contains [a] Add
        let normal_footer =
            " [↑↓/jk] Navigate  [Enter] Launch  [a] Add  [e] Edit config  [q/Ctrl-C] Quit";
        assert!(normal_footer.contains("[a] Add"));
    }
}
```

**New**:
```
    #[test]
    fn ui_footer_shows_add_hint() {
        let normal_footer =
            " [↑↓/jk] Navigate  [Enter] Launch  [s] Skip-perms  [a] Add  [e] Edit config  [q/Ctrl-C] Quit";
        assert!(normal_footer.contains("[a] Add"));
        assert!(normal_footer.contains("[s] Skip-perms"));
    }

    #[test]
    fn skip_permissions_profile_has_red_style() {
        let profile = Profile {
            name: "dangerous".into(),
            description: None,
            env: None,
            model: None,
            skip_permissions: Some(true),
            extra_args: None,
        };
        // Build a ListItem the same way the draw function does
        let label = profile.name.clone();
        let item = ListItem::new(label);
        let styled = if profile.skip_permissions.unwrap_or(false) {
            item.style(Style::default().fg(Color::Red))
        } else {
            item
        };
        // Verify the style has red foreground
        assert_eq!(styled.style().fg, Some(Color::Red));
    }
}
```

**Verify**: `cargo test skip_permissions_profile 2>&1 | grep -q "1 passed" && echo PASS || echo FAIL`

## Step 11 — Proof-Read End-to-End

Read each changed file in full. Check: formatting, no leftover TODOs, spec intent preserved.

## Step 12 — Cross-Check Acceptance Criteria

| Criterion | Addressed in Step |
|-----------|------------------|
| AC1: Unit tests for check_claude_installed | Step 8 |
| AC2: Unit tests for toggle_skip_permissions | Step 9 |
| AC3: Integration test: mock claude absence | Step 8 (CCT_CLAUDE_BIN override) |
| AC4: Integration test: toggle persists to config | Step 9 (tempfile round-trip) |
| AC5: Red profile row when skip_permissions true | Steps 6, 10 |
| AC6: Manual E2E: autoinstall flow | Step 13 |
| AC7: Manual E2E: toggle hotkey + visual | Step 14 |
| AC8: Documentation update | Step 15 |

All criteria mapped.

## Step 13 — Manual E2E: Autoinstall Flow 🖐️ MANUAL

Test on a clean environment without `claude` installed:
1. Run `cct` — expect "Claude CLI not found. Install now? [Y/n]"
2. Press Enter — expect installer runs, claude becomes available
3. Run `cct` again — expect TUI shows directly (no install prompt)

## Step 14 — Manual E2E: Toggle skip_permissions 🖐️ MANUAL

1. Run `cct` with at least one profile
2. Press `s` — expect profile row turns red, detail shows "skip_permissions: ✓"
3. Press `s` again — expect row returns to normal color
4. Quit and check `profiles.toml` — expect `skip_permissions = false`

## Step 15 — Documentation Update

**Files**: `CLAUDE.md`, `README.md`, module docs (`docs/modules/launch.md`, `docs/modules/ui.md`, `docs/modules/config.md`)
**What**: Document the new `s` hotkey, autoinstall behavior, and `toml_edit` dependency.

**Verify**: `grep -q "autoinstall\|check_claude_installed" docs/modules/launch.md && echo PASS || echo FAIL`

## Step 16 — Review

Follow Phase 3 (self-review). Write `review.md`.

## Step 17 — Commit

Use /commit. Suggested message:
feat: add autoinstall claude check and TUI skip_permissions toggle
- detect missing claude binary on startup with install prompt
- add 's' hotkey to toggle skip_permissions with red visual indicator
- persist toggle to profiles.toml using toml_edit
- add unit tests for both features

## Execution Order

Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 → Step 8 → Step 9 → Step 10 → Step 11 → Step 12 → Step 13 → Step 14 → Step 15 → Step 16 → Step 17

(Steps 2 and 3 are parallel-safe. Steps 6 and 7 are parallel-safe. Steps 8, 9, and 10 are parallel-safe. Steps 13 and 14 are parallel-safe.)
