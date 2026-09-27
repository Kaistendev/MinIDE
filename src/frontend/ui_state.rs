use crate::APP_NAME;

pub struct UiState {
    title: String,
    status: Option<String>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            title: APP_NAME.to_string(),
            status: None,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status = Some(message.into());
    }

    pub fn clear_status(&mut self) {
        self.status = None;
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_with_the_application_title_and_no_status() {
        let state = UiState::new();

        assert_eq!(state.title(), APP_NAME);
        assert_eq!(state.status(), None);
    }

    #[test]
    fn status_can_be_set_and_cleared() {
        let mut state = UiState::default();

        state.set_status("Build finished with 1 error");
        assert_eq!(state.status(), Some("Build finished with 1 error"));

        state.clear_status();
        assert_eq!(state.status(), None);
    }

    #[test]
    fn setting_a_status_replaces_the_previous_one() {
        let mut state = UiState::new();

        state.set_status("Compilando...");
        state.set_status("1 error");

        assert_eq!(state.status(), Some("1 error"));
    }
}
