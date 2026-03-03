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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn clap_routing_no_subcommand() {
        // No args → command should be None (TUI mode)
        let cli = Cli::try_parse_from(["cct"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn clap_routing_add_subcommand() {
        // "cct add" → command should be Some(Commands::Add)
        let cli = Cli::try_parse_from(["cct", "add"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Add)));
    }
}
