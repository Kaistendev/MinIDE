use std::fmt;

use crate::document::TextPosition;
use crate::project::ProjectRelativePath;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

impl DiagnosticLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Warning => "warning",
            DiagnosticLevel::Info => "info",
        }
    }
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticLocation {
    file: ProjectRelativePath,
    position: TextPosition,
}

impl DiagnosticLocation {
    pub fn new(file: ProjectRelativePath, position: TextPosition) -> Self {
        Self { file, position }
    }

    pub fn file(&self) -> &ProjectRelativePath {
        &self.file
    }

    pub fn position(&self) -> TextPosition {
        self.position
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    level: DiagnosticLevel,
    message: String,
    location: Option<DiagnosticLocation>,
}

impl Diagnostic {
    pub fn new(
        level: DiagnosticLevel,
        message: impl Into<String>,
        location: Option<DiagnosticLocation>,
    ) -> Self {
        Self {
            level,
            message: message.into(),
            location,
        }
    }

    pub fn level(&self) -> DiagnosticLevel {
        self.level
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn location(&self) -> Option<&DiagnosticLocation> {
        self.location.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn location() -> DiagnosticLocation {
        DiagnosticLocation::new(
            ProjectRelativePath::new("src/Program.cs").unwrap(),
            TextPosition::new(11, 4),
        )
    }

    #[test]
    fn diagnostic_stores_level_and_message() {
        let diagnostic = Diagnostic::new(DiagnosticLevel::Warning, "unused variable", None);

        assert_eq!(diagnostic.level(), DiagnosticLevel::Warning);
        assert_eq!(diagnostic.message(), "unused variable");
    }

    #[test]
    fn diagnostic_location_is_optional() {
        let diagnostic = Diagnostic::new(DiagnosticLevel::Info, "build started", None);

        assert_eq!(diagnostic.location(), None);
    }

    #[test]
    fn diagnostic_keeps_its_location() {
        let location = location();
        let diagnostic = Diagnostic::new(
            DiagnosticLevel::Error,
            "CS1002: ; expected",
            Some(location.clone()),
        );

        assert_eq!(diagnostic.location(), Some(&location));
        assert_eq!(
            diagnostic.location().unwrap().file().as_path(),
            Path::new("src/Program.cs")
        );
        assert_eq!(
            diagnostic.location().unwrap().position(),
            TextPosition::new(11, 4)
        );
    }

    #[test]
    fn diagnostic_level_exposes_a_stable_label() {
        assert_eq!(DiagnosticLevel::Error.as_str(), "error");
        assert_eq!(DiagnosticLevel::Warning.as_str(), "warning");
        assert_eq!(DiagnosticLevel::Info.as_str(), "info");
        assert_eq!(DiagnosticLevel::Error.to_string(), "error");
    }
}
