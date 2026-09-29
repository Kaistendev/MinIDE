//! El frontend de MiniIDE: la ventana, sus widgets y el estado visual.
//!
//! Aquí vive la interfaz y solo la interfaz. La interfaz consume el core y le emite
//! comandos, nunca al revés: el core no sabe que existe un frontend, y por eso puede
//! probarse y usarse sin ventana ninguna.
//!
//! El frontend no es la fuente de verdad de nada que ya tenga dueño:
//!
//! * El texto, el cursor, la selección y el historial son del core.
//! * El proyecto, sus archivos y su configuración son del core.
//! * La compilación, la ejecución y los diagnósticos son del core.
//! * El modelo del diseñador y el código que genera son del core.
//!
//! Lo que sí es de aquí es lo que no le importa al core: qué panel está abierto, qué
//! pestaña se ve, qué elemento está resaltado y cómo se ve la ventana.
//!
//! # Stack
//!
//! egui para los widgets y el dibujado, eframe para la ventana y el ciclo de
//! ejecución (`docs/frontend-plan.md` FD-01). No hay otro stack de UI en el MVP, y el
//! frontend no depende de C#, Java ni de ninguna otra tecnología del proyecto.
//!
//! # Qué hay y qué no
//!
//! Esto es el sitio y la puerta de entrada de la interfaz; lo que va dentro se añade
//! tarea a tarea, y cada una es una cosa:
//!
//! * La ventana mínima y el ciclo de ejecución: FE-003.
//! * El estado visual mínimo: FE-004.
//! * El layout, el menú, la barra de herramientas y la barra de estado: FE-005 a FE-008.
//! * Los paneles: FE-010 en adelante.
//!
//! Nada de esto se implementa antes de tiempo. Un widget que se añade sin su tarea se
//! queda sin criterio para decidir si va al core o se queda aquí.

mod acciones;
mod app;
mod atajos;
mod busqueda;
mod diagnosticos;
mod editor;
mod explorador;
mod icon;
mod layout;
mod menu;
mod salida;
mod status;
mod tabs;
mod toolbar;
mod ui_state;

pub use app::{opciones, run, titulo, ventana, App};
pub use icon::{icono, LADO, LOGO};
pub use ui_state::UiState;

#[cfg(test)]
mod tests {
    /// El frontend tiene que estar declarado en la raiz del crate para que se pueda
    /// abrir desde fuera. Es lo unico que este sitio necesita del crate: el resto de
    /// la estructura se comprueba desde fuera, con el modulo ya declarado.
    #[test]
    fn the_frontend_module_is_declared_in_the_crate_root() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
        let content = std::fs::read_to_string(&root).expect("readable crate root");

        assert!(
            content.contains("pub mod frontend;"),
            "src/lib.rs tiene que declarar `pub mod frontend;`"
        );
        assert!(
            !content.contains("pub mod ui;"),
            "`pub mod ui;` se ha sustituido por el modulo del frontend: el estado de la interfaz es suyo"
        );
    }
}
