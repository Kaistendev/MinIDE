use crate::core::CoreResult;
use crate::diagnostics::Diagnostic;
use crate::project::Project;
use crate::runtime::{run, ProcessOutput};
use crate::toolchain::ToolchainProvider;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildResult {
    succeeded: bool,
    exit_code: Option<i32>,
    standard_output: String,
    standard_error: String,
    diagnostics: Vec<Diagnostic>,
}

impl BuildResult {
    pub fn new(
        succeeded: bool,
        exit_code: Option<i32>,
        standard_output: impl Into<String>,
        standard_error: impl Into<String>,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        Self {
            succeeded,
            exit_code,
            standard_output: standard_output.into(),
            standard_error: standard_error.into(),
            diagnostics,
        }
    }

    pub fn succeeded(&self) -> bool {
        self.succeeded
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    pub fn standard_output(&self) -> &str {
        &self.standard_output
    }

    pub fn standard_error(&self) -> &str {
        &self.standard_error
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Normaliza la salida de una compilacion.
    ///
    /// La compilacion va bien si el proceso termino con codigo 0.
    pub fn from_output(output: ProcessOutput, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            succeeded: output.exit_code == Some(0),
            exit_code: output.exit_code,
            standard_output: output.standard_output,
            standard_error: output.standard_error,
            diagnostics,
        }
    }
}

/// Compila `project` con el proveedor dado y devuelve el resultado.
///
/// El proveedor prepara la llamada, la ejecuta `runtime` y lee su salida para
/// dejar diagnosticos. Es sincrono: no bloquea todavia, y por eso tampoco se
/// puede usar en el hilo de la interfaz sin dejar la UI congelada mientras
/// compila.
pub fn build_with(provider: &dyn ToolchainProvider, project: &Project) -> CoreResult<BuildResult> {
    let invocation = provider.build_invocation(project)?;

    let output = run(&invocation)?;
    let diagnostics = provider.parse_diagnostics(&output, project.root());

    Ok(BuildResult::from_output(output, diagnostics))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticLocation};
    use crate::document::TextPosition;
    use crate::project::ProjectRelativePath;
    use std::path::Path;

    #[test]
    fn build_result_reports_a_successful_build() {
        let result = BuildResult::new(true, Some(0), String::new(), String::new(), Vec::new());

        assert!(result.succeeded());
        assert_eq!(result.exit_code(), Some(0));
        assert_eq!(result.standard_output(), "");
        assert_eq!(result.standard_error(), "");
        assert!(result.diagnostics().is_empty());
    }

    #[test]
    fn build_result_reports_a_failed_build() {
        let result = BuildResult::new(
            false,
            Some(1),
            "Restaurando paquetes...",
            "Build FAILED.",
            Vec::new(),
        );

        assert!(!result.succeeded());
        assert_eq!(result.exit_code(), Some(1));
        assert_eq!(result.standard_output(), "Restaurando paquetes...");
        assert_eq!(result.standard_error(), "Build FAILED.");
    }

    #[test]
    fn build_result_reports_a_build_that_never_ran() {
        let diagnostic = Diagnostic::new(
            DiagnosticLevel::Error,
            "No se encontro el SDK de .NET.",
            None,
        );
        let result = BuildResult::new(false, None, String::new(), String::new(), vec![diagnostic]);

        assert!(!result.succeeded());
        assert_eq!(result.exit_code(), None);
        assert_eq!(result.diagnostics().len(), 1);
        assert_eq!(result.diagnostics()[0].level(), DiagnosticLevel::Error);
    }

    #[test]
    fn build_result_keeps_its_diagnostics_in_order() {
        let error = Diagnostic::new(
            DiagnosticLevel::Error,
            "CS1002: ; expected",
            Some(DiagnosticLocation::new(
                ProjectRelativePath::new("src/Program.cs").unwrap(),
                TextPosition::new(11, 4),
            )),
        );
        let warning = Diagnostic::new(
            DiagnosticLevel::Warning,
            "CS0219: variable asignada sin usar",
            None,
        );

        let result = BuildResult::new(
            false,
            Some(1),
            String::new(),
            String::new(),
            vec![error, warning],
        );

        assert_eq!(result.diagnostics().len(), 2);
        assert_eq!(result.diagnostics()[0].level(), DiagnosticLevel::Error);
        assert_eq!(result.diagnostics()[1].level(), DiagnosticLevel::Warning);
        assert_eq!(
            result.diagnostics()[0].location().unwrap().file().as_path(),
            Path::new("src/Program.cs")
        );
        assert_eq!(result.diagnostics()[1].location(), None);
    }

    #[test]
    fn a_process_that_finished_with_zero_is_a_successful_build() {
        let result = BuildResult::from_output(
            ProcessOutput {
                exit_code: Some(0),
                standard_output: "compilando".to_string(),
                standard_error: String::new(),
            },
            Vec::new(),
        );

        assert!(result.succeeded());
        assert_eq!(result.exit_code(), Some(0));
        assert_eq!(result.standard_output(), "compilando");
    }

    #[test]
    fn a_process_that_failed_is_not_a_successful_build() {
        let result = BuildResult::from_output(
            ProcessOutput {
                exit_code: Some(1),
                standard_output: String::new(),
                standard_error: "error MSB".to_string(),
            },
            Vec::new(),
        );

        assert!(!result.succeeded());
        assert_eq!(result.standard_error(), "error MSB");
    }

    #[test]
    fn a_process_without_exit_code_is_not_a_successful_build() {
        let result = BuildResult::from_output(
            ProcessOutput {
                exit_code: None,
                standard_output: String::new(),
                standard_error: String::new(),
            },
            Vec::new(),
        );

        assert!(
            !result.succeeded(),
            "sin codigo de salida no se puede afirmar"
        );
    }

    #[test]
    fn the_diagnostics_of_the_toolchain_reach_the_build_result() {
        let parsed = Diagnostic::new(DiagnosticLevel::Error, "CS1002: falta punto y coma", None);

        let result = BuildResult::from_output(
            ProcessOutput {
                exit_code: Some(1),
                standard_output: String::new(),
                standard_error: String::new(),
            },
            vec![parsed],
        );

        assert_eq!(result.diagnostics().len(), 1);
        assert_eq!(result.diagnostics()[0].level(), DiagnosticLevel::Error);
    }
}
