use std::path::Path;

use crate::core::LanguageId;

/// Como se edita el texto de un lenguaje: sus comentarios y su indentacion.
///
/// La da el proveedor del lenguaje para que el editor no tenga que preguntar
/// si el lenguaje es uno u otro. `indent` no puede estar vacio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditingConfiguration {
    line_comment: Option<&'static str>,
    block_comment: Option<(&'static str, &'static str)>,
    indent: &'static str,
}

impl EditingConfiguration {
    /// `line_comment` es el inicio de un comentario de una linea, y
    /// `block_comment` el inicio y el final de uno de varias lineas. Cualquiera
    /// de los dos puede no existir en el lenguaje.
    pub fn new(
        line_comment: Option<&'static str>,
        block_comment: Option<(&'static str, &'static str)>,
        indent: &'static str,
    ) -> Self {
        Self {
            line_comment,
            block_comment,
            indent,
        }
    }

    /// Inicio del comentario de una linea, si el lenguaje tiene.
    pub fn line_comment(&self) -> Option<&'static str> {
        self.line_comment
    }

    /// Inicio y final del comentario de varias lineas, si el lenguaje tiene.
    pub fn block_comment(&self) -> Option<(&'static str, &'static str)> {
        self.block_comment
    }

    /// Sangria de un nivel. No puede estar vacia.
    pub fn indent(&self) -> &'static str {
        self.indent
    }
}

/// Lo que el core puede consultar de un lenguaje sin saber cual es.
///
/// La implementan los soportes concretos, asi que anadir un lenguaje es anadir
/// un tipo que cumple este contrato y nada mas. El core nunca pregunta por un
/// lenguaje en concreto.
pub trait LanguageProvider {
    /// Identidad abstracta del lenguaje.
    fn id(&self) -> LanguageId;

    /// Extensiones de los archivos del lenguaje, sin el punto y en minusculas.
    fn extensions(&self) -> &'static [&'static str];

    /// Configuracion de edicion del lenguaje.
    fn editing(&self) -> EditingConfiguration;

    /// Si el archivo de `path` es de este lenguaje.
    ///
    /// No le importa al archivo que no tenga extension, y las extensiones se
    /// comparan sin distinguir mayusculas, que es como las da el proveedor.
    fn supports(&self, path: &Path) -> bool {
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            return false;
        };

        self.extensions()
            .iter()
            .any(|known| known.eq_ignore_ascii_case(extension))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::core::LanguageId;
    use crate::language::{EditingConfiguration, LanguageProvider};

    struct FakeLanguage {
        id: LanguageId,
        extensions: &'static [&'static str],
        editing: EditingConfiguration,
    }

    impl LanguageProvider for FakeLanguage {
        fn id(&self) -> LanguageId {
            self.id
        }

        fn extensions(&self) -> &'static [&'static str] {
            self.extensions
        }

        fn editing(&self) -> EditingConfiguration {
            self.editing
        }
    }

    fn editing() -> EditingConfiguration {
        EditingConfiguration::new(Some("//"), Some(("/*", "*/")), "    ")
    }

    fn csharp() -> FakeLanguage {
        FakeLanguage {
            id: LanguageId::CSharp,
            extensions: &["cs", "csx"],
            editing: editing(),
        }
    }

    fn java() -> FakeLanguage {
        FakeLanguage {
            id: LanguageId::Java,
            extensions: &["java"],
            editing: EditingConfiguration::new(Some("//"), Some(("/*", "*/")), "    "),
        }
    }

    #[test]
    fn the_language_identity_comes_from_the_provider() {
        let csharp = csharp();

        assert_eq!(csharp.id(), LanguageId::CSharp);
    }

    #[test]
    fn the_extensions_come_from_the_provider() {
        let csharp = csharp();

        assert_eq!(csharp.extensions(), &["cs", "csx"]);
    }

    #[test]
    fn the_editing_configuration_comes_from_the_provider() {
        let csharp = csharp();

        assert_eq!(csharp.editing().line_comment(), Some("//"));
    }

    #[test]
    fn several_languages_can_be_queried_through_the_same_interface() {
        let languages: [&dyn LanguageProvider; 2] = [&csharp(), &java()];

        let identities: Vec<LanguageId> = languages.iter().map(|l| l.id()).collect();
        let extensions: Vec<&[&str]> = languages.iter().map(|l| l.extensions()).collect();

        assert_eq!(identities, vec![LanguageId::CSharp, LanguageId::Java]);
        assert_eq!(extensions, vec![&["cs", "csx"][..], &["java"][..]]);
    }

    #[test]
    fn a_provider_recognizes_the_files_of_its_own_extensions() {
        let csharp = csharp();

        assert!(csharp.supports(Path::new("src/Main.cs")));
        assert!(csharp.supports(Path::new("Program.csx")));
    }

    #[test]
    fn a_provider_does_not_recognize_the_files_of_another_language() {
        let csharp = csharp();

        assert!(!csharp.supports(Path::new("src/Main.java")));
    }

    #[test]
    fn a_provider_ignores_the_case_of_the_extension() {
        let csharp = csharp();

        assert!(csharp.supports(Path::new("MAIN.CS")));
    }

    #[test]
    fn a_file_without_an_extension_is_not_recognized() {
        let csharp = csharp();

        assert!(!csharp.supports(Path::new("Makefile")));
        assert!(!csharp.supports(Path::new("src/Program")));
    }

    #[test]
    fn an_editing_configuration_exposes_its_comments_and_its_indent() {
        let configuration = editing();

        assert_eq!(configuration.line_comment(), Some("//"));
        assert_eq!(configuration.block_comment(), Some(("/*", "*/")));
        assert_eq!(configuration.indent(), "    ");
    }

    #[test]
    fn an_editing_configuration_can_have_no_comments() {
        let configuration = EditingConfiguration::new(None, None, "\t");

        assert_eq!(configuration.line_comment(), None);
        assert_eq!(configuration.block_comment(), None);
        assert_eq!(configuration.indent(), "\t");
    }
}
