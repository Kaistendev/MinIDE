//! Qué se puede ver de un proyecto: el editor y, si el framework la tiene, la vista de
//! diseño.
//!
//! Abrir un proyecto no es solo guardarlo: es decidir qué tiene sentido enseñar. Un
//! proyecto de C# con Windows Forms tiene editor y diseñador, y uno sin framework de
//! ventanas solo tiene editor. Quien lo decide es el framework que declara el proyecto, y
//! no la ventana: un `if framework == "winforms"` en el layout es exactamente el sitio donde
//! un segundo framework obliga a volver a tocar la interfaz (AGENTS.md §2.4).
//!
//! Lo que hay aquí son las dos respuestas que la ventana necesita y que son del framework:
//!
//! * Si hay una vista de diseño. Un framework sin generación de código no la tiene: no hay
//!   nada que colocar en un canvas que llegue a ser un archivo.
//! * Qué controles ofrece el toolbox de esa vista, con el nombre que tienen en ese
//!   framework. `Button` en uno y `JButton` en otro, y la ventana no elige: pregunta.
//!
//! # Dónde está
//!
//! Vive en la aplicación y no en [`crate::frontend::UiState`] porque lo decide el core:
//! `UiState` es de la ventana y no importa nada del core, y esto se calcula con `Supports`,
//! que es el registro de lenguajes y frameworks del core. FE-057.

use crate::core::ProjectType;
use crate::supports::Supports;

/// Las vistas que la ventana puede enseñar con un proyecto abierto.
///
/// Sin framework no hay nada que enseñar más allá del editor, y eso es lo que trae un
/// [`Default`]: una ventana que todavía no tiene proyecto abierto.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Vistas {
    diseniable: bool,
    controles: Vec<String>,
}

impl Vistas {
    /// Las vistas del framework que usa un proyecto de `project_type`.
    ///
    /// Se le pregunta al framework del proyecto y no al tipo de proyecto: el tipo dice qué
    /// combinación hay —C# con Windows Forms— y el framework es quien sabe qué se puede
    /// hacer con ella. Un framework que no esté registrado se trata como uno sin vistas,
    /// que es lo que hay: un framework que MiniIDE no conoce es un framework al que no se
    /// le pueden inventar controles, y un toolbox con nombres inventados es peor que un
    /// toolbox vacío.
    pub fn de(project_type: ProjectType, soportes: &Supports) -> Self {
        let Some(framework) = soportes.framework(project_type.framework()) else {
            return Self::default();
        };

        let capacidades = framework.capabilities();

        Self {
            diseniable: capacidades.generates_code(),
            controles: capacidades
                .components()
                .iter()
                .map(|control| (*control).to_owned())
                .collect(),
        }
    }

    /// Si hay una vista de diseño para este proyecto. FE-057.
    pub fn diseniable(&self) -> bool {
        self.diseniable
    }

    /// Los controles que ofrece el toolbox, con el nombre que tienen en este framework.
    ///
    /// Los del framework y no una lista de la ventana porque el nombre del control es lo
    /// que se escribe en el código generado: si la ventana lo nombrara, el día que
    /// MiniIDE abriera un proyecto de otro framework el canvas pondría un control que ese
    /// framework no tiene (FE-050).
    pub fn controles(&self) -> &[String] {
        &self.controles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un proyecto de C# con Windows Forms habilita las dos vistas. FE-057.
    ///
    /// Es el caso que pide FE-057: al abrir un proyecto de C# con Windows Forms tienen que
    /// poder verse el editor y la vista de diseño, porque es lo que declara su framework. Se
    /// pregunta con los soportes iniciales porque son los que trae MiniIDE: un registro
    /// vacío no es una ventana con menos vistas, es una ventana que no sabe qué proyecto
    /// tiene abierto.
    #[test]
    fn un_proyecto_de_winforms_habilita_el_editor_y_el_diseniador() {
        let vistas = Vistas::de(ProjectType::CSharpWinForms, &Supports::initial());

        assert!(
            vistas.diseniable(),
            "un framework que genera código tiene vista de diseño"
        );
        assert!(
            !vistas.controles().is_empty(),
            "y tiene que decir qué controles ofrece su toolbox"
        );
    }

    /// Un framework sin generación de código no tiene diseñador, pero el editor sigue
    /// estando.
    ///
    /// Va en su propio test porque es lo que hace que la pregunta tenga dos respuestas
    /// honestas: sin generación de código no se quita el editor —que está siempre— se quita
    /// la vista de diseño. Sin esto, "sin vistas" y "sin diseñador" serían la misma cosa, y
    /// un proyecto sin framework no tendría nada que enseñar.
    #[test]
    fn un_framework_sin_generacion_de_codigo_no_tiene_diseniador() {
        use crate::core::FrameworkId;
        use crate::framework::{FrameworkCapabilities, FrameworkProvider};

        struct SinGeneracion;

        impl FrameworkProvider for SinGeneracion {
            fn id(&self) -> FrameworkId {
                FrameworkId::WinForms
            }

            fn capabilities(&self) -> FrameworkCapabilities {
                FrameworkCapabilities::new("Form", &[], false)
            }
        }

        let mut soportes = Supports::initial();
        soportes.register_framework(Box::new(SinGeneracion));

        let vistas = Vistas::de(ProjectType::CSharpWinForms, &soportes);

        assert!(
            !vistas.diseniable(),
            "un framework que no genera código no tiene nada que poner en un canvas"
        );
    }

    /// Los controles del toolbox son los del framework, con su nombre.
    ///
    /// Se comparan las listas enteras porque lo que importa no es que haya controles sino
    /// que sean los de este framework: un toolbox de un framework con un control del otro
    /// pondría en el formulario algo que ese framework no tiene, y el error no se vería
    /// hasta que el proyecto dejara de compilar.
    #[test]
    fn los_controles_del_toolbox_son_los_del_framework() {
        let soportes = Supports::initial();

        let winforms = Vistas::de(ProjectType::CSharpWinForms, &soportes);
        let swing = Vistas::de(ProjectType::JavaSwing, &soportes);

        assert!(winforms.controles().contains(&"Button".to_owned()));
        assert!(!winforms.controles().contains(&"JButton".to_owned()));
        assert!(swing.controles().contains(&"JButton".to_owned()));
        assert!(!swing.controles().contains(&"Button".to_owned()));
    }

    /// Un proyecto de Java con Swing enseña Java y ofrece Swing. FE-061.
    ///
    /// Son las dos mitades de lo que el usuario ve al abrir un proyecto de Java: que sus
    /// archivos se reconozcan como Java —y se resalten con las palabras clave de Java— y que
    /// la vista de diseño ofrezca los controles de Swing. Las dos se preguntan al core por su
    /// identidad, sin mirar el tipo de proyecto en la ventana: por eso aquí no hay ningún
    /// `match` y el framework se cambia solo en `supports.rs`.
    #[test]
    fn un_proyecto_de_java_te_hace_ver_java_y_swing() {
        use std::path::Path;

        use crate::core::LanguageId;

        let soportes = Supports::initial();

        assert_eq!(
            ProjectType::JavaSwing.language(),
            LanguageId::Java,
            "un proyecto de Java con Swing está escrito en Java"
        );

        let lenguaje = soportes
            .language_for(Path::new("src/main/java/MainWindow.java"))
            .expect("un archivo .java es de un lenguaje que MiniIDE tiene");
        assert_eq!(
            lenguaje.id(),
            LanguageId::Java,
            "y el archivo se reconoce por su extensión, sin preguntar por el proyecto"
        );

        let vistas = Vistas::de(ProjectType::JavaSwing, &soportes);
        assert!(vistas.diseniable(), "Swing tiene vista de diseño");
        assert!(
            vistas.controles().contains(&"JButton".to_owned()),
            "con los controles de Swing: {:?}",
            vistas.controles()
        );
    }

    /// Un framework que MiniIDE no tiene registrado no habilita nada.
    ///
    /// Es lo que pasa con un proyecto de una tecnología que todavía no se soporta: la
    /// ventana no puede inventarse los controles de un framework que no conoce, así que lo
    /// trata como uno sin vistas en lugar de enseñarle un toolbox con nombres inventados.
    #[test]
    fn sin_soportes_no_hay_ninguna_vista_que_ensenar() {
        let vistas = Vistas::de(ProjectType::CSharpWinForms, &Supports::empty());

        assert!(
            !vistas.diseniable(),
            "sin framework registrado no se puede enseñar un diseñador"
        );
        assert!(
            vistas.controles().is_empty(),
            "y el toolbox no puede ofrecer controles: {vistas:?}"
        );
    }

    /// Una ventana sin proyecto abierto no tiene diseñador.
    ///
    /// Es el estado de partida de `App::new()` y no un caso raro: abrir MiniIDE sin abrir
    /// nada es lo primero que ve el usuario, y si saliera con un diseñador a medio hacer
    /// estaría enseñando un formulario de un proyecto que no ha visto.
    #[test]
    fn una_ventana_sin_proyecto_no_tiene_diseniador() {
        let vistas = Vistas::default();

        assert!(!vistas.diseniable());
        assert!(vistas.controles().is_empty());
    }

    /// Este módulo no nombra ningún lenguaje ni ningún framework.
    ///
    /// Es la regla de AGENTS.md §2.4 hecha comprobable: las vistas se calculan preguntando
    /// al framework del proyecto, y si el código de aquí nombrara una tecnología, el día que
    /// MiniIDE abriera un proyecto de otra esta ventana tendría que tocarse.
    ///
    /// Se leen las líneas de código y no el archivo entero porque los comentarios sí pueden
    /// nombrarlas —este necesita hacerlo para decir qué no se hace— y porque el test vive
    /// dentro del módulo y se encontraría a sí mismo con los nombres que prohibe.
    #[test]
    fn las_vistas_no_nombran_ningun_lenguaje_ni_framework() {
        let fuente =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file!()))
                .expect("el fuente de las vistas tiene que poder leerse");

        let codigo: Vec<&str> = fuente
            .split("#[cfg(test)]")
            .next()
            .expect("las vistas tienen que tener código antes de los tests")
            .lines()
            .map(str::trim)
            .filter(|linea| !linea.starts_with("//"))
            .collect();

        for prohibido in [
            "csharp", "CSharp", "java", "Java", "winforms", "WinForms", "swing", "Swing",
        ] {
            assert!(
                !codigo.iter().any(|linea| linea.contains(prohibido)),
                "las vistas no pueden nombrar {prohibido} en su código: se le pregunta al \
                 framework del proyecto: {codigo:?}"
            );
        }
    }
}
