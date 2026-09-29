//! Los atajos de la ventana.
//!
//! Un atajo es de la ventana y no de una zona: guardar tiene que funcionar estés donde
//! estés. Por eso están todos en una tabla y en un módulo, y no repartidos por los widgets
//! que los usan: si el atajo de guardar estuviera en el editor y el de buscar en otro sitio,
//! no habría forma de responder a "qué hace Ctrl+?".
//!
//! Cada atajo dice qué teclas lo pulsan y qué pide. Lo que pide es la acción del
//! vocabulario -el mismo nombre y el mismo comando que su botón del menú- o un diálogo de
//! la ventana, que no es un comando porque no es cosa del core.

use eframe::egui;

use super::acciones::{Accion, DESHACER, GUARDAR, REHACER};
use super::app::App;

/// Qué teclas hay que pulsar a la vez con el atajo.
///
/// Se guardan aquí y no con los modificadores de egui porque son una fila de una tabla
/// constante: una tabla que depende de cómo egui nombre sus modificadores cambiaría con
/// él, y una tabla de atajos es de la ventana y no del framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Teclas {
    pub ctrl: bool,
    pub shift: bool,
}

const CTRL: Teclas = Teclas {
    ctrl: true,
    shift: false,
};

/// Lo que hace un atajo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Efecto {
    /// Pedir lo que pide la acción, que es lo mismo que pediría su botón del menú.
    Pedir(Accion),
    /// Abrir un diálogo de la ventana, que no es un comando porque no es cosa del core.
    Abrir(Dialogo),
}

/// Los diálogos que un atajo puede abrir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialogo {
    /// Buscar en el documento.
    Busqueda,
    /// Buscar y reemplazar en el documento.
    BusquedaYReemplazo,
}

impl Dialogo {
    /// Si el diálogo trae el campo de reemplazo, que es lo que decide si se ve.
    pub fn tiene_reemplazo(self) -> bool {
        self == Dialogo::BusquedaYReemplazo
    }
}

/// Un atajo: qué teclas lo pulsan y qué pide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Atajo {
    pub tecla: egui::Key,
    pub teclas: Teclas,
    pub efecto: Efecto,
}

/// Los atajos de la ventana.
///
/// Guardar, deshacer, rehacer, buscar y reemplazar. Los que piden una acción piden la
/// misma del vocabulario que sus botones, para que un atajo y un botón no puedan pedir
/// cosas distintas: si el atajo de guardar escribiera su propio comando, el botón y el
/// atajo dejarían de ser la misma operación el día que uno de los dos cambiara.
pub const ATAJOS: &[Atajo] = &[
    Atajo {
        tecla: egui::Key::S,
        teclas: CTRL,
        efecto: Efecto::Pedir(GUARDAR),
    },
    Atajo {
        tecla: egui::Key::Z,
        teclas: CTRL,
        efecto: Efecto::Pedir(DESHACER),
    },
    Atajo {
        tecla: egui::Key::Y,
        teclas: CTRL,
        efecto: Efecto::Pedir(REHACER),
    },
    Atajo {
        tecla: egui::Key::F,
        teclas: CTRL,
        efecto: Efecto::Abrir(Dialogo::Busqueda),
    },
    Atajo {
        tecla: egui::Key::H,
        teclas: CTRL,
        efecto: Efecto::Abrir(Dialogo::BusquedaYReemplazo),
    },
];

/// El atajo de `tecla` con `teclas` pulsadas, si lo hay.
pub fn atajo_de(tecla: egui::Key, teclas: Teclas) -> Option<&'static Atajo> {
    ATAJOS
        .iter()
        .find(|atajo| atajo.tecla == tecla && atajo.teclas == teclas)
}

/// Mira lo que se ha pulsado en este frame y pide lo que corresponda.
///
/// Se mira lo que ha pasado y no lo que queda pulsado, porque un atajo se pide al
/// apretarlo y no mientras se mantenga apretado: si no, con Ctrl+S mantenido se pediría
/// guardar sin parar. La soltura se ignora a propósito, y por eso una pulsación pide una
/// vez y solo una.
pub fn manejar(ui: &egui::Ui, app: &mut App) {
    let pulsados: Vec<Efecto> = ui.input(|entrada| {
        entrada
            .events
            .iter()
            .filter_map(|evento| match evento {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => atajo_de(
                    *key,
                    Teclas {
                        ctrl: modifiers.ctrl,
                        shift: modifiers.shift,
                    },
                )
                .map(|atajo| atajo.efecto),
                _ => None,
            })
            .collect()
    });

    for efecto in pulsados {
        aplicar(ui, efecto, app);
    }
}

/// Hace lo que dice el efecto de un atajo.
pub fn aplicar(ui: &egui::Ui, efecto: Efecto, app: &mut App) {
    let _ = ui;

    match efecto {
        Efecto::Pedir(accion) => {
            if let Some(comando) = accion.comando {
                app.emitir(comando);
            }
        }
        Efecto::Abrir(dialogo) => app.state_mut().abrir_busqueda(dialogo.tiene_reemplazo()),
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::Command;
    use crate::frontend::acciones;
    use crate::frontend::App;
    use eframe::egui;

    use super::{atajo_de, Teclas, ATAJOS};

    /// Una ventana de trabajo normal, ni enorme ni mínima.
    const ANCHO: f32 = 1000.0;
    const ALTO: f32 = 800.0;

    const CTRL: Teclas = Teclas {
        ctrl: true,
        shift: false,
    };

    fn entrada_con(eventos: &[egui::Event]) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(ANCHO, ALTO),
            )),
            events: eventos.to_vec(),
            ..egui::RawInput::default()
        }
    }

    /// Una pulsación y su soltura, con su modificador.
    fn pulsar(tecla: egui::Key, ctrl: bool) -> Vec<egui::Event> {
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key: tecla,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl,
                    ..egui::Modifiers::default()
                },
            })
            .collect()
    }

    /// Un frame de la ventana con unos eventos de por medio.
    fn ventana(app: &mut App, eventos: &[egui::Event]) {
        let mut salida = egui::Context::default().run_ui(entrada_con(eventos), |ui| {
            ui.allocate_ui(egui::vec2(ANCHO, ALTO), |ui| {
                super::manejar(ui, app);
            });
        });
        salida.textures_delta.clear();
    }

    /// Ctrl+S pide guardar.
    #[test]
    fn ctrl_s_pide_guardar() {
        let mut app = App::new();

        ventana(&mut app, &pulsar(egui::Key::S, true));

        assert_eq!(app.peticiones(), vec![Command::Save]);
    }

    /// Una pulsación pide una vez, y la soltura no pide nada.
    ///
    /// Va en su propio test porque es la diferencia entre "pedir guardar" y "pedir guardar
    /// otra vez al soltar", y las dos se ven igual si solo se mira lo que pasa al pulsar.
    #[test]
    fn una_pulsacion_pide_una_sola_vez() {
        let mut app = App::new();

        ventana(&mut app, &pulsar(egui::Key::S, true));

        assert_eq!(app.peticiones(), vec![Command::Save]);
    }

    /// El atajo de guardar pide la misma acción que el botón de guardar del menú.
    ///
    /// Es lo que FE-029 pide: el atajo y el botón son la misma operación. Se comprueba
    /// comparando las dos acciones y no contando lo que piden, porque si fueran dos
    /// acciones distintas con el mismo comando, el día que una cambiara el atajo se
    /// quedaría atrás sin que nada lo notara: y la que cambiaría es la del menú, que es la
    /// que el usuario ve.
    #[test]
    fn el_atajo_de_guardar_es_el_boton_de_guardar() {
        let atajo = atajo_de(egui::Key::S, CTRL).expect("Ctrl+S es un atajo");

        let del_menu = crate::frontend::menu::MENUS
            .iter()
            .flat_map(|menu| menu.acciones.iter())
            .find(|accion| accion.nombre == acciones::GUARDAR.nombre)
            .copied()
            .expect("el menú tiene Guardar");

        assert_eq!(atajo.efecto, super::Efecto::Pedir(acciones::GUARDAR));
        assert_eq!(
            del_menu,
            acciones::GUARDAR,
            "el atajo y el botón son la misma acción"
        );
    }

    #[test]
    fn ctrl_z_pide_deshacer() {
        let mut app = App::new();

        ventana(&mut app, &pulsar(egui::Key::Z, true));

        assert_eq!(app.peticiones(), vec![Command::Undo]);
    }

    #[test]
    fn ctrl_y_pide_rehacer() {
        let mut app = App::new();

        ventana(&mut app, &pulsar(egui::Key::Y, true));

        assert_eq!(app.peticiones(), vec![Command::Redo]);
    }

    /// Ctrl+F abre la búsqueda y no pide nada por el camino.
    ///
    /// Que no pida nada es lo mismo que en FE-014 con el menú contextual: abrir una
    /// búsqueda no es buscar, y si abriera pidiendo, cada Ctrl+F sería una búsqueda de la
    /// consulta vacía.
    #[test]
    fn ctrl_f_abre_la_busqueda() {
        let mut app = App::new();

        ventana(&mut app, &pulsar(egui::Key::F, true));

        assert!(app.state().esta_abierta_la_busqueda());
        assert!(!app.state().la_busqueda_tiene_reemplazo());
        assert_eq!(app.peticiones(), Vec::<Command>::new());
    }

    #[test]
    fn ctrl_h_abre_la_busqueda_con_reemplazo() {
        let mut app = App::new();

        ventana(&mut app, &pulsar(egui::Key::H, true));

        assert!(app.state().esta_abierta_la_busqueda());
        assert!(app.state().la_busqueda_tiene_reemplazo());
    }

    /// Una tecla sin el modificador del atajo no hace nada.
    ///
    /// Sin esto, la S de escribir "S" guardaría el documento, y la Z de un identificador
    /// deshacería lo último escrito.
    #[test]
    fn una_tecla_sin_su_modificador_no_pide_nada() {
        let mut app = App::new();

        ventana(&mut app, &pulsar(egui::Key::S, false));
        ventana(&mut app, &pulsar(egui::Key::Z, false));

        assert_eq!(app.peticiones(), Vec::<Command>::new());
        assert!(!app.state().esta_abierta_la_busqueda());
    }

    /// Ninguna combinación de teclas está dos veces en la tabla.
    ///
    /// Una combinación repetida no falla, pero hace que la fila que esté después sea
    /// código muerto: la primera en találcase se lleva la pulsación.
    #[test]
    fn ningun_atajo_esta_repetido() {
        let mut combinaciones: Vec<(egui::Key, Teclas)> = ATAJOS
            .iter()
            .map(|atajo| (atajo.tecla, atajo.teclas))
            .collect();
        let antes = combinaciones.len();

        combinaciones.sort_by_key(|(tecla, teclas)| (*tecla, teclas.shift, teclas.ctrl));
        combinaciones.dedup();

        assert_eq!(
            combinaciones.len(),
            antes,
            "hay una combinación repetida: {ATAJOS:?}"
        );
    }
}
