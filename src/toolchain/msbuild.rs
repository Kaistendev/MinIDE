use std::path::Path;

use crate::diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticLocation};
use crate::document::TextPosition;
use crate::project::ProjectRelativePath;
use crate::runtime::ProcessOutput;

/// Traduce la salida de MSBuild en diagnosticos.
///
/// Cada linea con esta forma es un diagnostico:
///
/// ```text
/// <archivo>(<linea>,<columna>): <nivel> <codigo>: <mensaje> [<proyecto>]
/// ```
///
/// Las lineas de progreso y el resumen final ("0 Advertencias", "1 Errores")
/// no tienen esa forma y se ignoran. Un error del propio MSBuild que no trae
/// archivo, como `MSB1003`, tambien se reconoce y sale sin posicion.
///
/// Solo se lee stdout: es donde MSBuild escribe los diagnosticos. stderr
/// lleva fallos de la herramienta, que no speaking este formato.
///
/// MSBuild cuenta lineas y columnas desde 1 y el modelo cuenta desde 0, asi que
/// la posicion se corrige al construir el diagnostico.
pub fn parse(output: &ProcessOutput, root: &Path) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for line in output.standard_output().lines() {
        let line = line.trim_end();

        if let Some(diagnostic) = parse_line(line, root) {
            diagnostics.push(diagnostic);
        }
    }

    diagnostics
}

/// Un diagnostico de una linea, o `None` si la linea no es un diagnostico.
fn parse_line(line: &str, root: &Path) -> Option<Diagnostic> {
    let (level, head, rest) = match find_level(line) {
        Some((level, index, marker_len)) => (level, &line[..index], &line[index + marker_len..]),
        None => {
            // Un fallo de la herramienta sin nivel, como "MSB1003: ...". La
            // linea entera es el mensaje, con su codigo dentro.
            let index = line.find(':')?;

            if is_code(&line[..index]) {
                return Some(Diagnostic::new(
                    DiagnosticLevel::Error,
                    line.to_string(),
                    None,
                ));
            }

            return None;
        }
    };

    let (code, message) = split_code(rest)?;

    match position_of(head) {
        Some(Positioned::WithPosition(file, line_number, column)) => {
            let message = without_project_suffix(message);

            match relative_to(root, &file) {
                Some(relative) => Some(Diagnostic::new(
                    level,
                    format!("{code}: {message}"),
                    Some(DiagnosticLocation::new(
                        relative,
                        TextPosition::new(line_number.saturating_sub(1), column.saturating_sub(1)),
                    )),
                )),
                None => Some(Diagnostic::new(
                    level,
                    format!("{file}: {code}: {message}"),
                    None,
                )),
            }
        }
        Some(Positioned::WithoutPosition) => {
            Some(Diagnostic::new(level, format!("{code}: {message}"), None))
        }
        None => Some(Diagnostic::new(level, line.to_string(), None)),
    }
}

/// Busca `: error ` o `: warning ` y devuelve el nivel, donde empieza el
/// marcador y cuanto ocupa.
fn find_level(line: &str) -> Option<(DiagnosticLevel, usize, usize)> {
    for (marker, level) in [
        (": error ", DiagnosticLevel::Error),
        (": warning ", DiagnosticLevel::Warning),
    ] {
        if let Some(index) = line.find(marker) {
            return Some((level, index, marker.len()));
        }
    }

    None
}

/// `CS0246` o `MSB1003`: letras y digitos, empezando por letra.
fn is_code(candidate: &str) -> bool {
    !candidate.is_empty()
        && candidate
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
        && candidate
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic())
}

/// Separa el codigo del mensaje por su primer ':'.
fn split_code(rest: &str) -> Option<(&str, &str)> {
    let index = rest.find(':')?;

    let code = rest[..index].trim();
    if !is_code(code) {
        return None;
    }

    Some((code, rest[index + 1..].trim()))
}

enum Positioned {
    WithPosition(String, u32, u32),
    WithoutPosition,
}

/// La ruta y la posicion de `head`, que es lo que hay antes de `: error `.
///
/// Si `head` acaba en `)` y tiene un `(linea,columna)` al final, hay posicion.
fn position_of(head: &str) -> Option<Positioned> {
    if !head.ends_with(')') {
        return Some(Positioned::WithoutPosition);
    }

    let open = match head.rfind('(') {
        Some(open) => open,
        None => return Some(Positioned::WithoutPosition),
    };

    let file = &head[..open];
    let numbers = &head[open + 1..head.len() - 1];
    let (line_number, column) = match numbers.split_once(',') {
        Some((line_number, column)) => (line_number.trim(), column.trim()),
        None => return Some(Positioned::WithoutPosition),
    };

    match (line_number.parse::<u32>(), column.parse::<u32>()) {
        (Ok(line_number), Ok(column)) if !file.is_empty() => Some(Positioned::WithPosition(
            file.to_string(),
            line_number,
            column,
        )),
        _ => Some(Positioned::WithoutPosition),
    }
}

/// Sin el ` [archivo de proyecto]` que MSBuild anade al final.
fn without_project_suffix(message: &str) -> String {
    match message.rfind(" [") {
        Some(index) if message.ends_with(']') => message[..index].to_string(),
        _ => message.to_string(),
    }
}

/// La ruta del archivo relativa a la raiz del proyecto, si esta dentro.
fn relative_to(root: &Path, file: &str) -> Option<ProjectRelativePath> {
    let file = Path::new(file);
    let relative = file.strip_prefix(root).ok()?;

    ProjectRelativePath::new(relative).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{Diagnostic, DiagnosticLevel};
    use crate::document::TextPosition;
    use crate::project::ProjectRelativePath;
    use std::path::{Path, PathBuf};

    fn root() -> PathBuf {
        PathBuf::from("C:\\proyectos\\App")
    }

    /// La salida real de MSBuild que fallo en la plantilla.
    const REAL_ERROR: &str = "C:\\proyectos\\App\\Form1.cs(3,30): error CS0246: El nombre del tipo o del espacio de nombres 'Form' no se encontró (¿falta una directiva using o una referencia de ensamblado?) [C:\\proyectos\\App\\App.csproj]";

    fn parse_one(line: &str) -> Diagnostic {
        let diagnostics = parse(&ProcessOutput::new(Some(1), line.to_string(), ""), &root());

        assert_eq!(diagnostics.len(), 1, "se esperaba un diagnostico");

        diagnostics.into_iter().next().unwrap()
    }

    #[test]
    fn an_error_with_a_position_becomes_a_diagnostic() {
        let diagnostic = parse_one(REAL_ERROR);

        assert_eq!(diagnostic.level(), DiagnosticLevel::Error);
        assert!(
            diagnostic.message().contains("CS0246"),
            "{}",
            diagnostic.message()
        );
        assert!(
            diagnostic.message().contains("Form"),
            "{}",
            diagnostic.message()
        );
    }

    #[test]
    fn the_position_of_the_error_is_the_place_of_the_file() {
        let diagnostic = parse_one(REAL_ERROR);
        let location = diagnostic.location().expect("con posicion");

        assert_eq!(location.file().as_path(), Path::new("Form1.cs"));
        // MSBuild cuenta desde 1 y el modelo desde 0.
        assert_eq!(location.position(), TextPosition::new(2, 29));
    }

    #[test]
    fn the_project_in_brackets_is_not_part_of_the_message() {
        let diagnostic = parse_one(REAL_ERROR);

        assert!(
            !diagnostic.message().contains("App.csproj"),
            "{}",
            diagnostic.message()
        );
        assert!(
            !diagnostic.message().ends_with(']'),
            "{}",
            diagnostic.message()
        );
    }

    #[test]
    fn a_warning_becomes_a_diagnostic_with_the_warning_level() {
        let diagnostic = parse_one(
            "C:\\proyectos\\App\\Form1.cs(7,13): warning CS0168: la variable 'x' está declarada pero nunca se usa [C:\\proyectos\\App\\App.csproj]",
        );

        assert_eq!(diagnostic.level(), DiagnosticLevel::Warning);
        assert!(
            diagnostic.message().contains("CS0168"),
            "{}",
            diagnostic.message()
        );
    }

    #[test]
    fn a_file_in_a_subdirectory_is_relative_to_the_project() {
        let diagnostic = parse_one(
            "C:\\proyectos\\App\\src\\forms\\MainForm.cs(12,9): error CS0103: el nombre 'btn' no existe [C:\\proyectos\\App\\App.csproj]",
        );
        let location = diagnostic.location().expect("con posicion");

        assert_eq!(
            location.file().as_path(),
            Path::new("src").join("forms").join("MainForm.cs")
        );
    }

    #[test]
    fn an_error_without_a_position_is_still_a_diagnostic() {
        let diagnostic = parse_one(
            "MSB1003: Specify a project or solution file. The current working directory does not contain a project or solution file.",
        );

        assert_eq!(diagnostic.level(), DiagnosticLevel::Error);
        assert!(
            diagnostic.message().contains("MSB1003"),
            "{}",
            diagnostic.message()
        );
        assert_eq!(diagnostic.location(), None);
    }

    #[test]
    fn an_error_of_the_compiler_without_position_keeps_its_line_as_message() {
        let diagnostic =
            parse_one("C:\\proyectos\\App\\Form1.cs: error CS1010: falta llave de cierre");

        assert_eq!(diagnostic.level(), DiagnosticLevel::Error);
        assert!(
            diagnostic.message().contains("CS1010"),
            "{}",
            diagnostic.message()
        );
        assert_eq!(diagnostic.location(), None);
    }

    #[test]
    fn a_file_outside_the_project_has_no_location_but_keeps_its_path() {
        let diagnostic = parse_one(
            "C:\\librerias\\Externo.cs(4,1): error CS0246: tipo no encontrado [C:\\proyectos\\App\\App.csproj]",
        );

        assert_eq!(diagnostic.location(), None);
        assert!(
            diagnostic.message().contains("Externo.cs"),
            "el archivo no se puede perder: {}",
            diagnostic.message()
        );
    }

    #[test]
    fn the_summary_of_the_compilation_is_not_a_diagnostic() {
        let output = "    0 Advertencia(s)\n    1 Errores\nTiempo transcurrido 00:00:01.73\n";

        let diagnostics = parse(
            &ProcessOutput::new(Some(1), output.to_string(), ""),
            &root(),
        );

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn the_progress_of_the_compilation_is_not_a_diagnostic() {
        let output = "  Determinando los proyectos que se van a restaurar...\n  Se ha restaurado C:\\proyectos\\App\\App.csproj (en 256 ms).\n";

        let diagnostics = parse(
            &ProcessOutput::new(Some(0), output.to_string(), ""),
            &root(),
        );

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn several_errors_are_parsed_in_the_order_they_appear() {
        let output = format!(
            "{REAL_ERROR}\nC:\\proyectos\\App\\Form2.cs(9,5): error CS1002: falta punto y coma [C:\\proyectos\\App\\App.csproj]\n"
        );

        let diagnostics = parse(&ProcessOutput::new(Some(1), output, ""), &root());

        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics[0].message().contains("CS0246"));
        assert!(diagnostics[1].message().contains("CS1002"));
    }

    #[test]
    fn a_compilation_without_errors_has_no_diagnostics() {
        let diagnostics = parse(
            &ProcessOutput::new(Some(0), "  App -> C:\\proyectos\\App\\bin\\App.dll\n", ""),
            &root(),
        );

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn a_message_with_colons_keeps_all_of_its_text() {
        let diagnostic = parse_one(
            "C:\\proyectos\\App\\Form1.cs(3,5): error CS1002: ; esperado: falta algo mas",
        );

        assert!(
            diagnostic.message().contains("; esperado: falta algo mas"),
            "{}",
            diagnostic.message()
        );
    }

    #[test]
    fn the_stderr_of_the_compiler_is_not_read_as_diagnostics() {
        let diagnostics = parse(
            &ProcessOutput::new(Some(1), "", format!("{REAL_ERROR}\n")),
            &root(),
        );

        assert!(
            diagnostics.is_empty(),
            "MSBuild escribe los diagnosticos en stdout: {diagnostics:?}"
        );
    }

    #[test]
    fn a_parsed_diagnostic_can_be_built_by_hand_the_same_way() {
        let location = DiagnosticLocation::new(
            ProjectRelativePath::new("Form1.cs").unwrap(),
            TextPosition::new(2, 29),
        );
        let manual = Diagnostic::new(DiagnosticLevel::Error, "CS0246: algo", Some(location));

        assert_eq!(manual.level(), DiagnosticLevel::Error);
        assert_eq!(
            manual.location().unwrap().position(),
            TextPosition::new(2, 29)
        );
    }
}
