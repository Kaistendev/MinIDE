//! La lista de diagnósticos: lo que el core ha encontrado en el proyecto.
//!
//! Cada diagnóstico se ve con su gravedad, su archivo y su línea si los tiene, y su
//! mensaje. Pulsar uno con sitio pide abrir ese archivo, que es lo que se puede pedir
//! desde aquí: la línea va en la petición, que es de la de abrir un documento.
//!
//! Los diagnósticos son del core y aquí no se tocan: se muestran tal cual, con su
//! gravedad y su posición. Lo único que se hace es decidir si se pueden pulsar, que es si
//! tienen sitio al que ir.

use crate::commands::Command;
use crate::diagnostics::Diagnostic;
use eframe::egui;

use super::app::App;

/// Dibuja la lista de diagnósticos, y la deja pulsable si tiene dónde ir.
///
/// Sin diagnósticos no se dibuja nada: una lista vacía con un borde es un hueco, y en la
/// parte de abajo de la ventana un hueco parece que falta algo.
pub fn panel(ui: &mut egui::Ui, diagnosticos: &[Diagnostic], app: &mut App) {
    if diagnosticos.is_empty() {
        return;
    }

    for diagnostico in diagnosticos {
        fila(ui, diagnostico, app);
    }
}

/// Dibuja un diagnóstico.
///
/// Se dibuja entero como un botón y no con un botón y un texto al lado porque un
/// diagnóstico se pulsa entero: pedir la navegación por pulsar en el mensaje y no por
/// pulsar en la línea sería fallar en el sitio donde se mira.
fn fila(ui: &mut egui::Ui, diagnostico: &Diagnostic, app: &mut App) {
    let texto = texto_de(diagnostico);
    let tiene_sitio = diagnostico.location().is_some();
    let respuesta = if tiene_sitio {
        ui.button(texto)
    } else {
        ui.add_enabled(false, egui::Button::new(texto))
    };

    if respuesta.clicked() {
        app.emitir(Command::OpenDocument);
    }
}

/// Lo que se ve de un diagnóstico: su gravedad, dónde está y qué dice.
///
/// El archivo y la línea delante del mensaje porque es lo que se busca de un vistazo: un
/// mensaje de error sin saber de dónde es no dice nada. Los que no tienen sitio se ven sin
/// archivo, que es lo que son: un problema del proyecto entero y no de un archivo.
pub fn texto_de(diagnostico: &Diagnostic) -> String {
    let donde = match diagnostico.location() {
        Some(sitio) => format!(
            "{}:{} ",
            sitio.file().as_path().display(),
            sitio.position().line() + 1
        ),
        None => String::new(),
    };

    format!(
        "{} {}{}",
        diagnostico.level().as_str(),
        donde,
        diagnostico.message()
    )
}

#[cfg(test)]
mod tests {
    use crate::commands::Command;
    use crate::diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticLocation};
    use crate::document::TextPosition;
    use crate::frontend::App;
    use crate::project::ProjectRelativePath;
    use eframe::egui;

    /// Una ventana de trabajo normal, ni enorme ni mínima.
    const ANCHO: f32 = 1000.0;
    const ALTO: f32 = 800.0;

    /// El hueco donde se dibuja la lista, que es la parte de abajo de la ventana.
    const ALTO_DE_LA_LISTA: f32 = 200.0;

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

    /// Dibuja la lista y devuelve los rectángulos que ha pintado y lo que hay escrito.
    fn pintado(
        context: &egui::Context,
        app: &mut App,
        diagnosticos: &[Diagnostic],
        eventos: &[egui::Event],
    ) -> (Vec<egui::Rect>, Vec<String>) {
        let mut salida = context.run_ui(entrada_con(eventos), |ui| {
            ui.allocate_ui(egui::vec2(ANCHO, ALTO_DE_LA_LISTA), |ui| {
                super::panel(ui, diagnosticos, app);
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
        rectangulos.sort_by(|uno, otro| {
            uno.min
                .y
                .total_cmp(&otro.min.y)
                .then(uno.min.x.total_cmp(&otro.min.x))
        });
        rectangulos.dedup();

        let textos: Vec<String> = salida
            .shapes
            .iter()
            .filter_map(|forma| match &forma.shape {
                egui::Shape::Text(texto) => Some(texto.galley.job.text.to_string()),
                _ => None,
            })
            .collect();

        (rectangulos, textos)
    }

    fn ruta(valor: &str) -> ProjectRelativePath {
        ProjectRelativePath::new(valor).expect("ruta de prueba valida")
    }

    /// Un diagnóstico con archivo y línea, que es el caso normal.
    fn con_sitio(mensaje: &str) -> Diagnostic {
        Diagnostic::new(
            DiagnosticLevel::Error,
            mensaje.to_owned(),
            Some(DiagnosticLocation::new(
                ruta("src/Form1.cs"),
                TextPosition::new(11, 0),
            )),
        )
    }

    /// Un diagnóstico sin archivo, que es un problema del proyecto entero.
    fn sin_sitio(mensaje: &str) -> Diagnostic {
        Diagnostic::new(DiagnosticLevel::Warning, mensaje.to_owned(), None)
    }

    /// Un clic de verdad: la pulsación y la soltura, y el puntero donde se ha pulsado.
    ///
    /// Van en dos listas para pulsarlos en dos frames, porque egui solo cuenta un clic si
    /// la pulsación y la soltura caen en dos frames distintos del widget.
    fn clic(posicion: egui::Pos2) -> (Vec<egui::Event>, Vec<egui::Event>) {
        let pulsar = |pressed| {
            vec![
                egui::Event::PointerButton {
                    pos: posicion,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::default(),
                },
                egui::Event::PointerMoved(posicion),
            ]
        };

        (pulsar(true), pulsar(false))
    }

    /// Sin diagnósticos no se dibuja nada.
    #[test]
    fn sin_diagnosticos_no_se_dibuja_nada() {
        let mut app = App::new();
        let context = egui::Context::default();

        let (rectangulos, textos) = pintado(&context, &mut app, &[], &[]);

        assert!(
            rectangulos.is_empty() && textos.is_empty(),
            "una lista vacía es un hueco: {rectangulos:?} y {textos:?}"
        );
    }

    /// Un diagnóstico sale con su archivo, su línea y su mensaje.
    ///
    /// Con los tres porque de los tres se sirve el que mira la lista: qué pasa, dónde y en
    /// qué línea. La línea se enseña desde la primera -en un archivo la primera línea es la
    /// uno- y por eso el uno se suma a la del core, que cuenta desde cero.
    #[test]
    fn un_diagnostico_con_archivo_y_linea_aparece_en_la_lista() {
        let mut app = App::new();
        let context = egui::Context::default();

        let (_, textos) = pintado(
            &context,
            &mut app,
            &[con_sitio("no se encuentra Form")],
            &[],
        );

        let escrito = textos.join(" ");
        assert!(escrito.contains("no se encuentra Form"), "{escrito}");
        assert!(
            escrito.contains("src/Form1.cs"),
            "el archivo sale: {escrito}"
        );
        assert!(
            escrito.contains(":12"),
            "la línea sale contada desde uno: {escrito}"
        );
    }

    /// Un diagnóstico sin sitio sale sin archivo, y no se puede pulsar.
    ///
    /// Sin archivo porque es lo que es: un problema del proyecto y no de un archivo. Y no se
    /// pulsa porque no hay a dónde ir con él, y un botón que acepta el clic y no hace nada
    /// parece un fallo.
    #[test]
    fn un_diagnostico_sin_sitio_sale_sin_archivo_y_no_se_pulsa() {
        let mut app = App::new();
        let context = egui::Context::default();
        let diagnosticos = [sin_sitio("falta el SDK")];

        let (_, textos) = pintado(&context, &mut app, &diagnosticos, &[]);
        let escrito = textos.join(" ");

        assert!(escrito.contains("falta el SDK"), "{escrito}");
        assert!(
            !escrito.contains(':'),
            "sin archivo, sin dos puntos: {escrito}"
        );

        let (rectangulos, _) = pintado(&context, &mut app, &diagnosticos, &[]);
        if let Some(fila) = rectangulos.first() {
            let (pulsar, soltar) = clic(fila.center());
            pintado(&context, &mut app, &diagnosticos, &pulsar);
            pintado(&context, &mut app, &diagnosticos, &soltar);
        }

        assert_eq!(app.peticiones(), Vec::<Command>::new());
    }

    /// Pulsar un diagnóstico con sitio pide abrir el archivo donde está.
    ///
    /// Se comprueba que pide exactamente uno porque es la navegación entera: si pidiera dos
    /// veces, el documento se abriría dos veces.
    #[test]
    fn pulsar_un_diagnostico_con_ubicacion_pide_abrir_su_archivo() {
        let mut app = App::new();
        let context = egui::Context::default();
        let diagnosticos = [con_sitio("no se encuentra Form")];
        let (rectangulos, _) = pintado(&context, &mut app, &diagnosticos, &[]);
        let fila = rectangulos
            .first()
            .copied()
            .expect("el diagnóstico se pinta");

        let (pulsar, soltar) = clic(fila.center());
        pintado(&context, &mut app, &diagnosticos, &pulsar);
        pintado(&context, &mut app, &diagnosticos, &soltar);

        assert_eq!(
            app.peticiones(),
            vec![Command::OpenDocument],
            "pulsar un diagnóstico pide abrir el archivo donde está"
        );
    }

    /// Pulsar la lista sin tocar en ella no pide nada.
    ///
    /// Su propio test porque es el otro sentido del mismo pulsable: si pulsara en cualquier
    /// sitio de la lista, abriría el archivo de cualquier diagnóstico.
    #[test]
    fn pulsar_sin_tocar_un_diagnostico_no_pide_nada() {
        let mut app = App::new();
        let context = egui::Context::default();
        let diagnosticos = [con_sitio("no se encuentra Form")];

        let (pulsar, soltar) = clic(egui::pos2(ANCHO - 2.0, ALTO - 2.0));
        pintado(&context, &mut app, &diagnosticos, &pulsar);
        pintado(&context, &mut app, &diagnosticos, &soltar);

        assert_eq!(app.peticiones(), Vec::<Command>::new());
    }

    /// Los diagnósticos salen en el orden en que el core los dio.
    ///
    /// El core los agrupa por gravedad y ese orden es el que ha decided él; aquí no se
    /// reordenan, porque el primero de la lista es el error que más importa y si se
    /// reordenara por gusto dejaría de serlo.
    #[test]
    fn los_diagnosticos_salen_en_el_orden_del_core() {
        let mut app = App::new();
        let context = egui::Context::default();
        let diagnosticos = [
            con_sitio("primero"),
            sin_sitio("segundo"),
            con_sitio("tercero"),
        ];

        let (_, textos) = pintado(&context, &mut app, &diagnosticos, &[]);

        let primero = textos
            .iter()
            .position(|t| t.contains("primero"))
            .expect("primero");
        let segundo = textos
            .iter()
            .position(|t| t.contains("segundo"))
            .expect("segundo");
        let tercero = textos
            .iter()
            .position(|t| t.contains("tercero"))
            .expect("tercero");
        assert!(primero < segundo && segundo < tercero, "{textos:?}");
    }
}
