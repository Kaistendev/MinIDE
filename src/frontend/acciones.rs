//! Las acciones de la interfaz: lo que se ve y lo que se pide.
//!
//! Una acción es un nombre y el comando que ese nombre pide, y es la misma acción la
//! pulses donde la pulses: el "Guardar" del menú, el de la barra de herramientas, el del
//! diálogo de cerrar y el atajo son esta misma `Accion`, escrita una vez. Si cada sitio
//! escribiera su propio par de nombre y comando, el mismo botón podría pedir dos cosas
//! según por dónde se pulsara, y arreglarlo obligaría a buscar el nombre en cuatro sitios
//! a la vez.
//!
//! Y toda acción que existe pide algo. No hay acciones a medio hacer: una acción sin
//! comando se dibujaba apagada para decir la verdad —que el core todavía no tiene ese
//! comando—, pero un botón apagado en el menú sigue siendo un botón que el usuario ve y
//! que no puede usar, y FE-077 quita esos. Si el core gana el comando, la acción se
//! escribe aquí con su nombre y aparece; mientras tanto no está, y lo que no está no
//! ocupa sitio en la ventana. Qué comandos hay está en `crate::commands`, y añadir uno es
//! trabajo del core (T-096), no de aquí.
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
    /// Lo que se pide al pulsarla.
    pub comando: Command,
}

/// Las acciones de MiniIDE, cada una escrita una vez.
///
/// Los nombres son los que se ven en la ventana, y son los mismos en el menú, en la barra
/// de herramientas, en los diálogos y en los mensajes: FE-076 lo comprueba y este módulo
/// es el único donde se puede cambiar uno.
pub const NUEVO_PROYECTO: Accion = Accion {
    nombre: "Nuevo proyecto",
    comando: Command::NewProject,
};

pub const ABRIR_PROYECTO: Accion = Accion {
    nombre: "Abrir proyecto",
    comando: Command::OpenProject,
};

pub const GUARDAR: Accion = Accion {
    nombre: "Guardar",
    comando: Command::Save,
};

pub const DESHACER: Accion = Accion {
    nombre: "Deshacer",
    comando: Command::Undo,
};

pub const REHACER: Accion = Accion {
    nombre: "Rehacer",
    comando: Command::Redo,
};

/// Un archivo nuevo, y en qué carpeta, lo decide el core cuando ejecute el comando.
///
/// La acción no dice dónde va porque aquí no se sabe: lo único que se sabe es que se ha
/// pedido desde la fila que se ha pulsado, y de eso se encarga quien abra el diálogo.
pub const NUEVO_ARCHIVO: Accion = Accion {
    nombre: "Nuevo archivo",
    comando: Command::NewFile,
};

/// Véase [`NUEVO_ARCHIVO`].
pub const NUEVO_DIRECTORIO: Accion = Accion {
    nombre: "Nuevo directorio",
    comando: Command::NewDirectory,
};

pub const BUSCAR: Accion = Accion {
    nombre: "Buscar",
    comando: Command::Find,
};

/// Reemplazar lo que se busca por otra cosa. FE-033.
///
/// Ya tiene comando: es el que pidio T-096, y el botón de la barra de búsqueda y el
/// atajo piden este mismo nombre, que es lo que hace que un botón y un atajo sean la
/// misma operación.
pub const REEMPLAZAR: Accion = Accion {
    nombre: "Reemplazar",
    comando: Command::Replace,
};

pub const COMPILAR: Accion = Accion {
    nombre: "Compilar",
    comando: Command::Build,
};

pub const EJECUTAR: Accion = Accion {
    nombre: "Ejecutar",
    comando: Command::Run,
};

pub const DETENER: Accion = Accion {
    nombre: "Detener",
    comando: Command::Stop,
};

/// Dibuja una acción y pide su comando si la pulsan.
///
/// Un solo sitio para esto, y no uno por zona: si el menú y la barra tuvieran el suyo, el
/// mismo nombre se dibujaría dos veces y una de las dos se quedaría sin pedir nada. Aquí
/// un clic se convierte en un comando y se entrega a `App::emitir`, que es el único
/// camino que tiene un comando para salir de la ventana.
///
/// El botón se dibuja siempre encendido. No hay una rama que lo apague porque no hay
/// acciones apagadas: si un botón no puede hacer nada, no se dibuja (FE-077), y un botón
/// que acepta el clic y no hace nada parece un fallo del IDE.
pub fn boton(ui: &mut egui::Ui, app: &mut App, accion: Accion) {
    if ui.button(accion.nombre).clicked() {
        app.emitir(accion.comando);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::menu::MENUS;
    use crate::frontend::toolbar::BOTONES;
    use std::collections::{HashMap, HashSet};

    /// Un frame con una ventana de tamaño conocido.
    fn entrada() -> eframe::egui::RawInput {
        entrada_con(&[])
    }

    /// Un frame con una ventana de tamaño conocido y estos eventos de por medio.
    fn entrada_con(eventos: &[eframe::egui::Event]) -> eframe::egui::RawInput {
        eframe::egui::RawInput {
            screen_rect: Some(eframe::egui::Rect::from_min_size(
                eframe::egui::Pos2::ZERO,
                eframe::egui::vec2(300.0, 60.0),
            )),
            events: eventos.to_vec(),
            ..eframe::egui::RawInput::default()
        }
    }

    /// Los comandos que pide cada nombre en la superficie que le pasas.
    fn comandos_de(
        acciones: impl Iterator<Item = &'static Accion>,
    ) -> HashMap<&'static str, HashSet<Command>> {
        let mut comandos: HashMap<&'static str, HashSet<Command>> = HashMap::new();

        for accion in acciones {
            comandos
                .entry(accion.nombre)
                .or_default()
                .insert(accion.comando);
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
            assert!(
                del_menu.contains_key(accion.nombre),
                "el boton {:?} de la barra no esta en ningun menu: {del_menu:?}",
                accion.nombre
            );
            assert!(
                del_menu[accion.nombre].contains(&accion.comando),
                "el boton {:?} pide {:?} pero en el menu ese nombre pide otra cosa: {del_menu:?}",
                accion.nombre,
                accion.comando
            );
        }
    }

    /// Cada acción que el usuario ve hace algo cuando la pulsa. FE-069 y FE-077.
    ///
    /// Los tests de arriba miran las tablas —qué acción hay en el menú y cuál en la barra— y no
    /// el gesto: que "Guardar" tenga `Save` escrito en la tabla no dice que al pulsarlo salga un
    /// `Save`. Aquí se dibuja cada acción y se pulsa con un clic de verdad, y se recorre el menú
    /// y la barra enteros, así que una acción nueva no puede entrar en la ventana sin comprobarse.
    ///
    /// Y tiene que hacer algo siempre. Un elemento que se pinta y al que se le puede pulsar sin
    /// que pase nada es peor que uno que no está: parece que MiniIDE ha hecho caso y no lo ha
    /// hecho. FE-077 quita los que lo eran, y este test es lo que avisa si vuelve a entrar uno.
    #[test]
    fn cada_accion_visible_pide_su_comando_al_pulsarla() {
        let acciones: Vec<&'static Accion> = MENUS
            .iter()
            .flat_map(|menu| menu.acciones.iter())
            .chain(BOTONES.iter())
            .collect();

        assert!(
            !acciones.is_empty(),
            "si no hay acciones este test no comprueba nada"
        );

        for accion in acciones {
            let mut app = App::new();
            se_pinta(accion, &mut app);

            assert_eq!(
                app.peticiones(),
                vec![accion.comando],
                "pulsar {:?} tiene que pedir su comando",
                accion.nombre
            );
        }
    }

    /// Dibuja `accion`, la pulsa con un clic de verdad y dice si se ha pintado.
    ///
    /// El clic va en dos frames porque egui solo cuenta uno si la pulsación y la soltura caen
    /// en el mismo widget, y el rectángulo del botón se busca en lo que se ha pintado porque es
    /// el sitio donde el usuario haría clic.
    fn se_pinta(accion: &'static Accion, app: &mut App) -> bool {
        let contexto = eframe::egui::Context::default();
        let mut salida = contexto.run_ui(entrada(), |ui| {
            ui.allocate_ui(eframe::egui::vec2(300.0, 60.0), |ui| {
                super::boton(ui, app, *accion);
            });
        });
        salida.textures_delta.clear();

        let boton = salida
            .shapes
            .iter()
            .filter_map(|forma| match &forma.shape {
                eframe::egui::Shape::Rect(rectangulo) => Some(rectangulo.rect),
                _ => None,
            })
            .next()
            .unwrap_or_else(|| panic!("{:?} se tiene que pintar", accion.nombre));

        for pressed in [true, false] {
            let _ = contexto.run_ui(
                entrada_con(&[eframe::egui::Event::PointerButton {
                    pos: boton.center(),
                    button: eframe::egui::PointerButton::Primary,
                    pressed,
                    modifiers: eframe::egui::Modifiers::default(),
                }]),
                |ui| {
                    ui.allocate_ui(eframe::egui::vec2(300.0, 60.0), |ui| {
                        super::boton(ui, app, *accion);
                    });
                },
            );
        }

        boton.width() > 0.0 && boton.height() > 0.0
    }
}
