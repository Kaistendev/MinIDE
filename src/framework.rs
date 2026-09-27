use crate::core::FrameworkId;

/// Lo que WinForms puede hacer en el disenador.
mod winforms;

/// Lo que Swing puede hacer en el disenador.
mod swing;

pub use swing::SwingModel;
pub use winforms::WinFormsModel;

/// Lo que el core puede consultar de un framework sin saber cual es.
///
/// La implementan los toolkits concretos, asi que anadir un framework es anadir
/// un tipo que cumple este contrato y nada mas. El core nunca pregunta por un
/// framework en concreto.
pub trait FrameworkProvider {
    /// Identidad abstracta del toolkit.
    fn id(&self) -> FrameworkId;

    /// Capacidades visuales y de generacion del framework.
    fn capabilities(&self) -> FrameworkCapabilities;
}

/// Que puede hacer el disenador visual de un framework.
///
/// `root` es el tipo de la ventana o el formulario, y `components` los tipos de
/// control que el disenador puede poner dentro, con el nombre que tienen en ese
/// framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameworkCapabilities {
    root: &'static str,
    components: &'static [&'static str],
    generates_code: bool,
}

impl FrameworkCapabilities {
    /// Un framework sin componentes tiene una lista vacia, y uno que no genera
    /// codigo lo dice en `generates_code`.
    pub fn new(
        root: &'static str,
        components: &'static [&'static str],
        generates_code: bool,
    ) -> Self {
        Self {
            root,
            components,
            generates_code,
        }
    }

    /// Tipo de la ventana o el formulario raiz del disenador.
    pub fn root(&self) -> &'static str {
        self.root
    }

    /// Tipos de control disponibles, tal y como se llaman en este framework.
    pub fn components(&self) -> &'static [&'static str] {
        self.components
    }

    /// Si el framework tiene generacion de codigo.
    ///
    /// Aqui solo se declara que la tiene. El contrato que genera el codigo es
    /// el del generador, y el framework da su descriptor, no el generador.
    pub fn generates_code(&self) -> bool {
        self.generates_code
    }
}

#[cfg(test)]
mod tests {
    use crate::core::FrameworkId;

    use crate::framework::{FrameworkCapabilities, FrameworkProvider};

    struct FakeFramework {
        id: FrameworkId,
        capabilities: FrameworkCapabilities,
    }

    impl FrameworkProvider for FakeFramework {
        fn id(&self) -> FrameworkId {
            self.id
        }

        fn capabilities(&self) -> FrameworkCapabilities {
            self.capabilities
        }
    }

    fn winforms() -> FakeFramework {
        FakeFramework {
            id: FrameworkId::WinForms,
            capabilities: FrameworkCapabilities::new(
                "Form",
                &["Button", "Label", "TextBox", "Panel"],
                true,
            ),
        }
    }

    fn swing() -> FakeFramework {
        FakeFramework {
            id: FrameworkId::Swing,
            capabilities: FrameworkCapabilities::new(
                "JFrame",
                &["JButton", "JLabel", "JTextField", "JPanel"],
                true,
            ),
        }
    }

    #[test]
    fn the_framework_identity_comes_from_the_provider() {
        let winforms = winforms();

        assert_eq!(winforms.id(), FrameworkId::WinForms);
    }

    #[test]
    fn the_root_of_the_designer_comes_from_the_provider() {
        let winforms = winforms();
        let swing = swing();

        assert_eq!(winforms.capabilities().root(), "Form");
        assert_eq!(swing.capabilities().root(), "JFrame");
    }

    #[test]
    fn the_available_components_come_from_the_provider() {
        let winforms = winforms();

        assert_eq!(
            winforms.capabilities().components(),
            &["Button", "Label", "TextBox", "Panel"]
        );
    }

    #[test]
    fn the_generation_capability_comes_from_the_provider() {
        assert!(winforms().capabilities().generates_code());
    }

    #[test]
    fn several_frameworks_can_be_queried_through_the_same_interface() {
        let frameworks: [&dyn FrameworkProvider; 2] = [&winforms(), &swing()];

        let identities: Vec<FrameworkId> = frameworks.iter().map(|f| f.id()).collect();
        let roots: Vec<&str> = frameworks.iter().map(|f| f.capabilities().root()).collect();

        assert_eq!(identities, vec![FrameworkId::WinForms, FrameworkId::Swing]);
        assert_eq!(roots, vec!["Form", "JFrame"]);
    }

    #[test]
    fn a_framework_without_components_or_generation_declares_none_of_them() {
        let limited = FakeFramework {
            id: FrameworkId::Swing,
            capabilities: FrameworkCapabilities::new("JFrame", &[], false),
        };

        assert_eq!(limited.capabilities().components(), &[] as &[&str]);
        assert!(!limited.capabilities().generates_code());
    }
}
