use std::path::Path;

use crate::diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticLocation};
use crate::document::TextPosition;
use crate::project::ProjectRelativePath;
use crate::runtime::ProcessOutput;

/// Traduce la salida de `javac` en diagnosticos.
///
/// Cada linea con esta forma es un diagnostico:
///
/// ```text
/// <archivo>:<linea>:<nivel>: <mensaje>
/// <archivo>:<linea>:<columna>:<nivel>: <mensaje>
/// <nivel>: <mensaje>
/// ```
///
/// El resumen final ("1 error", "3 warnings"), las notas, el codigo fuente que
/// `javac` repite debajo de cada error, el cursor `^` y las lineas de ejemplo que
/// acompanan a un error no tienen esa forma y se ignoran.
///
/// Solo se lee stderr: es donde `javac` escribe los diagnosticos, y stdout lleva
/// la salida del propio compilador.
///
/// `javac` cuenta lineas desde 1 y el modelo cuenta desde 0, asi que la posicion
/// se corrige al construir el diagnostico. No lleva columna, asi que el
/// diagnostico apunta al principio de la linea.
pub fn parse(output: &ProcessOutput, root: &Path) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for line in output.standard_error().lines() {
        if let Some(diagnostic) = parse_line(line, root) {
            diagnostics.push(diagnostic);
        }
    }

    diagnostics
}

/// Un diagnostico de una linea, o `None` si la linea no es un diagnostico.
fn parse_line(line: &str, root: &Path) -> Option<Diagnostic> {
    let line = line.trim_end();
    let (level, index, marker_len) = find_level(line)?;
    let head = &line[..index];
    let message = line[index + marker_len..].trim();

    if message.is_empty() {
        return None;
    }

    let location = position_of(head, root);

    // Sin posicion el archivo no se ve en ningun sitio, asi que se queda en el
    // mensaje en vez de perderse.
    let message = match (&location, head.is_empty()) {
        (Some(_), _) => message.to_string(),
        (None, true) => message.to_string(),
        (None, false) => format!("{head}: {message}"),
    };

    Some(Diagnostic::new(level, message, location))
}

/// El nivel de la linea, donde empieza el mensaje y cuanto ocupa el marcador.
///
/// Un error con posicion llega como `archivo:5: error: mensaje`, y uno sin
/// posicion como `error: mensaje`, sin archivo delante. Los dos se reconocen por
/// su nivel, que es lo unico que no puede faltar.
fn find_level(line: &str) -> Option<(DiagnosticLevel, usize, usize)> {
    for (marker, level) in [
        ("error", DiagnosticLevel::Error),
        ("warning", DiagnosticLevel::Warning),
    ] {
        let at_start = format!("{marker}: ");

        if line.starts_with(&at_start) {
            return Some((level, 0, at_start.len()));
        }

        let after_file = format!(": {marker}: ");

        if let Some(index) = line.find(&after_file) {
            return Some((level, index, after_file.len()));
        }
    }

    None
}

/// Lo que hay antes del nivel: el archivo y, si los hay, la linea y la columna.
///
/// El texto es `archivo:linea` y, si `javac` imprime columna,
/// `archivo:linea:columna`. Una columna solo puede existir si detras de ella hay
/// una linea, y no al reves, asi que se mira antes de darla por buena.
///
/// La letra de la unidad no es un numero, de modo que `C:\proyectos\App\Main.java:5`
/// no se lee como si el 5 fuera una columna de un archivo llamado `C`.
fn position_of(head: &str, root: &Path) -> Option<DiagnosticLocation> {
    let mut before_level = head;
    let mut column = None;

    if let Some((before, number)) = number_at_end(before_level) {
        if number_at_end(before).is_some() {
            column = Some(number);
            before_level = before;
        }
    }

    let (file, line_number) = number_at_end(before_level)?;
    let file = relative_to(root, file)?;

    // `javac` cuenta desde 1 y el modelo desde 0. Sin columna, el diagnostico
    // apunta al principio de la linea.
    Some(DiagnosticLocation::new(
        file,
        TextPosition::new(
            line_number.saturating_sub(1),
            column.unwrap_or(1).saturating_sub(1),
        ),
    ))
}

/// `head` partido por su ultimo `:` cuando lo que va detras es un numero, con el
/// numero que es y lo que se queda delante.
fn number_at_end(head: &str) -> Option<(&str, u32)> {
    let (before, number) = head.rsplit_once(':')?;

    Some((before, number.parse().ok()?))
}

/// La ruta del archivo relativa a la raiz del proyecto, si esta dentro.
///
/// `javac` repite el archivo tal y como se le ha dado, y MiniIDE le da rutas
/// relativas a la raiz del proyecto: si ya es relativa, vale tal cual. Una ruta
/// absoluta solo sirve si esta dentro del proyecto, y una que se sale no es una
/// posicion: sale sin ella en vez de inventarse una.
fn relative_to(root: &Path, file: &str) -> Option<ProjectRelativePath> {
    let file = Path::new(file);

    if file.is_absolute() {
        return ProjectRelativePath::new(file.strip_prefix(root).ok()?).ok();
    }

    ProjectRelativePath::new(file).ok()
}

/// Tests del parseo de la salida de `javac`.
///
/// Los ejemplos son la salida real de `javac` 25 taken de la plantilla de Java,
/// incluido el codigo fuente que `javac` repite, el cursor `^` y el resumen final.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::DiagnosticLevel;
    use crate::document::TextPosition;
    use crate::project::ProjectRelativePath;
    use std::path::PathBuf;

    fn root() -> PathBuf {
        PathBuf::from("C:\\proyectos\\App")
    }

    /// La salida real de `javac` con un error y su resumen.
    const REAL_ERROR: &str = "src\\main\\java\\Main.java:1: error: <identifier> expected\npublic class Main { estaNoCompila }\n                                 ^\n1 error\n";

    fn parse_error(stderr: &str) -> Vec<Diagnostic> {
        parse(
            &ProcessOutput::new(Some(1), "", stderr.to_string()),
            &root(),
        )
    }

    fn parse_one(stderr: &str) -> Diagnostic {
        let diagnostics = parse_error(stderr);

        assert_eq!(diagnostics.len(), 1, "se esperaba un diagnostico");

        diagnostics.into_iter().next().unwrap()
    }

    #[test]
    fn an_error_with_a_position_becomes_a_diagnostic() {
        let diagnostic = parse_one("src\\main\\java\\Main.java:5: error: cannot find symbol\n");

        assert_eq!(diagnostic.level(), DiagnosticLevel::Error);
        assert_eq!(diagnostic.message(), "cannot find symbol");
    }

    #[test]
    fn the_position_of_the_error_is_the_place_of_the_file() {
        let diagnostic = parse_one("src\\main\\java\\Main.java:5: error: cannot find symbol\n");
        let location = diagnostic.location().expect("con posicion");

        // `javac` cuenta las lineas desde 1 y el modelo desde 0.
        assert_eq!(location.position(), TextPosition::new(4, 0));
        assert_eq!(
            location.file().as_path(),
            Path::new("src").join("main").join("java").join("Main.java")
        );
    }

    #[test]
    fn a_warning_becomes_a_diagnostic_with_the_warning_level() {
        let diagnostic =
            parse_one("src\\main\\java\\Main.java:5: warning: [rawtypes] found raw type: List\n");

        assert_eq!(diagnostic.level(), DiagnosticLevel::Warning);
        assert_eq!(
            diagnostic.message(),
            "[rawtypes] found raw type: List",
            "la categoria de javac es parte del mensaje"
        );
    }

    #[test]
    fn the_source_line_and_the_caret_are_not_diagnostics() {
        let diagnostics = parse_error(REAL_ERROR);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    }

    #[test]
    fn the_summary_of_the_compilation_is_not_a_diagnostic() {
        for summary in ["1 error\n", "2 errors\n", "3 warnings\n", "1 warning\n"] {
            let output = format!(
                "src\\main\\java\\Main.java:8: error: bad operand types for binary operator '+'\n{summary}"
            );

            let diagnostics = parse_error(&output);

            assert_eq!(diagnostics.len(), 1, "el resumen se cuela: {diagnostics:?}");
        }
    }

    #[test]
    fn the_explanation_of_a_diagnostic_is_not_another_diagnostic() {
        let output = "src\\main\\java\\Main.java:5: warning: [rawtypes] found raw type: List\n        List lista = new ArrayList();\n        ^\n  missing type arguments for generic class List<E>\n  where E is a type-variable:\n    E extends Object declared in interface List\n1 error\n3 warnings\n";

        let diagnostics = parse_error(output);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    }

    #[test]
    fn a_note_of_the_compiler_is_not_a_diagnostic() {
        let output = "Note: Some input files use unchecked or unsafe operations.\nNote: Recompile with -Xlint:unchecked for details.\n";

        let diagnostics = parse_error(output);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn an_error_without_a_position_is_still_a_diagnostic() {
        let diagnostic = parse_one("error: file not found: src\\NoExiste.java\n");

        assert_eq!(diagnostic.level(), DiagnosticLevel::Error);
        assert_eq!(diagnostic.message(), "file not found: src\\NoExiste.java");
        assert_eq!(diagnostic.location(), None);
    }

    #[test]
    fn a_file_outside_the_project_has_no_location_but_keeps_its_path() {
        let diagnostic =
            parse_one("C:\\librerias\\Externo.java:4: error: no se encuentra el simbolo\n");

        assert_eq!(diagnostic.location(), None);
        assert!(
            diagnostic.message().contains("Externo.java"),
            "el archivo no se puede perder: {}",
            diagnostic.message()
        );
    }

    #[test]
    fn a_file_above_the_project_has_no_location() {
        let diagnostic = parse_one("..\\Otro\\Main.java:1: error: error de compilacion\n");

        assert_eq!(diagnostic.location(), None);
    }

    #[test]
    fn a_file_in_a_subdirectory_is_relative_to_the_project() {
        let diagnostic = parse_one("src\\vista\\Panel.java:12: error: cannot find symbol\n");
        let location = diagnostic.location().expect("con posicion");

        assert_eq!(
            location.file().as_path(),
            Path::new("src").join("vista").join("Panel.java")
        );
    }

    #[test]
    fn an_absolute_file_inside_the_project_is_relative_to_it() {
        let diagnostic =
            parse_one("C:\\proyectos\\App\\src\\Main.java:3: error: cannot find symbol\n");
        let location = diagnostic.location().expect("con posicion");

        assert_eq!(
            location.file().as_path(),
            Path::new("src").join("Main.java")
        );
        assert_eq!(location.position(), TextPosition::new(2, 0));
    }

    #[test]
    fn a_column_is_used_when_javac_gives_one() {
        let diagnostic = parse_one("src\\Main.java:5:9: error: cannot find symbol\n");
        let location = diagnostic.location().expect("con posicion");

        assert_eq!(location.position(), TextPosition::new(4, 8));
    }

    #[test]
    fn a_message_with_colons_keeps_all_of_its_text() {
        let diagnostic = parse_one("src\\Main.java:1: error: expected one of: error: otro\n");

        assert_eq!(
            diagnostic.message(),
            "expected one of: error: otro",
            "el mensaje se corta en el primer nivel que parece"
        );
    }

    #[test]
    fn several_errors_are_parsed_in_the_order_they_appear() {
        let output = "src\\main\\java\\Main.java:5: warning: [rawtypes] found raw type: List\nsrc\\main\\java\\Main.java:8: error: bad operand types for binary operator '+'\nsrc\\vista\\Panel.java:3: error: cannot find symbol\n";

        let diagnostics = parse_error(output);

        assert_eq!(diagnostics.len(), 3);
        assert_eq!(diagnostics[0].level(), DiagnosticLevel::Warning);
        assert_eq!(diagnostics[1].level(), DiagnosticLevel::Error);
        assert!(diagnostics[1].message().contains("bad operand"));
        assert!(diagnostics[2].message().contains("cannot find symbol"));
    }

    #[test]
    fn a_compilation_without_errors_has_no_diagnostics() {
        let diagnostics = parse(&ProcessOutput::new(Some(0), "", ""), &root());

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    /// `javac` misused as a command prints its whole usage, and none of that is a
    /// compilation error.
    #[test]
    fn the_usage_of_javac_is_not_a_diagnostic() {
        let output = "Usage: javac <options> <source files>\nwhere possible options include:\n  -d <directory>               Specify where to place generated class files\n  -Werror                      Terminate compilation if warnings occur\n";

        let diagnostics = parse_error(output);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn the_standard_output_of_javac_is_not_read_as_diagnostics() {
        let diagnostics = parse(
            &ProcessOutput::new(Some(1), "src\\Main.java:1: error: cannot find symbol\n", ""),
            &root(),
        );

        assert!(
            diagnostics.is_empty(),
            "javac escribe los diagnosticos en stderr: {diagnostics:?}"
        );
    }

    #[test]
    fn a_parsed_diagnostic_can_be_built_by_hand_the_same_way() {
        let location = DiagnosticLocation::new(
            ProjectRelativePath::new("Main.java").unwrap(),
            TextPosition::new(0, 0),
        );
        let manual = Diagnostic::new(DiagnosticLevel::Error, "cannot find symbol", Some(location));

        assert_eq!(manual.level(), DiagnosticLevel::Error);
        assert_eq!(
            manual.location().unwrap().position(),
            TextPosition::new(0, 0)
        );
    }
}
