//! El estado visual de la interfaz.
//!
//! Aqui vive lo que se ve y lo que se ha visto, y nada mas: qué panel está abierto,
//! qué pestaña se está viendo, qué elemento está resaltado, qué dice la barra de
//! estado. Es estado de la ventana, y desaparece con ella.
//!
//! Lo que no puede vivir aqui es el estado del dominio. El texto de un documento, el
//! cursor, la selección, el historial, el proyecto y su configuración son del core y
//! se piden, no se guardan: si la interfaz guardara una copia, la copia se pondría al
//! día hasta que dejara de estarlo, y el IDE empezaría a guardar y mostrar cosas que
//! el core ya no sabe.
//!
//! Por eso este archivo no importa nada del core. No es una costumbre: es lo que hace
//! que la separación sea de verdad, y hay un test que lo comprueba.
/// El estado visual de la ventana.
///
/// Solo lo que se ve: qué dice la barra de estado, qué panel está abierto, qué
/// elemento está resaltado. Ahora mismo solo la barra de estado; el resto de las
/// zonas llega con la tarea de cada una.
///
/// Deliberadamente no guarda el titulo de la ventana. El titulo lo decide la
/// aplicación (`titulo`) y, cuando haya documentos abiertos, se compondrá con el
/// documento activo, que es del core. Guardarlo aqui seria tener la misma verdad en
/// dos sitios: la del titulo, que es fija, y la del documento, que no.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UiState {
    status: Option<String>,
}

impl UiState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Lo que dice la barra de estado, si dice algo.
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// Pone un mensaje en la barra de estado, sustituyendo el que hubiera.
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status = Some(message.into());
    }

    /// Quita el mensaje de la barra de estado.
    pub fn clear_status(&mut self) {
        self.status = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_visual_state_has_nothing_to_show_yet() {
        let state = UiState::new();

        assert_eq!(state.status(), None);
    }

    #[test]
    fn status_can_be_set_and_cleared() {
        let mut state = UiState::default();

        state.set_status("Compilando...");
        assert_eq!(state.status(), Some("Compilando..."));

        state.clear_status();
        assert_eq!(state.status(), None);
    }

    #[test]
    fn setting_a_status_replaces_the_previous_one() {
        let mut state = UiState::new();

        state.set_status("Compilando...");
        state.set_status("1 error");

        assert_eq!(state.status(), Some("1 error"));
    }

    /// Modulos del core. El estado visual no puede importar ninguno.
    const MODULOS_DEL_CORE: &[&str] = &[
        "build",
        "commands",
        "core",
        "diagnostics",
        "document",
        "editor",
        "framework",
        "generation",
        "language",
        "project",
        "runtime",
        "supports",
        "templates",
        "toolchain",
        "workspace",
    ];

    /// El estado visual no depende del core.
    ///
    /// Es la forma de comprobar que la interfaz no guarda el dominio: si este archivo
    /// importara el documento, el proyecto o el editor, el estado visual empezaria a
    /// depender de tipos que son del core, y con ellos a entrar en ellos. Ahi esta el
    /// peligro de la copia: un documento en la interfaz obliga a decidir quien tiene
    /// la verdad, y siempre se acaban teniendo las dos.
    ///
    /// Se comprueban los `use` y no el texto entero a proposito: la regla es que el
    /// estado visual no *importa* el core, y porque el test se escribe aqui dentro, un
    /// `contains` sobre todo el archivo se encontraria a si mismo con el nombre de
    /// cada modulo.
    #[test]
    fn the_visual_state_does_not_depend_on_the_core() {
        let source =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file!()))
                .expect("readable source of the visual state");

        let uses: Vec<&str> = source
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("use crate::"))
            .filter(|line| {
                MODULOS_DEL_CORE
                    .iter()
                    .any(|module| line.contains(&format!("::{module}")))
            })
            .collect();

        assert!(
            uses.is_empty(),
            "el estado visual es de la interfaz y no puede importar el core: {uses:?}"
        );
    }
}
