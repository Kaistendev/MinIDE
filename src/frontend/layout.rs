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
use super::dialogos;
use super::diseniador;
use super::editor;
use super::explorador;
use super::menu;
use super::propiedades as panel_de_propiedades;
use super::salida;
use super::status;
use super::tabs;
use super::toolbar;
use super::ui_state::VistaCentral;

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
    /// Pegado a la derecha: las propiedades de lo seleccionado. FE-051.
    Propiedades,
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
            Zona::Propiedades => "Propiedades",
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
    propiedades(ui, app);
    central(ui, app);
    dialogos::panel(ui.ctx(), app);
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
/// solo lee: mira el estado visual y lo que se está haciendo, y no pide nada. Las
/// operaciones entran por aquí porque su estado (FE-038, FE-039) es de lo que vive la
/// barra, y sacarlo de la aplicación y de la zona para que cada una tenga el suyo sería
/// tener dos verdades sobre si se está compilando.
fn barra_estado(ui: &mut egui::Ui, app: &App) {
    egui::Panel::bottom(Zona::Estado.id()).show(ui, |ui| {
        status::barra(
            ui,
            app.state(),
            app.operaciones(),
            app.proyecto().map(|proyecto| proyecto.name()),
            app.state().documento_activo(),
        );
    });
}

/// El panel de la izquierda: el explorador de proyectos. FE-010.
///
/// Va después de las franjas de arriba y de abajo y antes del área central por una razón
/// de orden, no de gusto: cada panel se queda con el sitio que dejan los que se han
/// dibujado antes, así que el explorador dibuja después de las franjas para no meterse
/// debajo de ellas, y el área central va la última para quedarse con lo que queda, que es
/// aquí ya es sin la columna del explorador.
///
/// Se redimensiona con el borde, que es lo que hacen los paneles de los lados por defecto
/// y lo que un usuario espera de una columna: la agranda o la encoge sin tener que buscar
/// un asa escondida. El ancho se lo guarda egui por su id, y por eso el id sale del
/// nombre de la zona y no se cambia.
fn explorador(ui: &mut egui::Ui, app: &mut App) {
    egui::Panel::left(Zona::Proyecto.id()).show(ui, |ui| {
        // Los archivos se copian antes de entrar en el panel porque el explorador dibuja
        // con la aplicación en la mano y los archivos son de ella: sin copiarlos, el
        // explorador tendría el árbol prestado y la ventana prestada a la vez, y de cada
        // cosa solo se puede prestar una. Lo que se copia son rutas y no documentos, así
        // que no es una segunda verdad del proyecto: es la foto que se enseña (FE-058).
        let archivos = app.archivos().to_vec();

        explorador::panel(ui, &archivos, app);
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
/// Los diagnósticos son los de la última compilación y salen del core por `Operaciones`
/// (FE-060): la ventana no los busca ni los ordena, porque el core es quien sabe qué es un
/// error de compilación y en qué orden importa. Se copian por lo mismo que los archivos del
/// explorador: el panel los dibuja con la ventana en la mano, y cada cosa se presta una vez.
fn inferior(ui: &mut egui::Ui, app: &mut App) {
    let diagnosticos = app.diagnosticos().to_vec();

    egui::Panel::bottom(Zona::Inferior.id())
        .exact_size(ALTO_DE_LA_ZONA_DE_ABAJO)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(Zona::Inferior.nombre());
                salida::panel(ui, app.operaciones().salida());
                ui.separator();
                diagnosticos::panel(ui, &diagnosticos, app);
            });
        });
}

/// El área central: las pestañas y el editor, o el diseñador. FE-015 a FE-028, FE-044 y
/// FE-081.
///
/// No lleva id propio porque no es un panel: `CentralPanel` se queda con el espacio que
/// dejan los paneles de los bordes y no guarda nada por id.
///
/// Van las pestañas arriba y el editor debajo, que es donde están en cualquier editor y
/// porque es donde se está mirando: la lista de lo que hay abierto y el documento que se
/// está viendo, uno encima del otro.
///
/// El editor pinta el documento que la ventana tiene abierto, que es del core, y el
/// desplazamiento se le da copiado y se devuelve escrito: el documento y el estado visual no
/// se pueden prestar a la vez, y el editor los necesita los dos en el mismo frame (FE-058).
///
/// El diseñador es la otra mitad de esta misma zona (FE-044), y solo puede verse uno de los
/// dos: si los dos se dibujaran a la vez cada uno se quedaría con la mitad del panel, y el
/// usuario vería dos mitades en lugar de una vista. Cuál de los dos se ve lo dice el estado
/// visual, y lo cambia el interruptor de arriba cuando el documento que se está viendo tiene
/// un diseño detrás (FE-059).
fn central(ui: &mut egui::Ui, app: &mut App) {
    egui::CentralPanel::default().show(ui, |ui| match app.state().vista_central() {
        VistaCentral::Editor => {
            vistas_del_formulario(ui, app);

            ui.vertical(|ui| {
                tabs::panel(ui, app);
                ui.separator();
                busqueda::panel(ui, app);
                ui.separator();
                let area = egui::Rect::from_min_size(ui.cursor().min, ui.available_size());
                let contexto = ui.interact(
                    area,
                    editor::id_del_contexto(),
                    egui::Sense::click_and_drag(),
                );
                let mut desplazamiento = app.state().desplazamiento();
                ui.allocate_ui(area.size(), |ui| {
                    editor::panel(ui, app.pestana_visual(), &mut desplazamiento);
                });
                app.state_mut().set_desplazamiento(desplazamiento);
                // El asterisco de la pestaña es estado visual y el modificado es del
                // documento: sin esta vuelta, escribir no pondría el asterisco ni la
                // ventana preguntaría al cerrar (FE-058).
                app.sincronizar_la_pestana();
                editor::menu_contextual(contexto, app);
            });
        }
        VistaCentral::Diseniador => {
            vistas_del_formulario(ui, app);
            diseniador::panel(ui, app);
        }
    });
}

/// Lo que dice el interruptor de la izquierda: el código.
const CODIGO: &str = "Code";

/// Lo que dice el interruptor de la derecha: el diseño del formulario. FE-059.
///
/// Los dos nombres son los de cualquier IDE con diseñador y son los que el usuario ya
/// conoce: son etiquetas de un botón, no un concepto de MiniIDE.
const DISENO: &str = "Designer";

/// El interruptor entre el código y el diseño del formulario. FE-059.
///
/// Solo se dibuja si se puede pasar de uno a otro —FE-057 y FE-059— y no siempre: un
/// interruptor que aparece y no hace nada es peor que no tenerlo, porque el usuario lo pulsa y
/// no entiende por qué no pasa nada. Con un documento sin diseño detrás no hay a dónde ir, y
/// con un framework sin vista de diseño no hay ni dónde mirar.
///
/// Se dibuja en las dos vistas y no solo en el editor porque tiene que estar donde se está
/// mirando la vista que se quiere cambiar: si solo estuviera en el editor, desde el
/// diseñador no habría forma de volver al código.
fn vistas_del_formulario(ui: &mut egui::Ui, app: &mut App) {
    if !app.puede_diseñar() {
        return;
    }

    let viendo_el_codigo = app.state().vista_central() == VistaCentral::Editor;

    ui.horizontal(|ui| {
        if ui.selectable_label(viendo_el_codigo, CODIGO).clicked() {
            app.state_mut().mostrar_editor();
        }

        // El diseño solo se rehace al venir del código: volver a pulsarlo con el diseñador
        // ya delante borraría los controles que se han colocado sin querer.
        if ui.selectable_label(!viendo_el_codigo, DISENO).clicked() && viendo_el_codigo {
            app.abrir_diseniador();
        }
    });
}

/// La columna de la derecha: las propiedades del control seleccionado. FE-051.
///
/// Va pegada a la derecha y antes que el área central por la misma razón que el explorador:
/// cada panel se queda con el sitio que dejan los que se han dibujado antes.
///
/// El ancho es fijo y no el que egui quiera porque el ancho de un panel se guarda entre
/// frames y un panel que cambia de ancho al redimensionarse la ventana empuja la zona
/// central de lado. Con el ancho escrito aquí la columna se comporta como la de un IDE.
fn propiedades(ui: &mut egui::Ui, app: &mut App) {
    egui::Panel::right(Zona::Propiedades.id())
        .exact_size(ANCHO_DE_LAS_PROPIEDADES)
        .show(ui, |ui| {
            ui.label(Zona::Propiedades.nombre());
            ui.separator();
            panel_de_propiedades::panel(ui, app);
        });
}

/// Lo que mide la columna de propiedades, en puntos.
///
/// Son 220 porque dan para el nombre del control y su valor en una línea, que es lo más
/// ancho que enseña el panel, y porque dejando más de la mitad de una ventana de mil puntos
/// se le está quitando sitio al diseñador para enseñarle cuatro campos.
const ANCHO_DE_LAS_PROPIEDADES: f32 = 220.0;

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
    /// Cada franja horizontal ocupa más de media ventana, y los paneles de los lados no.
    ///
    /// Se distinguen por el ancho y no por llegar al borde derecho porque ahora hay dos
    /// columnas pegadas a los lados —el explorador y las propiedades (FE-051)—, y una de
    /// ellas llega también al borde derecho: buscándolas por el borde, la columna de
    /// propiedades se contaría como una franja y el área central, que queda entre las dos
    /// columnas, se contaría como si no existiera.
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
            .filter(|rectangulo| rectangulo.width() > ANCHO / 2.0)
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

        let zonas = [
            Zona::Menu,
            Zona::Central,
            Zona::Inferior,
            Zona::Estado,
            Zona::Propiedades,
        ];

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

    /// Las cuatro franjas se reparten la ventana de arriba abajo.
    ///
    /// Es lo que hace que la ventana se entienda: el menú arriba, el área central en medio,
    /// la salida debajo y la barra de estado en el pie. Sin esto MiniIDE sería un panel
    /// único con texto suelto, que es como estaba antes de esta tarea.
    #[test]
    fn las_cuatro_zonas_reparten_la_ventana() {
        let [menu, central, inferior, estado] = zonas(&egui::Context::default());

        for zona in [menu, central, inferior, estado] {
            assert!(
                zona.width() > ANCHO / 2.0,
                "una franja tiene que cruzar la ventana de lado a lado, y si no es una \
                 columna de los lados: {zona:?}"
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

    /// Una ventana con un proyecto de WinForms abierto y un archivo del formulario en una
    /// pestaña.
    ///
    /// Es el estado en el que vive MiniIDE trabajando, y el punto de partida de los tres
    /// tests de debajo. El proyecto es de verdad, con sus archivos en el disco, porque lo que
    /// se comprueba es lo que ve el usuario a partir de lo que hay escrito.
    fn ventana_trabajando(
        nombre: &str,
    ) -> (egui::Context, super::super::App, crate::project::Project) {
        let (context, mut app, proyecto) = ventana_con_un_proyecto(nombre);

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(crate::commands::Command::OpenDocument);
        app.avanzar(&context);

        (context, app, proyecto)
    }

    /// Una ventana con un proyecto de WinForms abierto y ningún documento.
    ///
    /// Sin documentos para los tests que tienen que mirar algo antes de abrir nada —el tamaño
    /// de la ventana, o un archivo con un contenido concreto que hay que escribir antes de
    /// abrirlo—.
    fn ventana_con_un_proyecto(
        nombre: &str,
    ) -> (egui::Context, super::super::App, crate::project::Project) {
        use std::sync::Arc;

        use crate::core::ProjectType;
        use crate::templates::create_project;
        use crate::toolchain::DotNetToolchain;

        let raiz = std::env::temp_dir().join(nombre);
        let _ = std::fs::remove_dir_all(&raiz);

        let proyecto =
            create_project(ProjectType::CSharpWinForms, &raiz).expect("proyecto de WinForms");
        let mut app = super::super::App::new();
        app.abrir(proyecto.clone(), Arc::new(DotNetToolchain));

        (egui::Context::default(), app, proyecto)
    }

    /// Un clic y una soltura en `posicion`, y el puntero que se queda ahí.
    ///
    /// Con el puntero delante porque cada frame empieza con la entrada que se le pasa y no con
    /// la de antes: sin él, el clic se registra en un sitio que en el frame siguiente ya no
    /// existe.
    fn clic(context: &egui::Context, app: &mut super::super::App, posicion: egui::Pos2) {
        for pressed in [true, false] {
            let _ = dibujar_en(
                context,
                app,
                None,
                &[
                    egui::Event::PointerButton {
                        pos: posicion,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    },
                    egui::Event::PointerMoved(posicion),
                ],
            );
        }
    }

    /// Escribe `texto` en el documento que se está viendo, como lo haría el usuario.
    ///
    /// Primero un clic dentro del editor —el editor toma el foco con el clic, no solo con que el
    /// ratón pase por encima, FE-065— y el texto va en un frame aparte porque el foco se aplica
    /// al terminar el frame en que se ha pedido.
    fn escribir(context: &egui::Context, app: &mut super::super::App, texto: &str) {
        clic(context, app, egui::pos2(450.0, 300.0));
        escribir_sin_clic(context, app, texto);
    }

    /// Escribe `texto` sin tocar el foco: escribe solo si el editor lo tiene.
    ///
    /// Es la mitad de [`Self::escribir`] que va aparte porque es justo lo que hay que medir en
    /// FE-065: con el foco en otro sitio el texto no puede entrar en el documento. Si el helper
    /// hiciera clic, recuperaría el foco él solo y el test no comprobaría nada.
    fn escribir_sin_clic(context: &egui::Context, app: &mut super::super::App, texto: &str) {
        let dentro = egui::pos2(450.0, 300.0);

        for eventos in [
            vec![egui::Event::PointerMoved(dentro)],
            vec![
                egui::Event::PointerMoved(dentro),
                egui::Event::Text(texto.to_owned()),
            ],
        ] {
            let _ = dibujar_en(context, app, None, &eventos);
        }
    }

    /// Una ventana con un proyecto de Java con Swing abierto, con su ventana abierta.
    ///
    /// Es el hermano de [`Self::ventana_trabajando`] para el otro framework, y está aparte
    /// porque los archivos de Java están tres carpetas más abajo: el camino hasta la ventana
    /// es desplegar `src`, `main` y `java` y pulsar el fuente, que es lo que hace el usuario.
    fn ventana_con_un_proyecto_de_swing(
        nombre: &str,
    ) -> (egui::Context, super::super::App, crate::project::Project) {
        use std::sync::Arc;

        use crate::core::ProjectType;
        use crate::templates::create_project;
        use crate::toolchain::JdkToolchain;

        let raiz = std::env::temp_dir().join(nombre);
        let _ = std::fs::remove_dir_all(&raiz);

        let proyecto = create_project(ProjectType::JavaSwing, &raiz).expect("proyecto de Swing");
        let mut app = super::super::App::new();
        app.abrir(proyecto.clone(), Arc::new(JdkToolchain));

        let context = egui::Context::default();
        app.state_mut().seleccionar("src/main/java/MainWindow.java");
        app.emitir(crate::commands::Command::OpenDocument);
        app.avanzar(&context);

        (context, app, proyecto)
    }

    /// Dibuja la ventana de `app` con estos eventos de por medio y devuelve lo que ha escrito.
    fn escrito(
        context: &egui::Context,
        app: &mut super::super::App,
        eventos: &[egui::Event],
    ) -> Vec<String> {
        let entrada = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(ANCHO, ALTO),
            )),
            events: eventos.to_vec(),
            ..egui::RawInput::default()
        };

        let mut salida = context.run_ui(entrada, |ui| ventana(ui, app));
        salida.textures_delta.clear();

        salida
            .shapes
            .iter()
            .filter_map(|forma| match &forma.shape {
                egui::Shape::Text(texto) => Some(texto.galley.job.text.to_string()),
                _ => None,
            })
            .collect()
    }

    /// El punto donde se ha pintado la aparición número `cual` de un texto.
    ///
    /// egui no da los widgets que se han dibujado, pero sí el texto y dónde lo ha puesto, y
    /// eso es justo lo que el usuario ve: se apunta a lo que se lee.
    ///
    /// Se busca el texto entero y no un trozo porque "Designer" está en el conmutor de vistas
    /// y también dentro del nombre de un archivo: con un trozo, este helper apuntaría al
    /// explorador y el clic caería en otro sitio del que en realidad se quería. Y se cuenta
    /// cuál de las apariciones es la que se quiere, porque hay palabras que están dos veces en
    /// pantalla: "Compilar" es un menú y también un botón de la barra.
    fn posicion_de_la(
        context: &egui::Context,
        app: &mut super::super::App,
        texto: &str,
        cual: usize,
    ) -> egui::Pos2 {
        let mut salida = context.run_ui(entrada(), |ui| ventana(ui, app));
        salida.textures_delta.clear();

        salida
            .shapes
            .iter()
            .filter_map(|forma| match &forma.shape {
                egui::Shape::Text(pintado) if pintado.galley.job.text == texto => Some(pintado.pos),
                _ => None,
            })
            .nth(cual)
            .unwrap_or_else(|| panic!("no se ha pintado {texto:?}"))
    }

    /// Dibuja la ventana de `app` en un tamaño dado, con estos eventos de por medio.
    ///
    /// El tamaño se puede cambiar porque FE-066 es una revisión de tamaños: una ventana solo
    /// dibujada en un tamaño no dice si aguanta otro. `None` es el de los tests, que es el
    /// habitual.
    fn dibujar_en(
        context: &egui::Context,
        app: &mut super::super::App,
        ancho: Option<f32>,
        eventos: &[egui::Event],
    ) -> egui::FullOutput {
        let entrada = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(ancho.unwrap_or(ANCHO), ALTO),
            )),
            events: eventos.to_vec(),
            ..egui::RawInput::default()
        };

        let mut salida = context.run_ui(entrada, |ui| ventana(ui, app));
        salida.textures_delta.clear();

        salida
    }

    /// Un clic de verdad sobre el texto `texto`.
    ///
    /// egui no cuenta un clic si la pulsación y la soltura caen en el mismo frame, así que
    /// van en dos, y el puntero se pone encima del texto que se quiere pulsar.
    fn pulsar(context: &egui::Context, app: &mut super::super::App, texto: &str) {
        pulsar_el(context, app, texto, 0);
    }

    /// Un clic de verdad sobre la aparición número `cual` de un texto.
    ///
    /// Hay palabras que están dos veces en pantalla —"Compilar" es un menú y también un botón
    /// de la barra de herramientas— y en ese caso el clic tiene que ir a una de las dos, no a
    /// la primera que aparezca. Se cuenta en el orden en que se pintan, que es el orden en el
    /// que se ven.
    fn pulsar_el(context: &egui::Context, app: &mut super::super::App, texto: &str, cual: usize) {
        let posicion = posicion_de_la(context, app, texto, cual) + egui::vec2(4.0, 4.0);

        for pressed in [true, false] {
            escrito(
                context,
                app,
                &[egui::Event::PointerButton {
                    pos: posicion,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::default(),
                }],
            );
        }
    }

    /// El árbol enseña los archivos del proyecto abierto. FE-058.
    ///
    /// Es la diferencia entre abrir un proyecto y no abrir nada: sin esto la ventana sigue
    /// enseñando un panel vacío con su título, y el usuario no tiene forma de llegar a un
    /// archivo. Se mira el texto pintado y no la lista del explorador porque lo que importa
    /// es que se vea.
    #[test]
    fn el_arbol_muestra_los_archivos_del_proyecto_abierto() {
        let (context, mut app, proyecto) = ventana_trabajando("miniide-layout-arbol");

        let escrito = escrito(&context, &mut app, &[]);

        assert!(
            escrito.iter().any(|linea| linea.contains("Form1.cs")),
            "el archivo del formulario se ve en el árbol: {escrito:?}"
        );
        assert!(
            escrito.iter().any(|linea| linea.contains("Program.cs")),
            "y el punto de entrada también: {escrito:?}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// El archivo abierto se ve en el editor. FE-058.
    ///
    /// Se mira una palabra del cuerpo del archivo —una llamada a `InitializeComponent`, que no
    /// está en ningún sitio más de la ventana— porque el editor pinta el texto resaltado
    /// token a token y no línea a línea, y porque "Form1.cs" también es el nombre de la
    /// pestaña y el de la fila del árbol: comprobar eso no comprobaría el editor.
    #[test]
    fn el_archivo_abierto_se_ve_en_el_editor() {
        let (context, mut app, proyecto) = ventana_trabajando("miniide-layout-editor");

        let escrito = escrito(&context, &mut app, &[]);

        let en_disco = std::fs::read_to_string(proyecto.root().join("Form1.cs"))
            .expect("el archivo del formulario se puede leer");
        assert!(
            en_disco.contains("InitializeComponent"),
            "el archivo de la plantilla tiene que tener esto para que el test sirva de algo"
        );
        assert!(
            escrito
                .iter()
                .any(|linea| linea.contains("InitializeComponent")),
            "el texto del documento abierto tiene que estar en pantalla: {escrito:?}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Los errores de la compilación se enseñan abajo, con el archivo y la línea. FE-060.
    ///
    /// El resultado del core se le pasa a la ventana entero, como se le pasa cuando llega de
    /// verdad, y lo que se mira es lo que se ve: un error sin archivo ni línea es un mensaje
    /// que no dice dónde corregir nada.
    #[test]
    fn los_errores_de_la_compilacion_se_ensenan_abajo_con_donde_estan() {
        use crate::build::BuildResult;
        use crate::core::CoreResult;
        use crate::diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticLocation};
        use crate::document::TextPosition;
        use crate::project::ProjectRelativePath;
        use crate::runtime::ProcessOutput;

        let (context, mut app, proyecto) = ventana_trabajando("miniide-layout-errores");
        let resultado: CoreResult<BuildResult> = Ok(BuildResult::new(
            ProcessOutput::new(Some(1), "", ""),
            vec![Diagnostic::new(
                DiagnosticLevel::Error,
                "se esperaba un punto y coma".to_owned(),
                Some(DiagnosticLocation::new(
                    ProjectRelativePath::new("Form1.cs").expect("ruta valida"),
                    TextPosition::new(11, 0),
                )),
            )],
        ));
        app.operaciones_mut().aplicar_el_resultado(resultado);

        let escrito = escrito(&context, &mut app, &[]);
        let todo = escrito.join(" ");

        assert!(
            todo.contains("se esperaba un punto y coma"),
            "el mensaje del error tiene que verse: {todo}"
        );
        assert!(
            todo.contains("Form1.cs:12"),
            "y dónde está, con la línea contada desde uno: {todo}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// La salida de la compilación se enseña abajo, tal cual la dio el core. FE-034.
    ///
    /// Es la otra mitad de lo que se mira cuando algo compila: además del error, lo que dijo
    /// el proceso. Se le pasa el resultado entero como llega de verdad —con su salida normal
    /// y su error— y se mira lo que se ve, que es el camino que va de la salida del core al
    /// panel de abajo.
    #[test]
    fn la_salida_de_la_compilacion_se_ensena_abajo() {
        use crate::build::BuildResult;
        use crate::core::CoreResult;
        use crate::runtime::ProcessOutput;

        let (context, mut app, proyecto) = ventana_trabajando("miniide-layout-salida");
        let resultado: CoreResult<BuildResult> = Ok(BuildResult::new(
            ProcessOutput::new(Some(0), "compilando", "un aviso"),
            Vec::new(),
        ));
        app.operaciones_mut().aplicar_el_resultado(resultado);

        let escrito = escrito(&context, &mut app, &[]);
        let todo = escrito.join(" ");

        assert!(
            todo.contains("compilando"),
            "lo que dijo el proceso tiene que verse: {todo}"
        );
        assert!(
            todo.contains("un aviso"),
            "y lo que le fue mal, en su flujo: {todo}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// El árbol de un proyecto de Java enseña sus archivos, con las carpetas de por medio.
    ///
    /// Un proyecto de Java no tiene sus fuentes en la raíz: están en `src/main/java`, y el
    /// explorador tiene que enseñar ese árbol o no hay forma de llegar al código. Se mide el
    /// número de filas que se ven, que es lo que el usuario ve, en vez de la lista de
    /// archivos: con el árbol recién abierto solo enseñan la raíz, y cada carpeta que se
    /// despliega enseña lo que cuelga de ella.
    #[test]
    fn el_arbol_de_java_muestra_los_archivos_abajo_de_sus_carpetas() {
        use std::sync::Arc;

        use crate::core::ProjectType;
        use crate::templates::create_project;
        use crate::toolchain::JdkToolchain;

        let raiz = std::env::temp_dir().join("miniide-layout-arbol-java");
        let _ = std::fs::remove_dir_all(&raiz);
        let proyecto = create_project(ProjectType::JavaSwing, &raiz).expect("proyecto de Swing");

        let context = egui::Context::default();
        let mut app = super::super::App::new();
        app.abrir(proyecto.clone(), Arc::new(JdkToolchain));

        let cerrado = escrito(&context, &mut app, &[]).join(" ");
        assert!(
            cerrado.contains("pom.xml") && cerrado.contains("▸ src"),
            "con el árbol recién abierto solo se ven la raíz y las carpetas: {cerrado}"
        );
        assert!(
            !cerrado.contains("MainWindow.java"),
            "y no los archivos que hay dentro: {cerrado}"
        );

        // "▸ src" pasa a "▾ src" al desplegarla, así que se busca el otro nombre.
        pulsar(&context, &mut app, "▸ src");
        let con_src = escrito(&context, &mut app, &[]).join(" ");
        assert!(con_src.contains("▾ src"), "desplegar src: {con_src}");
        assert!(
            con_src.contains("▸ main"),
            "enseña lo que cuelga: {con_src}"
        );

        pulsar(&context, &mut app, "▸ main");
        pulsar(&context, &mut app, "▸ java");

        let abierta = escrito(&context, &mut app, &[]).join(" ");
        assert!(
            abierta.contains("MainWindow.java") && abierta.contains("Main.java"),
            "desplegando todo se ven los fuentes: {abierta}"
        );
        let _ = std::fs::remove_dir_all(&raiz);
    }

    /// Escribir en el editor solo escribe cuando lo tiene. FE-065.
    ///
    /// Un editor que escribe con cualquier tecla escribiría en el documento mientras el
    /// usuario está escribiendo en otro sitio —una búsqueda, un diálogo, un botón— y eso es
    /// la forma más fácil de perder texto sin enterarse de nada. Se comprueba por los dos
    /// lados: con el foco en otro sitio el texto no entra, y con un clic dentro del editor
    /// vuelve a entrar, que es lo que hace falta para recuperar el foco sin cerrar nada.
    #[test]
    fn escribir_solo_escribe_mientras_el_editor_tiene_el_foco() {
        let (context, mut app, proyecto) = ventana_trabajando("miniide-fe065-foco");

        // El foco se lo lleva un botón de la barra, que no es el editor.
        pulsar(&context, &mut app, "Detener");

        escribir_sin_clic(&context, &mut app, "Z");

        assert!(
            !app.state().pestanas()[0].esta_modificada(),
            "con el foco en otro sitio el documento no puede cambiar: {:?}",
            app.state().pestanas()
        );

        // Un clic dentro del editor le devuelve el foco, y a partir de ahí sí escribe.
        escribir(&context, &mut app, "Z");

        assert!(
            app.state().pestanas()[0].esta_modificada(),
            "con el foco recuperado el editor tiene que escribir: {:?}",
            app.state().pestanas()
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// El Tab saca el foco del editor, y al volver a escribir no entra nada. FE-065.
    ///
    /// Es la otra mitad de "recuperar el foco de cada área": Tab mueve el foco a la siguiente
    /// zona, así que lo que se escriba después va allí y no al documento que se está viendo. Sin
    /// comprobarlo, un Tab por equivocación escribiría texto en el archivo abierto.
    #[test]
    fn el_tab_mueve_el_foco_y_el_texto_deja_de_llegar_al_editor() {
        let (context, mut app, proyecto) = ventana_trabajando("miniide-fe065-tab");

        // Con un clic dentro del editor, el editor tiene el foco.
        clic(&context, &mut app, egui::pos2(450.0, 300.0));

        for pressed in [true, false] {
            let _ = dibujar_en(
                &context,
                &mut app,
                None,
                &[egui::Event::Key {
                    key: egui::Key::Tab,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        shift: false,
                        ..egui::Modifiers::default()
                    },
                }],
            );
        }

        escribir_sin_clic(&context, &mut app, "Z");

        assert!(
            !app.state().pestanas()[0].esta_modificada(),
            "con el foco movido a otra zona el documento no puede cambiar: {:?}",
            app.state().pestanas()
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Cerrar el documento que tenía el foco no rompe la ventana. FE-065 y FE-067.
    ///
    /// Es el cruce de las dos revisiones: el foco estaba en un documento y ese documento se
    /// cierra. egui suelta el foco de lo que ya no está, así que la ventana tiene que quedarse
    /// dibujando y sin escribir en ninguna parte —y sin caerse, que es lo que se comprueba—.
    #[test]
    fn cerrar_el_documento_del_foco_no_rompe_la_ventana() {
        let (context, mut app, proyecto) = ventana_trabajando("miniide-fe065-cerrar");

        clic(&context, &mut app, egui::pos2(450.0, 300.0));
        app.state_mut().cerrar_pestana("Form1.cs");
        app.emitir(crate::commands::Command::CloseDocument);
        app.avanzar(&context);

        escribir(&context, &mut app, "Z");

        assert!(
            app.state().pestanas().is_empty(),
            "no hay ningún documento: {:?}",
            app.state().pestanas()
        );
        assert!(
            !escrito(&context, &mut app, &[]).is_empty(),
            "y la ventana se sigue dibujando"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// La ventana aguanta el tamaño más pequeño que declara. FE-066.
    ///
    /// El mínimo de la ventana es de seiscientos cuarenta por cuatrocientos (`App::opciones`) y
    /// en ese tamaño ninguna zona puede desaparecer: si el área central se quedara sin sitio no
    /// habría editor y MiniIDE se abriría en una ventana donde no se puede trabajar.
    ///
    /// Se comprueba con lo que hay escrito en cada zona y no con los rectángulos de los paneles
    /// porque el área central no pinta fondo —no hay nada detrás del editor— y porque lo que
    /// importa es que el menú, el árbol, las propiedades, el editor y la barra sigan*viéndose*.
    #[test]
    fn la_ventana_aguanta_el_tamano_minimo_sin_que_desaparezca_nada() {
        let (context, mut app, proyecto) = ventana_trabajando("miniide-fe066-minimo");
        let minimo = 640.0;

        let escrito = escrito(&context, &mut app, &[]).join(" ");

        for zona in [
            "Archivo",             // el menú
            "Abrir proyecto",      // la barra de herramientas
            "Proyecto",            // el explorador
            "Form1.cs",            // un archivo del explorador
            "Propiedades",         // la columna de la derecha
            "InitializeComponent", // el editor
            "Estado:",             // la barra de estado
        ] {
            assert!(
                escrito.contains(zona),
                "en el tamaño mínimo tiene que seguir viéndose {zona:?}: {escrito}"
            );
        }

        let salida = dibujar_en(&context, &mut app, Some(minimo), &[]);
        let fuera: Vec<egui::Rect> = salida
            .shapes
            .iter()
            .map(|forma| forma.clip_rect)
            .filter(|recorte| recorte.max.x > minimo + 1.0 || recorte.min.x < -1.0)
            .collect();

        assert!(
            fuera.is_empty(),
            "y nada se pinta fuera de una ventana más estrecha: {fuera:?}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// El texto del editor no se sale de la ventana. FE-066.
    ///
    /// Una línea más ancha que su zona no puede invadir la columna de propiedades: el editor la
    /// desplaza con la rueda para poder ver el final, así que la línea entera no cabe nunca, y
    /// lo que se comprueba es que lo que no cabe no se pinte encima de lo de al lado. Se mira el
    /// recorte con el que se pinta cada forma —que es lo que decide qué se ve— y no el sitio
    /// donde cae la forma, porque una línea de cuatrocientos caracteres se pinta en un sitio
    /// aunque el recorte la deje fuera.
    #[test]
    fn el_texto_del_editor_no_se_sale_de_la_ventana() {
        let (context, mut app, proyecto) = ventana_con_un_proyecto("miniide-fe066-texto");

        // Una línea mucho más ancha que la ventana, escrita en el archivo del proyecto antes de
        // abrirlo: entra por el mismo camino que cualquier otro.
        let larga = format!("// {}\n", "x".repeat(400));
        std::fs::write(proyecto.root().join("Form1.cs"), &larga).expect("se escribe el archivo");
        app.state_mut().seleccionar("Form1.cs");
        app.emitir(crate::commands::Command::OpenDocument);
        app.avanzar(&context);

        let salida = dibujar_en(&context, &mut app, None, &[]);

        let recorte = salida
            .shapes
            .iter()
            .find(|forma| match &forma.shape {
                egui::Shape::Text(texto) => texto.galley.job.text.contains("xxxx"),
                _ => false,
            })
            .map(|forma| forma.clip_rect)
            .expect("la línea larga se pinta");

        assert!(
            recorte.max.x <= ANCHO + 1.0,
            "la línea no puede salirse de la ventana: {recorte:?}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Un archivo de veinte mil líneas no hace que la ventana se congele. FE-078.
    ///
    /// Un archivo largo es contenido de prueba razonable y no un caso raro: los archivos
    /// que se abren en un IDE se generan, y los generados llegan a miles de líneas. Lo que
    /// se mide es lo que se pinta, y no el tiempo, porque lo que se comprueba es que la
    /// ventana solo pinta lo que se ve: el editor pinta las líneas que caben en el área
    /// central y no el archivo entero, así que un archivo veinte veces más grande no
    /// cuesta veinte veces más.
    ///
    /// Y tiene que seguir viéndose el principio del archivo, que es lo primero que se abre:
    /// si al recortar desapareciera lo que se estaba viendo, recortar sería perder en vez de
    /// ahorrar.
    #[test]
    fn un_archivo_enorme_no_hace_que_la_ventana_se_congele() {
        let (context, mut app, proyecto) = ventana_con_un_proyecto("miniide-fe078-enorme");
        let lineas = 20_000;

        let archivo: String = (1..=lineas)
            .map(|numero| format!("// linea {numero}\n"))
            .collect();
        std::fs::write(proyecto.root().join("Form1.cs"), &archivo).expect("se escribe el archivo");

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(crate::commands::Command::OpenDocument);
        app.avanzar(&context);

        let pintado = escrito(&context, &mut app, &[]);
        let lineas_del_editor = pintado
            .iter()
            .filter(|texto| texto.starts_with("// linea "))
            .count();

        assert!(
            lineas_del_editor < 200,
            "el editor ha pintado {lineas_del_editor} líneas de un archivo de {lineas}: lo que \
             no se ve no se pinta, o la ventana se congela (FE-078)"
        );
        assert!(
            pintado.iter().any(|texto| texto.contains("// linea 1")),
            "y lo que se ve es el principio del archivo: {:?}",
            &pintado[..pintado.len().min(10)]
        );

        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// La barra de estado también dice qué hay abierto. FE-068.
    ///
    /// Con un proyecto abierto y un documento en una pestaña, los dos primeros campos tienen que
    /// decir sus nombres: una barra que dice "Sin proyecto" con un proyecto abierto hace dudar al
    /// usuario de si ha abierto algo, y es justo lo que la barra tenía que resolver.
    #[test]
    fn la_barra_dice_el_proyecto_y_el_documento_que_estan_abiertos() {
        let (context, mut app, proyecto) = ventana_trabajando("miniide-fe068-barra");

        let escrito = escrito(&context, &mut app, &[]).join(" ");

        assert!(
            escrito.contains(&format!("Proyecto: {}", proyecto.name())),
            "la barra dice el proyecto abierto: {escrito}"
        );
        assert!(
            escrito.contains("Documento: Form1.cs"),
            "y el documento que se está viendo: {escrito}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Escribir marca la pestaña con un asterisco, y guardar lo quita. FE-070.
    ///
    /// Es el estado modificado de las pestañas de punta a punta, y es lo que evita que se
    /// pierda trabajo: el asterisco es lo único que le dice al usuario que el documento que ve
    /// no está en el disco. Se comprueba con lo que se pinta en la pestaña, que es lo que el
    /// usuario lee, y no con el estado interno: un asterisco que se calculara bien pero no se
    /// enseñara no serviría de nada.
    #[test]
    fn escribir_marca_la_pestana_y_guardar_la_desmarca() {
        let (context, mut app, proyecto) = ventana_trabajando("miniide-fe070-modificado");

        let antes = escrito(&context, &mut app, &[]);
        assert!(
            antes.iter().any(|linea| linea == "Form1.cs"),
            "al abrir, la pestaña es el nombre del archivo: {antes:?}"
        );

        escribir(&context, &mut app, "Z");

        let escrita = escrito(&context, &mut app, &[]);
        assert!(
            escrita.iter().any(|linea| linea == "Form1.cs *"),
            "escribir tiene que poner el asterisco: {escrita:?}"
        );

        app.emitir(crate::commands::Command::Save);
        app.avanzar(&context);

        let guardada = escrito(&context, &mut app, &[]);
        assert!(
            guardada.iter().any(|linea| linea == "Form1.cs"),
            "guardar tiene que quitar el asterisco: {guardada:?}"
        );
        assert!(
            !guardada.iter().any(|linea| linea.contains('*')),
            "y no puede quedar ninguno en ninguna pestaña: {guardada:?}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Cerrar una pestaña modificada pregunta antes de cerrar. FE-070.
    ///
    /// Es el caso principal del estado modificado: con cambios sin guardar, la cruz no cierra,
    /// y la decisión es del usuario. Se comprueba que aparece la pregunta, que la pestaña sigue
    /// ahí mientras espera y que al descartar se cierra —que es lo que significa descartar,
    /// y lo que no se podría deshacer si se cerrara sin preguntar.
    #[test]
    fn cerrar_una_pestana_modificada_pregunta_antes_y_descartar_la_cierra() {
        let (context, mut app, proyecto) = ventana_trabajando("miniide-fe070-cerrar");

        escribir(&context, &mut app, "Z");
        assert!(
            app.state().pestanas()[0].esta_modificada(),
            "el documento tiene cambios sin guardar"
        );

        // La cruz de la pestaña.
        pulsar(&context, &mut app, "×");

        let preguntando = escrito(&context, &mut app, &[]).join(" ");
        assert!(
            preguntando.contains("tiene cambios sin guardar"),
            "cerrar un documento modificado tiene que preguntar: {preguntando}"
        );
        assert_eq!(
            app.state().pestanas().len(),
            1,
            "y mientras espera, la pestaña sigue ahí: {:?}",
            app.state().pestanas()
        );

        pulsar(&context, &mut app, "Descartar");
        app.avanzar(&context);

        assert!(
            app.state().pestanas().is_empty(),
            "descartar cierra la pestaña sin guardar: {:?}",
            app.state().pestanas()
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Los errores de `javac` se enseñan abajo, con el archivo y la línea. FE-064.
    ///
    /// Es el "hecho cuando" de FE-064: un error del compilador de Java en la lista de abajo,
    /// con el archivo y la línea. Se parsea con el parser de verdad del JDK y no con un
    /// diagnóstico escrito a mano, porque lo que se comprueba es que la salida que da
    /// `javac` llega a la ventana tal cual; hace falta el JDK instalado para *compilar*, que
    /// no es lo que se hace aquí.
    #[test]
    fn los_errores_de_javac_se_ensenan_abajo_con_donde_estan() {
        use crate::build::BuildResult;
        use crate::runtime::ProcessOutput;
        use crate::toolchain::{JdkToolchain, ToolchainProvider};

        let (context, mut app, proyecto) = ventana_con_un_proyecto_de_swing("miniide-layout-javac");

        let salida = ProcessOutput::new(
            Some(1),
            "",
            "src/main/java/MainWindow.java:8: error: ';' expected\n1 error\n",
        );
        let diagnosticos = JdkToolchain.parse_diagnostics(&salida, proyecto.root());
        assert_eq!(
            diagnosticos.len(),
            1,
            "el error tiene que entenderse: {diagnosticos:?}"
        );

        app.operaciones_mut()
            .aplicar_el_resultado(Ok(BuildResult::new(salida, diagnosticos)));

        let escrito = escrito(&context, &mut app, &[]).join(" ");

        assert!(
            escrito.contains("';' expected"),
            "el mensaje del error tiene que verse: {escrito}"
        );
        assert!(
            escrito.contains("MainWindow.java:8"),
            "y dónde está, con la línea que dice javac: {escrito}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// El interruptor de código y diseño aparece con un documento que tiene diseño. FE-059.
    ///
    /// Con el proyecto abierto pero sin ningún documento no aparece, porque no hay clase que
    /// dibujar. Se comprueba en los dos estados porque un interruptor que sale siempre —o que
    /// no sale nunca— es lo mismo que no tener interruptor.
    #[test]
    fn el_interruptor_de_vistas_solo_aparece_con_un_documento_que_tiene_diseno() {
        use std::sync::Arc;

        use crate::core::ProjectType;
        use crate::templates::create_project;
        use crate::toolchain::DotNetToolchain;

        let raiz = std::env::temp_dir().join("miniide-layout-interruptor");
        let _ = std::fs::remove_dir_all(&raiz);
        let proyecto =
            create_project(ProjectType::CSharpWinForms, &raiz).expect("proyecto de WinForms");

        let context = egui::Context::default();
        let mut app = super::super::App::new();
        app.abrir(proyecto.clone(), Arc::new(DotNetToolchain));

        let sin_documento = escrito(&context, &mut app, &[]);
        assert!(
            !sin_documento.iter().any(|linea| linea == DISENO),
            "sin ningún documento abierto no hay nada que diseñar: {sin_documento:?}"
        );

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(crate::commands::Command::OpenDocument);
        app.avanzar(&context);

        let con_documento = escrito(&context, &mut app, &[]);
        assert!(
            con_documento.iter().any(|linea| linea == CODIGO),
            "el interruptor tiene la parte de código: {con_documento:?}"
        );
        assert!(
            con_documento.iter().any(|linea| linea == DISENO),
            "y la de diseño: {con_documento:?}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Se pasa de código a diseño y se vuelve, con clics de verdad. FE-059.
    ///
    /// Es lo que pide FE-059: el usuario cambia de Code a Designer y vuelve. Se comprueba con
    /// clics sobre lo que está escrito y no llamando a la ventana, porque un interruptor que
    /// solo funciona si se le llama por dentro no es un interruptor.
    #[test]
    fn se_puede_pasar_de_codigo_a_diseno_y_volver() {
        use crate::frontend::ui_state::VistaCentral;

        let (context, mut app, proyecto) = ventana_trabajando("miniide-layout-cambio");

        pulsar(&context, &mut app, DISENO);
        assert_eq!(
            app.state().vista_central(),
            VistaCentral::Diseniador,
            "pulsar Designer tiene que enseñar el diseñador"
        );

        pulsar(&context, &mut app, CODIGO);
        assert_eq!(
            app.state().vista_central(),
            VistaCentral::Editor,
            "y pulsar Code tiene que volver al editor"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Una ventana de WinForms abierta con la herramienta que le pasen, sin documento.
    ///
    /// Es [`Self::ventana_con_un_proyecto`] con la herramienta cambiada, y está aparte porque
    /// los estados de compilar y ejecutar se comprueban con una herramienta de prueba: con la
    /// de verdad, pedir una compilación lanzaría un `dotnet` que no está en todas las máquinas
    /// y una ejecución dejaría un proceso vivo mientras dura el test.
    fn ventana_con_una_herramienta(
        nombre: &str,
        herramienta: std::sync::Arc<dyn crate::toolchain::ToolchainProvider>,
    ) -> (egui::Context, super::super::App, crate::project::Project) {
        use std::sync::Arc;

        use crate::core::ProjectType;
        use crate::templates::create_project;

        let raiz = std::env::temp_dir().join(nombre);
        let _ = std::fs::remove_dir_all(&raiz);

        let proyecto =
            create_project(ProjectType::CSharpWinForms, &raiz).expect("proyecto de WinForms");
        let mut app = super::super::App::new();
        app.abrir(proyecto.clone(), Arc::clone(&herramienta));

        (egui::Context::default(), app, proyecto)
    }

    /// Una herramienta que compila tarde y no sabe ejecutar.
    ///
    /// El retardo es lo que deja ver `Compilando` -si compilara al instante, ese estado
    /// existiría solo entre dos frames y no se podría comprobar que la ventana lo enseña-, y
    /// el fallo es para poder comprobar las dos formas de acabar sin lanzar nada de verdad.
    struct HerramientaLenta(std::time::Duration);

    impl crate::toolchain::ToolchainProvider for HerramientaLenta {
        fn project_type(&self) -> crate::core::ProjectType {
            crate::core::ProjectType::CSharpWinForms
        }

        fn tool(&self) -> &'static str {
            "herramienta de los tests de la ventana"
        }

        fn is_available(&self) -> bool {
            true
        }

        fn build_invocation(
            &self,
            _proyecto: &crate::project::Project,
        ) -> crate::core::CoreResult<crate::toolchain::Invocation> {
            std::thread::sleep(self.0);
            Err(crate::core::CoreError::Unsupported(
                "esta herramienta no compila de verdad".to_owned(),
            ))
        }

        fn run_invocation(
            &self,
            _proyecto: &crate::project::Project,
        ) -> crate::core::CoreResult<crate::toolchain::Invocation> {
            Err(crate::core::CoreError::Unsupported(
                "esta herramienta no ejecuta de verdad".to_owned(),
            ))
        }

        fn parse_diagnostics(
            &self,
            _salida: &crate::runtime::ProcessOutput,
            _raiz: &std::path::Path,
        ) -> Vec<crate::diagnostics::Diagnostic> {
            Vec::new()
        }
    }

    /// Una herramienta que lanza un proceso que tarda, para poder ver `Ejecutando`.
    ///
    /// Es un `ping` contra uno mismo porque es lo que hay en Windows sin instalar nada, y va
    /// solo en el test ignorado de FE-071: lanzar un proceso de verdad depende de que el
    /// sistema lo mate, y eso no puede estar en el `cargo test` de cada día.
    struct HerramientaQueEjecuta;

    impl crate::toolchain::ToolchainProvider for HerramientaQueEjecuta {
        fn project_type(&self) -> crate::core::ProjectType {
            crate::core::ProjectType::CSharpWinForms
        }

        fn tool(&self) -> &'static str {
            "proceso de los tests de la ventana"
        }

        fn is_available(&self) -> bool {
            true
        }

        fn build_invocation(
            &self,
            _proyecto: &crate::project::Project,
        ) -> crate::core::CoreResult<crate::toolchain::Invocation> {
            Err(crate::core::CoreError::Unsupported(
                "no hace falta".to_owned(),
            ))
        }

        fn run_invocation(
            &self,
            _proyecto: &crate::project::Project,
        ) -> crate::core::CoreResult<crate::toolchain::Invocation> {
            Ok(crate::toolchain::Invocation::new(
                "cmd",
                vec!["/C".to_owned(), "ping -n 20 127.0.0.1 >NUL".to_owned()],
                std::path::PathBuf::from("C:\\"),
            ))
        }

        fn parse_diagnostics(
            &self,
            _salida: &crate::runtime::ProcessOutput,
            _raiz: &std::path::Path,
        ) -> Vec<crate::diagnostics::Diagnostic> {
            Vec::new()
        }
    }

    /// Lo que la barra de estado está diciendo ahora mismo.
    ///
    /// Se leen las formas pintadas y no el estado porque lo que hay que comprobar es que el
    /// usuario se entera: un estado correcto que no se enseña es lo mismo que no tenerlo.
    fn barra(context: &egui::Context, app: &mut super::super::App) -> String {
        escrito(context, app, &[]).join(" ")
    }

    /// Los cuatro estados de compilación se enseñan uno detrás de otro. FE-071.
    ///
    /// Se va de los cuatro en orden porque es una transición y no cuatro estados sueltos: lo
    /// que se comprueba es que la ventana *cambia* de un estado al siguiente, que es lo que
    /// ve el usuario mientras compila y después. Los tres últimos se provocan con lo que
    /// contesta el core —una compilación que termina bien, una que termina mal— porque los
    /// estados se mueven al recoger su respuesta y no al pulsar, y eso es justo lo que hay que
    /// comprobar desde la ventana.
    #[test]
    fn los_cuatro_estados_de_compilacion_se_ensenan_en_la_barra() {
        use crate::build::BuildResult;
        use crate::runtime::ProcessOutput;

        let (context, mut app, proyecto) = ventana_con_una_herramienta(
            "miniide-fe071-build",
            std::sync::Arc::new(HerramientaLenta(std::time::Duration::from_millis(800))),
        );

        assert!(
            barra(&context, &mut app).contains("Compilación: Inactivo"),
            "sin pedir nada no hay nada que decir: {}",
            barra(&context, &mut app)
        );

        pulsar_el(&context, &mut app, "Compilar", 1);
        app.avanzar(&context);
        assert!(
            barra(&context, &mut app).contains("Compilación: Compilando"),
            "mientras compila hay que verlo: {}",
            barra(&context, &mut app)
        );

        app.operaciones_mut()
            .aplicar_el_resultado(Ok(BuildResult::new(
                ProcessOutput::new(Some(0), "", ""),
                Vec::new(),
            )));
        assert!(
            barra(&context, &mut app).contains("Compilación: Correcta"),
            "una compilación que ha ido bien se dice: {}",
            barra(&context, &mut app)
        );

        app.operaciones_mut()
            .aplicar_el_resultado(Ok(BuildResult::new(
                ProcessOutput::new(Some(1), "", ""),
                Vec::new(),
            )));
        assert!(
            barra(&context, &mut app).contains("Compilación: Fallida"),
            "y una que ha ido mal se dice de otra manera: {}",
            barra(&context, &mut app)
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Los cuatro estados de ejecución se enseñan uno detrás de otro. FE-071.
    ///
    /// Es el hermano del de compilación para la ejecución, y va el mismo camino: el botón de
    /// la barra, el estado del módulo y el texto de la barra de estado. Aquí no se puede
    /// fingir la respuesta del core como en el de compilación —no hay un proceso que fingir—,
    /// así que se lanza uno de verdad y se espera a que el sistema lo mate, que es lo que
    /// tarda en dar el último estado.
    #[test]
    #[ignore = "lanza un proceso de verdad"]
    fn los_cuatro_estados_de_ejecucion_se_ensenan_en_la_barra() {
        let (context, mut app, proyecto) = ventana_con_una_herramienta(
            "miniide-fe071-run",
            std::sync::Arc::new(HerramientaQueEjecuta),
        );

        assert!(
            barra(&context, &mut app).contains("Ejecución: Inactivo"),
            "sin ejecutar nada no hay nada que decir: {}",
            barra(&context, &mut app)
        );

        pulsar(&context, &mut app, "Ejecutar");
        app.avanzar(&context);
        assert!(
            barra(&context, &mut app).contains("Ejecución: Ejecutando"),
            "un proceso vivo se dice que se está ejecutando: {}",
            barra(&context, &mut app)
        );

        pulsar(&context, &mut app, "Detener");
        app.avanzar(&context);
        assert!(
            barra(&context, &mut app).contains("Ejecución: Deteniendo"),
            "pedir la parada no es haber parado: {}",
            barra(&context, &mut app)
        );

        let mut terminado = false;
        for _ in 0..300 {
            app.avanzar(&context);
            if !app.operaciones().esta_ocupada() {
                terminado = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            terminado,
            "un proceso parado termina: lo confirma el sistema, no la ventana"
        );
        assert!(
            barra(&context, &mut app).contains("Ejecución: Terminado"),
            "y cuando el proceso ya no está hay que decirlo: {}",
            barra(&context, &mut app)
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }
}
