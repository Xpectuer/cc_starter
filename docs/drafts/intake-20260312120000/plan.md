---
title: "Plan: --continue one-shot launch key for cct TUI"
doc_type: proc
brief: "Add 'c' key binding that prepends --continue to claude CLI args via bool param on build_args/exec_claude"
confidence: verified
created: 2026-03-12
updated: 2026-03-12
revision: 1
---

# Plan: --continue One-Shot Launch Key

See [spec.md](./spec.md) and [requirements.md](./requirements.md) for full context.

## Files Changed

| File | Change Type |
|------|-------------|
| `src/launch.rs` | Major edit — signature changes + new tests |
| `src/main.rs` | Minor edit — update Enter call + add `c` arm |
| `src/ui.rs` | Minor edit — footer string + test assertion |

---

## Step 1 — Update `build_args` signature and prepend logic

**File**: `src/launch.rs`
**What**: Add `with_continue: bool` parameter; prepend `"--continue"` when true.

**Old**:
```
pub fn build_args(profile: &Profile) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(model) = &profile.model {
```

**New**:
```
pub fn build_args(profile: &Profile, with_continue: bool) -> Vec<String> {
    let mut args = Vec::new();
    if with_continue {
        args.push("--continue".to_string());
    }
    if let Some(model) = &profile.model {
```

**Verify**: `grep -n 'pub fn build_args' src/launch.rs` shows `with_continue: bool` in signature.

---

## Step 2 — Update `exec_claude` signature and thread bool

**File**: `src/launch.rs`
**What**: Add `with_continue: bool`; pass it through to `build_args`.

**Old**:
```
pub fn exec_claude(profile: &Profile) -> anyhow::Error {
    if let Some(env_map) = &profile.env {
        for (k, v) in env_map {
            env::set_var(k, v);
        }
    }
    let args = build_args(profile);
```

**New**:
```
pub fn exec_claude(profile: &Profile, with_continue: bool) -> anyhow::Error {
    if let Some(env_map) = &profile.env {
        for (k, v) in env_map {
            env::set_var(k, v);
        }
    }
    let args = build_args(profile, with_continue);
```

**Verify**: `grep -n 'pub fn exec_claude' src/launch.rs` shows `with_continue: bool` in signature.

---

## Step 3 — Update existing test calls to pass `false`

**File**: `src/launch.rs` (test module — three occurrences)

**Edit 3a** — `build_args_empty`:

Old: `assert!(build_args(&profile(None, None, None)).is_empty());`
New: `assert!(build_args(&profile(None, None, None), false).is_empty());`

**Edit 3b** — `build_args_model_only`:

Old:
```
        assert_eq!(
            build_args(&profile(Some("kimi-k1.5"), None, None)),
```
New:
```
        assert_eq!(
            build_args(&profile(Some("kimi-k1.5"), None, None), false),
```

**Edit 3c** — `build_args_full`:

Old: `            build_args(&p),`
New: `            build_args(&p, false),`

**Verify**: `grep 'build_args(' src/launch.rs | grep -v 'pub fn'` — no call missing `, false` or `, true`.

---

## Step 4 — Add two new continue unit tests

**File**: `src/launch.rs`
**What**: Insert `build_args_continue_only` and `build_args_continue_with_flags` after `build_args_full`.

**Old** (end of test module, after Step 3 has run):
```
    #[test]
    fn build_args_full() {
        let p = profile(Some("opus"), Some(true), Some(vec!["--verbose"]));
        assert_eq!(
            build_args(&p, false),
            vec![
                "--model",
                "opus",
                "--dangerously-skip-permissions",
                "--verbose"
            ]
        );
    }
}
```

**New**:
```
    #[test]
    fn build_args_full() {
        let p = profile(Some("opus"), Some(true), Some(vec!["--verbose"]));
        assert_eq!(
            build_args(&p, false),
            vec![
                "--model",
                "opus",
                "--dangerously-skip-permissions",
                "--verbose"
            ]
        );
    }

    #[test]
    fn build_args_continue_only() {
        assert_eq!(
            build_args(&profile(None, None, None), true),
            vec!["--continue"]
        );
    }

    #[test]
    fn build_args_continue_with_flags() {
        let p = profile(Some("opus"), Some(true), Some(vec!["--verbose"]));
        assert_eq!(
            build_args(&p, true),
            vec![
                "--continue",
                "--model",
                "opus",
                "--dangerously-skip-permissions",
                "--verbose",
            ]
        );
    }
}
```

**Verify**: `cargo test build_args_continue` passes with two tests found.

---

## Step 5 — Update Enter handler and add `c` key arm in `main.rs`

**File**: `src/main.rs`
**What**: Pass `false` to existing Enter launch; insert `c` arm passing `true` before the `e` arm.

**Old**:
```
                    (KeyCode::Enter, _) if !app.profiles.is_empty() => {
                        launch::restore_terminal();
                        let err = launch::exec_claude(&app.profiles[app.selected]);
                        eprintln!("Error: {err:#}");
                        std::process::exit(1);
                    }
                    (KeyCode::Char('e'), _) => {
```

**New**:
```
                    (KeyCode::Enter, _) if !app.profiles.is_empty() => {
                        launch::restore_terminal();
                        let err = launch::exec_claude(&app.profiles[app.selected], false);
                        eprintln!("Error: {err:#}");
                        std::process::exit(1);
                    }
                    (KeyCode::Char('c'), _) if !app.profiles.is_empty() => {
                        launch::restore_terminal();
                        let err = launch::exec_claude(&app.profiles[app.selected], true);
                        eprintln!("Error: {err:#}");
                        std::process::exit(1);
                    }
                    (KeyCode::Char('e'), _) => {
```

**Verify**: `grep -n "Char('c')" src/main.rs` shows the new arm with `with_continue=true`.

---

## Step 6 — Update footer string in `ui.rs`

**File**: `src/ui.rs`
**What**: Insert `[c] Resume` after `[Enter] Launch` in the Normal mode footer.

**Old**:
```
        AppMode::Normal => {
            " [↑↓/jk] Navigate  [Enter] Launch  [s] Skip-perms  [a] Add  [e] Edit config  [q/Ctrl-C] Quit"
        }
```

**New**:
```
        AppMode::Normal => {
            " [↑↓/jk] Navigate  [Enter] Launch  [c] Resume  [s] Skip-perms  [a] Add  [e] Edit config  [q/Ctrl-C] Quit"
        }
```

**Verify**: `grep '\[c\] Resume' src/ui.rs` returns a match.

---

## Step 7 — Update footer test in `ui.rs`

**File**: `src/ui.rs`
**What**: Update `ui_footer_shows_add_hint` to reference the new string and assert `[c] Resume`.

**Old**:
```
    fn ui_footer_shows_add_hint() {
        let normal_footer =
            " [↑↓/jk] Navigate  [Enter] Launch  [s] Skip-perms  [a] Add  [e] Edit config  [q/Ctrl-C] Quit";
        assert!(normal_footer.contains("[a] Add"));
        assert!(normal_footer.contains("[s] Skip-perms"));
    }
```

**New**:
```
    fn ui_footer_shows_add_hint() {
        let normal_footer =
            " [↑↓/jk] Navigate  [Enter] Launch  [c] Resume  [s] Skip-perms  [a] Add  [e] Edit config  [q/Ctrl-C] Quit";
        assert!(normal_footer.contains("[a] Add"));
        assert!(normal_footer.contains("[s] Skip-perms"));
        assert!(normal_footer.contains("[c] Resume"));
    }
```

**Verify**: `cargo test ui_footer_shows_add_hint` passes.

---

## Step 8 — Proof-Read End-to-End

Read `src/launch.rs`, `src/main.rs`, `src/ui.rs` in full. Check: no leftover old signatures, no
`build_args(profile)` calls missing the bool, no stray TODOs, spec intent preserved.

---

## Step 9 — Cross-Check Acceptance Criteria

| Criterion | Addressed in Step |
|-----------|------------------|
| `c` key exec-replaces with `claude --continue [args...]` | Step 5 (key arm) + Steps 1-2 (bool threaded) |
| `Enter` launch unaffected | Step 5 (passes `false`) |
| Footer shows `[c] Resume` | Step 6 |
| `cargo test` passes with new tests | Step 4 |
| `cargo clippy` passes | Step 8 (proof-read catches warnings) |

All criteria map to steps. No gaps.

---

## Step 10 — Review

Follow [03-self-review.md](../../../.claude/plugins/cache/alex-marketplace/lb-dev/4.15.0/idea/references/plan-gen/03-self-review.md). Writes `review.md`.

---

## Step 11 — Commit

Use `/lb-dev:commit`. Suggested message:

```
feat: add 'c' key to launch selected profile with --continue

- build_args/exec_claude gain with_continue: bool param
- 'c' in TUI Normal mode passes true; Enter passes false
- footer updated: [c] Resume hint added
- new unit tests: build_args_continue_only, build_args_continue_with_flags
```

---

## Execution Order

```
Step 1 → Step 2 → Step 3 → Step 4   (launch.rs — sequential)
Step 5                                (main.rs — parallel with launch.rs steps)
Step 6 → Step 7                       (ui.rs — parallel with launch.rs steps)
                    ↓ all complete
Step 8 → Step 9 → Step 10 → Step 11
```

Parallel-safe: Steps [1→2→3→4], [5], [6→7] can run concurrently since they touch different files.
