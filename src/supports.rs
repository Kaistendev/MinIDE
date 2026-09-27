use std::path::Path;

use crate::core::{FrameworkId, LanguageId, ProjectType};
use crate::framework::{FrameworkCapabilities, FrameworkProvider};
use crate::language::{EditingConfiguration, LanguageProvider};
use crate::toolchain::{DotNetToolchain, ToolchainProvider};

/// Soporte de C#.
pub struct CSharp;

impl LanguageProvider for CSharp {
    fn id(&self) -> LanguageId {
        LanguageId::CSharp
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["cs", "csx"]
    }

    fn editing(&self) -> EditingConfiguration {
        EditingConfiguration::new(Some("//"), Some(("/*", "*/")), "    ")
    }
}

/// Soporte de Java.
pub struct Java;

impl LanguageProvider for Java {
    fn id(&self) -> LanguageId {
        LanguageId::Java
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["java"]
    }

    fn editing(&self) -> EditingConfiguration {
        EditingConfiguration::new(Some("//"), Some(("/*", "*/")), "    ")
    }
}

/// Soporte de Windows Forms.
pub struct WinForms;

impl FrameworkProvider for WinForms {
    fn id(&self) -> FrameworkId {
        FrameworkId::WinForms
    }

    fn capabilities(&self) -> FrameworkCapabilities {
        FrameworkCapabilities::new(
            "Form",
            &["Button", "Label", "TextBox", "CheckBox", "ListBox", "Panel"],
            true,
        )
    }
}

/// Soporte de Swing.
pub struct Swing;

impl FrameworkProvider for Swing {
    fn id(&self) -> FrameworkId {
        FrameworkId::Swing
    }

    fn capabilities(&self) -> FrameworkCapabilities {
        FrameworkCapabilities::new(
            "JFrame",
            &[
                "JButton",
                "JLabel",
                "JTextField",
                "JCheckBox",
                "JList",
                "JPanel",
            ],
            true,
        )
    }
}

/// Los soportes tecnologicos que MiniIDE tiene, y como se llegan a ellos.
///
/// Anadir un lenguaje o un framework es registrar su soporte aqui, no anadir una
/// comprobacion condicional en el core. Quien necesita un soporte lo pide por su
/// identidad, y el core nunca pregunta si el lenguaje es este o el otro.
#[derive(Default)]
pub struct Supports {
    languages: Vec<Box<dyn LanguageProvider>>,
    frameworks: Vec<Box<dyn FrameworkProvider>>,
    toolchains: Vec<Box<dyn ToolchainProvider>>,
}

impl Supports {
    /// Registro vacio.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Los soportes iniciales: C# con Windows Forms y Java con Swing.
    pub fn initial() -> Self {
        let mut supports = Self::empty();

        supports.register_language(Box::new(CSharp));
        supports.register_language(Box::new(Java));
        supports.register_framework(Box::new(WinForms));
        supports.register_framework(Box::new(Swing));
        supports.register_toolchain(Box::new(DotNetToolchain));

        supports
    }

    /// Registra un lenguaje. Si ya habia uno con la misma identidad, el ultimo
    /// registrado lo sustituye.
    pub fn register_language(&mut self, language: Box<dyn LanguageProvider>) {
        let id = language.id();

        self.languages.retain(|known| known.id() != id);
        self.languages.push(language);
    }

    /// Registra un framework. Si ya habia uno con la misma identidad, el ultimo
    /// registrado lo sustituye.
    pub fn register_framework(&mut self, framework: Box<dyn FrameworkProvider>) {
        let id = framework.id();

        self.frameworks.retain(|known| known.id() != id);
        self.frameworks.push(framework);
    }

    /// Registra una toolchain. Si ya habia una para el mismo tipo de proyecto,
    /// la ultima registrada la sustituye.
    pub fn register_toolchain(&mut self, toolchain: Box<dyn ToolchainProvider>) {
        let project_type = toolchain.project_type();

        self.toolchains
            .retain(|known| known.project_type() != project_type);
        self.toolchains.push(toolchain);
    }

    /// Toolchain registrada para ese tipo de proyecto, si la hay.
    ///
    /// Java todavia no tiene toolchain, asi que se devuelve `None` para ella.
    pub fn toolchain(&self, project_type: ProjectType) -> Option<&dyn ToolchainProvider> {
        self.toolchains
            .iter()
            .find(|toolchain| toolchain.project_type() == project_type)
            .map(|toolchain| toolchain.as_ref())
    }

    /// Toolchains registradas, en orden de registro.
    pub fn toolchain_types(&self) -> Vec<ProjectType> {
        self.toolchains
            .iter()
            .map(|toolchain| toolchain.project_type())
            .collect()
    }

    /// Lenguaje registrado con esa identidad.
    pub fn language(&self, id: LanguageId) -> Option<&dyn LanguageProvider> {
        self.languages
            .iter()
            .find(|language| language.id() == id)
            .map(|language| language.as_ref())
    }

    /// Lenguaje que maneja el archivo de `path`.
    ///
    /// Es el sentido inverso de `language`: el editor recibe un archivo y asi
    /// sabe que lenguaje es y que configuracion de edicion le toca, sin
    /// preguntar si el archivo es de C# o de Java. Devuelve `None` si ningun
    /// lenguaje registrado maneja esa extension.
    ///
    /// Dos lenguajes no pueden reclamar la misma extension, asi que el
    /// resultado no es ambiguo.
    pub fn language_for(&self, path: &Path) -> Option<&dyn LanguageProvider> {
        self.languages
            .iter()
            .find(|language| language.supports(path))
            .map(|language| language.as_ref())
    }

    /// Framework registrado con esa identidad.
    pub fn framework(&self, id: FrameworkId) -> Option<&dyn FrameworkProvider> {
        self.frameworks
            .iter()
            .find(|framework| framework.id() == id)
            .map(|framework| framework.as_ref())
    }

    /// Idiomas registrados, en orden de registro.
    pub fn languages(&self) -> Vec<LanguageId> {
        self.languages
            .iter()
            .map(|language| language.id())
            .collect()
    }

    /// Frameworks registrados, en orden de registro.
    pub fn frameworks(&self) -> Vec<FrameworkId> {
        self.frameworks
            .iter()
            .map(|framework| framework.id())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::core::{FrameworkId, LanguageId, ProjectType};
    use crate::framework::{FrameworkCapabilities, FrameworkProvider};
    use crate::language::{EditingConfiguration, LanguageProvider};
    use crate::supports::Supports;

    #[test]
    fn the_initial_supports_have_csharp() {
        let supports = Supports::initial();

        let csharp = supports.language(LanguageId::CSharp).unwrap();

        assert_eq!(csharp.id(), LanguageId::CSharp);
        assert!(
            csharp.extensions().contains(&"cs"),
            "{:?}",
            csharp.extensions()
        );
        assert!(csharp.supports(Path::new("src/Main.cs")));
    }

    #[test]
    fn the_initial_supports_have_java() {
        let supports = Supports::initial();

        let java = supports.language(LanguageId::Java).unwrap();

        assert_eq!(java.id(), LanguageId::Java);
        assert_eq!(java.extensions(), &["java"]);
        assert!(java.supports(Path::new("src/Main.java")));
    }

    #[test]
    fn the_initial_supports_have_winforms() {
        let supports = Supports::initial();

        let winforms = supports.framework(FrameworkId::WinForms).unwrap();

        assert_eq!(winforms.id(), FrameworkId::WinForms);
        assert_eq!(winforms.capabilities().root(), "Form");
    }

    #[test]
    fn the_initial_supports_have_swing() {
        let supports = Supports::initial();

        let swing = supports.framework(FrameworkId::Swing).unwrap();

        assert_eq!(swing.id(), FrameworkId::Swing);
        assert_eq!(swing.capabilities().root(), "JFrame");
    }

    #[test]
    fn every_supported_combination_resolves_its_language_and_its_framework() {
        let supports = Supports::initial();

        for project_type in ProjectType::ALL {
            assert!(
                supports.language(project_type.language()).is_some(),
                "{project_type:?} no tiene lenguaje"
            );
            assert!(
                supports.framework(project_type.framework()).is_some(),
                "{project_type:?} no tiene framework"
            );
        }
    }

    #[test]
    fn the_languages_declare_their_editing_configuration() {
        let supports = Supports::initial();

        for language in [LanguageId::CSharp, LanguageId::Java] {
            let editing = supports.language(language).unwrap().editing();

            assert_eq!(editing.line_comment(), Some("//"), "{language}");
            assert!(!editing.indent().is_empty(), "{language}");
        }
    }

    #[test]
    fn each_framework_declares_its_own_control_names() {
        let supports = Supports::initial();

        let winforms = supports.framework(FrameworkId::WinForms).unwrap();
        let swing = supports.framework(FrameworkId::Swing).unwrap();

        let winforms_components = winforms.capabilities().components();
        let swing_components = swing.capabilities().components();

        assert!(winforms_components.contains(&"Button"));
        assert!(winforms_components.contains(&"Label"));
        assert!(swing_components.contains(&"JButton"));
        assert!(swing_components.contains(&"JLabel"));
        assert!(!winforms_components.contains(&"JButton"));
        assert!(!swing_components.contains(&"Button"));
    }

    #[test]
    fn a_registered_language_is_found_without_touching_the_registry() {
        struct Fake;

        impl LanguageProvider for Fake {
            fn id(&self) -> LanguageId {
                LanguageId::CSharp
            }

            fn extensions(&self) -> &'static [&'static str] {
                &["cs", "csx", "cake"]
            }

            fn editing(&self) -> EditingConfiguration {
                EditingConfiguration::new(Some("//"), Some(("/*", "*/")), "  ")
            }
        }

        let mut supports = Supports::initial();

        supports.register_language(Box::new(Fake));

        assert!(supports
            .language(LanguageId::CSharp)
            .unwrap()
            .supports(Path::new("src/cake.csx")));
    }

    #[test]
    fn an_empty_registry_resolves_nothing() {
        let supports = Supports::empty();

        assert!(supports.language(LanguageId::CSharp).is_none());
        assert!(supports.framework(FrameworkId::Swing).is_none());
    }

    #[test]
    fn a_registered_framework_is_found_without_touching_the_registry() {
        struct Fake;

        impl FrameworkProvider for Fake {
            fn id(&self) -> FrameworkId {
                FrameworkId::Swing
            }

            fn capabilities(&self) -> FrameworkCapabilities {
                FrameworkCapabilities::new("JDialog", &["JTree"], true)
            }
        }

        let mut supports = Supports::initial();

        supports.register_framework(Box::new(Fake));

        assert_eq!(
            supports
                .framework(FrameworkId::Swing)
                .unwrap()
                .capabilities()
                .root(),
            "JDialog"
        );
    }

    fn initial() -> Supports {
        Supports::initial()
    }

    fn registered_languages(supports: &Supports) -> Vec<&dyn LanguageProvider> {
        supports
            .languages()
            .iter()
            .filter_map(|id| supports.language(*id))
            .collect()
    }

    fn registered_frameworks(supports: &Supports) -> Vec<&dyn FrameworkProvider> {
        supports
            .frameworks()
            .iter()
            .filter_map(|id| supports.framework(*id))
            .collect()
    }

    #[test]
    fn every_registered_language_has_extensions() {
        for language in registered_languages(&initial()) {
            assert!(
                !language.extensions().is_empty(),
                "{:?} no declara extensiones",
                language.id()
            );
        }
    }

    #[test]
    fn every_extension_is_lowercase_and_has_no_dot() {
        for language in registered_languages(&initial()) {
            for extension in language.extensions() {
                assert!(
                    !extension.is_empty(),
                    "{:?}: extension vacia",
                    language.id()
                );
                assert_eq!(
                    extension.to_lowercase(),
                    *extension,
                    "{:?}: la extension {extension} no esta en minusculas",
                    language.id()
                );
                assert!(
                    !extension.contains('.'),
                    "{:?}: la extension {extension} lleva punto",
                    language.id()
                );
            }
        }
    }

    #[test]
    fn no_two_languages_claim_the_same_extension() {
        let supports = initial();
        let mut seen: Vec<(&str, LanguageId)> = Vec::new();

        for language in registered_languages(&supports) {
            for extension in language.extensions() {
                let previous = seen
                    .iter()
                    .find(|(claimed, _)| *claimed == *extension)
                    .map(|(_, owner)| *owner);

                assert!(
                    previous.is_none() || previous == Some(language.id()),
                    "la extension {extension} la piden dos lenguajes"
                );

                seen.push((extension, language.id()));
            }
        }
    }

    #[test]
    fn a_file_of_one_language_is_not_recognized_by_the_other() {
        let supports = initial();
        let languages = registered_languages(&supports);

        for language in &languages {
            for extension in language.extensions() {
                let path = format!("src/Main.{extension}");

                for other in &languages {
                    if other.id() == language.id() {
                        continue;
                    }

                    assert!(
                        !other.supports(Path::new(&path)),
                        "{:?} toma un archivo de {:?}",
                        other.id(),
                        language.id()
                    );
                }
            }
        }
    }

    #[test]
    fn a_file_with_an_unknown_extension_is_recognized_by_no_language() {
        for language in registered_languages(&initial()) {
            assert!(!language.supports(Path::new("notas.txt")));
            assert!(!language.supports(Path::new("Makefile")));
            assert!(!language.supports(Path::new("src/")));
        }
    }

    #[test]
    fn every_registered_language_has_a_usable_editing_configuration() {
        for language in registered_languages(&initial()) {
            let editing = language.editing();

            if let Some(line_comment) = editing.line_comment() {
                assert!(!line_comment.is_empty(), "{:?}", language.id());
            }

            if let Some((begin, end)) = editing.block_comment() {
                assert!(!begin.is_empty() && !end.is_empty(), "{:?}", language.id());
                assert_ne!(begin, end, "{:?}", language.id());
            }

            assert!(!editing.indent().is_empty(), "{:?}", language.id());
        }
    }

    #[test]
    fn every_registered_framework_has_a_root_and_components() {
        for framework in registered_frameworks(&initial()) {
            let capabilities = framework.capabilities();

            assert!(!capabilities.root().is_empty(), "{:?}", framework.id());
            assert!(
                !capabilities.components().is_empty(),
                "{:?} no declara controles",
                framework.id()
            );
        }
    }

    #[test]
    fn a_framework_does_not_repeat_a_component() {
        for framework in registered_frameworks(&initial()) {
            let components = framework.capabilities().components();

            for (index, component) in components.iter().enumerate() {
                assert!(!component.is_empty(), "{:?}", framework.id());
                assert!(
                    !components[..index].contains(component),
                    "{:?} repite el control {component}",
                    framework.id()
                );
            }
        }
    }

    #[test]
    fn a_framework_that_generates_code_has_components_to_generate() {
        for framework in registered_frameworks(&initial()) {
            let capabilities = framework.capabilities();

            if capabilities.generates_code() {
                assert!(
                    !capabilities.components().is_empty(),
                    "{:?} dice que genera codigo sin controles que generar",
                    framework.id()
                );
            }
        }
    }

    #[test]
    fn a_csharp_file_is_associated_with_csharp() {
        let supports = initial();

        assert_eq!(
            supports
                .language_for(Path::new("src/Main.cs"))
                .unwrap()
                .id(),
            LanguageId::CSharp
        );
        assert_eq!(
            supports
                .language_for(Path::new("Program.csx"))
                .unwrap()
                .id(),
            LanguageId::CSharp
        );
    }

    #[test]
    fn a_java_file_is_associated_with_java() {
        let supports = initial();

        assert_eq!(
            supports
                .language_for(Path::new("src/Main.java"))
                .unwrap()
                .id(),
            LanguageId::Java
        );
    }

    #[test]
    fn the_associated_file_hands_over_its_editing_configuration() {
        let supports = initial();

        let editing = supports
            .language_for(Path::new("src/Main.cs"))
            .unwrap()
            .editing();

        assert_eq!(editing.line_comment(), Some("//"));
        assert_eq!(editing.block_comment(), Some(("/*", "*/")));
        assert_eq!(editing.indent(), "    ");
    }

    #[test]
    fn the_association_ignores_the_case_of_the_extension() {
        let supports = initial();

        assert_eq!(
            supports.language_for(Path::new("MAIN.CS")).unwrap().id(),
            LanguageId::CSharp
        );
    }

    #[test]
    fn a_file_with_an_unknown_extension_has_no_language() {
        let supports = initial();

        assert!(supports.language_for(Path::new("notas.txt")).is_none());
        assert!(supports.language_for(Path::new("Makefile")).is_none());
        assert!(supports.language_for(Path::new("src/")).is_none());
    }

    #[test]
    fn an_empty_registry_associates_no_file_with_any_language() {
        let supports = Supports::empty();

        assert!(supports.language_for(Path::new("Main.cs")).is_none());
    }

    #[test]
    fn a_file_of_a_registered_language_is_associated_without_naming_that_language() {
        struct Fake;

        impl LanguageProvider for Fake {
            fn id(&self) -> LanguageId {
                LanguageId::CSharp
            }

            fn extensions(&self) -> &'static [&'static str] {
                &["cake"]
            }

            fn editing(&self) -> EditingConfiguration {
                EditingConfiguration::new(Some("#"), None, "\t")
            }
        }

        let mut supports = Supports::initial();
        supports.register_language(Box::new(Fake));

        let language = supports.language_for(Path::new("src/cake.cake")).unwrap();

        assert_eq!(language.extensions(), &["cake"]);
        assert_eq!(language.editing().line_comment(), Some("#"));
    }
}
