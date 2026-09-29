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

    /// Palabras clave del lenguaje, en minusculas y sin punto y coma.
    ///
    /// Las da el proveedor porque son suyas: el editor sabe pintar una
    /// palabra clave cuando se lo dicen, pero no sabe cuales lo son en cada
    /// lenguaje, y si las tuviera escritas seria del lenguaje y habria que
    /// tocarlo cada vez que se anadiera uno.
    fn keywords(&self) -> &'static [&'static str];

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

        fn keywords(&self) -> &'static [&'static str] {
            &["FAKE"]
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
    fn las_palabras_clave_las_da_el_proveedor() {
        struct SinPalabras;

        impl LanguageProvider for SinPalabras {
            fn id(&self) -> LanguageId {
                LanguageId::CSharp
            }

            fn extensions(&self) -> &'static [&'static str] {
                &["txt"]
            }

            fn editing(&self) -> EditingConfiguration {
                editing()
            }

            fn keywords(&self) -> &'static [&'static str] {
                &["PRUEBA"]
            }
        }

        let provider = SinPalabras;

        assert_eq!(provider.keywords(), &["PRUEBA"]);
    }

    /// Cada lenguaje dice cuales son sus palabras clave, y son distintas.
    ///
    /// Son suyas y no del editor porque el editor no puede saber que en C# se escribe
    /// `namespace` y en Java `package`: si las tuviera escritas, el editor seria del
    /// lenguaje, y para añadir un lenguaje habria que tocarlo. El resaltado sabe pintar
    /// comentarios, cadenas y palabras; lo que es una palabra clave se lo pregunta al
    /// proveedor.
    #[test]
    fn cada_lenguaje_declara_sus_palabras_clave() {
        let csharp = crate::supports::CSharp;
        let java = crate::supports::Java;

        assert!(
            csharp.keywords().contains(&"namespace"),
            "namespace es de C#: {:?}",
            csharp.keywords()
        );
        assert!(
            java.keywords().contains(&"package"),
            "package es de Java: {:?}",
            java.keywords()
        );
        assert!(
            !csharp.keywords().contains(&"package"),
            "package no es de C#: {:?}",
            csharp.keywords()
        );
        assert!(
            !java.keywords().contains(&"namespace"),
            "namespace no es de Java: {:?}",
            java.keywords()
        );
    }

    /// Los dos lenguajes comparten las palabras que los dos tienen.
    ///
    /// Va en su propio test porque es lo que hace que el resaltado de los dos se parezca:
    /// si un lenguaje no tuviera `class`, un archivo de C# y otro de Java se verían
    /// distintos por todo, y no por el lenguaje del que son.
    #[test]
    fn los_dos_lenguajes_comparten_lo_que_tienen_en_comun() {
        let csharp = crate::supports::CSharp;
        let java = crate::supports::Java;

        for comun in [
            "class", "public", "void", "return", "if", "new", "true", "null",
        ] {
            assert!(
                csharp.keywords().contains(&comun) && java.keywords().contains(&comun),
                "{comun} es de los dos: {:?} y {:?}",
                csharp.keywords(),
                java.keywords()
            );
        }
    }

    /// Las palabras clave no tienen mayusculas ni punto y coma.
    ///
    /// Se comprueba porque el resaltado las compara con la palabra tal y como está
    /// escrita, y una lista con `Class` o con `class;` no se encontraría nunca. Va en su
    /// propio test para que el fallo diga qué está mal y no que un archivo no se resalta.
    #[test]
    fn las_palabras_clave_estan_las_que_son() {
        let csharp = crate::supports::CSharp;
        let java = crate::supports::Java;

        for provider in [&csharp as &dyn LanguageProvider, &java] {
            assert!(
                !provider.keywords().is_empty(),
                "{:?} tiene que decir sus palabras clave",
                provider.id()
            );

            for palabra in provider.keywords() {
                assert!(
                    palabra
                        .chars()
                        .all(|letra| letra.is_ascii_alphabetic() || letra == '_'),
                    "{palabra:?} de {:?} solo puede llevar letras y guiones bajos",
                    provider.id()
                );
            }
        }
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
