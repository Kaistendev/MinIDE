//! La barra de herramientas: los botones de arriba.
//!
//! Son los cinco botones que un IDE tiene siempre a mano —abrir proyecto, guardar,
//! compilar, ejecutar y detener— porque son las operaciones que se repiten mientras se
//! trabaja (RF-14 y T-039).
//!
//! Los botones no hacen nada por su cuenta: son [`acciones::Accion`] de las mismas que usa
//! el menú, se dibujan con [`acciones::boton`] y piden por el mismo sitio. Lo que aquí solo
//! decide es cuáles se ven y en qué orden.
//!
//! Piden y no ejecutan, y no por descuido: a la ventana todavía no le llega nada del core,
//! porque `App` lleva el estado visual y las peticiones, y no tiene un core al que
//! preguntarle. Lo que hace un clic es dejar escrito qué comando se ha pedido, y FE-040 a
//! FE-042 son las que enchufen la ejecución donde ya está el sitio.
//!
//! Por eso estos botones van encendidos y lo que no se puede hacer no está: un botón
//! apagado en la barra sería un botón que se ve y no se puede usar (FE-077).

use eframe::egui;

use super::acciones::{self, Accion, ABRIR_PROYECTO, COMPILAR, DETENER, EJECUTAR, GUARDAR};
use super::app::App;

/// Los botones de la barra, en el orden en que salen.
///
/// Son los cinco que T-039 pide accesibles desde la UI, y los mismos que están en sus
/// menús. No está "Guardar todo" porque su comando todavía no existe (T-096), y un botón
/// que pide un comando que no está fallaría justo cuando alguien lo pulse.
pub const BOTONES: &[Accion] = &[ABRIR_PROYECTO, GUARDAR, COMPILAR, EJECUTAR, DETENER];

/// Dibuja la barra de herramientas, una fila de botones.
///
/// Cada botón se dibuja y se pide con `acciones::boton`, que es el único sitio donde un
/// clic se convierte en un comando. Esta función solo dice cuáles se ven y en qué orden,
/// que es lo único de la barra que no es de las acciones.
pub fn barra(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        for accion in BOTONES {
            acciones::boton(ui, app, *accion);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Command;
    use crate::frontend::App;

    /// Una ventana de trabajo normal, ni enorme ni mínima.
    const ANCHO: f32 = 1000.0;
    const ALTO: f32 = 800.0;

    /// Un frame de la ventana con estos eventos de por medio, y lo que se ha pedido.
    ///
    /// Se recoge lo que devuelve la ventana porque los botones no ejecutan nada: lo
    /// único que hacen es apuntar su comando, y ese es el retorno de la ventana.
    fn dibujar(context: &egui::Context, app: &mut App, eventos: &[egui::Event]) {
        let mut salida = context.run_ui(entrada(eventos), |ui| {
            app.dibujar(ui);
        });

        // egui avisa si se tiran sin aplicar las texturas que ha creado, y en una
        // aplicación de verdad las aplica el renderizador. Aquí no hay renderizador, así
        // que se vacían a propósito: lo que se prueba son los botones, no los píxeles.
        salida.textures_delta.clear();
    }

    /// Un frame con una ventana de tamaño conocido.
    fn entrada(eventos: &[egui::Event]) -> eframe::egui::RawInput {
        eframe::egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(ANCHO, ALTO),
            )),
            events: eventos.to_vec(),
            ..eframe::egui::RawInput::default()
        }
    }

    /// Dibuja la ventana y devuelve los rectángulos que se han pintado.
    fn pintar(context: &egui::Context) -> Vec<egui::Rect> {
        let mut app = App::new();

        let mut salida = context.run_ui(entrada(&[]), |ui| {
            app.dibujar(ui);
        });
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

    /// La franja de arriba de la ventana, que es donde vive la barra.
    fn franja_de_arriba(context: &egui::Context) -> egui::Rect {
        let mut zonas: Vec<egui::Rect> = pintar(context)
            .into_iter()
            .filter(|rectangulo| rectangulo.width() >= ANCHO)
            .collect();
        zonas.sort_by(|uno, otro| uno.min.y.total_cmp(&otro.min.y));

        zonas.first().copied().expect("la ventana tiene zonas")
    }

    /// Los botones de la barra: los de la fila de debajo del menú.
    ///
    /// La franja de arriba tiene dos filas, la del menú y la de la barra, y se separan
    /// por su `min.y` y no por una posición fija: las filas cambian de alto según lo que
    /// llevan encima, y un test que se sabe los píxeles se rompe con el primer cambio.
    ///
    /// Los rectángulos se descubren mirando lo que se pinta en vez de escribir unas
    /// coordenadas, porque el ancho de un botón depende del texto y de la fuente. Cada
    /// botón pinta su fondo y su borde en el mismo rectángulo, así que los repetidos de
    /// la lista son el mismo botón.
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

        let fila_del_menu = rectangulos
            .iter()
            .map(|rectangulo| rectangulo.min.y)
            .fold(f32::INFINITY, f32::min);
        rectangulos.retain(|rectangulo| rectangulo.min.y > fila_del_menu);

        assert_eq!(
            rectangulos.len(),
            BOTONES.len(),
            "la barra tiene que pintar un botón por cada botón que tiene: {rectangulos:?}"
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

    /// La barra tiene los cinco botones del RF-14, y ninguno se repite.
    ///
    /// Son los que T-039 pide accesibles desde la UI, y ninguno se repite porque dos
    /// botones con el mismo comando son el mismo botón puesto dos veces, que es la
    /// duplicación que FE-009 viene a evitar.
    ///
    /// De cada botón se mira además su texto, porque un botón que dice "Guardar" y pide
    /// otra cosa es un botón que miente, y eso no se ve en la ventana: solo aparece al
    /// pulsarlo, cuando ya es tarde.
    #[test]
    fn la_barra_tiene_los_cinco_botones_del_rf_14() {
        use std::collections::HashSet;

        assert!(
            !BOTONES.is_empty(),
            "la barra tiene que tener los botones del RF-14"
        );

        for (nombre, comando) in [
            ("Abrir proyecto", Command::OpenProject),
            ("Guardar", Command::Save),
            ("Compilar", Command::Build),
            ("Ejecutar", Command::Run),
            ("Detener", Command::Stop),
        ] {
            let boton = BOTONES
                .iter()
                .find(|boton| boton.nombre == nombre)
                .unwrap_or_else(|| panic!("la barra no tiene ningun boton que diga {nombre:?}"));

            assert_eq!(
                boton.comando, comando,
                "el boton {nombre:?} tiene que pedir {comando:?}"
            );
        }

        let comandos: HashSet<Command> = BOTONES.iter().map(|boton| boton.comando).collect();
        assert_eq!(
            comandos.len(),
            BOTONES.len(),
            "dos botones con el mismo comando son el mismo botón en dos sitios: {comandos:?}"
        );

        let nombres: HashSet<&str> = BOTONES.iter().map(|boton| boton.nombre).collect();
        assert_eq!(
            nombres.len(),
            BOTONES.len(),
            "dos botones con el mismo texto no se distinguen en la barra: {nombres:?}"
        );
        assert!(
            nombres.iter().all(|nombre| !nombre.is_empty()),
            "un botón sin texto no se ve: {nombres:?}"
        );
    }

    /// Cada botón pide su comando al pulsarlo, y no pide nada si nadie pulsa.
    ///
    /// Un botón que no pide su comando es un botón que no hace nada, y eso es justo lo que
    /// esta tarea tiene que evitar: la barra de herramientas existe para que abrir, guardar,
    /// compilar, ejecutar y detener se puedan pedir desde la ventana.
    #[test]
    fn cada_boton_pide_su_comando_al_pulsarlo() {
        assert!(
            !BOTONES.is_empty(),
            "si no hay botones, este test no comprueba que ningún botón pida su comando"
        );

        for (indice, boton) in BOTONES.iter().enumerate() {
            let context = egui::Context::default();
            let mut app = App::new();

            dibujar(&context, &mut app, &[]);
            assert!(
                app.peticiones().is_empty(),
                "sin pulsar nada no se pide nada, y se pidió {:?}",
                app.peticiones()
            );

            let botones = botones(&context);
            clic(&context, &mut app, botones[indice].center());
            dibujar(&context, &mut app, &[]);

            assert_eq!(
                app.peticiones(),
                [boton.comando],
                "el botón {:?} tiene que pedir su comando y solo ese",
                boton.nombre
            );
        }
    }
}
