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
