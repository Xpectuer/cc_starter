---
title: "Plan: Config Add Functionality for cct"
doc_type: proc
brief: "Implementation plan for CLI and TUI profile add flows"
confidence: verified
created: 2026-03-03
updated: 2026-03-03
revision: 1
---

# Plan: Config Add Functionality for cct

Spec: [./spec.md](./spec.md)
Requirements: [./requirements.md](./requirements.md)

## Files Changed

| File | Change Type |
|------|-------------|
| `Cargo.toml` | Minor edit — add `clap` dependency |
| `src/config.rs` | Major edit — add `NewProfile`, `append_profile()`, `profile_name_exists()` |
| `src/app.rs` | Major edit — add `AppMode`, `FormState`, form input handling |
| `src/ui.rs` | Major edit — render inline add form, confirmation panel, updated footer |
| `src/cli.rs` | New file — `run_add()` interactive prompt flow |
| `src/lib.rs` | Minor edit — add `pub mod cli;` |
| `src/main.rs` | Major edit — clap routing, extract `run_tui()` |

## Step 1 — Add clap dependency

**File**: `Cargo.toml`
**What**: Add `clap` with `derive` feature to dependencies.

**Old**:
```
[dependencies]
ratatui   = "0.29"
```

**New**:
```
[dependencies]
clap      = { version = "4", features = ["derive"] }
ratatui   = "0.29"
```

**Verify**: `cargo check 2>&1 | head -5` — no unresolved import errors for clap.

## Step 2 — Add config writing utilities

**File**: `src/config.rs`
**What**: Add `NewProfile` struct, `profile_name_exists()`, and `append_profile()` functions with unit tests.

**Old**:
```
#[cfg(test)]
mod tests {
```

**New**:
```
pub struct NewProfile {
    pub name: String,
    pub description: Option<String>,
    pub model: Option<String>,
}

pub fn profile_name_exists(name: &str) -> Result<bool> {
    let profiles = load_profiles()?;
    Ok(profiles
        .iter()
        .any(|p| p.name.eq_ignore_ascii_case(name)))
}

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

#[cfg(test)]
mod tests {
```

**Verify**: `cargo test --lib config -- profile_name_exists append_profile 2>&1 | tail -5` — tests pass.

## Step 3 — Add config writing unit tests

**File**: `src/config.rs`
**What**: Add tests for `append_profile` and `profile_name_exists` inside the existing `tests` module.

**Old**:
```
    #[test]
    fn default_config_is_valid_toml() {
        let _: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
    }
}
```

**New**:
```
    #[test]
    fn default_config_is_valid_toml() {
        let _: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
    }

    #[test]
    fn append_profile_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, DEFAULT_CONFIG).unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let new = NewProfile {
            name: "test-profile".into(),
            description: Some("A test".into()),
            model: Some("claude-sonnet-4-6".into()),
        };
        append_profile(&new).unwrap();
        let profiles = load_profiles().unwrap();
        assert!(profiles.iter().any(|p| p.name == "test-profile"));
        assert_eq!(profiles.len(), 2); // default + new

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    fn append_preserves_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        let original = "# My comment\n\n[[profiles]]\nname = \"orig\"\n";
        std::fs::write(&path, original).unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let new = NewProfile {
            name: "added".into(),
            description: None,
            model: None,
        };
        append_profile(&new).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("# My comment"));
        let profiles = load_profiles().unwrap();
        assert_eq!(profiles.len(), 2);

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    fn profile_name_exists_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, "[[profiles]]\nname = \"MyProfile\"\n").unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        assert!(profile_name_exists("myprofile").unwrap());
        assert!(profile_name_exists("MYPROFILE").unwrap());
        assert!(!profile_name_exists("other").unwrap());

        std::env::remove_var("CCT_CONFIG");
    }

    #[test]
    fn append_minimal_profile() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, DEFAULT_CONFIG).unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        let new = NewProfile {
            name: "minimal".into(),
            description: None,
            model: None,
        };
        append_profile(&new).unwrap();
        let profiles = load_profiles().unwrap();
        let p = profiles.iter().find(|p| p.name == "minimal").unwrap();
        assert!(p.description.is_none());
        assert!(p.model.is_none());

        std::env::remove_var("CCT_CONFIG");
    }
}
```

**Verify**: `cargo test --lib config 2>&1 | tail -10` — all config tests pass.

## Step 4 — Add AppMode and FormState to app.rs

**File**: `src/app.rs`
**What**: Add `AppMode` enum, `FormState` struct, and form navigation methods. Update `App` to include `mode` field.

**Old**:
```
use crate::config::Profile;

pub struct App {
    pub profiles: Vec<Profile>,
    pub selected: usize,
}

impl App {
    pub fn new(profiles: Vec<Profile>) -> Self {
        Self {
            profiles,
            selected: 0,
        }
    }
```

**New**:
```
use crate::config::Profile;

pub const FIELD_LABELS: [&str; 3] = ["Name *", "Description", "Model"];

pub enum AppMode {
    Normal,
    AddForm(FormState),
}

pub struct FormState {
    pub fields: [String; 3],
    pub active_field: usize,
    pub confirming: bool,
    pub error: Option<String>,
}

impl FormState {
    pub fn new() -> Self {
        Self {
            fields: [String::new(), String::new(), String::new()],
            active_field: 0,
            confirming: false,
            error: None,
        }
    }

    pub fn next_field(&mut self) {
        self.active_field = (self.active_field + 1).min(2);
    }

    pub fn prev_field(&mut self) {
        self.active_field = self.active_field.saturating_sub(1);
    }
}

pub struct App {
    pub profiles: Vec<Profile>,
    pub selected: usize,
    pub mode: AppMode,
}

impl App {
    pub fn new(profiles: Vec<Profile>) -> Self {
        Self {
            profiles,
            selected: 0,
            mode: AppMode::Normal,
        }
    }
```

**Verify**: `cargo check 2>&1 | tail -3` — compiles without errors.

## Step 5 — Create cli.rs module

**File**: `src/cli.rs` (new file)
**What**: Implement `run_add()` with interactive prompts, validation, and config writing.

**New** (full file):
```rust
use anyhow::Result;
use std::io::{self, BufRead, Write};

use crate::config::{self, NewProfile};

pub fn run_add() -> Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout();

    // Name (required)
    let name = loop {
        print!("Name: ");
        stdout.flush()?;
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            println!("Name is required.");
            continue;
        }
        if config::profile_name_exists(&trimmed)? {
            eprintln!("Error: profile '{}' already exists.", trimmed);
            std::process::exit(1);
        }
        break trimmed;
    };

    // Description (optional)
    print!("Description (optional): ");
    stdout.flush()?;
    let mut desc_line = String::new();
    reader.read_line(&mut desc_line)?;
    let description = {
        let t = desc_line.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    };

    // Model (optional)
    print!("Model (optional): ");
    stdout.flush()?;
    let mut model_line = String::new();
    reader.read_line(&mut model_line)?;
    let model = {
        let t = model_line.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    };

    // Summary
    println!();
    println!("--- New Profile ---");
    println!("  Name:        {}", name);
    println!("  Description: {}", description.as_deref().unwrap_or("(none)"));
    println!("  Model:       {}", model.as_deref().unwrap_or("(none)"));
    println!();

    // Confirm
    print!("Save? (y/n): ");
    stdout.flush()?;
    let mut confirm = String::new();
    reader.read_line(&mut confirm)?;
    if confirm.trim().to_lowercase() != "y" {
        println!("Cancelled.");
        return Ok(());
    }

    let profile = NewProfile {
        name,
        description,
        model,
    };
    config::append_profile(&profile)?;
    println!("Profile '{}' added.", profile.name);
    Ok(())
}
```

**Verify**: `cargo check 2>&1 | tail -3` — compiles without errors.

## Step 6 — Register cli module in lib.rs

**File**: `src/lib.rs`
**What**: Add `pub mod cli;` to export the new module.

**Old**:
```
pub mod app;
pub mod config;
pub mod launch;
pub mod ui;
```

**New**:
```
pub mod app;
pub mod cli;
pub mod config;
pub mod launch;
pub mod ui;
```

**Verify**: `cargo check 2>&1 | tail -3` — compiles without errors.

## Step 7 — Add clap routing to main.rs

**File**: `src/main.rs`
**What**: Replace direct TUI launch with clap-based routing. Extract TUI loop into `run_tui()`. Route `cct add` to `cli::run_add()`.

**Old** (full file replacement):
```
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use app::App;
use cct::{app, config, launch, ui};

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

**New**:
```
use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use cct::app::{App, AppMode, FormState};
use cct::{cli, config, launch, ui};

#[derive(Parser)]
#[command(name = "cct", about = "Terminal UI launcher for Claude Code")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new profile interactively
    Add,
}

fn main() -> Result<()> {
    config::ensure_default_config()?;

    let args = Cli::parse();
    match args.command {
        Some(Commands::Add) => cli::run_add(),
        None => run_tui(),
    }
}

fn run_tui() -> Result<()> {
    let profiles = config::load_profiles()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut tui = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new(profiles);

    loop {
        tui.draw(|f| ui::draw(&app, f))?;

        if let Event::Key(key) = event::read()? {
            match &mut app.mode {
                AppMode::Normal => match (key.code, key.modifiers) {
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
                    (KeyCode::Char('a'), _) => {
                        app.mode = AppMode::AddForm(FormState::new());
                    }
                    _ => {}
                },
                AppMode::AddForm(form) => {
                    if form.confirming {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                let name = form.fields[0].trim().to_string();
                                if name.is_empty() {
                                    form.error = Some("Name is required.".into());
                                    form.confirming = false;
                                    continue;
                                }
                                match config::profile_name_exists(&name) {
                                    Ok(true) => {
                                        form.error =
                                            Some(format!("Profile '{}' already exists.", name));
                                        form.confirming = false;
                                        continue;
                                    }
                                    Ok(false) => {}
                                    Err(e) => {
                                        form.error = Some(format!("Error: {e:#}"));
                                        form.confirming = false;
                                        continue;
                                    }
                                }
                                let desc = form.fields[1].trim().to_string();
                                let model = form.fields[2].trim().to_string();
                                let new_profile = config::NewProfile {
                                    name,
                                    description: if desc.is_empty() { None } else { Some(desc) },
                                    model: if model.is_empty() { None } else { Some(model) },
                                };
                                if let Err(e) = config::append_profile(&new_profile) {
                                    form.error = Some(format!("Save failed: {e:#}"));
                                    form.confirming = false;
                                    continue;
                                }
                                // Reload and auto-select
                                match config::load_profiles() {
                                    Ok(updated) => {
                                        app.selected = updated.len().saturating_sub(1);
                                        app.profiles = updated;
                                    }
                                    Err(e) => {
                                        eprintln!("Warning: reload failed: {e:#}");
                                    }
                                }
                                app.mode = AppMode::Normal;
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                form.confirming = false;
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char(c) => {
                                form.error = None;
                                form.fields[form.active_field].push(c);
                            }
                            KeyCode::Backspace => {
                                form.error = None;
                                form.fields[form.active_field].pop();
                            }
                            KeyCode::Tab | KeyCode::Down => form.next_field(),
                            KeyCode::BackTab | KeyCode::Up => form.prev_field(),
                            KeyCode::Enter => {
                                if form.active_field < 2 {
                                    form.next_field();
                                } else {
                                    form.confirming = true;
                                }
                            }
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
```

**Verify**: `cargo check 2>&1 | tail -3` — compiles without errors.

## Step 8 — Update ui.rs to render AddForm

**File**: `src/ui.rs`
**What**: When `app.mode` is `AddForm`, render the inline form in the detail panel instead of profile details. Update footer with `[a]` hint.

**Old**:
```
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
    let footer =
        Paragraph::new(" [↑↓/jk] Navigate  [Enter] Launch  [e] Edit config  [q/Ctrl-C] Quit")
            .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, outer[1]);
```

**New**:
```
    // --- Detail panel ---
    match &app.mode {
        AppMode::AddForm(form) => {
            let detail_lines = build_form_lines(form);
            let detail = Paragraph::new(detail_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Add Profile "),
                )
                .wrap(Wrap { trim: false });
            frame.render_widget(detail, content[1]);
        }
        AppMode::Normal => {
            let detail_lines = if app.profiles.is_empty() {
                vec![Line::from("Select a profile to see details.")]
            } else {
                build_detail(&app.profiles[app.selected])
            };
            let detail = Paragraph::new(detail_lines)
                .block(Block::default().borders(Borders::ALL).title(" Details "))
                .wrap(Wrap { trim: false });
            frame.render_widget(detail, content[1]);
        }
    }

    // --- Footer ---
    let footer_text = match &app.mode {
        AppMode::Normal => {
            " [↑↓/jk] Navigate  [Enter] Launch  [a] Add  [e] Edit config  [q/Ctrl-C] Quit"
        }
        AppMode::AddForm(form) if form.confirming => " [y] Save  [n/Esc] Back",
        AppMode::AddForm(_) => " [Tab/↓] Next field  [Shift-Tab/↑] Prev  [Enter] Confirm  [Esc] Cancel",
    };
    let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, outer[1]);
```

**Verify**: `cargo check 2>&1 | tail -3` — compiles without errors.

## Step 9 — Add form rendering helper to ui.rs

**File**: `src/ui.rs`
**What**: Add `build_form_lines` function and import `AppMode`, `FormState`, `FIELD_LABELS`.

**Old**:
```
use crate::app::App;
use crate::config::Profile;
```

**New**:
```
use crate::app::{App, AppMode, FormState, FIELD_LABELS};
use crate::config::Profile;
```

And add the `build_form_lines` function before the `#[cfg(test)]` block:

**Old**:
```
    lines
}

#[cfg(test)]
```

**New**:
```
    lines
}

fn build_form_lines(form: &FormState) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

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
    } else {
        for (i, label) in FIELD_LABELS.iter().enumerate() {
            let prefix = if i == form.active_field { "> " } else { "  " };
            let style = if i == form.active_field {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            lines.push(Line::from(format!("{}{}: {}", prefix, label, form.fields[i])).style(style));
        }
    }

    if let Some(err) = &form.error {
        lines.push(Line::from(""));
        lines.push(Line::from(err.clone()).style(Style::default().fg(Color::Red)));
    }

    lines
}

#[cfg(test)]
```

**Verify**: `cargo check 2>&1 | tail -3` — compiles without errors.

## Step 10 — Proof-Read End-to-End

Read each changed file in full. Check: formatting, no leftover TODOs, spec intent preserved.

## Step 11 — Cross-Check Acceptance Criteria

| Criterion | Addressed in Step |
|-----------|------------------|
| `cct add` prompts for name, description, model interactively | Step 5 |
| `cct add` rejects duplicate profile names with a clear error message | Step 5 |
| `cct add` shows a summary and asks for confirmation before saving | Step 5 |
| `cct add` appends a valid `[[profiles]]` block to `profiles.toml` | Step 2 |
| Pressing `a` in the TUI opens an inline form for name, description, model | Steps 4, 7, 8, 9 |
| In-TUI form validates for duplicate names before saving | Step 7 |
| In-TUI form shows confirmation before writing to disk | Steps 7, 8, 9 |
| After saving via TUI form, cursor auto-selects the new profile | Step 7 |
| Existing profiles are not modified or corrupted when a new one is appended | Steps 2, 3 |
| `clap` handles `cct` (no subcommand → TUI) and `cct add` (→ interactive add flow) | Step 7 |
| `cct --help` shows available subcommands | Step 7 (clap auto-generates) |

All criteria mapped.

## Step 12 — Review

Follow Phase 3. Writes `review.md`.

## Step 13 — Commit

Use /commit. Suggested message:
feat: add profile creation via CLI subcommand and TUI inline form
- Add `cct add` interactive CLI for creating profiles
- Add inline form in TUI triggered by 'a' keybind
- Add clap for subcommand routing
- Validate duplicate profile names, confirm before save

## Execution Order

Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 → Step 8 → Step 9 → Step 10 → Step 11 → Step 12 → Step 13

Steps 2-3 (config) and Steps 4 (app) are parallel-safe.
Steps 5-6 (cli) and Steps 8-9 (ui) are parallel-safe after Step 4.
