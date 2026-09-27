use std::any::Any;
use std::error::Error;
use std::fmt;
use std::io;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    InvalidName(String),
    InvalidPath(String),
    InvalidPosition(String),
    NotFound(String),
    Unsupported(String),
    /// Ya existe algo en la ruta que se queria crear.
    AlreadyExists(String),
    /// El archivo tiene una zona generada que no esta cerrada.
    MalformedSource(String),
    /// El sistema de archivos ha rechazado la operacion.
    Io(String),
    /// Un fallo de MiniIDE que no se esperaba y que el usuario no ha podido
    /// provocar.
    ///
    /// Es el unico que no describe un problema del proyecto ni del sistema, y por eso
    /// va aparte: no se corrige mirando el codigo del usuario ni el disco, sino el de
    /// MiniIDE. Va separado para que no se confunda con los demas al contar que ha
    /// fallado y de donde.
    Internal(String),
}

impl CoreError {
    /// Traduce un error del sistema de archivos al error del core que mejor lo
    /// describe.
    ///
    /// Vive aqui para que ningun modulo tenga que decidir por su cuenta que
    /// error de disco es cual.
    pub fn from_io(path: &Path, error: &io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound(path.display().to_string()),
            io::ErrorKind::AlreadyExists => Self::AlreadyExists(path.display().to_string()),
            _ => Self::Io(format!("{}: {error}", path.display())),
        }
    }

    /// Convierte lo que lleva un panic en el error del core que lo describe.
    ///
    /// Un panic nunca deberia llegar aqui: es un fallo del propio MiniIDE, no algo
    /// que el usuario pueda haber hecho. Pero si llega, tiene que volver por el camino
    /// normal de los errores y no hundir el hilo en el que ocurre, o el IDE se
    /// quedaria sin poder hacer nada con el fallo.
    ///
    /// El texto del panic se enseña dentro del mensaje y no suelto, porque el
    /// usuario necesita ver que ha pasado y porque asi se puede buscar despues. No
    /// lleva mas contexto que este: quien recoge el panic sabe en que parte del
    /// trabajo ha ocurrido, y anadirlo aqui seria suponerlo.
    ///
    /// Lo que lleva el panic se toma en propiedad, y no por referencia, porque es lo
    /// que entrega `catch_unwind` y porque borrowed ahi no encuentra el tipo: sobre
    /// una referencia el downcast da que no hay detalle y el fallo se queda sin decir
    /// nada, que es justo lo que no puede pasar.
    pub fn from_panic(panic: Box<dyn Any + Send>) -> Self {
        let detail = panic
            .downcast_ref::<&str>()
            .map(|text| (*text).to_string())
            .or_else(|| panic.downcast_ref::<String>().cloned());

        match detail {
            Some(detail) => Self::Internal(format!("fallo inesperado del IDE: {detail}")),
            // Un panic puede no decir nada. El error sigue teniendo que decir algo,
            // porque un mensaje vacio no se puede enseñar ni buscar.
            None => Self::Internal("fallo inesperado del IDE, sin detalle".to_string()),
        }
    }

    pub fn message(&self) -> &str {
        match self {
            CoreError::InvalidName(value)
            | CoreError::InvalidPath(value)
            | CoreError::InvalidPosition(value)
            | CoreError::NotFound(value)
            | CoreError::Unsupported(value)
            | CoreError::AlreadyExists(value)
            | CoreError::MalformedSource(value)
            | CoreError::Io(value)
            | CoreError::Internal(value) => value,
        }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            CoreError::InvalidName(_) => "invalid name",
            CoreError::InvalidPath(_) => "invalid path",
            CoreError::InvalidPosition(_) => "invalid position",
            CoreError::NotFound(_) => "not found",
            CoreError::Unsupported(_) => "unsupported",
            CoreError::AlreadyExists(_) => "already exists",
            CoreError::MalformedSource(_) => "malformed source",
            CoreError::Io(_) => "io error",
            CoreError::Internal(_) => "internal error",
        };

        write!(f, "{label}: {}", self.message())
    }
}

impl Error for CoreError {}

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LanguageId {
    CSharp,
    Java,
}

impl LanguageId {
    pub const ALL: [LanguageId; 2] = [LanguageId::CSharp, LanguageId::Java];

    pub fn as_str(&self) -> &'static str {
        match self {
            LanguageId::CSharp => "csharp",
            LanguageId::Java => "java",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            LanguageId::CSharp => "C#",
            LanguageId::Java => "Java",
        }
    }
}

impl fmt::Display for LanguageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

impl FromStr for LanguageId {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "csharp" => Ok(LanguageId::CSharp),
            "java" => Ok(LanguageId::Java),
            _ => Err(CoreError::Unsupported(value.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FrameworkId {
    WinForms,
    Swing,
}

impl FrameworkId {
    pub const ALL: [FrameworkId; 2] = [FrameworkId::WinForms, FrameworkId::Swing];

    pub fn as_str(&self) -> &'static str {
        match self {
            FrameworkId::WinForms => "winforms",
            FrameworkId::Swing => "swing",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            FrameworkId::WinForms => "Windows Forms",
            FrameworkId::Swing => "Swing",
        }
    }
}

impl fmt::Display for FrameworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

impl FromStr for FrameworkId {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "winforms" => Ok(FrameworkId::WinForms),
            "swing" => Ok(FrameworkId::Swing),
            _ => Err(CoreError::Unsupported(value.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProjectType {
    CSharpWinForms,
    JavaSwing,
}

impl ProjectType {
    pub const ALL: [ProjectType; 2] = [ProjectType::CSharpWinForms, ProjectType::JavaSwing];

    pub fn language(&self) -> LanguageId {
        match self {
            ProjectType::CSharpWinForms => LanguageId::CSharp,
            ProjectType::JavaSwing => LanguageId::Java,
        }
    }

    pub fn framework(&self) -> FrameworkId {
        match self {
            ProjectType::CSharpWinForms => FrameworkId::WinForms,
            ProjectType::JavaSwing => FrameworkId::Swing,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProjectType::CSharpWinForms => "C# / Windows Forms",
            ProjectType::JavaSwing => "Java / Swing",
        }
    }

    pub fn from_parts(language: LanguageId, framework: FrameworkId) -> CoreResult<Self> {
        match (language, framework) {
            (LanguageId::CSharp, FrameworkId::WinForms) => Ok(ProjectType::CSharpWinForms),
            (LanguageId::Java, FrameworkId::Swing) => Ok(ProjectType::JavaSwing),
            _ => Err(CoreError::Unsupported(format!(
                "{} / {}",
                language.display_name(),
                framework.display_name()
            ))),
        }
    }
}

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn language_id_exposes_a_stable_identifier_and_a_display_name() {
        assert_eq!(LanguageId::CSharp.as_str(), "csharp");
        assert_eq!(LanguageId::CSharp.display_name(), "C#");
        assert_eq!(LanguageId::Java.as_str(), "java");
        assert_eq!(LanguageId::Java.display_name(), "Java");
    }

    #[test]
    fn language_id_parses_its_identifier_ignoring_case() {
        assert_eq!(LanguageId::from_str("CSharp"), Ok(LanguageId::CSharp));
        assert_eq!(LanguageId::from_str("JAVA"), Ok(LanguageId::Java));
    }

    #[test]
    fn language_id_rejects_an_unknown_identifier() {
        let result = LanguageId::from_str("rust");

        assert_eq!(result, Err(CoreError::Unsupported("rust".to_string())));
    }

    #[test]
    fn language_id_displays_the_display_name() {
        assert_eq!(LanguageId::CSharp.to_string(), "C#");
        assert_eq!(LanguageId::Java.to_string(), "Java");
    }

    #[test]
    fn framework_id_exposes_a_stable_identifier_and_a_display_name() {
        assert_eq!(FrameworkId::WinForms.as_str(), "winforms");
        assert_eq!(FrameworkId::WinForms.display_name(), "Windows Forms");
        assert_eq!(FrameworkId::Swing.as_str(), "swing");
        assert_eq!(FrameworkId::Swing.display_name(), "Swing");
    }

    #[test]
    fn framework_id_parses_its_identifier_ignoring_case() {
        assert_eq!(FrameworkId::from_str("WinForms"), Ok(FrameworkId::WinForms));
        assert_eq!(FrameworkId::from_str("SWING"), Ok(FrameworkId::Swing));
    }

    #[test]
    fn framework_id_rejects_an_unknown_identifier() {
        let result = FrameworkId::from_str("javafx");

        assert_eq!(result, Err(CoreError::Unsupported("javafx".to_string())));
    }

    #[test]
    fn framework_id_displays_the_display_name() {
        assert_eq!(FrameworkId::WinForms.to_string(), "Windows Forms");
        assert_eq!(FrameworkId::Swing.to_string(), "Swing");
    }

    #[test]
    fn framework_id_all_lists_every_supported_framework() {
        assert_eq!(
            FrameworkId::ALL,
            [FrameworkId::WinForms, FrameworkId::Swing]
        );
    }

    #[test]
    fn project_type_exposes_its_language_and_framework() {
        assert_eq!(ProjectType::CSharpWinForms.language(), LanguageId::CSharp);
        assert_eq!(
            ProjectType::CSharpWinForms.framework(),
            FrameworkId::WinForms
        );
        assert_eq!(ProjectType::JavaSwing.language(), LanguageId::Java);
        assert_eq!(ProjectType::JavaSwing.framework(), FrameworkId::Swing);
    }

    #[test]
    fn project_type_exposes_a_display_name() {
        assert_eq!(
            ProjectType::CSharpWinForms.display_name(),
            "C# / Windows Forms"
        );
        assert_eq!(ProjectType::JavaSwing.display_name(), "Java / Swing");
        assert_eq!(ProjectType::JavaSwing.to_string(), "Java / Swing");
    }

    #[test]
    fn project_type_all_lists_every_supported_project_type() {
        assert_eq!(
            ProjectType::ALL,
            [ProjectType::CSharpWinForms, ProjectType::JavaSwing]
        );
    }

    #[test]
    fn project_types_cover_distinct_language_framework_pairs() {
        let mut pairs: Vec<(LanguageId, FrameworkId)> = ProjectType::ALL
            .iter()
            .map(|project_type| (project_type.language(), project_type.framework()))
            .collect();

        let total = pairs.len();
        pairs.sort();
        pairs.dedup();

        assert_eq!(pairs.len(), total, "two project types share the same pair");
    }

    #[test]
    fn project_type_from_parts_accepts_every_supported_combination() {
        for project_type in ProjectType::ALL {
            let result = ProjectType::from_parts(project_type.language(), project_type.framework());

            assert_eq!(result, Ok(project_type));
        }
    }

    #[test]
    fn project_type_from_parts_rejects_an_unsupported_combination() {
        let result = ProjectType::from_parts(LanguageId::CSharp, FrameworkId::Swing);

        assert_eq!(
            result,
            Err(CoreError::Unsupported("C# / Swing".to_string()))
        );
    }

    #[test]
    fn message_returns_the_offending_value() {
        let error = CoreError::InvalidPath("../outside".to_string());

        assert_eq!(error.message(), "../outside");
    }

    #[test]
    fn display_includes_kind_and_value() {
        let error = CoreError::NotFound("MainForm.cs".to_string());

        assert_eq!(error.to_string(), "not found: MainForm.cs");
    }

    #[test]
    fn core_result_carries_core_error() {
        let result: CoreResult<u8> = Err(CoreError::Unsupported("WPF".to_string()));

        assert_eq!(result, Err(CoreError::Unsupported("WPF".to_string())));
    }

    /// Un fallo que MiniIDE no esperaba es un fallo interno, y como tal se
    /// clasifica: no es ni un problema del usuario ni del disco ni de la
    /// herramienta, y mezclarlo con ellos haria que se corrigiera lo que no toca.
    #[test]
    fn an_unexpected_failure_is_classified_as_internal() {
        let panic: Box<dyn std::any::Any + Send> = Box::new("la toolchain ha fallado por dentro");

        let error = CoreError::from_panic(panic);

        assert_eq!(
            error,
            CoreError::Internal(
                "fallo inesperado del IDE: la toolchain ha fallado por dentro".to_string()
            )
        );
        assert_eq!(
            error.to_string(),
            "internal error: fallo inesperado del IDE: la toolchain ha fallado por dentro"
        );
    }

    /// Un panic puede no decir nada, y tampoco por eso puede quedar un error sin
    /// texto: un mensaje vacio no se puede enseñar ni buscar.
    #[test]
    fn an_unexpected_failure_without_detail_still_has_a_message() {
        let panic: Box<dyn std::any::Any + Send> = Box::new(7_u8);

        let error = CoreError::from_panic(panic);

        assert_eq!(
            error,
            CoreError::Internal("fallo inesperado del IDE, sin detalle".to_string())
        );
        assert!(!error.message().is_empty());
    }

    /// Un panic cuyo texto viene en un `String` se enseña igual que si viniera
    /// prestado: los dos son el caso de verdad, y el segundo es el que se ve.
    #[test]
    fn an_unexpected_failure_can_carry_an_owned_message() {
        let panic: Box<dyn std::any::Any + Send> = Box::new(String::from("indice fuera de rango"));

        let error = CoreError::from_panic(panic);

        assert_eq!(
            error.message(),
            "fallo inesperado del IDE: indice fuera de rango"
        );
    }
}
