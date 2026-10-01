//! La lista de diagnósticos: lo que el core ha encontrado en el proyecto.
//!
//! Cada diagnóstico se ve con su gravedad, su archivo y su línea si los tiene, y su
//! mensaje. Pulsar uno con sitio pide abrir ese archivo y lo deja marcado en el explorador,
//! que es lo que se puede pedir desde aquí: un comando no lleva datos, así que el archivo al
//! que se va se dice marcando su fila, y la línea a la que salta la navegación es de FE-072.
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
///
/// Pulsar uno con sitio pide abrir el archivo que señala, lo deja seleccionado en el
/// explorador con el mismo nombre de fila que usa allí (FE-060) y anota a qué línea de ese
/// archivo hay que ir (FE-072). Son las tres cosas porque el comando no lleva datos —es un
/// nombre de operación, no un destino— y es quien lo ejecuta quien mira qué fila está marcada
/// y a qué línea se va: si solo se pidiera abrir el archivo sin marcarlo, el explorador
/// seguiría enseñando como seleccionado otro archivo y el usuario no vería a dónde ha ido; y
/// si no se anotara la línea, el cursor se quedaría al principio del archivo.
fn fila(ui: &mut egui::Ui, diagnostico: &Diagnostic, app: &mut App) {
    let texto = texto_de(diagnostico);
    let Some(sitio) = diagnostico.location() else {
        ui.add_enabled(false, egui::Button::new(texto));

        return;
    };

    if !ui.button(texto).clicked() {
        return;
    }

    let clave = super::explorador::clave(sitio.file().as_path());
    app.state_mut().seleccionar(&clave);
    app.state_mut().ir_a(clave, sitio.position().line());
    app.emitir(Command::OpenDocument);
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

    /// Pulsar un diagnóstico deja marcado en el explorador el archivo que señala. FE-060.
    ///
    /// El comando que se pide es el mismo de siempre —abrir un documento— porque no lleva
    /// datos: es un nombre de operación y no un destino. Por eso el archivo al que hay que ir
    /// tiene que quedar marcado con la misma clave que el explorador usa para sus filas: si
    /// uno de los dos lo nombrara de otra manera, el explorador no reconocería la fila y el
    /// usuario vería que ha pulsado un error y que no pasa nada.
    #[test]
    fn pulsar_un_diagnostico_deja_marcado_el_archivo_que_senala() {
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
            app.state().seleccion(),
            Some("src/Form1.cs"),
            "el archivo del error es el que queda marcado en el explorador"
        );
        assert_eq!(
            app.peticiones(),
            vec![Command::OpenDocument],
            "y es el abrir ese documento lo que se pide"
        );
    }

    /// Pulsar un diagnóstico con ubicación anota a qué archivo y a qué línea hay que ir. FE-072.
    ///
    /// Es lo que convierte "pulsar un error" en "ver el error": el comando que se pide no
    /// lleva la línea, porque `Command::OpenDocument` es un nombre de operación y no un
    /// destino, así que la línea tiene que quedarse escrita en la ventana. Sin esto, abrir el
    /// archivo dejaría el cursor al principio y el usuario tendría que buscarse el error él.
    #[test]
    fn pulsar_un_diagnostico_con_ubicacion_anota_el_archivo_y_la_linea() {
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

        let destino = app.state().destino().expect("hay un destino pendiente");
        assert_eq!(destino.ruta(), "src/Form1.cs", "el destino es su archivo");
        assert_eq!(
            destino.linea(),
            11,
            "y su línea tal como la cuenta el core, desde cero"
        );
    }

    /// Pulsar un diagnóstico sin ubicación no anota ningún destino. FE-072.
    ///
    /// Un problema del proyecto entero no tiene archivo al que ir, así que no puede dejar un
    /// destino a medias: si lo dejaba, el siguiente documento que se abriera —el que fuera—
    /// llevaría el cursor a una línea de otro archivo que el usuario no había pedido.
    #[test]
    fn pulsar_un_diagnostico_sin_ubicacion_no_anota_destino() {
        let mut app = App::new();
        let context = egui::Context::default();
        let diagnosticos = [sin_sitio("falta el paquete NuGet")];

        // La fila se pinta pero no se puede pulsar, así que el destino solo se puede anotar
        // pulsándola. Sin fila pulsable no hay destino, y eso es lo que se comprueba.
        let (pulsar, soltar) = clic(egui::pos2(ANCHO / 2.0, ALTO_DE_LA_LISTA / 2.0));
        pintado(&context, &mut app, &diagnosticos, &pulsar);
        pintado(&context, &mut app, &diagnosticos, &soltar);

        assert_eq!(
            app.state().destino(),
            None,
            "un error sin archivo no puede decir a dónde ir"
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
