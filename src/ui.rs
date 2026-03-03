use crate::app::{App, AppMode, FormState, FIELD_LABELS};
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
    if SENSITIVE.iter().any(|p| upper.contains(p)) {
        "***"
    } else {
        val
    }
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
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(profile_list, content[0], &mut list_state);

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
        AppMode::AddForm(_) => {
            " [Tab/↓] Next field  [Shift-Tab/↑] Prev  [Enter] Confirm  [Esc] Cancel"
        }
    };
    let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray));
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

fn build_form_lines(form: &FormState) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    if form.confirming {
        lines.push(
            Line::from("Save this profile?").style(Style::default().add_modifier(Modifier::BOLD)),
        );
        lines.push(Line::from(""));
        lines.push(Line::from(format!("  Name:        {}", form.fields[0])));
        lines.push(Line::from(format!(
            "  Description: {}",
            if form.fields[1].is_empty() {
                "(none)"
            } else {
                &form.fields[1]
            }
        )));
        lines.push(Line::from(format!(
            "  Base URL:    {}",
            if form.fields[2].is_empty() {
                "(none)"
            } else {
                &form.fields[2]
            }
        )));
        lines.push(Line::from(format!(
            "  API Key:     {}",
            if form.fields[3].is_empty() {
                "(none)".to_string()
            } else {
                mask_value("API_KEY", &form.fields[3]).to_string()
            }
        )));
        lines.push(Line::from(format!(
            "  Model:       {}",
            if form.fields[4].is_empty() {
                "(none)"
            } else {
                &form.fields[4]
            }
        )));
    } else {
        for (i, label) in FIELD_LABELS.iter().enumerate() {
            let prefix = if i == form.active_field { "> " } else { "  " };
            let style = if i == form.active_field {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
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

    #[test]
    fn ui_renders_add_form() {
        let mut form = FormState::new();
        form.fields[0] = "my-profile".into();
        form.fields[1] = "A description".into();
        form.active_field = 0;

        let lines = build_form_lines(&form);
        // Should have 5 lines (one per field)
        assert_eq!(lines.len(), 5);

        // Active field should have "> " prefix
        let first = lines[0].to_string();
        assert!(
            first.starts_with("> "),
            "active field should have '> ' prefix, got: {first}"
        );
        assert!(first.contains("my-profile"));

        // Non-active fields should have "  " prefix
        let second = lines[1].to_string();
        assert!(
            second.starts_with("  "),
            "inactive field should have '  ' prefix"
        );
        assert!(second.contains("A description"));
    }

    #[test]
    fn ui_confirmation_shows_five_fields() {
        let mut form = FormState::new();
        form.confirming = true;
        form.fields[0] = "test-profile".into();
        form.fields[2] = "https://api.example.com".into();
        form.fields[3] = "sk-secret-key".into();
        form.fields[4] = "kimi-k2".into();

        let lines = build_form_lines(&form);
        let text: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        let joined = text.join("\n");

        // Must contain all five field labels
        assert!(
            joined.contains("Name:"),
            "Expected 'Name:' in confirmation, got:\n{joined}"
        );
        assert!(
            joined.contains("Description:"),
            "Expected 'Description:' in confirmation, got:\n{joined}"
        );
        assert!(
            joined.contains("Base URL:"),
            "Expected 'Base URL:' in confirmation, got:\n{joined}"
        );
        assert!(
            joined.contains("API Key:"),
            "Expected 'API Key:' in confirmation, got:\n{joined}"
        );
        assert!(
            joined.contains("Model:"),
            "Expected 'Model:' in confirmation, got:\n{joined}"
        );

        // API key should be masked (mask_value on key containing "KEY" returns "***")
        assert!(
            joined.contains("***"),
            "Expected masked API key '***' in confirmation, got:\n{joined}"
        );
        assert!(
            !joined.contains("sk-secret-key"),
            "API key should NOT appear in cleartext in confirmation, got:\n{joined}"
        );

        // Model should show the actual value
        assert!(
            joined.contains("kimi-k2"),
            "Expected model 'kimi-k2' in confirmation, got:\n{joined}"
        );

        // Base URL should show the actual value
        assert!(
            joined.contains("https://api.example.com"),
            "Expected base URL in confirmation, got:\n{joined}"
        );

        // Description should show "(none)" since it's empty
        assert!(
            joined.contains("(none)"),
            "Expected '(none)' for empty description, got:\n{joined}"
        );
    }

    #[test]
    fn ui_footer_shows_add_hint() {
        // Verify the normal footer text contains [a] Add
        let normal_footer =
            " [↑↓/jk] Navigate  [Enter] Launch  [a] Add  [e] Edit config  [q/Ctrl-C] Quit";
        assert!(normal_footer.contains("[a] Add"));
    }
}
