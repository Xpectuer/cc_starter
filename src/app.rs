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
