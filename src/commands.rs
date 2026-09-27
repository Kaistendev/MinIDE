use std::fmt;

use crate::core::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    NewProject,
    OpenProject,
    CloseProject,
    Save,
    Build,
    Run,
    Stop,
    Undo,
    Redo,
    Find,
}

impl Command {
    pub const ALL: [Command; 10] = [
        Command::NewProject,
        Command::OpenProject,
        Command::CloseProject,
        Command::Save,
        Command::Build,
        Command::Run,
        Command::Stop,
        Command::Undo,
        Command::Redo,
        Command::Find,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Command::NewProject => "new_project",
            Command::OpenProject => "open_project",
            Command::CloseProject => "close_project",
            Command::Save => "save",
            Command::Build => "build",
            Command::Run => "run",
            Command::Stop => "stop",
            Command::Undo => "undo",
            Command::Redo => "redo",
            Command::Find => "find",
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
    Completed,
    Cancelled,
    Rejected(CoreError),
}

impl CommandOutcome {
    pub fn is_completed(&self) -> bool {
        matches!(self, CommandOutcome::Completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_is_the_only_successful_outcome() {
        assert!(CommandOutcome::Completed.is_completed());
        assert!(!CommandOutcome::Cancelled.is_completed());
        assert!(!CommandOutcome::Rejected(CoreError::NotFound("a".to_string())).is_completed());
    }

    #[test]
    fn rejected_keeps_the_core_error() {
        let outcome = CommandOutcome::Rejected(CoreError::Unsupported("WPF".to_string()));

        assert_eq!(
            outcome,
            CommandOutcome::Rejected(CoreError::Unsupported("WPF".to_string()))
        );
    }

    #[test]
    fn the_command_set_covers_the_main_actions() {
        assert_eq!(
            Command::ALL,
            [
                Command::NewProject,
                Command::OpenProject,
                Command::CloseProject,
                Command::Save,
                Command::Build,
                Command::Run,
                Command::Stop,
                Command::Undo,
                Command::Redo,
                Command::Find,
            ]
        );
    }

    #[test]
    fn every_command_has_a_unique_identifier() {
        let mut identifiers: Vec<&str> = Command::ALL.iter().map(Command::as_str).collect();

        for identifier in &identifiers {
            assert!(!identifier.is_empty());
        }

        identifiers.sort_unstable();
        identifiers.dedup();

        assert_eq!(identifiers.len(), Command::ALL.len());
    }

    #[test]
    fn a_command_displays_its_own_identifier() {
        for command in Command::ALL {
            assert_eq!(command.to_string(), command.as_str());
        }
    }
}
