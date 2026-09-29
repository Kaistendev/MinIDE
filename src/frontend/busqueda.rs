//! La búsqueda y el reemplazo de texto del documento.
//!
//! Es una barra, no una ventana aparte: aparece encima del editor con lo que se está
//! buscando y con lo que se va a reemplazar, y se cierra con su ×. Está cerrada por
//! defecto, y abierta o cerrada es estado de la ventana (FE-032 y FE-033).
//!
//! Lo que se escribe en el campo de búsqueda no se busca aquí: se pide al core con
//! `Find`, que es lo que sabe dónde está cada coincidencia. Lo de reemplazar se pide
//! con el botón, y no al escribir el texto de reemplazo: escribir "b" en el campo no es
//! pedir que se reemplace nada, y si lo fuera, cada tecla de un texto de reemplazo
//! borraría el documento.
//!
//! Las coincidencias y por dónde está la siguiente no se ven aquí todavía: las trae
//! otra tarea, y hasta que las traiga, la barra dice qué se busca y pregunta por el resto.

use crate::commands::Command;
use eframe::egui;

use super::acciones::REEMPLAZAR;
use super::app::App;

/// Lo que mide de ancho un campo de la barra, en puntos.
const ANCHO_DE_UN_CAMPO: f32 = 200.0;

/// Dibuja la barra de búsqueda si hay alguna búsqueda abierta.
///
/// Sin búsqueda no se dibuja nada, ni una barra vacía ni un hueco: una barra de búsqueda
/// siempre abierta sería un recordatorio permanente de algo que no se está buscando, y un
/// hueco entre las pestañas y el texto se lleva por delante las primeras líneas.
pub fn panel(ui: &mut egui::Ui, app: &mut App) {
    if !app.state().esta_abierta_la_busqueda() {
        return;
    }

    ui.horizontal(|ui| {
        consulta(ui, app);

        if app.state().la_busqueda_tiene_reemplazo() {
            reemplazo(ui, app);
            super::acciones::boton(ui, app, REEMPLAZAR);
        }

        if ui.small_button("×").clicked() {
            app.state_mut().cerrar_busqueda();
        }
    });
}

/// El campo de lo que se busca, que es lo que pide al core cada vez que cambia.
fn consulta(ui: &mut egui::Ui, app: &mut App) {
    let respuesta = {
        let estado = app.state_mut();
        let busqueda = estado
            .busqueda_mut()
            .expect("la busqueda se comprueba antes de dibujar sus campos");

        egui::TextEdit::singleline(busqueda.consulta_mut())
            .hint_text("Buscar")
            .desired_width(ANCHO_DE_UN_CAMPO)
            .show(ui)
    };

    if respuesta.response.changed() {
        app.emitir(Command::Find);
    }
}

/// El campo de con qué se reemplaza, que no pide nada por sí solo.
fn reemplazo(ui: &mut egui::Ui, app: &mut App) {
    let Some(busqueda) = app.state_mut().busqueda_mut() else {
        return;
    };
    let Some(reemplazo) = busqueda.reemplazo_mut() else {
        return;
    };

    let _ = egui::TextEdit::singleline(reemplazo)
        .hint_text("Reemplazar por")
        .desired_width(ANCHO_DE_UN_CAMPO)
        .show(ui);
}

#[cfg(test)]
mod tests {
    use crate::commands::Command;
    use crate::frontend::App;
    use eframe::egui;

    /// Una ventana de trabajo normal, ni enorme ni mínima.
    const ANCHO: f32 = 1000.0;
    const ALTO: f32 = 800.0;

    /// El hueco donde se dibuja la barra. Alto a propósito pequeño: la barra es una fila.
    const ALTO_DE_LA_BARRA: f32 = 40.0;

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

    /// Dibuja la barra y devuelve los rectángulos que ha pintado, de izquierda a derecha.
    ///
    /// De izquierda a derecha y no de arriba abajo porque la barra es una fila, y el
    /// orden de los rectángulos es el que se ve: campo de consulta, campo de reemplazo,
    /// botón de reemplazar y el de cerrar. Los tests los toman por su sitio, que es
    /// también el sitio donde se puede pulsar cada cosa.
    fn barra(context: &egui::Context, app: &mut App, eventos: &[egui::Event]) -> Vec<egui::Rect> {
        let mut salida = context.run_ui(entrada_con(eventos), |ui| {
            ui.allocate_ui(egui::vec2(ANCHO, ALTO_DE_LA_BARRA), |ui| {
                super::panel(ui, app);
            });
        });
        salida.textures_delta.clear();

        let mut rectangulos: Vec<egui::Rect> = salida
            .shapes
            .iter()
            .filter_map(|forma| match &forma.shape {
                egui::Shape::Rect(rectangulo) => Some(rectangulo.rect),
                _ => None,
            })
            .collect();
        rectangulos.sort_by(|uno, otro| uno.min.x.total_cmp(&otro.min.x));
        rectangulos.dedup();

        rectangulos
    }

    /// Un clic de verdad, con su pulsación y su soltura.
    fn clic(posicion: egui::Pos2) -> Vec<egui::Event> {
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::PointerButton {
                pos: posicion,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            })
            .chain(std::iter::once(egui::Event::PointerMoved(posicion)))
            .collect()
    }

    /// Lo que se escribe a máquina, con el puntero en el campo.
    fn escribir(texto: &str, posicion: egui::Pos2) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(posicion),
            egui::Event::Text(texto.to_owned()),
        ]
    }

    /// El rectángulo `indice` de la barra, contando de izquierda a derecha.
    fn elemento(context: &egui::Context, app: &mut App, indice: usize) -> egui::Rect {
        let rectangulos = barra(context, app, &[]);

        rectangulos
            .get(indice)
            .copied()
            .unwrap_or_else(|| panic!("la barra no tiene el elemento {indice}: {rectangulos:?}"))
    }

    /// Sin búsqueda no se pinta nada, ni una barra vacía.
    #[test]
    fn sin_busqueda_no_se_dibuja_nada() {
        let mut app = App::new();
        let context = egui::Context::default();

        assert!(
            barra(&context, &mut app, &[]).is_empty(),
            "una barra de búsqueda siempre abierta sería un recordatorio de algo que no se busca"
        );
    }

    /// Con Ctrl+F se pinta el campo de lo que se busca y nada más.
    #[test]
    fn ctrl_f_muestra_el_campo_de_consulta() {
        let mut app = App::new();
        let context = egui::Context::default();
        app.state_mut().abrir_busqueda(false);

        let barra = barra(&context, &mut app, &[]);

        assert_eq!(
            barra.len(),
            2,
            "el campo de lo que se busca y el de cerrar, y nada más: {barra:?}"
        );
    }

    /// Con Ctrl+H se pintan los dos campos y el botón de reemplazar.
    #[test]
    fn ctrl_h_muestra_tambien_el_campo_de_reemplazo() {
        let mut app = App::new();
        let context = egui::Context::default();
        app.state_mut().abrir_busqueda(true);

        let barra = barra(&context, &mut app, &[]);

        assert_eq!(
            barra.len(),
            4,
            "consulta, reemplazo, reemplazar y cerrar: {barra:?}"
        );
    }

    /// Escribir en el campo de lo que se busca pide buscar al core.
    ///
    /// Se escribe con un clic antes porque un campo de texto solo recibe lo que se escribe
    /// si tiene el foco, y el foco se lo da el clic: es lo mismo que le pasa al editor.
    #[test]
    fn escribir_la_consulta_pide_buscar_al_core() {
        let mut app = App::new();
        let context = egui::Context::default();
        app.state_mut().abrir_busqueda(false);
        let campo = elemento(&context, &mut app, 0).center();
        barra(&context, &mut app, &clic(campo));

        barra(&context, &mut app, &escribir("form", campo));

        assert_eq!(
            app.state().busqueda().map(|b| b.consulta().to_owned()),
            Some("form".to_owned())
        );
        assert_eq!(
            app.peticiones(),
            vec![Command::Find],
            "escribir la consulta la pide al core, y no en cada frame"
        );
    }

    /// Escribir el texto de reemplazo no pide reemplazar nada.
    ///
    /// Su propio test porque es lo contrario de lo que hace el campo de consulta: escribir
    /// lo que se busca busca, y escribir con qué se reemplaza no reemplaza. Si pidiera, cada
    /// tecla de un texto de reemplazo borraría el documento.
    #[test]
    fn escribir_el_reemplazo_no_pide_reemplazar() {
        let mut app = App::new();
        let context = egui::Context::default();
        app.state_mut().abrir_busqueda(true);
        let campo = elemento(&context, &mut app, 1).center();
        barra(&context, &mut app, &clic(campo));

        barra(&context, &mut app, &escribir("dos", campo));

        assert_eq!(
            app.state()
                .busqueda()
                .and_then(|b| b.reemplazo().map(str::to_owned)),
            Some("dos".to_owned())
        );
        assert_eq!(app.peticiones(), Vec::<Command>::new());
    }

    /// El botón de reemplazar pide reemplazar al core.
    #[test]
    fn el_boton_de_reemplazar_pide_reemplazar_al_core() {
        let mut app = App::new();
        let context = egui::Context::default();
        app.state_mut().abrir_busqueda(true);
        let boton = elemento(&context, &mut app, 2).center();

        barra(&context, &mut app, &clic(boton));

        assert_eq!(app.peticiones(), vec![Command::Replace]);
    }

    /// El × cierra la búsqueda y la deja de ver.
    #[test]
    fn cerrar_la_busqueda_la_esconde() {
        let mut app = App::new();
        let context = egui::Context::default();
        app.state_mut().abrir_busqueda(true);
        let cerrar = elemento(&context, &mut app, 3).center();

        barra(&context, &mut app, &clic(cerrar));

        assert!(!app.state().esta_abierta_la_busqueda());
        assert!(barra(&context, &mut app, &[]).is_empty());
    }

    /// Abrir la búsqueda otra vez no borra lo que se estaba buscando.
    ///
    /// Quien abre la búsqueda con Ctrl+F para cambiar el término no quiere perder el
    /// anterior, y borrándoselo se lo pierde sin avisar.
    #[test]
    fn abrir_la_busqueda_otra_vez_no_borra_la_consulta() {
        let mut app = App::new();
        let context = egui::Context::default();
        app.state_mut().abrir_busqueda(false);
        let campo = elemento(&context, &mut app, 0).center();
        barra(&context, &mut app, &clic(campo));
        barra(&context, &mut app, &escribir("form", campo));

        app.state_mut().abrir_busqueda(false);

        assert_eq!(
            app.state().busqueda().map(|b| b.consulta().to_owned()),
            Some("form".to_owned())
        );
    }
}
