//! La barra de estado: el pie de la ventana.
//!
//! Es la única zona de la ventana que no enseña el contenido de MiniIDE sino lo que
//! MiniIDE está haciendo: qué proyecto hay abierto, qué documento se está viendo y qué ha
//! pasado. Vive en el pie porque es lo único que tiene que verse sin apartar la vista de
//! lo que se está haciendo.
//!
//! Solo hay un campo vivo ahora, el del estado de operación, y sale del estado visual
//! (FE-004). Los otros dos —el proyecto y el documento— son datos del core: el proyecto
//! activo lo sabe `Workspace` y el documento activo es T-097, que todavía no existe. La
//! interfaz no tiene un core al que preguntarle, así que esos dos campos dicen que no hay
//! nada, que es verdad: MiniIDE todavía no abre nada. Cuando la interfaz tenga core
//! (FE-009) y se puedan abrir proyectos y documentos (FE-058, FE-062), se llenan desde ahí
//! y no desde aquí.

use eframe::egui;

use super::ui_state::UiState;

/// Lo que dice el estado cuando MiniIDE está esperando y no ha pasado nada.
const LISTO: &str = "Listo";

/// Lo que dice el proyecto mientras no haya ninguno abierto.
const SIN_PROYECTO: &str = "Sin proyecto";

/// Lo que dice el documento mientras no haya ninguno abierto.
const SIN_DOCUMENTO: &str = "Sin documento";

/// Dibuja la barra de estado en `ui`, con los tres campos de la ventana.
///
/// Los campos van en fila y en Weak porque es texto de consulta, no el contenido: se lee
/// de reojo mientras se trabaja y no tiene que competir con el editor. Con el mismo
/// criterio, el estado va el último: es el campo que más cambia, y al final es donde el
/// ojo vuelve cuando algo pasa.
pub fn barra(ui: &mut egui::Ui, estado: &UiState) {
    ui.horizontal(|ui| {
        for (nombre, valor) in campos(estado) {
            ui.weak(format!("{nombre}: {valor}"));
        }
    });
}

/// Los tres campos de la barra: qué se llama cada uno y qué dice.
///
/// El del estado sale del estado visual, que es lo único que la interfaz sabe de lo que está
/// pasando. Los del proyecto y del documento son los del core y todavía no se pueden
/// preguntar, así que dicen que no hay: es mejor eso que un nombre que seCalcula aquí,
/// porque un nombre inventado en la barra hace creer que hay un proyecto abierto.
fn campos(estado: &UiState) -> [(&'static str, &str); 3] {
    [
        ("Proyecto", SIN_PROYECTO),
        ("Documento", SIN_DOCUMENTO),
        ("Estado", estado.status().unwrap_or(LISTO)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::App;
    use eframe::egui;

    /// Una ventana de trabajo normal, ni enorme ni mínima.
    const ANCHO: f32 = 1000.0;
    const ALTO: f32 = 800.0;

    /// Dibuja la ventana una vez y devuelve lo que se ha pintado.
    ///
    /// Se dibuja la ventana entera y no solo la barra porque lo que se quiere comprobar es
    /// que la barra está en la ventana y no en un sitio aparte: la zona del pie la decide
    /// el layout, no esta función.
    fn dibujar(context: &egui::Context, app: &mut App) {
        let mut salida = context.run_ui(entrada(), |ui| {
            app.dibujar(ui);
        });
        salida.textures_delta.clear();
    }

    /// Un frame con una ventana de tamaño conocido.
    fn entrada() -> eframe::egui::RawInput {
        eframe::egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(ANCHO, ALTO),
            )),
            ..eframe::egui::RawInput::default()
        }
    }

    /// La barra de estado tiene los tres campos que RF-13 pide poder ver.
    ///
    /// RF-13 es la salida de los procesos y el estado de la ventana, y una barra de estado
    /// que no dice qué proyecto, qué documento y qué está pasando no sirve para nada: es
    /// una franja decorada. Los tres campos tienen que estar aunque todavía no haya nada
    /// que enseñar en ellos, porque un campo que solo aparece cuando hay algo esconde
    /// justo lo que hace falta saber cuando no hay nada.
    #[test]
    fn la_barra_tiene_los_tres_campos_de_rf_13() {
        let estado = UiState::new();

        let campos = campos(&estado);
        let nombres: Vec<&str> = campos.iter().map(|(nombre, _)| *nombre).collect();

        assert_eq!(
            nombres,
            vec!["Proyecto", "Documento", "Estado"],
            "la barra tiene que decir el proyecto, el documento y el estado: {campos:?}"
        );

        for (_, valor) in campos {
            assert!(
                !valor.is_empty(),
                "un campo que no dice nada parece que no está: {campos:?}"
            );
        }
    }

    /// La barra dice que no hay proyecto y que no hay documento.
    ///
    /// Es la verdad de ahora mismo: MiniIDE todavía no abre nada, y una barra que
    /// dijera un nombre inventado Mentiría sobre lo que está abierto. Cuando haya algo
    /// abierto, aquí se pone lo que diga el core.
    #[test]
    fn la_barra_no_inventa_un_proyecto_ni_un_documento() {
        let estado = UiState::new();
        let campos = campos(&estado);

        assert_eq!(
            campos[0],
            ("Proyecto", SIN_PROYECTO),
            "sin proyecto abierto la barra tiene que decirlo: {campos:?}"
        );
        assert_eq!(
            campos[1],
            ("Documento", SIN_DOCUMENTO),
            "sin documento abierto la barra tiene que decirlo: {campos:?}"
        );
    }

    /// La barra refleja lo que está pasando, y dice que no está pasando nada cuando no
    /// pasa nada.
    ///
    /// El estado de operación es el campo que se mueve: lo pone el estado visual cuando
    /// alguien compila, guarda o abre algo. Y cuando no hay nada que decir, la barra no se
    /// queda en blanco, porque un pie en blanco parece una ventana que no ha terminado de
    /// dibujarse.
    #[test]
    fn la_barra_refleja_el_estado_de_operacion() {
        let mut estado = UiState::new();
        assert_eq!(
            campos(&estado)[2],
            ("Estado", LISTO),
            "una ventana quieta tiene que decir que está quieta, no nada"
        );

        estado.set_status("Compilando...");
        assert_eq!(
            campos(&estado)[2],
            ("Estado", "Compilando..."),
            "la barra tiene que decir lo que está pasando"
        );

        estado.set_status("1 error");
        assert_eq!(
            campos(&estado)[2],
            ("Estado", "1 error"),
            "el estado de la barra cambia con lo que pasa"
        );

        estado.clear_status();
        assert_eq!(
            campos(&estado)[2],
            ("Estado", LISTO),
            "sin nada que decir la barra vuelve a decir que está quieta"
        );
    }

    /// La barra se dibuja en la ventana, en el pie.
    ///
    /// Los campos pueden estar bien escritos y aun así no verse: si la función no llega a
    /// dibujarse, la barra es una zona vacía. Se dibuja la ventana entera y se mira que la
    /// franja de abajo siga ahí, que es la zona que el layout reserva para la barra.
    #[test]
    fn la_barra_se_dibuja_en_el_pie_de_la_ventana() {
        let context = egui::Context::default();
        let mut app = App::new();

        dibujar(&context, &mut app);

        let salida = context.run_ui(entrada(), |ui| {
            app.dibujar(ui);
        });

        let mut zonas: Vec<egui::Rect> = salida
            .shapes
            .into_iter()
            .filter_map(|forma| match forma.shape {
                egui::Shape::Rect(rectangulo) => Some(rectangulo.rect),
                _ => None,
            })
            .filter(|rectangulo| rectangulo.width() >= ANCHO)
            .collect();
        zonas.sort_by(|uno, otro| uno.min.y.total_cmp(&otro.min.y));

        let pie = zonas.last().copied().expect("la ventana tiene zonas");
        assert!(
            (ALTO - pie.max.y).abs() <= 2.0 && 0.0 < pie.height() && pie.height() < ALTO / 4.0,
            "la barra de estado es la franja de abajo y es una franja, no media ventana: {pie:?}"
        );
    }

    /// La aplicación muestra la barra con su estado visual.
    ///
    /// La barra recibe el estado de la aplicación y no uno suyo, y ese estado es el mismo
    /// que se ve desde cualquier otra parte de la ventana: si la barra tuviera el suyo,
    /// escribir en la barra y en el resto de la ventana serían dos verdades.
    #[test]
    fn la_aplicacion_muestra_la_barra_con_su_estado() {
        let context = egui::Context::default();
        let mut app = App::new();
        app.state_mut().set_status("Compilando...");

        let mut salida = context.run_ui(entrada(), |ui| {
            app.dibujar(ui);
        });
        salida.textures_delta.clear();

        assert_eq!(
            campos(app.state())[2],
            ("Estado", "Compilando..."),
            "la barra de la aplicación tiene que decir lo que dice su estado visual"
        );
    }

    /// La barra no saca nada del core.
    ///
    /// El proyecto y el documento son del core, y la barra no puede quedarse con una copia
    /// de su nombre para teachlo sola: la copia se quedaría vieja sin que nadie se enterase,
    /// y es el error que RNF-08 prohíbe. La barra muestra lo que la interfaz sabe —el
    /// estado visual— y lo que el core le diga, y por eso no lo llama.
    ///
    /// Solo se mira el código y no los tests, porque el test sí llama a la ventana: mira
    /// que la barra se dibuja, y para eso tiene que llegar a ella por el camino de
    /// siempre.
    #[test]
    fn la_barra_no_saca_nada_del_core() {
        let source =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file!()))
                .expect("el fuente de la barra tiene que poder leerse");

        let codigo = source
            .split("#[cfg(test)]")
            .next()
            .expect("el fuente de la barra tiene que tener código antes de los tests");

        let uses: Vec<&str> = codigo
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("use crate::"))
            .collect();

        assert!(
            uses.is_empty(),
            "la barra no puede importar el core: lo que sepa de un proyecto o de un documento se lo dira el core, no una copia que guarde aqui: {uses:?}"
        );
    }
}
