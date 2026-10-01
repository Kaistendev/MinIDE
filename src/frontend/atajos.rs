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

use super::acciones::{Accion, COMPILAR, DESHACER, DETENER, EJECUTAR, GUARDAR, REHACER};
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

/// Solo `Shift`, que es lo que separa ejecutar de detener.
///
/// Sin un `const` para eso habría que escribir la pareja suelta en la fila, y una fila que
/// no se parece a las de al lado se lee peor que el resto de la tabla.
const SHIFT: Teclas = Teclas {
    ctrl: false,
    shift: true,
};

/// Ningún modificador, que es como se pulsan las teclas de función.
///
/// F5 y Shift+F5 se separan por el `Shift`, así que F5 necesita decir que no lleva
/// ninguno: si la fila dejara el modificador en `false` por omisión y `atajo_de` no lo
/// comprobara, las dos filas serían la misma.
const NINGUNA: Teclas = Teclas {
    ctrl: false,
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
/// Guardar, deshacer, rehacer, buscar y reemplazar, y desde FE-040 a FE-042 también
/// compilar, ejecutar y detener.
///
/// Los que piden una acción piden la misma del vocabulario que sus botones, para que un
/// atajo y un botón no puedan pedir cosas distintas: si el atajo de guardar escribiera su
/// propio comando, el botón y el atajo dejarían de ser la misma operación el día que uno de
/// los dos cambiara. Compilar, ejecutar y detener no son una excepción: salen de la misma
/// `Accion` que sus botones del menú y de la barra de herramientas, y por eso las tres
/// superficies son la misma operación y no tres parecidas.
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
    Atajo {
        tecla: egui::Key::B,
        teclas: CTRL,
        efecto: Efecto::Pedir(COMPILAR),
    },
    Atajo {
        tecla: egui::Key::F5,
        teclas: NINGUNA,
        efecto: Efecto::Pedir(EJECUTAR),
    },
    Atajo {
        tecla: egui::Key::F5,
        teclas: SHIFT,
        efecto: Efecto::Pedir(DETENER),
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
        Efecto::Pedir(accion) => app.emitir(accion.comando),
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
        pulsar_con(tecla, ctrl, false)
    }

    /// Una pulsación y su soltura, con los modificadores que se le pidan.
    ///
    /// Es una sola función y no una por cada combinación porque los atajos de compilar,
    /// ejecutar y detener usan modificadores distintos: Ctrl+B lleva `ctrl`, F5 no lleva
    /// ninguno y Shift+F5 lleva `shift`.
    fn pulsar_con(tecla: egui::Key, ctrl: bool, shift: bool) -> Vec<egui::Event> {
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key: tecla,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl,
                    shift,
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

    /// La acción del menú o de la barra que se llama `nombre`.
    ///
    /// Se busca por el nombre y no por el comando porque es el nombre lo que las tres
    /// superficies tienen que compartir: si el menú y el atajo pidieran el mismo comando con
    /// nombres distintos, serían dos operaciones que se parecen.
    fn accion_de_la_ventana(nombre: &str) -> acciones::Accion {
        crate::frontend::menu::MENUS
            .iter()
            .flat_map(|menu| menu.acciones.iter())
            .find(|accion| accion.nombre == nombre)
            .copied()
            .unwrap_or_else(|| panic!("la ventana no tiene ninguna acción que diga {nombre:?}"))
    }

    /// El atajo de `tecla` con esas modificadores, si lo hay.
    fn atajo(tecla: egui::Key, teclas: Teclas) -> &'static super::Atajo {
        super::atajo_de(tecla, teclas)
            .unwrap_or_else(|| panic!("{tecla:?} con {teclas:?} no es un atajo"))
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

    /// Ctrl+B pide compilar. FE-040.
    #[test]
    fn ctrl_b_pide_compilar() {
        let mut app = App::new();

        ventana(&mut app, &pulsar_con(egui::Key::B, true, false));

        assert_eq!(app.peticiones(), vec![Command::Build]);
    }

    /// F5 pide ejecutar y Shift+F5 pide detener. FE-041 y FE-042.
    ///
    /// Van en el mismo test porque son la misma tecla y lo que los distingue es el
    /// modificador: si se comprobaran por separado, un F5 que pidiera las dos cosas pasaría
    /// los dos tests y seguiría estando mal.
    #[test]
    fn f5_pide_ejecutar_y_con_shift_pide_detener() {
        let mut app = App::new();
        ventana(&mut app, &pulsar_con(egui::Key::F5, false, false));
        assert_eq!(app.peticiones(), vec![Command::Run]);

        let mut app = App::new();
        ventana(&mut app, &pulsar_con(egui::Key::F5, false, true));
        assert_eq!(app.peticiones(), vec![Command::Stop]);
    }

    /// Compilar, ejecutar y detener piden lo mismo desde el atajo, el menú y la barra.
    /// FE-040, FE-041 y FE-042.
    ///
    /// Las tres tareas piden lo mismo —que las tres superficies disparen la misma
    /// operación— y por eso van en un solo test: separadas, dejarían abierta la
    /// posibilidad de que un atajo pidiera `Build` mientras su botón pidiera otra cosa y los
    /// dos testsasen.
    ///
    /// Se comparan las tres `Accion` enteras y no los comandos que piden, porque si fueran
    /// acciones distintas con el mismo comando, el día que una de las dos cambiara la otra se
    /// quedaría atrás sin que nada lo notara: y la que cambiaría es la que ve el usuario.
    #[test]
    fn el_atajo_el_menu_y_la_barra_piden_la_misma_accion() {
        use crate::frontend::toolbar::BOTONES;

        for (tecla, teclas, nombre) in [
            (egui::Key::B, super::CTRL, "Compilar"),
            (egui::Key::F5, super::NINGUNA, "Ejecutar"),
            (egui::Key::F5, super::SHIFT, "Detener"),
        ] {
            let del_atajo = atajo(tecla, teclas);
            let del_menu = accion_de_la_ventana(nombre);
            let de_la_barra = BOTONES
                .iter()
                .find(|accion| accion.nombre == nombre)
                .copied();

            assert_eq!(
                del_atajo.efecto,
                super::Efecto::Pedir(del_menu),
                "el atajo de {nombre:?} tiene que pedir la acción del menú"
            );
            assert_eq!(
                de_la_barra,
                Some(del_menu),
                "el botón de {nombre:?} de la barra tiene que ser la misma acción"
            );
        }
    }

    /// Cada atajo de la tabla produce lo que dice, y la tabla entera está probada. FE-069.
    ///
    /// Los tests de arriba comprueban atajos sueltos, uno por uno, y por eso un atajo nuevo se
    /// podría añadir a la tabla sin que nadie lo probara: la tabla crecería y el conjunto de
    /// tests se quedaría igual. Aquí se recorre la tabla entera y cada fila tiene que hacer lo
    /// que dice a través del camino de verdad —una pulsación por la ventana—, así que lo que
    /// no esté comprobado es que la fila no esté en la tabla.
    ///
    /// Se comprueba por filas y no contando: contar comprobaría que hay ocho filas, no que las
    /// ocho hagan lo que deben.
    #[test]
    fn cada_atajo_de_la_tabla_hace_lo_que_dice() {
        for atajo in ATAJOS {
            let mut app = App::new();
            let Teclas { ctrl, shift } = atajo.teclas;

            ventana(&mut app, &pulsar_con(atajo.tecla, ctrl, shift));

            match atajo.efecto {
                super::Efecto::Pedir(accion) => assert_eq!(
                    app.peticiones(),
                    vec![accion.comando],
                    "el atajo de {atajo:?} tiene que pedir lo que dice su acción"
                ),
                super::Efecto::Abrir(_) => assert!(
                    app.peticiones().is_empty(),
                    "el atajo de {atajo:?} abre un diálogo, que no es un comando: {:?}",
                    app.peticiones()
                ),
            }
        }
    }

    /// Ningún atajo se queda sin probar. FE-069.
    ///
    /// Es la otra mitad del test anterior y va aparte porque es la que falla cuando alguien
    /// añade una fila a la tabla sin escribir su caso: el recorrido anterior seguiría en verde
    /// porque comprobaría la fila nueva contra sí misma. Aquí la lista de atajos comprobados a
    /// mano tiene que ser la tabla entera, y si no lo es el test dice cuál falta.
    #[test]
    fn ningun_atajo_se_queda_sin_probar() {
        let comprobados: Vec<(egui::Key, Teclas)> = vec![
            (egui::Key::S, CTRL),
            (egui::Key::Z, CTRL),
            (egui::Key::Y, CTRL),
            (egui::Key::F, CTRL),
            (egui::Key::H, CTRL),
            (egui::Key::B, CTRL),
            (egui::Key::F5, super::NINGUNA),
            (egui::Key::F5, super::SHIFT),
        ];

        let en_la_tabla: Vec<(egui::Key, Teclas)> = ATAJOS
            .iter()
            .map(|atajo| (atajo.tecla, atajo.teclas))
            .collect();

        let sin_probar: Vec<(egui::Key, Teclas)> = en_la_tabla
            .iter()
            .filter(|atajo| !comprobados.contains(atajo))
            .copied()
            .collect();

        assert!(
            sin_probar.is_empty(),
            "hay atajos en la tabla que ningún test comprueba uno por uno: {sin_probar:?}. \
             O se quita el atajo de la lista de arriba, o se escribe su test: la lista es \
             para que un atajo no se quede sin mirar."
        );
        assert_eq!(
            comprobados.len(),
            en_la_tabla.len(),
            "y la lista no puede tener atajos que no estén en la tabla: {comprobados:?}"
        );
    }

    /// Ctrl no aplasta a Shift, ni al revés.
    ///
    /// Sin esto, Shift+F5 pediría compilar y ejecutar a la vez: la tabla busca la primera
    /// fila que coincide y, con el `ctrl` de una sin comprobar, dos combinaciones distintas
    /// se parecerían lo suficiente como para que la pulsación cayera en la fila que no
    /// tocaba.
    #[test]
    fn los_modificadores_no_se_confunden_entre_si() {
        let con_ctrl_y_shift = Teclas {
            ctrl: true,
            shift: true,
        };

        assert_eq!(
            atajo(egui::Key::F5, super::NINGUNA).efecto,
            super::Efecto::Pedir(crate::frontend::acciones::EJECUTAR),
            "F5 sin modificadores ejecuta"
        );
        assert_eq!(
            atajo(egui::Key::F5, super::SHIFT).efecto,
            super::Efecto::Pedir(crate::frontend::acciones::DETENER),
            "Shift+F5 detiene"
        );
        assert!(
            atajo_de(egui::Key::F5, con_ctrl_y_shift).is_none(),
            "Ctrl+Shift+F5 no es un atajo: no hay fila que lo atienda a propósito"
        );

        let mut app = App::new();
        ventana(&mut app, &pulsar_con(egui::Key::F5, true, true));

        assert_eq!(
            app.peticiones(),
            Vec::<Command>::new(),
            "una combinación que no está en la tabla no pide nada"
        );
    }
}
