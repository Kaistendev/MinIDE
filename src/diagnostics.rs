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

/// Un grupo de diagnosticos del mismo nivel.
///
/// El panel los enseia juntos: los errores en un sitio y las advertencias en otro,
/// no mezclados en una sola lista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticGroup {
    level: DiagnosticLevel,
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticGroup {
    /// Un grupo del nivel `level`, sin diagnosticos.
    pub fn new(level: DiagnosticLevel) -> Self {
        Self {
            level,
            diagnostics: Vec::new(),
        }
    }

    /// Anade `diagnostic` al grupo. Solo si es del nivel del grupo: si no, el
    /// grupo dejaria de ser un grupo.
    ///
    /// Devuelve si lo ha admitido, para que quien agrupe no se equivoque.
    pub fn push(&mut self, diagnostic: Diagnostic) -> bool {
        if diagnostic.level() != self.level {
            return false;
        }

        self.diagnostics.push(diagnostic);

        true
    }

    pub fn level(&self) -> DiagnosticLevel {
        self.level
    }

    /// Los diagnosticos del grupo, en el orden en que los dio la herramienta.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Los diagnosticos de una operacion, como los muestra el panel.
///
/// Van agrupados por nivel y con los errores primero: en una lista mezclada, los
/// errores se pierden entre las advertencias, y son lo que hay que arreglar antes
/// de seguir.
///
/// El panel guarda cual de ellos ha elegido el usuario, y de ahi sale el sitio al
/// que hay que ir cuando el diagnostico tiene posicion. El que dibuja el panel es
/// el frontend: aqui solo esta lo que se enseña y en que orden.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiagnosticsPanel {
    groups: Vec<DiagnosticGroup>,
    selected: Option<usize>,
}

impl DiagnosticsPanel {
    /// Panel con `diagnostics` agrupados por nivel, con los errores primero.
    ///
    /// Los diagnosticos se copian: un panel es una foto de como estaba la
    /// compilacion, y el resultado del que viene puede seguir cambiando mientras se
    /// mira. Por eso se construye desde una referencia y no desde el resultado en
    /// propiedad.
    ///
    /// Un grupo vacio no se crea: si no hay errores, no hay grupo de errores, y
    /// un titulo de "errores (0)" solo haria ruido.
    pub fn new(diagnostics: &[Diagnostic]) -> Self {
        let mut sorted = diagnostics.to_vec();
        let mut panel = Self::default();

        // Se ordena por nivel y se agrupa de uno en uno. `sort_by_key` es estable,
        // asi que dentro de un grupo los diagnosticos siguen el orden en que los
        // dio la herramienta.
        sorted.sort_by_key(|diagnostic| order_of(diagnostic.level()));

        for diagnostic in sorted {
            let group = match panel.groups.last_mut() {
                Some(group) if group.level() == diagnostic.level() => group,
                _ => {
                    panel.groups.push(DiagnosticGroup::new(diagnostic.level()));
                    panel.groups.last_mut().expect("el grupo recien creado")
                }
            };

            let accepted = group.push(diagnostic);

            debug_assert!(accepted, "el grupo es del nivel del diagnostico");
        }

        panel
    }

    /// Los grupos del panel, en el orden en que los muestra.
    pub fn groups(&self) -> &[DiagnosticGroup] {
        &self.groups
    }

    /// Cuantos diagnosticos hay en el panel, sin contar por grupos.
    pub fn len(&self) -> usize {
        self.groups.iter().map(DiagnosticGroup::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// Los diagnosticos de un nivel, en el orden en que los dio la herramienta.
    ///
    /// Es lo que va bajo un titulo del panel, con su cuenta. Un nivel que no esta
    /// en el panel es un nivel que no hay, y sale vacio.
    pub fn of_level(&self, level: DiagnosticLevel) -> &[Diagnostic] {
        self.groups
            .iter()
            .find(|group| group.level() == level)
            .map_or(&[], DiagnosticGroup::diagnostics)
    }

    /// Elige el diagnostico que ocupa la posicion `index` de lo que muestra el
    /// panel: los grupos en orden y, dentro de cada grupo, sus diagnosticos en
    /// orden.
    ///
    /// Devuelve si ha podido. Una posicion que no existe es un `false` y deja la
    /// seleccion como estaba: en el panel se pincha en una fila, y una fila que no
    /// hay no es un error.
    pub fn select(&mut self, index: usize) -> bool {
        if index >= self.len() {
            return false;
        }

        self.selected = Some(index);

        true
    }

    /// El diagnostico elegido, si hay alguno.
    pub fn selection(&self) -> Option<&Diagnostic> {
        self.selected.and_then(|index| self.at(index))
    }

    /// Quita la seleccion.
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    /// A donde lleva el diagnostico elegido, si lleva.
    ///
    /// Es `None` cuando no hay nada elegido y cuando el diagnostico no tiene
    /// posicion: un error sin archivo no lleva a ninguna parte, y el panel no
    /// puede inventarse una.
    pub fn target(&self) -> Option<&DiagnosticLocation> {
        self.selection()?.location()
    }

    /// El diagnostico de la posicion `index` de lo que muestra el panel.
    fn at(&self, index: usize) -> Option<&Diagnostic> {
        let mut left = index;

        for group in &self.groups {
            if left < group.len() {
                return group.diagnostics.get(left);
            }

            left -= group.len();
        }

        None
    }
}

/// Los niveles en el orden en que los muestra el panel: lo que hay que arreglar
/// antes, primero.
const ORDERED_LEVELS: [DiagnosticLevel; 3] = [
    DiagnosticLevel::Error,
    DiagnosticLevel::Warning,
    DiagnosticLevel::Info,
];

/// La posicion de `level` en el orden del panel. Un nivel que no esta en la lista
/// va al final, para que no se quede fuera del panel.
fn order_of(level: DiagnosticLevel) -> usize {
    ORDERED_LEVELS
        .iter()
        .position(|known| *known == level)
        .unwrap_or(ORDERED_LEVELS.len())
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

    fn located(level: DiagnosticLevel, message: &str, file: &str, line: u32) -> Diagnostic {
        Diagnostic::new(
            level,
            message,
            Some(DiagnosticLocation::new(
                ProjectRelativePath::new(file).unwrap(),
                TextPosition::new(line, 0),
            )),
        )
    }

    fn plain(level: DiagnosticLevel, message: &str) -> Diagnostic {
        Diagnostic::new(level, message, None)
    }

    /// Un panel con errores, advertencias y avisos, como los deja una compilacion
    /// que no va bien del todo.
    fn panel_with_a_mix() -> DiagnosticsPanel {
        DiagnosticsPanel::new(&[
            located(
                DiagnosticLevel::Warning,
                "CS0219: sin usar",
                "src/Main.cs",
                3,
            ),
            plain(DiagnosticLevel::Error, "no se encontro el SDK"),
            located(
                DiagnosticLevel::Error,
                "CS1002: falta punto y coma",
                "src/Main.cs",
                11,
            ),
            plain(DiagnosticLevel::Info, "compilando en Release"),
        ])
    }

    #[test]
    fn the_errors_come_before_the_warnings_and_the_warnings_before_the_infos() {
        let panel = panel_with_a_mix();

        let levels: Vec<DiagnosticLevel> =
            panel.groups().iter().map(DiagnosticGroup::level).collect();

        assert_eq!(
            levels,
            vec![
                DiagnosticLevel::Error,
                DiagnosticLevel::Warning,
                DiagnosticLevel::Info
            ]
        );
    }

    #[test]
    fn the_diagnostics_are_grouped_by_level() {
        let panel = panel_with_a_mix();

        assert_eq!(panel.groups().len(), 3);
        assert_eq!(panel.groups()[0].len(), 2);
        assert_eq!(panel.groups()[1].len(), 1);
        assert_eq!(panel.groups()[2].len(), 1);
    }

    #[test]
    fn a_group_keeps_the_order_in_which_its_diagnostics_arrived() {
        let panel = panel_with_a_mix();

        let errors: Vec<&str> = panel
            .of_level(DiagnosticLevel::Error)
            .iter()
            .map(|diagnostic| diagnostic.message())
            .collect();

        assert_eq!(
            errors,
            vec!["no se encontro el SDK", "CS1002: falta punto y coma"]
        );
    }

    #[test]
    fn a_panel_can_say_how_many_diagnostics_of_each_level_there_are() {
        let panel = panel_with_a_mix();

        assert_eq!(panel.of_level(DiagnosticLevel::Error).len(), 2);
        assert_eq!(panel.of_level(DiagnosticLevel::Warning).len(), 1);
        assert_eq!(panel.of_level(DiagnosticLevel::Info).len(), 1);
        assert_eq!(panel.len(), 4);
    }

    #[test]
    fn a_panel_without_diagnostics_has_no_groups() {
        let panel = DiagnosticsPanel::new(&[]);

        assert!(panel.is_empty());
        assert_eq!(panel.len(), 0);
        assert!(panel.groups().is_empty());
    }

    #[test]
    fn a_group_is_never_empty() {
        let panel = panel_with_a_mix();

        for group in panel.groups() {
            assert!(
                !group.is_empty(),
                "un grupo vacio solo haria ruido en el panel"
            );
        }
    }

    /// El panel se elige por la posicion que ocupa en lo que se muestra, y lo que
    /// se muestra va por grupos.
    #[test]
    fn a_diagnostic_can_be_selected_by_its_position_in_the_panel() {
        let mut panel = panel_with_a_mix();

        assert!(panel.select(0));

        assert_eq!(
            panel.selection().map(|diagnostic| diagnostic.message()),
            Some("no se encontro el SDK")
        );
    }

    #[test]
    fn a_position_that_is_not_there_selects_nothing() {
        let mut panel = panel_with_a_mix();

        assert!(!panel.select(99));
        assert_eq!(panel.selection(), None);
    }

    #[test]
    fn selecting_another_diagnostic_replaces_the_previous_one() {
        let mut panel = panel_with_a_mix();
        panel.select(0);

        panel.select(1);

        assert_eq!(
            panel.selection().map(|diagnostic| diagnostic.message()),
            Some("CS1002: falta punto y coma")
        );
    }

    #[test]
    fn the_selection_can_be_cleared() {
        let mut panel = panel_with_a_mix();
        panel.select(0);

        panel.clear_selection();

        assert_eq!(panel.selection(), None);
    }

    /// Seleccionar un diagnostico con posicion es pedir que se abra su sitio.
    #[test]
    fn a_selected_diagnostic_with_position_says_where_to_go() {
        let mut panel = panel_with_a_mix();
        panel.select(1);

        let target = panel.target().expect("hay a donde ir");

        assert_eq!(target.file().as_path(), Path::new("src/Main.cs"));
        assert_eq!(target.position(), TextPosition::new(11, 0));
    }

    /// Un error sin posicion no lleva a ninguna parte: el panel no puede
    /// inventarse un archivo al que ir.
    #[test]
    fn a_selected_diagnostic_without_position_has_nowhere_to_go() {
        let mut panel = panel_with_a_mix();
        panel.select(0);

        assert!(panel.target().is_none());
    }

    #[test]
    fn with_nothing_selected_there_is_nowhere_to_go() {
        let panel = panel_with_a_mix();

        assert_eq!(panel.target(), None);
    }

    /// Un panel de una compilacion que no ha dado ningun problema no muestra
    /// ningun grupo, tampoco vacios.
    #[test]
    fn a_clean_compilation_has_an_empty_panel() {
        let panel = DiagnosticsPanel::new(&[]);

        assert!(panel.is_empty());
        assert!(panel.of_level(DiagnosticLevel::Error).is_empty());
    }

    #[test]
    fn a_group_only_takes_diagnostics_of_its_own_level() {
        let mut group = DiagnosticGroup::new(DiagnosticLevel::Error);

        assert!(group.push(plain(DiagnosticLevel::Error, "CS1002: falta punto y coma")));
        assert!(
            !group.push(plain(DiagnosticLevel::Warning, "CS0219: sin usar")),
            "un aviso en un grupo de errores dejaria el grupo mintiendo"
        );
        assert_eq!(group.len(), 1);
    }
}
