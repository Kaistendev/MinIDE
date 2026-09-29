//! El layout raíz de la ventana: las cuatro zonas de MiniIDE.
//!
//! La ventana se reparte siempre igual y este módulo es el que decide cómo:
//!
//! ```text
//! ┌──────────────────────────────────────┐
//! │ Menú                                 │
//! ├──────────────────────────────────────┤
//! │                                      │
//! │ Área central                         │
//! │                                      │
//! ├──────────────────────────────────────┤
//! │ Panel inferior                       │
//! ├──────────────────────────────────────┤
//! │ Barra de estado                      │
//! └──────────────────────────────────────┘
//! ```
//!
//! Aquí no hay contenido de zonas, solo el sitio de cada una. Lo que va dentro lo trae su
//! tarea: el menú y la barra de herramientas en FE-006 y FE-007, el editor y el diseñador
//! en FE-016, FE-019 y FE-044, la salida y los diagnósticos en FE-034 y FE-036, y la barra de
//! estado en FE-008. Lo que se decide aquí es lo que no depende de lo que haya dentro: que
//! la ventana se reparte igual todos los días.
//!
//! Las zonas de los bordes son `Panel` de egui, arriba y abajo, porque lo que se cuelga de
//! los lados son el explorador de proyectos (FE-010) y las propiedades (FE-051). El área
//! central se dibuja la última a propósito: `CentralPanel` se queda con todo el espacio
//! que dejan los paneles de alrededor, así que si se dibuja antes, su rectángulo queda
//! debajo de ellos en vez de entre ellos.
//!
//! La barra de estado se dibuja antes que el panel inferior porque un panel de abajo se
//! queda siempre con el espacio de más abajo, y el pie de la ventana es la barra de
//! estado, no la salida del proceso.

use eframe::egui;

use super::app::App;
use super::busqueda;
use super::diagnosticos;
use super::editor;
use super::explorador;
use super::menu;
use super::salida;
use super::status;
use super::tabs;
use super::toolbar;

/// Una zona del layout raíz.
///
/// Su nombre es lo que se ve en la zona mientras no tenga contenido, y es también de
/// donde sale su id: egui recuerda cada panel por su id —el tamaño al que el usuario lo
/// ha redimensionado—, así que dos zonas con el mismo nombre serían el mismo panel.
///
/// Vive aquí y no repartida porque el layout es quien decide el sitio de cada zona, y el
/// nombre de una zona y su id tienen que ser la misma cosa: si cada módulo|Sportara el suyo,
/// el explorador y su zona acabarían siendo dos zonas distintas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zona {
    /// Arriba: el menú y la barra de herramientas. FE-006 y FE-007.
    Menu,
    /// En medio: el editor y el diseñador. FE-019 y FE-044.
    Central,
    /// Abajo, encima de la barra de estado: la salida y los diagnósticos. FE-034 y
    /// FE-036.
    Inferior,
    /// El pie de la ventana: la barra de estado. FE-008.
    Estado,
    /// Pegado a la izquierda: el explorador de proyectos. FE-010 a FE-014.
    Proyecto,
}

impl Zona {
    /// Lo que pone la zona mientras no tenga contenido de verdad.
    pub fn nombre(&self) -> &'static str {
        match self {
            Zona::Menu => "Menú",
            Zona::Central => "Área central",
            Zona::Inferior => "Panel inferior",
            Zona::Estado => "Barra de estado",
            Zona::Proyecto => "Proyecto",
        }
    }

    /// El id con el que egui recuerda el panel de la zona.
    ///
    /// Sale del nombre y no al revés para que no puedan separarse. El id es fijo porque
    /// si cambiara, la zona perdería su tamaño cada vez que se redibuja la ventana.
    pub fn id(&self) -> egui::Id {
        egui::Id::new(("miniide.layout", self.nombre()))
    }
}

/// Dibuja el layout raíz: las cuatro zonas de la ventana, cada una en su sitio.
///
/// La aplicación entra entera y no solo su estado visual porque las zonas pueden pedir
/// cosas -FE-006, FE-007, FE-009- y la que las recoge es la aplicación: el layout solo
/// dibuja, y quien decide qué pasa con una petición es `App::emitir`.
///
/// El orden es el que necesita egui y no el que se ve. Arriba y abajo se dibujan primero,
/// porque cada panel se queda con el espacio de su borde y el que se dibuja después se
/// apila contra el anterior. La barra de estado va antes que el panel inferior para que el
/// pie de la ventana sea la barra de estado y no la salida del proceso. Y el área central
/// va la última, porque `CentralPanel` se queda con todo lo que queda libre.
pub fn layout(ui: &mut egui::Ui, app: &mut App) {
    menu(ui, app);

    barra_estado(ui, app);
    inferior(ui, app);
    explorador(ui, app);
    central(ui, app);
}

/// La zona de arriba: el menú y la barra de herramientas.
///
/// El menú ya está (FE-006), la barra de herramientas también (FE-007) y las dos piden por
/// el mismo camino (FE-009), así que la zona lleva las dos cosas que se pulsan y su
/// contenido no necesita saber nada de lo que hacen con lo que se les pide.
///
/// No se redimensiona, que es lo que hace `Panel::top` por defecto, porque su alto lo
/// pondrán los menús y los botones que traigan encima y ese es su alto natural.
fn menu(ui: &mut egui::Ui, app: &mut App) {
    egui::Panel::top(Zona::Menu.id()).show(ui, |ui| {
        menu::barra(ui, app);
        toolbar::barra(ui, app);
    });
}

/// El pie de la ventana: la barra de estado.
///
/// Se dibuja antes que el panel inferior para acabar debajo de él, y es la única zona que
/// solo lee: mira el estado visual y no pide nada.
fn barra_estado(ui: &mut egui::Ui, app: &App) {
    egui::Panel::bottom(Zona::Estado.id()).show(ui, |ui| status::barra(ui, app.state()));
}

/// El panel de la izquierda: el explorador de proyectos. FE-010.
///
/// Va después de las franjas de arriba y de abajo y antes del área central por una razón
/// de orden, no de gusto: cada panel se queda con el sitio que dejan los que se han
/// dibujado antes, así que el explorador dibuja después de las franjas para no meterse
/// debajo de ellas, y el área central va la última para quedarse con lo que queda, que
/// aquí ya es sin la columna del explorador.
///
/// Se redimensiona con el borde, que es lo que hacen los paneles de los lados por defecto
/// y lo que un usuario espera de una columna: la agranda o la encoge sin tener que buscar
/// un asa escondida. El ancho se lo guarda egui por su id, y por eso el id sale del
/// nombre de la zona y no se cambia.
fn explorador(ui: &mut egui::Ui, app: &mut App) {
    egui::Panel::left(Zona::Proyecto.id()).show(ui, |ui| {
        // Sin archivos: a la ventana todavía no le llega nada del core, así que no hay
        // proyecto abierto y no hay archivos que enseñar. FE-058 es la que abre un
        // proyecto, y entonces aquí se le pasan los suyos en vez de una lista vacía.
        explorador::panel(ui, &[], app);
    });
}

/// El alto que tiene el panel de abajo la primera vez, en puntos.
///
/// Alto de verdad y no "lo que ocupa su contenido": si el panel cogiera el alto de lo que
/// tiene dentro, abrir una salida con veinte líneas o cerrarla movería el editor entero, y
/// con ello el cursor y lo que se estaba viendo. Un panel de salida es de un alto, y se
/// redimensiona si el usuario quiere -FE-066-.
///
/// Son 160 porque en una ventana de ochocientas deja al editor más de la mitad, que es lo
/// que se quiere: el panel de abajo se mira cuando se está esperando, no mientras se
/// escribe.
const ALTO_DE_LA_ZONA_DE_ABAJO: f32 = 160.0;

/// La zona de abajo, encima de la barra de estado: la salida y los diagnósticos.
/// FE-034, FE-035 y FE-036.
///
/// Van uno encima del otro y no con pestañas porque son los dos que hay: la salida de lo
/// que se ha ejecutado arriba, que es lo que se mira mientras dura, y los diagnósticos
/// debajo, que son los que se miran cuando ha terminado. Con dos cosas que se miran en
/// momentos distintos, una debajo de otra se pasa de la una a la otra con la mirada, y
/// unas pestañas harían falta solo para poderarlas.
///
/// Los dos reciben nada porque a la ventana todavía no le llega nada del core: no hay
/// proyecto abierto y no hay procesos lanzados, que es lo que traería la salida (FE-058 y
/// T-098). FE-036 y FE-037 ya enseñan un diagnóstico de verdad cuando se les da.
fn inferior(ui: &mut egui::Ui, app: &mut App) {
    egui::Panel::bottom(Zona::Inferior.id())
        .exact_size(ALTO_DE_LA_ZONA_DE_ABAJO)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(Zona::Inferior.nombre());
                salida::panel(ui, &crate::runtime::ProcessOutput::empty());
                ui.separator();
                diagnosticos::panel(ui, &[], app);
            });
        });
}

/// El área central: las pestañas y el editor. FE-015 a FE-028 y FE-081.
///
/// No lleva id propio porque no es un panel: `CentralPanel` se queda con el espacio que
/// dejan los paneles de los bordes y no guarda nada por id.
///
/// Van las pestañas arriba y el editor debajo, que es donde están en cualquier editor y
/// porque es donde se está mirando: la lista de lo que hay abierto y el documento que se
/// está viendo, uno encima del otro.
///
/// El editor entra vacío porque a la ventana todavía no le llega ningún documento: no hay
/// proyecto abierto, y sin documento no hay nada que enseñar. FE-058 es la que abre un
/// proyecto y el documento que tenga abierto, y entonces aquí se le pasa su documento y el
/// lenguaje del archivo en vez de nada.
fn central(ui: &mut egui::Ui, app: &mut App) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.vertical(|ui| {
            tabs::panel(ui, app);
            ui.separator();
            busqueda::panel(ui, app);
            ui.separator();
            editor::panel(ui, editor::PestanaVisual::vacio(), app);
        });
    });
}

/// El marcador de una zona ya no hace falta: la última zona sin contenido propio era la de
/// abajo, y ahora las cuatro zonas enseñan algo. Volverá con la de propiedades (FE-051),
/// que es la siguiente en quedarse sin contenido.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ventana;

    /// Una ventana de trabajo normal, ni enorme ni minima.
    const ANCHO: f32 = 1000.0;
    const ALTO: f32 = 800.0;

    /// Lo que le cuesta a egui cada frontera entre zonas.
    ///
    /// egui reserva fuera del rectángulo que pinta cada panel el ancho de su línea
    /// separadora, así que entre dos zonas hay un hueco de un píxel. Comparar al píxel
    /// sería medir el marco de egui y no el layout de MiniIDE.
    const MARGEN: f32 = 2.0;

    /// Dibuja la ventana de verdad y devuelve las franjas horizontales que se pintan.
    ///
    /// Se dibuja por [`ventana`] y no por [`layout`] a propósito: lo que se comprueba es
    /// lo que se ve en la ventana, no una lista de zonas que el layout dice tener. egui
    /// no enseña el texto de los widgets en la salida de un frame, pero sí los
    /// rectángulos que pinta cada panel, y eso es justo lo que hace que las zonas se vean
    /// unas debajo de otras y no todas en el mismo sitio.
    ///
    /// Cada zona llega al borde derecho de la ventana, así que las franjas son las de las
    /// cuatro zonas. Se distinguen por eso y no por ocupar el ancho entero, porque en cuanto
    /// hay un panel pegado a un lado (el explorador de proyectos, FE-010) el área central
    /// empieza más allá y deja de cruzar la ventana de lado a lado. Los rectángulos que no
    /// llegan al borde derecho son esos paneles de los lados, y las líneas separadoras de
    /// egui no son rectángulos.
    fn franjas(context: &egui::Context) -> Vec<egui::Rect> {
        let mut app = super::super::app::App::new();
        let mut salida = context.run_ui(entrada(), |ui| {
            ventana(ui, &mut app);
        });

        // egui avisa si se tiran sin aplicar las texturas que ha creado, y en una
        // aplicación de verdad las aplica el renderizador. Aquí no hay renderizador, así
        // que se vacían a propósito: lo que se prueba es el layout, no los píxeles.
        salida.textures_delta.clear();

        let mut franjas: Vec<egui::Rect> = salida
            .shapes
            .into_iter()
            .filter_map(|forma| match forma.shape {
                egui::Shape::Rect(rectangulo) => Some(rectangulo.rect),
                _ => None,
            })
            .filter(|rectangulo| rectangulo.max.x >= ANCHO - MARGEN)
            .collect();

        franjas.sort_by(|uno, otro| uno.min.y.total_cmp(&otro.min.y));
        franjas
    }

    /// Las franjas de la ventana, de arriba abajo, y solo cuatro.
    ///
    /// Que sean cuatro y en este orden es la mitad de lo que hay que comprobar, así que
    /// la cuenta va aquí y no repetida en cada test: quien llame a esto ya sabe que tiene
    /// el menú, el área central, el panel inferior y la barra de estado, en ese orden de
    /// arriba abajo.
    fn zonas(context: &egui::Context) -> [egui::Rect; 4] {
        let franjas = franjas(context);

        assert_eq!(
            franjas.len(),
            4,
            "la ventana se divide en menu, area central, panel inferior y barra de estado: {franjas:?}"
        );

        [franjas[0], franjas[1], franjas[2], franjas[3]]
    }

    /// Un frame con una ventana de tamaño conocido.
    fn entrada() -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(ANCHO, ALTO),
            )),
            ..egui::RawInput::default()
        }
    }

    /// Las cuatro zonas del layout se distinguen unas de otras.
    ///
    /// El id de una zona sale de su nombre, y egui usa el id para acordarse de cada
    /// panel: dos zonas con el mismo nombre serían el mismo panel, y el tamaño al que el
    /// usuario redimensionase una movería también la otra.
    #[test]
    fn las_zonas_del_layout_no_se_confunden_entre_si() {
        use std::collections::HashSet;

        let zonas = [Zona::Menu, Zona::Central, Zona::Inferior, Zona::Estado];

        let nombres: HashSet<&str> = zonas.iter().map(Zona::nombre).collect();
        assert_eq!(
            nombres.len(),
            zonas.len(),
            "cada zona necesita un nombre propio, si no no se sabe cual se esta viendo: {zonas:?}"
        );
        assert!(
            nombres.iter().all(|nombre| !nombre.is_empty()),
            "una zona sin nombre no se ve: {zonas:?}"
        );

        let ids: HashSet<egui::Id> = zonas.iter().map(Zona::id).collect();
        assert_eq!(
            ids.len(),
            zonas.len(),
            "dos zonas con el mismo id son el mismo panel para egui, y el tamaño al que se redimensione una mueve la otra: {zonas:?}"
        );
    }

    /// Las cuatro zonas se reparten la ventana de arriba abajo.
    ///
    /// Es lo que hace que la ventana se entienda: el menú arriba, el área central en
    /// medio, la salida debajo y la barra de estado en el pie. Sin esto MiniIDE sería un
    /// panel único con texto suelto, que es como estaba antes de esta tarea.
    #[test]
    fn las_cuatro_zonas_reparten_la_ventana() {
        let [menu, central, inferior, estado] = zonas(&egui::Context::default());

        for zona in [menu, central, inferior, estado] {
            assert!(
                zona.max.x >= ANCHO - MARGEN,
                "una zona tiene que llegar al borde derecho de la ventana, y si no es un panel de los lados: {zona:?}"
            );
            assert!(
                0.0 < zona.height(),
                "una zona con altura cero no se ve: {zona:?}"
            );
        }

        assert!(
            menu.min.y.abs() <= MARGEN,
            "la primera zona es la de arriba y llega al borde superior: {menu:?}"
        );
        assert!(
            (ALTO - estado.max.y).abs() <= MARGEN,
            "la ultima zona es la de abajo y llega al borde inferior: {estado:?}"
        );

        for [arriba, abajo] in [[menu, central], [central, inferior], [inferior, estado]] {
            assert!(
                (arriba.max.y - abajo.min.y).abs() <= MARGEN,
                "las zonas van unas debajo de otras sin huecos de verdad: {arriba:?} {abajo:?}"
            );
            assert!(
                arriba.max.y <= abajo.min.y + MARGEN,
                "las zonas no se pisan entre si: {arriba:?} {abajo:?}"
            );
        }
    }

    /// El área central es la zona grande, no una franja más.
    ///
    /// Es donde va el editor y el diseñador, que es donde se pasa el rato. Si el área
    /// central no se lleva la ventana, el reparto está mal aunque las cuatro zonas se
    /// vean.
    #[test]
    fn el_area_central_es_la_mas_grande() {
        let [_menu, central, _inferior, _estado] = zonas(&egui::Context::default());

        assert!(
            central.height() > ALTO / 2.0,
            "el area central tiene que ser la mayor parte de la ventana, no una franja como las otras: {central:?}"
        );
    }

    /// La ventana se reparte igual en todos los frames.
    ///
    /// egui recuerda el tamaño de un panel entre frames, y recordar algo mal se ve como
    /// una ventana que se descuadra sola mientras no se la toca. Se dibujan dos frames
    /// seguidos y las cuatro zonas tienen que caer en el mismo sitio.
    #[test]
    fn el_layout_no_se_mueve_al_redibujarse() {
        let context = egui::Context::default();

        let primera = zonas(&context);
        let segunda = zonas(&context);

        for (antes, ahora) in primera.iter().zip(segunda.iter()) {
            assert!(
                (antes.min.y - ahora.min.y).abs() <= MARGEN,
                "una zona se ha movido al redibujar: {antes:?} {ahora:?}"
            );
            assert!(
                (antes.height() - ahora.height()).abs() <= MARGEN,
                "una zona ha cambiado de alto al redibujar: {antes:?} {ahora:?}"
            );
        }
    }
}
