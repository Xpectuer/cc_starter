---
title: "Plan: cct — Claude Code TUI Launcher"
doc_type: proc
brief: "Build Rust/ratatui profile-picker TUI that exec-replaces current process with claude"
confidence: verified
created: 2026-03-02
updated: 2026-03-02
revision: 1
---

## Files Changed

| File | Change Type |
|------|-------------|
| `Cargo.toml` | New file |
| `src/app.rs` | New file |
| `src/config.rs` | New file |
| `src/launch.rs` | New file |
| `src/ui.rs` | New file |
| `src/main.rs` | New file |
| `CLAUDE.md` | Minor edit |

---

## Step 1 — Write Cargo.toml

**File**: `Cargo.toml`
**What**: Create Cargo manifest with all required dependencies.

**New**:
```toml
[package]
name = "cct"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "cct"
path = "src/main.rs"

[dependencies]
ratatui   = "0.29"
crossterm = "0.28"
serde     = { version = "1", features = ["derive"] }
toml      = "0.8"
dirs      = "5"
anyhow    = "1"
```

**Verify**: `grep -c 'ratatui' Cargo.toml` → outputs `1`

---

## Step 2 — Write src/app.rs

**File**: `src/app.rs`
**What**: Define `App` struct holding profile list and selection cursor with `next`/`prev` methods.

**New**:
```rust
use crate::config::Profile;

pub struct App {
    pub profiles: Vec<Profile>,
    pub selected: usize,
}

impl App {
    pub fn new(profiles: Vec<Profile>) -> Self {
        Self { profiles, selected: 0 }
    }

    pub fn next(&mut self) {
        if !self.profiles.is_empty() {
            self.selected = (self.selected + 1) % self.profiles.len();
        }
    }

    pub fn prev(&mut self) {
        if !self.profiles.is_empty() {
            if self.selected == 0 {
                self.selected = self.profiles.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }
}
```

**Verify**: `grep 'pub struct App' src/app.rs` → outputs the struct line

---

## Step 3 — Write src/config.rs

**File**: `src/config.rs`
**What**: Define `Profile` struct, TOML parsing, XDG config path, and first-run default writer.

**New**:
```rust
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

#[derive(Debug, Deserialize)]
struct Config {
    profiles: Vec<Profile>,
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("cc-tui")
        .join("profiles.toml")
}

const DEFAULT_CONFIG: &str = r#"# cct — Claude Code TUI profile configuration
# Each [[profiles]] block defines one launch profile.

[[profiles]]
name = "default"
description = "Default Claude Code"
# model = "claude-sonnet-4-6"
# skip_permissions = false
# extra_args = []

# [profiles.env]
# ANTHROPIC_API_KEY = "sk-ant-..."
"#;

pub fn ensure_default_config() -> Result<()> {
    let path = config_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create config dir {parent:?}"))?;
        }
        fs::write(&path, DEFAULT_CONFIG)
            .with_context(|| format!("write default config to {path:?}"))?;
    }
    Ok(())
}

pub fn load_profiles() -> Result<Vec<Profile>> {
    let path = config_path();
    let content = fs::read_to_string(&path)
        .with_context(|| format!("read config {path:?}"))?;
    let config: Config = toml::from_str(&content)
        .with_context(|| format!("parse TOML in {path:?}"))?;
    Ok(config.profiles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_profile() {
        let src = r#"
[[profiles]]
name = "kclaude"
description = "Kimi AI"
model = "kimi-k1.5"
skip_permissions = true
extra_args = ["--verbose"]

[profiles.env]
ANTHROPIC_BASE_URL = "https://api.example.com"
ANTHROPIC_AUTH_TOKEN = "sk-secret"
"#;
        let cfg: Config = toml::from_str(src).unwrap();
        assert_eq!(cfg.profiles.len(), 1);
        let p = &cfg.profiles[0];
        assert_eq!(p.name, "kclaude");
        assert_eq!(p.model.as_deref(), Some("kimi-k1.5"));
        assert_eq!(p.skip_permissions, Some(true));
        assert_eq!(p.extra_args.as_deref(), Some(&["--verbose".to_string()][..]));
        let env = p.env.as_ref().unwrap();
        assert_eq!(
            env.get("ANTHROPIC_BASE_URL").map(String::as_str),
            Some("https://api.example.com")
        );
    }

    #[test]
    fn parse_minimal_profile() {
        let src = "[[profiles]]\nname = \"default\"";
        let cfg: Config = toml::from_str(src).unwrap();
        assert_eq!(cfg.profiles[0].name, "default");
        assert!(cfg.profiles[0].description.is_none());
        assert!(cfg.profiles[0].env.is_none());
    }

    #[test]
    fn default_config_is_valid_toml() {
        let _: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
    }
}
```

**Verify**: `grep 'pub struct Profile' src/config.rs` → outputs the struct line

---

## Step 4 — Write src/launch.rs

**File**: `src/launch.rs`
**What**: Implement `restore_terminal`, pure `build_args`, `exec_claude`, and `open_editor`.

**New**:
```rust
use crate::config::Profile;
use anyhow::{Context, Result};
use crossterm::{execute, terminal::LeaveAlternateScreen};
use std::{env, io, os::unix::process::CommandExt, path::Path, process::Command};

/// Restore terminal to cooked mode. Must be called before exec or editor spawn.
pub fn restore_terminal() {
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
}

/// Build the CLI argument list for `claude` from a profile. Pure — no side effects.
pub fn build_args(profile: &Profile) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(model) = &profile.model {
        args.push("--model".to_string());
        args.push(model.clone());
    }
    if profile.skip_permissions.unwrap_or(false) {
        args.push("--dangerously-skip-permissions".to_string());
    }
    if let Some(extra) = &profile.extra_args {
        args.extend(extra.iter().cloned());
    }
    args
}

/// Inject profile env vars and exec-replace the current process with `claude`.
/// Returns only on error (process was not replaced).
pub fn exec_claude(profile: &Profile) -> anyhow::Error {
    if let Some(env_map) = &profile.env {
        for (k, v) in env_map {
            env::set_var(k, v);
        }
    }
    let args = build_args(profile);
    let err = Command::new("claude").args(&args).exec();
    anyhow::anyhow!("exec claude: {err}")
}

/// Suspend TUI, open $EDITOR (or vi) on path, block until editor exits.
pub fn open_editor(path: &Path) -> Result<()> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    Command::new(&editor)
        .arg(path)
        .status()
        .with_context(|| format!("spawn editor {editor:?}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Profile;

    fn profile(model: Option<&str>, skip: Option<bool>, extra: Option<Vec<&str>>) -> Profile {
        Profile {
            name: "t".into(),
            description: None,
            env: None,
            model: model.map(Into::into),
            skip_permissions: skip,
            extra_args: extra.map(|v| v.into_iter().map(Into::into).collect()),
        }
    }

    #[test]
    fn build_args_empty() {
        assert!(build_args(&profile(None, None, None)).is_empty());
    }

    #[test]
    fn build_args_model_only() {
        assert_eq!(
            build_args(&profile(Some("kimi-k1.5"), None, None)),
            vec!["--model", "kimi-k1.5"]
        );
    }

    #[test]
    fn build_args_full() {
        let p = profile(Some("opus"), Some(true), Some(vec!["--verbose"]));
        assert_eq!(
            build_args(&p),
            vec!["--model", "opus", "--dangerously-skip-permissions", "--verbose"]
        );
    }
}
```

**Verify**: `grep 'pub fn build_args' src/launch.rs` → outputs the function signature line

---

## Step 5 — Write src/ui.rs

**File**: `src/ui.rs`
**What**: Implement `draw` (three-zone layout: list | detail | footer) and `mask_value`.

**New**:
```rust
use crate::app::App;
use crate::config::Profile;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

const SENSITIVE: &[&str] = &["TOKEN", "KEY", "SECRET"];

/// Returns `"***"` if `key` (case-insensitive) contains TOKEN, KEY, or SECRET.
pub fn mask_value<'a>(key: &str, val: &'a str) -> &'a str {
    let upper = key.to_uppercase();
    if SENSITIVE.iter().any(|p| upper.contains(p)) { "***" } else { val }
}

pub fn draw(app: &App, frame: &mut Frame) {
    // Outer: content area + 1-line footer
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    // Content: 35% list | 65% detail
    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(outer[0]);

    // --- Profile list ---
    let items: Vec<ListItem> = if app.profiles.is_empty() {
        vec![ListItem::new("No profiles. Press 'e' to edit config.")]
    } else {
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
    };

    let mut list_state = ListState::default();
    if !app.profiles.is_empty() {
        list_state.select(Some(app.selected));
    }

    let profile_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Profiles "))
        .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(profile_list, content[0], &mut list_state);

    // --- Detail panel ---
    let detail_lines = if app.profiles.is_empty() {
        vec![Line::from("Select a profile to see details.")]
    } else {
        build_detail(&app.profiles[app.selected])
    };

    let detail = Paragraph::new(detail_lines)
        .block(Block::default().borders(Borders::ALL).title(" Details "))
        .wrap(Wrap { trim: false });
    frame.render_widget(detail, content[1]);

    // --- Footer ---
    let footer = Paragraph::new(
        " [↑↓/jk] Navigate  [Enter] Launch  [e] Edit config  [q/Ctrl-C] Quit",
    )
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, outer[1]);
}

fn build_detail(profile: &Profile) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    if let Some(desc) = &profile.description {
        lines.push(Line::from(desc.clone()));
        lines.push(Line::from(""));
    }
    if let Some(model) = &profile.model {
        lines.push(Line::from(format!("model: {model}")));
    }
    if profile.skip_permissions.unwrap_or(false) {
        lines.push(Line::from("skip_permissions: ✓"));
    }
    if let Some(extra) = &profile.extra_args {
        if !extra.is_empty() {
            lines.push(Line::from(format!("extra_args: {}", extra.join(" "))));
        }
    }
    if let Some(env_map) = &profile.env {
        if !env_map.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from("ENV:"));
            let mut pairs: Vec<(&String, &String)> = env_map.iter().collect();
            pairs.sort_by_key(|(k, _)| k.as_str());
            for (k, v) in &pairs {
                let display = mask_value(k.as_str(), v.as_str());
                lines.push(Line::from(format!("  {} = {}", k, display)));
            }
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_auth_token() {
        assert_eq!(mask_value("ANTHROPIC_AUTH_TOKEN", "sk-secret"), "***");
    }

    #[test]
    fn mask_api_key() {
        assert_eq!(mask_value("OPENAI_API_KEY", "sk-key"), "***");
    }

    #[test]
    fn mask_secret() {
        assert_eq!(mask_value("MY_SECRET", "s3cr3t"), "***");
    }

    #[test]
    fn no_mask_url() {
        assert_eq!(
            mask_value("ANTHROPIC_BASE_URL", "https://api.example.com"),
            "https://api.example.com"
        );
    }
}
```

**Verify**: `grep 'pub fn mask_value' src/ui.rs` → outputs the function signature line

---

## Step 6 — Write src/main.rs

**File**: `src/main.rs`
**What**: Terminal init, event loop dispatching key events, suspend/resume for editor.

**New**:
```rust
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

mod app;
mod config;
mod launch;
mod ui;

use app::App;

fn main() -> Result<()> {
    config::ensure_default_config()?;
    let profiles = config::load_profiles()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut tui = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new(profiles);

    loop {
        tui.draw(|f| ui::draw(&app, f))?;

        if let Event::Key(key) = event::read()? {
            match (key.code, key.modifiers) {
                (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    launch::restore_terminal();
                    return Ok(());
                }
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.next(),
                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.prev(),
                (KeyCode::Enter, _) if !app.profiles.is_empty() => {
                    launch::restore_terminal();
                    let err = launch::exec_claude(&app.profiles[app.selected]);
                    eprintln!("Error: {err:#}");
                    std::process::exit(1);
                }
                (KeyCode::Char('e'), _) => {
                    launch::restore_terminal();
                    launch::open_editor(&config::config_path())?;
                    enable_raw_mode()?;
                    execute!(io::stdout(), EnterAlternateScreen)?;
                    tui.clear()?;
                    match config::load_profiles() {
                        Ok(updated) => {
                            app.profiles = updated;
                            if app.selected >= app.profiles.len() {
                                app.selected = app.profiles.len().saturating_sub(1);
                            }
                        }
                        Err(e) => eprintln!("Warning: profile reload failed: {e:#}"),
                    }
                }
                _ => {}
            }
        }
    }
}
```

**Verify**: `grep 'fn main' src/main.rs` → outputs the `fn main() -> Result<()>` line

---

## Step 7 — Update CLAUDE.md

**File**: `CLAUDE.md`
**What**: Replace placeholder sections with actual project description and structure.

**Old**:
```
(Replace with a brief description of the project)
```

**New**:
```
`cct` — a minimal Rust/ratatui TUI that presents named Claude Code launch profiles,
injects env vars, and exec-replaces the current process with `claude`. Profiles are
stored in `~/.config/cc-tui/profiles.toml`.
```

**Old**:
```
(Replace with directory tree and descriptions)
```

**New**:
```
src/
├── main.rs      # terminal lifecycle + event loop
├── app.rs       # App struct (profiles, selected cursor)
├── config.rs    # Profile struct, TOML parse, first-run default
├── ui.rs        # ratatui rendering: list+detail layout, mask_value
└── launch.rs    # restore_terminal, exec_claude, open_editor
Cargo.toml       # cct binary, ratatui/crossterm/serde/toml/dirs/anyhow deps
```

**Old**:
```
- (Add project-specific conventions here)
```

**New**:
```
- Always call `launch::restore_terminal()` before `exec_claude()` or `open_editor()`.
- `build_args()` in launch.rs is pure — keep it side-effect-free and unit-testable.
- Token masking: any env key containing TOKEN, KEY, or SECRET → display as `***`.
- Config path: `~/.config/cc-tui/profiles.toml` (XDG via `dirs::config_dir()`).
```

**Verify**: `grep 'cct' CLAUDE.md | wc -l` → outputs a number ≥ 3

---

## Step 8 — Proof-Read End-to-End

Read each file written in Steps 1–7 in full. Check:
- No leftover TODO comments
- Import paths consistent across modules (e.g. `crate::config::Profile` used in app.rs, launch.rs, ui.rs)
- `mod` declarations in main.rs match file names (`app`, `config`, `launch`, `ui`)
- `frame.area()` used (not deprecated `frame.size()`)
- `tui` variable name in main.rs (not `terminal`, which would conflict with `crossterm::terminal` imports)

---

## Step 9 — Cross-Check Acceptance Criteria

| Criterion | Addressed in Step |
|-----------|------------------|
| `cargo build --release` produces binary `cct` | Step 1 (Cargo.toml `[[bin]]`) |
| Renders profile list from `profiles.toml` | Step 3 (load_profiles) + Step 5 (draw) |
| Arrow keys move selection; Enter launches with correct env+args | Step 6 (event loop) + Step 4 (exec_claude) |
| `e` suspends TUI, opens `$EDITOR`, resumes on exit | Step 4 (open_editor) + Step 6 (Char 'e' branch) |
| `q` / `Ctrl-C` exits cleanly with code 0 | Step 6 (Char 'q' / Ctrl-C branch) |
| Token/key values shown as `***` | Step 5 (mask_value + build_detail) |
| No config → default created with example profile | Step 3 (ensure_default_config) |
| `extra_args`, `skip_permissions`, `model` → CLI flags | Step 4 (build_args) |

All criteria map to a step. ✓

---

## Step 10 — Review

Follow Phase 3 (03-self-review.md). Writes `review.md` in this draft directory.

---

## Step 11 — Commit

Use `/commit`. Suggested message:
```
feat: add cct Rust/ratatui TUI launcher for Claude Code profiles
- profile list + detail sidebar with token masking
- exec-replaces process on launch; $EDITOR suspend/resume
- first-run default config at ~/.config/cc-tui/profiles.toml
```

## Execution Order

Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 → Step 8 → Step 9 → Step 10 → Step 11

Steps 2–6 are independent new files and can be written in parallel.
