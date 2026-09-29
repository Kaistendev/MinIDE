//! Las acciones de la interfaz: lo que se ve y lo que se pide.
//!
//! Una acción es un nombre y el comando que ese nombre pide, y es la misma acción la
//! pulses donde la pulses: el "Guardar" del menú, el de la barra de herramientas y el del
//! atajo de mañana son esta misma `Accion`, escrita una vez. Si cada sitio escribiera su
//! propio par de nombre y comando, el mismo botón podría pedir dos cosas según por dónde
//! se pulsara, y arreglarlo obligaría a buscar el nombre en cuatro sitios a la vez.
//!
//! Las acciones sin comando también están aquí, y no en el menú o en la barra, por lo
//! mismo: "Copiar" es un elemento del menú hoy y un botón de la barra mañana, y lo que hoy
//! no se puede pulsar es que el core no tiene todavía ese comando, no que el menú se le
//! haya olvidado. Qué comandos hay está en `crate::commands`, y añadir uno es trabajo del
//! core (T-096), no de aquí.
//!
//! Y de aquí sale el clic: [`boton`] es el único sitio donde un clic se convierte en una
//! petición, y la pide por `App::emitir`, que es el único sitio por el que un comando sale
//! de la ventana. El menú, la barra y los atajos pasan por los dos o no llegan a nada.

use crate::commands::Command;
use eframe::egui;

use super::app::App;

/// Una acción de la interfaz: lo que se ve y lo que se pide al pulsarla.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Accion {
    /// Lo que se ve, en el botón o en el elemento del menú.
    pub nombre: &'static str,
    /// Lo que se pide, si hay algo que pedir.
    ///
    /// Sin comando la acción no se puede pulsar y se dibuja apagada: existe y está
    /// situada, pero el core todavía no tiene el comando que la haría funcionar.
    pub comando: Option<Command>,
}

/// Las acciones de MiniIDE, cada una escrita una vez.
///
/// Los nombres son los que se ven en la ventana, y FE-076 es la que los unifica con los de
/// la barra de herramientas y con los de los mensajes; mientras tanto, cada nombre está en
/// un sitio solo, y por eso cambiar uno obliga a cambiarlo en todas partes.
pub const NUEVO_PROYECTO: Accion = Accion {
    nombre: "Nuevo proyecto",
    comando: Some(Command::NewProject),
};

pub const ABRIR_PROYECTO: Accion = Accion {
    nombre: "Abrir proyecto",
    comando: Some(Command::OpenProject),
};

pub const GUARDAR: Accion = Accion {
    nombre: "Guardar",
    comando: Some(Command::Save),
};

pub const DESHACER: Accion = Accion {
    nombre: "Deshacer",
    comando: Some(Command::Undo),
};

pub const REHACER: Accion = Accion {
    nombre: "Rehacer",
    comando: Some(Command::Redo),
};

/// Un archivo nuevo, y en qué carpeta, lo decide el core cuando ejecute el comando.
///
/// La acción no dice dónde va porque aquí no se sabe: lo único que se sabe es que se ha
/// pedido desde la fila que se ha pulsado, y de eso se encarga quien abra el diálogo.
pub const NUEVO_ARCHIVO: Accion = Accion {
    nombre: "Nuevo archivo",
    comando: Some(Command::NewFile),
};

/// Véase [`NUEVO_ARCHIVO`].
pub const NUEVO_DIRECTORIO: Accion = Accion {
    nombre: "Nuevo directorio",
    comando: Some(Command::NewDirectory),
};

pub const BUSCAR: Accion = Accion {
    nombre: "Buscar",
    comando: Some(Command::Find),
};

/// El portapapeles no es un comando del core, ni hay tarea que lo añada.
pub const COPIAR: Accion = Accion {
    nombre: "Copiar",
    comando: None,
};

/// Véase [`COPIAR`].
pub const CORTAR: Accion = Accion {
    nombre: "Cortar",
    comando: None,
};

/// Véase [`COPIAR`].
pub const PEGAR: Accion = Accion {
    nombre: "Pegar",
    comando: None,
};

/// Reemplazar lo que se busca por otra cosa. FE-033.
///
/// Ya tiene comando: es el que pidio T-096, y el botón de la barra de búsqueda y el
/// atajo piden este mismo nombre, que es lo que hace que un botón y un atajo sean la
/// misma operación.
pub const REEMPLAZAR: Accion = Accion {
    nombre: "Reemplazar",
    comando: Some(Command::Replace),
};

pub const COMPILAR: Accion = Accion {
    nombre: "Compilar",
    comando: Some(Command::Build),
};

pub const EJECUTAR: Accion = Accion {
    nombre: "Ejecutar",
    comando: Some(Command::Run),
};

pub const DETENER: Accion = Accion {
    nombre: "Detener",
    comando: Some(Command::Stop),
};

/// Mostrar un panel no es un comando: es estado de la ventana, y eso lo lleva FE-010.
pub const PANEL_DE_PROYECTO: Accion = Accion {
    nombre: "Panel de proyecto",
    comando: None,
};

/// Véase [`PANEL_DE_PROYECTO`].
pub const PANEL_DE_PROPIEDADES: Accion = Accion {
    nombre: "Panel de propiedades",
    comando: None,
};

/// Dibuja una acción y pide su comando si la pulsan.
///
/// Un solo sitio para esto, y no uno por zona: si el menú y la barra tuvieran el suyo, el
/// mismo nombre se dibujaría dos veces y una de las dos se quedaría sin pedir nada. Aquí
/// un clic se convierte en un comando y se entrega a `App::emitir`, que es el único
/// camino que tiene un comando para salir de la ventana.
///
/// La acción sin comando se dibuja apagada. No es que esté pendiente de este módulo: es que
/// no hay comando que pedir, y un botón que acepta el clic y no hace nada parece un fallo
/// del IDE.
pub fn boton(ui: &mut egui::Ui, app: &mut App, accion: Accion) {
    let Some(comando) = accion.comando else {
        ui.add_enabled(false, egui::Button::new(accion.nombre));
        return;
    };

    if ui.button(accion.nombre).clicked() {
        app.emitir(comando);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::menu::MENUS;
    use crate::frontend::toolbar::BOTONES;
    use std::collections::{HashMap, HashSet};

    /// Los comandos que pide cada nombre en la superficie que le pasas.
    fn comandos_de(
        acciones: impl Iterator<Item = &'static Accion>,
    ) -> HashMap<&'static str, HashSet<Command>> {
        let mut comandos: HashMap<&'static str, HashSet<Command>> = HashMap::new();

        for accion in acciones {
            if let Some(comando) = accion.comando {
                comandos.entry(accion.nombre).or_default().insert(comando);
            }
        }

        comandos
    }

    /// Lo que pide cada nombre en el menú, y solo en el menú.
    fn del_menu() -> HashMap<&'static str, HashSet<Command>> {
        comandos_de(MENUS.iter().flat_map(|menu| menu.acciones.iter()))
    }

    /// Un nombre no pide nunca dos cosas distintas.
    ///
    /// Se mira el menú y la barra a la vez porque el nombre es lo que el usuario ve: si
    /// "Guardar" significa una cosa en el menú y otra en la barra, el usuario no tiene
    /// forma de saber cuál de las dos es la buena, y el sitio donde se cruzaron los dos
    /// nombres es un botón que hace algo que no dice. Cuando el core añada un comando y
    /// alguien escriba un nombre en un sitio nuevo, este test es el que dice si ese nombre
    /// significa lo mismo que antes.
    #[test]
    fn un_nombre_no_pide_dos_cosas_distintas() {
        let mut todo = del_menu();
        for (nombre, comandos) in comandos_de(BOTONES.iter()) {
            todo.entry(nombre).or_default().extend(comandos);
        }

        for (nombre, comandos) in todo {
            assert_eq!(
                comandos.len(),
                1,
                "el nombre {nombre:?} pide más de un comando: {comandos:?}"
            );
        }
    }

    /// Lo que está en la barra de herramientas está también en el menú.
    ///
    /// FE-009 pide que un botón y un menú puedan pedir lo mismo, y no como dos acciones
    /// parecidas: como la misma. Los cinco botones de la barra son operaciones de las que
    /// T-039 dice que tienen que ser accesibles desde la UI, y si estuvieran solo en la
    /// barra serían media operación partida en dos.
    ///
    /// Se mira solo el menú y no las dos superficies, porque si se miraran las dos el test
    /// no comprobaría nada: la barra estaría en el mapa por sí sola y siempre encontraría
    /// lo que busca.
    #[test]
    fn lo_que_esta_en_la_barra_tambien_esta_en_el_menu() {
        let del_menu = del_menu();

        for accion in BOTONES {
            let comando = accion
                .comando
                .expect("un boton de la barra tiene que pedir algo");

            assert!(
                del_menu.contains_key(accion.nombre),
                "el boton {:?} de la barra no esta en ningun menu: {del_menu:?}",
                accion.nombre
            );
            assert!(
                del_menu[accion.nombre].contains(&comando),
                "el boton {:?} pide {:?} pero en el menu ese nombre pide otra cosa: {del_menu:?}",
                accion.nombre,
                comando
            );
        }
    }
}
