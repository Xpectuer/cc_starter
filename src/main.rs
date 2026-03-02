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
