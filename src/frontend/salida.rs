//! La salida de los procesos: lo que un build o una ejecución han escrito.
//!
//! Son dos flujos y se ven como dos: lo normal y el error. No se juntan porque el error
//! es lo que se busca primero, y en una lista mezclada habría que leerlo entero para
//! encontrarlo.
//!
//! Lo que hay aquí son las líneas ya recibidas y su origen, que es de la ventana: el
//! proceso y sus flujos son del core, y aquí solo se enseña lo que llegó. Sin salida no
//! se dibuja nada, y lo que se dibuja es lo que se ve en la pantalla.

use eframe::egui;

/// De qué flujo viene una línea de la salida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origen {
    /// Salida normal del proceso.
    Normal,
    /// Error del proceso.
    Error,
}

/// Una línea de la salida, con el flujo de la que viene.
///
/// Lleva su origen y no solo su texto porque el color sale de ahí: una línea sin origen
/// no se puede pintar de una manera u otra, y el texto solo no lo dice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineaDeSalida {
    texto: String,
    origen: Origen,
}

impl LineaDeSalida {
    /// Una línea del flujo indicado.
    pub fn nueva(texto: impl Into<String>, origen: Origen) -> Self {
        Self {
            texto: texto.into(),
            origen,
        }
    }

    /// Lo que escribió el proceso en esta línea.
    pub fn texto(&self) -> &str {
        &self.texto
    }

    /// De qué flujo viene.
    pub fn origen(&self) -> Origen {
        self.origen
    }

    /// El color con el que se ve: el error se ve distinto para que se encuentre de un
    /// vistazo.
    pub fn color(&self) -> egui::Color32 {
        match self.origen() {
            Origen::Normal => COLOR_DE_LA_SALIDA,
            Origen::Error => COLOR_DEL_ERROR,
        }
    }
}

const COLOR_DE_LA_SALIDA: egui::Color32 = egui::Color32::from_rgb(0xd4, 0xd4, 0xd4);
const COLOR_DEL_ERROR: egui::Color32 = egui::Color32::from_rgb(0xe0, 0x6c, 0x75);

/// Las líneas de la salida de un proceso, con su flujo.
///
/// Se leen los dos flujos del core y se ponen uno detrás del otro, el normal primero, sin
/// mezclarlos: mezclados no se sabría de qué flujo viene cada línea y no se podría pintar
/// de dos colores. El core los tiene separados -`standard_output` y `standard_error`- y
/// aquí no se juntan.
pub fn lineas_de(salida: &crate::runtime::ProcessOutput) -> Vec<LineaDeSalida> {
    let mut lineas: Vec<LineaDeSalida> = salida
        .standard_output()
        .lines()
        .map(|linea| LineaDeSalida::nueva(linea, Origen::Normal))
        .collect();

    lineas.extend(
        salida
            .standard_error()
            .lines()
            .map(|linea| LineaDeSalida::nueva(linea, Origen::Error)),
    );

    lineas
}

/// Dibuja la salida de los procesos, línea por línea.
///
/// Sin líneas no se dibuja nada: un panel de salida vacío con un borde es un hueco, y en
/// la parte de abajo de la ventana un hueco parece que falta algo.
pub fn panel(ui: &mut egui::Ui, salida: &crate::runtime::ProcessOutput) {
    let lineas = lineas_de(salida);

    if lineas.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        for linea in lineas {
            ui.label(
                egui::RichText::new(linea.texto())
                    .monospace()
                    .color(linea.color()),
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::runtime::ProcessOutput;
    use eframe::egui;

    use super::{lineas_de, LineaDeSalida, Origen};

    /// Una ventana de trabajo normal, ni enorme ni mínima.
    const ANCHO: f32 = 1000.0;
    const ALTO: f32 = 800.0;

    /// El hueco donde se dibuja la salida, que es la parte de abajo de la ventana.
    const ALTO_DE_LA_SALIDA: f32 = 200.0;

    /// La salida de un proceso a la que pertenece lo que se ha dibujado.
    ///
    /// Se monta desde las líneas que hay probadas para poder dibujar la mezcla que se
    /// quiera, con lo normal y el error en el orden que se le dé, que es lo que no se
    /// puede hacer con la salida de un proceso de verdad y sí con una lista.
    fn salida_de(lineas: &[LineaDeSalida]) -> ProcessOutput {
        let de_un_flujo = |origen: Origen| {
            lineas
                .iter()
                .filter(|linea| linea.origen() == origen)
                .map(|linea| linea.texto())
                .collect::<Vec<_>>()
                .join("\n")
        };

        ProcessOutput::new(
            Some(0),
            de_un_flujo(Origen::Normal),
            de_un_flujo(Origen::Error),
        )
    }

    fn entrada() -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(ANCHO, ALTO),
            )),
            ..egui::RawInput::default()
        }
    }

    /// Dibuja la salida y devuelve lo que se ha escrito en ella, con su color.
    fn pintado(lineas: &[LineaDeSalida]) -> Vec<(String, egui::Color32)> {
        let mut salida = egui::Context::default().run_ui(entrada(), |ui| {
            ui.allocate_ui(egui::vec2(ANCHO, ALTO_DE_LA_SALIDA), |ui| {
                super::panel(ui, &salida_de(lineas));
            });
        });
        salida.textures_delta.clear();

        let mut pintados: Vec<(String, egui::Color32)> = salida
            .shapes
            .iter()
            .filter_map(|forma| match &forma.shape {
                egui::Shape::Text(texto) => {
                    let color = texto
                        .galley
                        .job
                        .sections
                        .first()
                        .map_or(egui::Color32::WHITE, |s| s.format.color);

                    Some((texto.galley.job.text.to_string(), color))
                }
                _ => None,
            })
            .collect();
        pintados.sort_by(|uno, otra| uno.0.cmp(&otra.0));

        pintados
    }

    /// Sin salida no se dibuja nada.
    #[test]
    fn sin_salida_no_se_dibuja_nada() {
        assert!(
            pintado(&[]).is_empty(),
            "un panel de salida vacío es un hueco, y un hueco parece que falta algo"
        );
    }

    /// Cada línea que se ha recibido sale en la salida, y en el orden en que llegaron.
    #[test]
    fn cada_linea_sale_se_ve() {
        let lineas = vec![
            LineaDeSalida::nueva("primera", Origen::Normal),
            LineaDeSalida::nueva("segunda", Origen::Normal),
        ];

        let pintados = pintado(&lineas);

        assert_eq!(
            pintados.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>(),
            vec!["primera", "segunda"],
            "las líneas salen tal cual y en orden"
        );
    }

    /// La salida normal y el error se ven distintos.
    ///
    /// Distintos por el color y no por la forma porque lo que se busca primero son los
    /// errores, y en una lista de un solo color habría que leerla entera para encontrarlos.
    #[test]
    fn la_salida_normal_y_el_error_se_ven_distintos() {
        let lineas = vec![
            LineaDeSalida::nueva("normal", Origen::Normal),
            LineaDeSalida::nueva("error", Origen::Error),
        ];

        let pintados = pintado(&lineas);

        assert_ne!(
            pintados[0].1, pintados[1].1,
            "el error tiene que verse distinto de la salida normal"
        );
    }

    /// Los dos flujos de un proceso llegan separados, y cada línea sabe del suyo.
    ///
    /// Es lo que FE-035 pide: sin mezclar la fuente del dato. Se construye una salida de
    /// verdad, con sus dos flujos, y se mira que las líneas normales son las suyas y las
    /// de error las suyas: si se mezclaran, el error saldría pintado como salida normal y
    /// no se podría ni encontrar ni distinguir.
    #[test]
    fn la_salida_normal_y_el_error_no_se_mezclan() {
        let salida = ProcessOutput::new(Some(0), "compilando\nenlazando", "error: no existe");

        let lineas = lineas_de(&salida);

        assert_eq!(
            lineas
                .iter()
                .map(|linea| (linea.texto(), linea.origen()))
                .collect::<Vec<_>>(),
            vec![
                ("compilando", Origen::Normal),
                ("enlazando", Origen::Normal),
                ("error: no existe", Origen::Error),
            ],
            "cada línea lleva su flujo: el normal primero y el error detrás"
        );
    }

    /// Una salida sin error no inventa líneas de error.
    #[test]
    fn una_salida_sin_error_no_tiene_lineas_de_error() {
        let salida = ProcessOutput::new(Some(0), "todo bien", "");

        let lineas = lineas_de(&salida);

        assert_eq!(lineas.len(), 1);
        assert_eq!(lineas[0].origen(), Origen::Normal);
    }
}
