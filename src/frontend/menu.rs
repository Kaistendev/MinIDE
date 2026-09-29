//! El menú principal de la ventana.
//!
//! Aquí está la barra de menús de arriba: qué menús hay y qué acciones tienen. Los menús
//! se abren, se ven y lo que hay dentro pide su comando por el mismo sitio que la barra de
//! herramientas (FE-009), porque los dos son la misma lista de acciones puesta de otra
//! forma.
//!
//! Las acciones sin comando siguen apagadas, y ahora eso significa algo concreto: el core
//! todavía no tiene ese comando. Lo que va a estar en la barra de herramientas es lo mismo
//! —las acciones con comando—, y se dibujan con el mismo [`acciones::boton`], así que un
//! "Guardar" del menú y el de la barra piden lo mismo por el mismo camino.
//!
//! Un botón apagado y no un botón que no hace nada: un botón que no hace nada parece un
//! fallo del IDE, y uno apagado dice la verdad, que es que esa acción todavía no está.

use eframe::egui;

use super::acciones::{
    self, Accion, ABRIR_PROYECTO, BUSCAR, COMPILAR, COPIAR, CORTAR, DESHACER, DETENER, EJECUTAR,
    GUARDAR, NUEVO_PROYECTO, PANEL_DE_PROPIEDADES, PANEL_DE_PROYECTO, PEGAR, REEMPLAZAR, REHACER,
};
use super::app::App;

/// Un menú de la barra.
#[derive(Debug)]
pub struct Menu {
    /// Lo que se ve en la barra.
    pub titulo: &'static str,
    /// Las acciones del menú, en el orden en que salen.
    pub acciones: &'static [Accion],
}

/// Los menús de la barra, en el orden en que salen.
///
/// Las acciones son las que el proyecto ya tiene escritas en algún sitio —RF-14, T-039,
/// RF-03 y el layout de `frontend-plan.md`—, no ideas nuevas: un menú con acciones que
/// no están en ningún requisito sería un menú que promete algo que nadie ha pedido.
pub const MENUS: &[Menu] = &[
    Menu {
        titulo: "Archivo",
        acciones: &[NUEVO_PROYECTO, ABRIR_PROYECTO, GUARDAR],
    },
    Menu {
        titulo: "Editar",
        acciones: &[DESHACER, REHACER, COPIAR, CORTAR, PEGAR, BUSCAR, REEMPLAZAR],
    },
    Menu {
        titulo: "Ver",
        acciones: &[PANEL_DE_PROYECTO, PANEL_DE_PROPIEDADES],
    },
    Menu {
        titulo: "Compilar",
        acciones: &[COMPILAR, EJECUTAR, DETENER],
    },
];

/// Dibuja la barra de menús en `ui`, y con ella sus cuatro menús.
///
/// Los menús son los de [`MENUS`] y sale de esa tabla toda la barra: añadir un menú es
/// añadir una entrada a la tabla, y no un `menu_button` suelto por el medio, que es como
/// la barra se llena de menús que solo se ven si alguien lee el código.
///
/// `Ui::menu_button` es el que hace el trabajo porque egui sabe si lo que está pintando
/// es una barra o el interior de un menú, y en una barra el menú se abre con clic y no
/// porque el ratón pase por encima.
pub fn barra(ui: &mut egui::Ui, app: &mut App) {
    egui::MenuBar::new().ui(ui, |ui| {
        for menu in MENUS {
            ui.menu_button(menu.titulo, |ui| acciones(ui, app, menu));
        }
    });
}

/// Las acciones de un menú, cada una con el mismo botón que usa la barra.
///
/// Lo que hay de específico del menú es solo la lista: el botón, y con él la manera de
/// pedir el comando, es el de `acciones::boton`. Si el menú dibujara sus propios botones,
/// el mismo nombre se dibujaría dos veces y la mitad de las veces sin pedir nada.
fn acciones(ui: &mut egui::Ui, app: &mut App, menu: &Menu) {
    for accion in menu.acciones {
        acciones::boton(ui, app, *accion);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::App;

    /// Una ventana de trabajo normal, ni enorme ni mínima.
    const ANCHO: f32 = 1000.0;
    const ALTO: f32 = 800.0;

    /// Un frame de la ventana con estos eventos de por medio.
    fn entrada(eventos: &[egui::Event]) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(ANCHO, ALTO),
            )),
            events: eventos.to_vec(),
            ..egui::RawInput::default()
        }
    }

    /// Dibuja la ventana y devuelve los rectángulos que se han pintado.
    fn dibujar(context: &egui::Context, app: &mut App, eventos: &[egui::Event]) -> Vec<egui::Rect> {
        let mut salida = context.run_ui(entrada(eventos), |ui| {
            app.dibujar(ui);
        });

        // egui avisa si se tiran sin aplicar las texturas que ha creado, y en una
        // aplicación de verdad las aplica el renderizador. Aquí no hay renderizador, así
        // que se vacían a propósito: lo que se prueba son los menús, no los píxeles.
        salida.textures_delta.clear();

        salida
            .shapes
            .into_iter()
            .filter_map(|forma| match forma.shape {
                egui::Shape::Rect(rectangulo) => Some(rectangulo.rect),
                _ => None,
            })
            .collect()
    }

    /// Dibuja la ventana con una aplicación de sobra para mirar, y devuelve lo pintado.
    fn pintar(context: &egui::Context) -> Vec<egui::Rect> {
        let mut app = App::new();

        dibujar(context, &mut app, &[])
    }

    /// La franja de arriba de la ventana, que es donde vive la barra de menús.
    ///
    /// Es la primera zona que cruza la ventana de lado a lado, y el layout ya tiene sus
    /// tests para eso: aquí solo hace falta saber dónde está.
    fn franja_de_arriba(context: &egui::Context) -> egui::Rect {
        let mut zonas: Vec<egui::Rect> = pintar(context)
            .into_iter()
            .filter(|rectangulo| rectangulo.width() >= ANCHO)
            .collect();
        zonas.sort_by(|uno, otro| uno.min.y.total_cmp(&otro.min.y));

        zonas.first().copied().expect("la ventana tiene zonas")
    }

    /// Los botones de la barra, de izquierda a derecha, uno por menú.
    ///
    /// Se descubren mirando lo que se pinta en vez de escribir unas coordenadas: el ancho
    /// de un botón depende del texto del menú y de la fuente, y un test que se sabe las
    /// posiciones se rompe con el primer cambio de nombre. Cada botón pinta su fondo y su
    /// borde en el mismo rectángulo, así que los repetidos de la lista son el mismo botón.
    ///
    /// La franja de arriba tiene dos filas, la del menú y la de la barra de herramientas, y
    /// aquí solo interesa la de arriba: se separa por su `min.y` y no por una posición
    /// fija, porque las filas cambian de alto según lo que llevan encima.
    fn botones(context: &egui::Context) -> Vec<egui::Rect> {
        let franja = franja_de_arriba(context);

        let mut rectangulos: Vec<egui::Rect> = pintar(context)
            .into_iter()
            .filter(|rectangulo| {
                rectangulo.min.y >= franja.min.y && rectangulo.max.y <= franja.max.y
            })
            .filter(|rectangulo| rectangulo.width() < ANCHO)
            .collect();

        rectangulos.sort_by(|uno, otro| {
            uno.min
                .x
                .total_cmp(&otro.min.x)
                .then(uno.min.y.total_cmp(&otro.min.y))
        });
        rectangulos.dedup();

        let fila_de_arriba = rectangulos
            .iter()
            .map(|rectangulo| rectangulo.min.y)
            .fold(f32::INFINITY, f32::min);
        rectangulos.retain(|rectangulo| rectangulo.min.y <= fila_de_arriba);

        assert_eq!(
            rectangulos.len(),
            MENUS.len(),
            "la barra tiene que pintar un botón por menú: {rectangulos:?}"
        );

        rectangulos
    }

    /// Los elementos de un menú abierto, de arriba abajo y uno por acción.
    ///
    /// Un menú abierto se pinta debajo de la franja de arriba, y sus elementos son
    /// estrechos y no están pegados a la ventana: un menú que flota va con su botón, que
    /// tampoco toca el borde. Con esas dos cosas fuera se descartan el explorador de
    /// proyectos, que está pegado al borde izquierdo (FE-010), y el área central, que llega
    /// al borde derecho, y lo que sale son los botones de las acciones del menú.
    ///
    /// Da igual que los elementos estén apagados: también se pintan, y una acción apagada
    /// que no se pintara sería invisible en vez de desactivada.
    fn elementos(context: &egui::Context, app: &mut App, esperados: usize) -> Vec<egui::Rect> {
        let franja = franja_de_arriba(context);

        let mut rectangulos: Vec<egui::Rect> = dibujar(context, app, &[])
            .into_iter()
            .filter(|rectangulo| rectangulo.max.y > franja.max.y)
            .filter(|rectangulo| rectangulo.min.x > 0.0)
            .filter(|rectangulo| rectangulo.width() < ANCHO / 2.0)
            .collect();

        rectangulos.sort_by(|uno, otro| uno.min.y.total_cmp(&otro.min.y));
        rectangulos.dedup();

        assert_eq!(
            rectangulos.len(),
            esperados,
            "un menú abierto tiene que pintar un elemento por acción: {rectangulos:?}"
        );

        rectangulos
    }

    /// Pulsa y suelta el botón izquierdo del ratón en `pos`.
    ///
    /// Son dos frames y no uno porque egui solo cuenta un clic si la pulsación y la
    /// soltura caen las dos en el widget, y cada frame procesa los eventos que le llegan.
    fn clic(context: &egui::Context, app: &mut App, pos: egui::Pos2) {
        for pressed in [true, false] {
            let boton = egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            };

            dibujar(context, app, &[boton]);
        }
    }

    /// Abre un menú y pulsa una de sus acciones, y devuelve la aplicación que lo ha pedido.
    ///
    /// Es el camino completo de una acción de menú: abrir el menú, pulsar el elemento y
    /// mirar qué ha pedido la aplicación. Se hace con clics de verdad y no con una llamada
    /// a la función, porque un clic que no pasa por el botón es un clic que no existe.
    fn pulsar_accion(indice_menu: usize, indice_accion: usize, acciones_del_menu: usize) -> App {
        let context = egui::Context::default();
        let mut app = App::new();

        let botones = botones(&context);
        clic(&context, &mut app, botones[indice_menu].center());
        dibujar(&context, &mut app, &[]);

        assert!(
            context.any_popup_open(),
            "el menú {indice_menu} no se ha abierto y su acción no se puede pulsar"
        );

        let elementos = elementos(&context, &mut app, acciones_del_menu);
        clic(&context, &mut app, elementos[indice_accion].center());
        dibujar(&context, &mut app, &[]);

        app
    }

    /// Los menús de la barra se distinguen unos de otros y enseñan acciones con nombre.
    ///
    /// Un menú sin título se confunde con el de al lado, y un menú sin acciones es un
    /// menú que se abre y no dice nada: en los dos casos la barra miente sobre lo que
    /// MiniIDE puede hacer.
    #[test]
    fn cada_menu_se_distingue_y_enseña_acciones_con_nombre() {
        assert!(
            !MENUS.is_empty(),
            "la barra tiene que tener los menús que MiniIDE ofrece"
        );

        let mut titulos: Vec<&str> = MENUS.iter().map(|menu| menu.titulo).collect();
        titulos.sort_unstable();
        titulos.dedup();

        assert_eq!(
            titulos.len(),
            MENUS.len(),
            "dos menús con el mismo título son el mismo menú en la barra: {titulos:?}"
        );

        for menu in MENUS {
            assert!(
                !menu.titulo.is_empty(),
                "un menú sin título no se ve en la barra: {menu:?}"
            );
            assert!(
                !menu.acciones.is_empty(),
                "un menú que se abre y no tiene acciones no dice nada: {menu:?}"
            );

            for accion in menu.acciones {
                assert!(
                    !accion.nombre.is_empty(),
                    "una acción sin nombre es un botón en blanco: {menu:?}"
                );
            }
        }
    }

    /// El menú no llama al core: pide por el punto único.
    ///
    /// El menú y la barra de herramientas son dos superficies de lo mismo, y lo que las hace
    /// lo mismo es que las dos piden por `App::emitir`. En cuanto el menú llamara al core
    /// por su cuenta habría dos caminos para la misma operación, y el mismo nombre pediría
    /// una cosa en el menú y otra en la barra sin que nada lo dijera.
    ///
    /// Se miran los `use` y no el texto entero a propósito, porque el test está escrito
    /// aquí dentro: un `contains` sobre todo el archivo se encontraría a sí mismo con el
    /// nombre del módulo del core.
    #[test]
    fn el_menu_no_llama_al_core_pide_por_el_punto_unico() {
        let source =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file!()))
                .expect("el fuente del menú tiene que poder leerse");

        let uses: Vec<&str> = source
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("use crate::"))
            .filter(|line| line.contains("::commands"))
            .collect();

        assert!(
            uses.is_empty(),
            "el menú pide por el punto único, no llama al core por su cuenta: {uses:?}"
        );
    }

    /// La barra pinta un botón por menú, y todos en la misma fila de arriba.
    ///
    /// Los menús tienen que verse como una fila y no como un menú suelto en cualquier
    /// sitio: es la señal de que hay más cosas detrás, que es lo que hace que se busquen.
    #[test]
    fn la_barra_dibuja_una_fila_de_botones_arriba() {
        let context = egui::Context::default();

        let franja = franja_de_arriba(&context);
        let botones = botones(&context);

        for boton in &botones {
            assert!(
                boton.min.x >= 0.0 && boton.max.x <= ANCHO,
                "un botón se sale de la ventana: {boton:?}"
            );
            assert!(
                boton.min.y >= franja.min.y && boton.max.y <= franja.max.y,
                "un botón se sale de la franja de arriba: {boton:?} {franja:?}"
            );
        }

        for par in botones.windows(2) {
            assert!(
                par[0].max.x <= par[1].min.x,
                "los botones de la barra van uno detrás de otro: {par:?}"
            );
        }
    }

    /// Cada menú se abre al pulsar su botón.
    ///
    /// Es lo único que hace falta para que un menú sea un menú: que abra. Se comprueba con
    /// clics de verdad, pulsando donde egui ha pintado cada botón, y no leyendo un estado
    /// que el propio menú dice de sí mismo.
    #[test]
    fn cada_menu_se_abre_al_pulsar_su_boton() {
        assert!(
            !MENUS.is_empty(),
            "si no hay menús, este test no comprueba que ningún menú se abra"
        );

        for (indice, menu) in MENUS.iter().enumerate() {
            let context = egui::Context::default();
            let mut app = App::new();
            let botones = botones(&context);

            assert!(
                !context.any_popup_open(),
                "sin pulsar nada no hay ningún menú abierto: {:?}",
                menu.titulo
            );

            clic(&context, &mut app, botones[indice].center());
            dibujar(&context, &mut app, &[]);

            assert!(
                context.any_popup_open(),
                "el menú {:?} no se abre al pulsar su botón: {botones:?}",
                menu.titulo
            );
        }
    }

    /// Cada acción de menú pide su comando, y pide el mismo que su gemela de la barra.
    ///
    /// Esto es lo que FE-009 viene a conseguir: que un elemento de menú y un botón de la
    /// barra de herramientas pidan lo mismo porque son la misma acción, y no dos acciones
    /// parecidas. Se abre cada menú, se pulsa cada elemento con comando y se mira lo que
    /// ha pedido la aplicación; la parte de que el botón de la barra pida lo mismo la
    /// comprueba `acciones`, que es donde están las dos listas.
    #[test]
    fn cada_accion_de_menu_pide_su_comando() {
        let mut con_comando = 0;

        for (indice_menu, menu) in MENUS.iter().enumerate() {
            for (indice_accion, accion) in menu.acciones.iter().enumerate() {
                let Some(comando) = accion.comando else {
                    continue;
                };
                con_comando += 1;

                let app = pulsar_accion(indice_menu, indice_accion, menu.acciones.len());

                assert_eq!(
                    app.peticiones(),
                    [comando],
                    "el elemento {:?} del menú {:?} tiene que pedir {:?}",
                    accion.nombre,
                    menu.titulo,
                    comando
                );
            }
        }

        assert!(
            con_comando > 0,
            "si no hay ninguna acción con comando, este test no comprueba nada"
        );
    }

    /// Una acción sin comando no pide nada, porque no hay nada que pedir.
    ///
    /// Las acciones sin comando están apagadas, y no por un descuido de la interfaz: el
    /// core todavía no tiene ese comando. Si se pudieran pulsar y no pasara nada, el
    /// usuario vería un botón que acepta el clic y no hace nada, que es peor que uno
    /// apagado porque parece que MiniIDE ha hecho caso.
    #[test]
    fn una_accion_sin_comando_no_pide_nada() {
        let mut sin_comando = 0;

        for (indice_menu, menu) in MENUS.iter().enumerate() {
            for (indice_accion, accion) in menu.acciones.iter().enumerate() {
                if accion.comando.is_some() {
                    continue;
                }
                sin_comando += 1;

                let app = pulsar_accion(indice_menu, indice_accion, menu.acciones.len());

                assert!(
                    app.peticiones().is_empty(),
                    "la acción {:?} no tiene comando y no puede pedir nada, y pidió {:?}",
                    accion.nombre,
                    app.peticiones()
                );
            }
        }

        assert!(
            sin_comando > 0,
            "si no hay ninguna acción sin comando, este test no comprueba nada"
        );
    }
}
