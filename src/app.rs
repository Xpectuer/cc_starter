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

impl Default for FormState {
    fn default() -> Self {
        Self::new()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_state_field_navigation() {
        let mut form = FormState::new();
        assert_eq!(form.active_field, 0);

        form.next_field();
        assert_eq!(form.active_field, 1);

        form.next_field();
        assert_eq!(form.active_field, 2);

        // Should clamp at max (2)
        form.next_field();
        assert_eq!(form.active_field, 2);

        form.prev_field();
        assert_eq!(form.active_field, 1);

        form.prev_field();
        assert_eq!(form.active_field, 0);

        // Should clamp at min (0)
        form.prev_field();
        assert_eq!(form.active_field, 0);
    }

    #[test]
    fn app_mode_transitions() {
        let app = App::new(vec![]);
        assert!(matches!(app.mode, AppMode::Normal));

        // Transition to AddForm
        let mut app = App::new(vec![]);
        app.mode = AppMode::AddForm(FormState::new());
        match &app.mode {
            AppMode::AddForm(form) => {
                assert_eq!(form.active_field, 0);
                assert!(!form.confirming);
                assert!(form.error.is_none());
            }
            _ => panic!("expected AddForm mode"),
        }

        // Transition back to Normal
        app.mode = AppMode::Normal;
        assert!(matches!(app.mode, AppMode::Normal));
    }
}
