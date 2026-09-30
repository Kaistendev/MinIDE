//! La barra de estado: el pie de la ventana.
//!
//! Es la única zona de la ventana que no enseña el contenido de MiniIDE sino lo que
//! MiniIDE está haciendo: qué proyecto hay abierto, qué documento se está viendo, en qué
//! está la compilación y la ejecución, y qué ha pasado. Vive en el pie porque es lo único
//! que tiene que verse sin apartar la vista de lo que se está haciendo.
//!
//! Los campos de compilación y ejecución (FE-038 y FE-039) salen de lo que está haciendo
//! MiniIDE y no del estado visual, porque son dos hechos —si hay una compilación en marcha y
//! si hay un proceso vivo— que no se pueden escribir a mano en un texto: un estado que
//! alguien pone y nadie cambia se queda diciendo "compilando" cuando la compilación ya ha
//! terminado. Van en sus propios campos y no dentro del estado genérico porque compilar y
//! ejecutar pueden estar pasando a la vez, y un solo campo de texto tendría que elegir a
//! cuál de los dos le está enseñando algo al usuario.
//!
//! Los campos del proyecto y del documento son datos del core: el proyecto activo lo sabe
//! `Workspace` y el documento activo es T-097. La interfaz no tiene todavía forma de
//! preguntarles, así que esos dos campos dicen que no hay nada, que es verdad: MiniIDE
//! todavía no abre nada. Cuando la interfaz sepa abrirlos (FE-058, FE-062), se llenan desde
//! ahí y no desde aquí.

use eframe::egui;

use super::operaciones::Operaciones;
use super::ui_state::UiState;

/// Lo que dice el estado cuando MiniIDE está esperando y no ha pasado nada.
const LISTO: &str = "Listo";

/// Lo que dice el proyecto mientras no haya ninguno abierto.
const SIN_PROYECTO: &str = "Sin proyecto";

/// Lo que dice el documento mientras no haya ninguno abierto.
const SIN_DOCUMENTO: &str = "Sin documento";

/// Dibuja la barra de estado en `ui`, con los campos de la ventana.
///
/// Los campos van en fila y en Weak porque es texto de consulta, no el contenido: se lee
/// de reojo mientras se trabaja y no tiene que competir con el editor. Con el mismo
/// criterio, el estado va el último: es el campo que más cambia, y al final es donde el
/// ojo vuelve cuando algo pasa.
pub fn barra(ui: &mut egui::Ui, estado: &UiState, operaciones: &Operaciones) {
    ui.horizontal(|ui| {
        for (nombre, valor) in campos(estado, operaciones) {
            ui.weak(format!("{nombre}: {valor}"));
        }
    });
}

/// Los campos de la barra: qué se llama cada uno y qué dice.
///
/// Los de compilación y ejecución son los que se están moviendo, así que van antes del
/// estado genérico: son los que el usuario mira para saber si puede tocar algo o tiene que
/// esperar. El del proyecto y el del documento no los puede poner todavía la ventana, así
/// que dicen que no hay: es mejor eso que un nombre que se calcula aquí, porque un nombre
/// inventado en la barra hace creer que hay un proyecto abierto.
fn campos(estado: &UiState, operaciones: &Operaciones) -> [(&'static str, String); 5] {
    [
        ("Proyecto", SIN_PROYECTO.to_owned()),
        ("Documento", SIN_DOCUMENTO.to_owned()),
        (
            "Compilación",
            operaciones.estado_de_build().as_str().to_owned(),
        ),
        ("Ejecución", operaciones.estado_de_run().as_str().to_owned()),
        ("Estado", estado.status().unwrap_or(LISTO).to_owned()),
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

    /// Dibuja la ventana una vez y deja quietas las texturas que ha creado.
    ///
    /// Se dibuja la ventana entera y no solo la barra porque lo que se quiere comprobar es
    /// que la barra está en la ventana y no en un sitio aparte: la zona del pie la decide el
    /// layout, no esta función.
    fn dibujar(contexto: &egui::Context, app: &mut App) {
        let mut salida = contexto.run_ui(entrada(), |ui| {
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

    /// La barra tiene los campos que RF-13 pide poder ver, más los de FE-038 y FE-039.
    ///
    /// RF-13 es la salida de los procesos y el estado de la ventana, y una barra de estado
    /// que no dice qué proyecto, qué documento y qué está pasando no sirve para nada: es
    /// una franja decorada. Los campos tienen que estar aunque todavía no haya nada que
    /// enseñar en ellos, porque un campo que solo aparece cuando hay algo esconde justo lo
    /// que hace falta saber cuando no hay nada.
    ///
    /// Son cinco y no tres porque compilar y ejecutar tienen su propio campo cada uno, y no
    /// caben en el del estado genérico: pueden estar pasando a la vez, y un solo texto
    /// tendría que elegir a cuál de los dos le enseña algo al usuario.
    #[test]
    fn la_barra_tiene_los_campos_de_rf_13() {
        let campos = campos(&UiState::new(), &Operaciones::new());
        let nombres: Vec<&str> = campos.iter().map(|(nombre, _)| *nombre).collect();

        assert_eq!(
            nombres,
            vec![
                "Proyecto",
                "Documento",
                "Compilación",
                "Ejecución",
                "Estado"
            ],
            "la barra tiene que decir el proyecto, el documento, los dos estados y el \
             estado de operación: {campos:?}"
        );

        for (nombre, valor) in &campos {
            assert!(
                !valor.is_empty(),
                "el campo {nombre:?} no dice nada y parece que no está: {campos:?}"
            );
        }
    }

    /// La barra dice que no hay proyecto y que no hay documento.
    ///
    /// Es la verdad de ahora mismo: MiniIDE todavía no abre nada, y una barra que dijera un
    /// nombre inventado mentiría sobre lo que está abierto. Cuando haya algo abierto, aquí
    /// se pone lo que diga el core.
    #[test]
    fn la_barra_no_inventa_un_proyecto_ni_un_documento() {
        let campos = campos(&UiState::new(), &Operaciones::new());

        assert_eq!(
            (campos[0].0, campos[0].1.as_str()),
            ("Proyecto", SIN_PROYECTO),
            "sin proyecto abierto la barra tiene que decirlo: {campos:?}"
        );
        assert_eq!(
            (campos[1].0, campos[1].1.as_str()),
            ("Documento", SIN_DOCUMENTO),
            "sin documento abierto la barra tiene que decirlo: {campos:?}"
        );
    }

    /// La barra enseña el estado que tienen las operaciones, y no otro. FE-038 y FE-039.
    ///
    /// Es lo que las dos tareas piden: lo que MiniIDE está haciendo tiene que verse en la
    /// ventana. Se compara el texto de la barra con el del módulo de operaciones porque lo
    /// que cambia entre un estado y otro es el texto, y el texto es lo que el usuario lee;
    /// que además llegue a pintarse lo comprueba `la_barra_se_dibuja_en_el_pie_de_la_ventana`,
    /// y que los estados cambien de verdad, `operaciones`.
    ///
    /// Con una ventana quieta solo se puede comprobar el estado de reposo. Los estados con
    /// trabajo en marcha se comprueban desde la aplicación, que es quien los pone.
    #[test]
    fn la_barra_ensena_el_estado_que_tienen_las_operaciones() {
        let operaciones = Operaciones::new();

        let campos = campos(&UiState::new(), &operaciones);

        assert_eq!(
            (campos[2].0, campos[2].1.as_str()),
            ("Compilación", operaciones.estado_de_build().as_str()),
            "la barra tiene que decir en qué está la compilación: {campos:?}"
        );
        assert_eq!(
            (campos[3].0, campos[3].1.as_str()),
            ("Ejecución", operaciones.estado_de_run().as_str()),
            "la barra tiene que decir en qué está la ejecución: {campos:?}"
        );
    }

    /// La barra refleja lo que está pasando, y dice que no está pasando nada cuando no pasa
    /// nada.
    ///
    /// El estado de operación es el campo que se mueve: lo pone el estado visual cuando
    /// alguien compila, guarda o abre algo. Y cuando no hay nada que decir, la barra no se
    /// queda en blanco, porque un pie en blanco parece una ventana que no ha terminado de
    /// dibujarse.
    #[test]
    fn la_barra_refleja_el_estado_de_operacion() {
        let operaciones = Operaciones::new();
        let mut estado = UiState::new();

        assert_eq!(
            campos(&estado, &operaciones)[4],
            ("Estado", LISTO.to_owned()),
            "una ventana quieta tiene que decir que está quieta, no nada"
        );

        estado.set_status("Compilando...");
        assert_eq!(
            campos(&estado, &operaciones)[4],
            ("Estado", "Compilando...".to_owned()),
            "la barra tiene que decir lo que está pasando"
        );

        estado.set_status("1 error");
        assert_eq!(
            campos(&estado, &operaciones)[4],
            ("Estado", "1 error".to_owned()),
            "el estado de la barra cambia con lo que pasa"
        );

        estado.clear_status();
        assert_eq!(
            campos(&estado, &operaciones)[4],
            ("Estado", LISTO.to_owned()),
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
        let contexto = egui::Context::default();
        let mut app = App::new();

        dibujar(&contexto, &mut app);

        let salida = contexto.run_ui(entrada(), |ui| {
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

    /// La aplicación muestra la barra con su estado visual y con lo que está haciendo.
    ///
    /// La barra recibe el estado de la aplicación y no uno suyo, y ese estado es el mismo que
    /// se ve desde cualquier otra parte de la ventana: si la barra tuviera el suyo, lo que
    /// dijera la barra y lo que dijera el resto serían dos verdades.
    #[test]
    fn la_aplicacion_muestra_la_barra_con_su_estado() {
        let contexto = egui::Context::default();
        let mut app = App::new();
        app.state_mut().set_status("Compilando...");

        let mut salida = contexto.run_ui(entrada(), |ui| {
            app.dibujar(ui);
        });
        salida.textures_delta.clear();

        assert_eq!(
            campos(app.state(), app.operaciones())[4],
            ("Estado", "Compilando...".to_owned()),
            "la barra de la aplicación tiene que decir lo que dice su estado visual"
        );
    }

    /// La barra no saca nada del core.
    ///
    /// El proyecto y el documento son del core, y la barra no puede quedarse con una copia
    /// de su nombre para enseñarla sola: la copia se quedaría vieja sin que nadie se enterase,
    /// y es el error que RNF-08 prohíbe. La barra muestra lo que la interfaz sabe —el estado
    /// visual y lo que está haciendo— y lo que el core le diga, y por eso no lo llama.
    ///
    /// Solo se mira el código y no los tests, porque el test sí llega a la barra: mira que
    /// se dibuja, y para eso tiene que llegar a ella por el camino de siempre.
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
            "la barra no puede importar el core: lo que sepa de un proyecto o de un \
             documento se lo dira el core, no una copia que guarde aqui: {uses:?}"
        );
    }
}
