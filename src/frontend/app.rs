//! La ventana mínima de MiniIDE.
//!
//! Es la ventana de verdad, de eframe, con su ciclo de ejecución. A partir de aquí
//! MiniIDE se abre en una ventana y se cierra como cualquier otra.
//!
//! Lo que hay dentro es lo mínimo: un panel central con el nombre de la aplicación.
//! El contenido de verdad lo traen las siguientes tareas, y cada una lo añade en su
//! sitio (layout en FE-005, menú en FE-006, estado visual en FE-004).
//!
//! Dibujar y abrir están separados a propósito: `ventana` pinta en un `Ui` y se puede
//! recorrer en un test sin abrir nada, mientras que `run` es lo único que abre la
//! ventana de verdad. Así el contenido de la interfaz se puede probar casi entero sin
//! ventana, y lo único que hay que mirar a mano es que la ventana aparezca.

use std::sync::Arc;

use eframe::egui;

use super::atajos;
use super::icon::icono;
use super::layout::layout;
use super::ui_state::UiState;
use crate::commands::Command;

/// Titulo de la ventana.
///
/// Es el nombre de la aplicación y no el del binario ni el de un módulo, porque es lo
/// que el usuario ve en la barra de tareas y en el conmutador de ventanas.
pub fn titulo() -> &'static str {
    crate::APP_NAME
}

/// Como se abre la ventana.
///
/// El tamaño se fija aquí y se deja que se pueda redimensionar: sin tamaño, cada
/// equipo abriría MiniIDE con el que le toque, y una ventana mínima que cambia de
/// tamaño no es una ventana.
///
/// El mínimo es más pequeño que el inicial a propósito: abrir pequeño en una pantalla
/// pequeña y no impedir agrandar nunca.
pub fn opciones() -> eframe::NativeOptions {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(titulo())
        .with_inner_size([1100.0, 700.0])
        .with_min_inner_size([640.0, 400.0]);

    // El icono se pone en el campo y no con `with_icon` porque el logo puede no
    // cargar, y `with_icon` no admite "sin icono". Si no hay logo, no hay icono, que es
    // como tiene que abrir un IDE al que se le ha roto la marca: con el de egui, no
    // con un rectangulo en blanco.
    viewport.icon = icono().map(Arc::new);

    eframe::NativeOptions {
        viewport,
        // Por defecto eframe sigue vivo despues de cerrar la ventana, que es lo que
        // hace una aplicacion que se puede volver a abrir. Un IDE no es eso: se cierra
        // la ventana y se cierra MiniIDE, o el proceso se queda ahi sin ventana y sin
        // forma de cerrarlo.
        run_and_return: false,
        ..eframe::NativeOptions::default()
    }
}

/// La aplicación: la ventana de MiniIDE y su estado visual.
///
/// El estado visual vive aquí y no en la ventana, porque egui redibuja miles de veces
/// por segundo y cada redibujado es una llamada a `ui`: lo que tiene que recordarse
/// entre frames —qué panel está abierto, qué se está viendo, qué dice la barra de
/// estado— necesita vivir en la aplicación, que es la misma para todos.
///
/// Lo que lleva es estado visual y lo que el usuario ha pedido, y nada más. El texto de un documento, el proyecto o
/// el estado de una compilación no están aquí: se piden al core cuando hacen falta.
/// Guardarlos sería tener dos verdades, y la copia se quedaría vieja sin que nadie se
/// enterase.
#[derive(Debug, Default)]
pub struct App {
    estado: UiState,
    /// Los comandos que el usuario ha pedido y que todavía no ha ejecutado nadie.
    ///
    /// Los deja [`Self::emitir`], que es el único sitio por el que se piden. Se guardan
    /// porque hace falta un sitio al que lleguen: sin este campo la ventana los soltaría
    /// al vacío y no habría forma de saber que se ha pulsado un botón.
    peticiones: Vec<Command>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    /// El estado visual de la ventana.
    pub fn state(&self) -> &UiState {
        &self.estado
    }

    /// El estado visual, para cambiarlo.
    pub fn state_mut(&mut self) -> &mut UiState {
        &mut self.estado
    }

    /// El único camino por el que un comando sale de la ventana.
    ///
    /// Todo lo que se pulsa pasa por aquí: el menú, la barra de herramientas y los atajos
    /// de FE-029 a FE-033. Que haya un solo sitio del que salir es lo que evita que el
    /// mismo nombre pida dos cosas distintas según por dónde se pulse, y lo que hace que
    /// un atajo pueda ejecutar lo mismo que un botón sin escribir la operación dos veces.
    ///
    /// Hoy no ejecuta nada: la ventana todavía no tiene un core al que mandarle el
    /// comando, y sin core no hay a quién preguntarle. Lo que hace es dejar el comando
    /// escrito, que es la forma de que la operación no se pierda. Cuando el core llegue, se
    /// llama aquí, y todo lo demás —el menú, la barra y los atajos— sigue igual porque nunca
    /// supieron quién lo iba a ejecutar.
    pub fn emitir(&mut self, comando: Command) {
        self.peticiones.push(comando);
    }

    /// Dibuja la ventana y deja escritas las peticiones que se hayan hecho en ella.
    pub fn dibujar(&mut self, ui: &mut egui::Ui) {
        ventana(ui, self);
    }

    /// Lo que el usuario ha pedido desde la ventana y nadie ha ejecutado todavía.
    pub fn peticiones(&self) -> &[Command] {
        &self.peticiones
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.dibujar(ui);
    }
}

/// Dibuja el contenido de la ventana.
///
/// Ahora es el layout raíz y nada más: las cuatro zonas de la ventana, cada una en su
/// sitio. El contenido de cada zona lo pone su tarea -el menú, el editor, la salida, la
/// barra de estado- y lo que va aquí es lo único que no depende de lo que haya dentro.
///
/// Recibe la aplicación y no un estado suelto porque las zonas pueden pedir cosas -FE-007,
/// FE-006, FE-009- y la que recoge esas peticiones es la aplicación. Pasarle el estado
/// visual obligaría a que cada zona tenga su propio sitio donde dejar lo que pide.
pub fn ventana(ui: &mut egui::Ui, app: &mut App) {
    atajos::manejar(ui, app);
    layout(ui, app);
}

/// Abre la ventana y no vuelve hasta que se cierra.
///
/// La ventana se abre desde aquí y no desde el core: el core no sabe que existe una
/// interfaz, y por eso sigue pudiendo probarse sin abrir nada.
///
/// Devuelve el error si la ventana no se puede abrir, para que quien la llama pueda
/// contarlo. Una ventana que no abre y no dice nada parece un IDE que no funciona.
pub fn run() -> eframe::Result {
    eframe::run_native(
        titulo(),
        opciones(),
        Box::new(|_creation| Ok(Box::new(App::new()) as Box<dyn eframe::App>)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::icon::LADO;

    /// La ventana se puede dibujar sin ventana.
    ///
    /// Es lo que permite probar el contenido de la interfaz sin abrir nada: egui
    /// sabe ejecutar un frame con un contexto propio, asi que lo que se dibuja se
    /// puede recorrer en un test. Lo que no se comprueba asi es que la ventana
    /// aparezca en la pantalla y se pueda cerrar, que se mira a mano.
    #[test]
    fn the_window_content_can_be_drawn_without_a_window() {
        let context = eframe::egui::Context::default();

        let mut frame = context.run_ui(eframe::egui::RawInput::default(), |ui| {
            ventana(ui, &mut App::new());
        });

        // egui avisa si se tiran sin aplicar las texturas que ha creado, y en una
        // aplicacion de verdad las aplica el renderizador. Aqui no hay renderizador,
        // asi que se vacian a proposito: lo que se prueba es que el contenido se
        // dibuja, no los pixeles.
        frame.textures_delta.clear();

        assert!(
            frame.pixels_per_point > 0.0,
            "un frame dibujado tiene que decir a que escala se dibuja"
        );
    }

    /// La ventana se titula con el nombre de la aplicacion, y no con el del binario
    /// ni con el de un modulo: es lo que el usuario ve en la barra de tareas.
    #[test]
    fn the_window_is_titled_with_the_name_of_the_application() {
        assert_eq!(titulo(), crate::APP_NAME);
    }

    /// La ventana tiene un tamano, porque si no la decide el sistema y MiniIDE
    /// abriria con la que le salga a cada equipo.
    #[test]
    fn the_window_starts_with_a_size() {
        let options = opciones();
        let size = options
            .viewport
            .inner_size
            .expect("la ventana tiene que tener un tamano");

        assert!(
            size.x > 0.0 && size.y > 0.0,
            "el tamano tiene que servir: {size:?}"
        );
    }

    /// La ventana lleva el logo de la aplicacion.
    ///
    /// El icono de la ventana y el del ejecutable son cosas distintas: este lo pone
    /// egui, y el otro lo incrusta la compilacion. Los dos vienen del mismo PNG, y
    /// este es el que se ve en la barra de tareas mientras MiniIDE esta abierto.
    #[test]
    fn the_window_carries_the_logo_as_its_icon() {
        let icon = opciones()
            .viewport
            .icon
            .expect("la ventana tiene que llevar el logo");

        assert_eq!((icon.width, icon.height), (LADO, LADO));
        assert!(
            icon.rgba.iter().any(|byte| *byte != 0),
            "un icono en blanco no es el logo"
        );
    }

    /// La aplicacion lleva el estado visual, y solo ese.
    ///
    /// egui redibuja la ventana muchas veces por segundo y la misma aplicacion
    /// atiende a todas: si el estado visual no vive en ella, cada frame tendria que
    /// inventarselo otra vez y no habria forma de que un panel siguiera donde se
    /// quedo. Y si en vez del estado visual lleva el estado del core, la ventana deja
    /// de mirar y pasa a ser la fuente de verdad, que es justo lo que no puede pasar.
    #[test]
    fn the_app_carries_the_visual_state() {
        let mut app = App::new();

        assert_eq!(
            app.state().status(),
            None,
            "una ventana nueva no tiene estado"
        );

        app.state_mut().set_status("Compilando...");

        assert_eq!(
            app.state().status(),
            Some("Compilando..."),
            "el estado visual se conserva entre frames: es de la aplicacion, no del frame"
        );
    }

    /// La aplicacion es una aplicacion de eframe, y no un tipo suelto: sin esto el
    /// contenido no llega a dibujarse nunca.
    ///
    /// Que este test compile ya es la comprobacion: `eframe::App` es lo que
    /// `run_native` necesita para dibujar y para cerrar.
    #[test]
    fn the_app_is_an_eframe_app() {
        fn es_eframe_app<T: eframe::App>() {}

        es_eframe_app::<App>();
    }

    /// La aplicación guarda lo que el usuario pide, y no inventa peticiones.
    ///
    /// La aplicación recoge las peticiones y no las ejecuta todavía, y no las inventa: sin
    /// un clic detrás no hay nada que recoger, y recoger algo por la espalda sería mandar
    /// trabajo al core que el usuario no ha pedido.
    #[test]
    fn the_app_keeps_what_the_window_asks_for() {
        let context = eframe::egui::Context::default();
        let mut app = App::new();

        let mut frame = context.run_ui(eframe::egui::RawInput::default(), |ui| {
            app.dibujar(ui);
        });
        frame.textures_delta.clear();

        assert!(
            app.peticiones().is_empty(),
            "una ventana en la que no se ha pulsado nada no pide nada: {:?}",
            app.peticiones()
        );
    }

    /// Los comandos de la ventana salen por un solo sitio.
    ///
    /// FE-009 pide un punto único, y lo que lo protege de volverse dos es que el menú y la
    /// barra de herramientas usen el mismo. Se comprueba leyendo el código y no la ventana
    /// porque la duplicación no se ve desde fuera: si el menú se dibujara sus propios
    /// botones, los dos caminos pedirían lo mismo y los dos funcionarían, y el día que uno
    /// cambiara el otro se quedaría atrás sin que nada lo notara.
    #[test]
    fn the_commands_of_the_window_leave_through_one_place() {
        let frontend = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("frontend");

        let fuente = |archivo: &str| {
            std::fs::read_to_string(frontend.join(archivo))
                .unwrap_or_else(|error| panic!("{archivo} tiene que poder leerse: {error}"))
        };

        let define: Vec<String> = [
            "acciones.rs",
            "app.rs",
            "icon.rs",
            "layout.rs",
            "explorador.rs",
            "menu.rs",
            "status.rs",
            "atajos.rs",
            "tabs.rs",
            "toolbar.rs",
            "ui_state.rs",
        ]
        .into_iter()
        .filter(|archivo| fuente(archivo).contains("fn emitir"))
        .map(String::from)
        .collect();

        assert_eq!(
            define,
            vec!["app.rs".to_owned()],
            "`emitir` es el unico sitio por el que un comando sale de la ventana"
        );

        for consumidor in ["menu.rs", "toolbar.rs", "busqueda.rs"] {
            assert!(
                fuente(consumidor).contains("acciones::boton("),
                "{consumidor} tiene que pedir por `acciones::boton`, que es el unico sitio donde un clic se convierte en un comando"
            );
        }

        for consumidor in ["explorador.rs", "tabs.rs"] {
            assert!(
                fuente(consumidor).contains("app.emitir("),
                "{consumidor} tiene que pedir por `App::emitir`, que es el unico sitio por el que un comando sale de la ventana"
            );
        }

        assert!(
            fuente("explorador.rs").contains("acciones::boton("),
            "el explorador pide sus acciones por `acciones::boton` y el resto por `App::emitir`"
        );
    }

    /// Cerrar la ventana cierra MiniIDE.
    ///
    /// eframe sigue vivo despues de cerrar la ventana salvo que se le diga lo
    /// contrario, y en un IDE eso es un fallo: el usuario cierra la ventana y el
    /// proceso se queda ahi, sin ventana y sin forma de cerrarlo.
    ///
    /// Este test viene de mirar la ventana de verdad, no de imaginarlo: se abrio,
    /// se cerro y el proceso seguia.
    #[test]
    fn closing_the_window_closes_the_application() {
        assert!(
            !opciones().run_and_return,
            "cerrar la ventana tiene que cerrar MiniIDE"
        );
    }
}
