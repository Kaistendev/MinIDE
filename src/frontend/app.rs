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
use std::time::Duration;

use eframe::egui;

use super::atajos;
use super::dialogos::Dialogo;
use super::icon::icono;
use super::layout::layout;
use super::operaciones::Operaciones;
use super::ui_state::UiState;
use super::{Diseniador, GestoDelDiseniador};
use crate::commands::Command;
use crate::project::Project;
use crate::toolchain::ToolchainProvider;

/// Cada cuánto mira la ventana si algo ha terminado, en milisegundos.
///
/// Se mira con retardo y noredibujando sin parar porque la ventana no tiene nada que
/// enseñar mientras espera: pedir un repintado por cada frame sin trabajo sería wake up del
/// sistema unas sesenta veces por segundo para pintar lo mismo. Con este retardo la ventana
/// se entera antes de que se note y el equipo no se calienta.
const CADA_CUANTO_MIRA: Duration = Duration::from_millis(50);

/// Título de la ventana.
///
/// Es el nombre de la aplicación y no el del binario ni el de un módulo, porque es lo
/// que el usuario ve en la barra de tareas y en el conmutador de ventanas.
pub fn titulo() -> &'static str {
    crate::APP_NAME
}

/// Lo que dice la barra de estado cuando se pide algo que necesita un proyecto y no hay.
///
/// Lo dice el botón y no el core porque el core no está: todavía no hay a quién preguntarle
/// por el proyecto activo (FE-058), así que la ventana responde por su cuenta. Callarse no
/// valía: un Compilar que no hace nada parece un IDE roto.
const SIN_PROYECTO: &str = "No hay proyecto abierto";

/// Lo que dice la barra de estado cuando hay proyecto pero no se sabe qué herramienta
/// construye.
///
/// En una ventana normal no pasa: FE-058 abre el proyecto y le entrega su herramienta al
/// mismo tiempo. Va escrito aparte para que el caso sea visible si algún día se abre un
/// proyecto sin herramienta en lugar de inventarse una.
const SIN_HERRAMIENTA: &str = "No hay herramienta para este proyecto";

/// Lo que dice la barra de estado mientras hay una compilación en marcha.
const COMPILANDO: &str = "Compilando...";

/// Lo que dice la barra de estado mientras hay una ejecución en marcha.
const EJECUTANDO: &str = "Ejecutando...";

/// Lo que dice la barra de estado desde que se pide parar hasta que el proceso se va.
const DETENIENDO: &str = "Deteniendo...";

/// Lo que dice la barra de estado mientras hay un aviso de que falta la herramienta.
///
/// Es corto porque el mensaje entero lo dice el diálogo, y ponerlo entero en la barra lo
/// partiría en varias líneas y empujaría el resto de los campos. FE-056.
const FALTA_LA_HERRAMIENTA: &str = "Falta la herramienta";

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
/// Lo que lleva es estado visual y lo que el usuario ha pedido, y nada más. El texto de un documento y el
/// estado de una compilación no están en [`UiState`]: el primero se pide al core cuando hace
/// falta, y el segundo se recoge del core cuando contesta. Guardar cualquiera de los dos aquí
/// sería tener dos verdades, y la copia se quedaría vieja sin que nadie se enterase.
#[derive(Default)]
pub struct App {
    estado: UiState,
    /// Los comandos que el usuario ha pedido y que todavía no ha ejecutado nadie.
    ///
    /// Los deja [`Self::emitir`], que es el único sitio por el que se piden. Se guardan
    /// porque hace falta un sitio al que lleguen: sin este campo la ventana los soltaría
    /// al vacío y no habría forma de saber que se ha pulsado un botón.
    peticiones: Vec<Command>,
    /// Lo que MiniIDE tiene compilando o ejecutándose, y cómo se ve (FE-038, FE-039).
    operaciones: Operaciones,
    /// El proyecto abierto, si lo hay.
    ///
    /// Vive aquí y no en `Workspace` porque la ventana es la que consume el core, y porque
    /// de momento solo necesita una cosa de él: qué compilar. FE-058 es la que abre
    /// proyectos de verdad y la que le pasa a la ventana lo que ha abierto.
    proyecto: Option<Project>,
    /// La herramienta que construye y ejecuta `proyecto`.
    ///
    /// La elige quien abre el proyecto y no la ventana: decidir qué toolchain va con qué
    /// tipo de proyecto es del core, y una ventana que hiciera ese `match` tendría dentro
    /// los nombres de los lenguajes, que es justo lo que AGENTS.md §2.4 prohíbe.
    proveedor: Option<Arc<dyn ToolchainProvider>>,
    /// El diseñador: el modelo del core y lo que se ha hecho con él. FE-044.
    ///
    /// Vive en la aplicación y no en `UiState` porque guarda el modelo del diseñador, que es
    /// del core. `UiState` es para lo que no tiene dueño, y este lo tiene.
    diseniador: Diseniador,
}

/// Se enseña lo que la ventana está haciendo, y no la herramienta que tiene debajo.
///
/// El proveedor no se enseña porque es de otra cosa: `ToolchainProvider` no es `Debug`
/// porque no tiene por qué serlo, y hacer que lo fuera obligaría a las herramientas de
/// verdad a justificarse en un `println!`. Lo que interesa al que mira es si hay algo
/// compilando, y eso ya está en las operaciones.
impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("estado", &self.estado)
            .field("peticiones", &self.peticiones)
            .field("operaciones", &self.operaciones)
            .field("proyecto", &self.proyecto.as_ref().map(Project::name))
            .field("diseniador", &self.diseniador)
            .finish()
    }
}

impl App {
    /// Una ventana nueva, sin proyecto abierto y sin nada en marcha.
    ///
    /// Se escribe y no se deja en `default()` porque las operaciones no tienen nada que
    /// inventar: una ventana quieta de verdad es la que empieza sin compilar ni ejecutar,
    /// y dejarlo a `Default` sería dejar que cada campo se inicialice por su cuenta.
    pub fn new() -> Self {
        Self {
            operaciones: Operaciones::new(),
            ..Self::default()
        }
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
    /// de FE-029 a FE-033, y los de compilar, ejecutar y detener de FE-040 a FE-042. Que
    /// haya un solo sitio del que salir es lo que evita que el mismo nombre pida dos cosas
    /// distintas según por dónde se pulse, y lo que hace que un atajo pueda ejecutar lo
    /// mismo que un botón sin escribir la operación dos veces.
    ///
    /// Lo que hace es dejar el comando escrito, no ejecutarlo: el clic se convierte en
    /// comando aquí y se ejecuta en [`Self::avanzar`], que es donde la ventana tiene el core
    /// al que mandárselo. Entre medias solo está la lista de lo pendiente, y [`Self::avanzar`]
    /// la vacía en cuanto la ejecuta.
    pub fn emitir(&mut self, comando: Command) {
        self.peticiones.push(comando);
    }

    /// Abre un proyecto y le entrega a la ventana la herramienta que lo construye.
    ///
    /// Es lo que hace falta para que Compilar y Ejecutar tengan a quién preguntar. FE-058
    /// es la que abre proyectos de verdad, y lo que hace es llamar a esto: a partir de aquí
    /// los botones de la barra dejan de ser peticiones sin destino.
    pub fn abrir(&mut self, proyecto: Project, proveedor: Arc<dyn ToolchainProvider>) {
        self.proyecto = Some(proyecto);
        self.proveedor = Some(proveedor);
    }

    /// Lo que MiniIDE tiene compilando o ejecutándose, y cómo se ve.
    ///
    /// Lo consulta la barra de estado para enseñar los estados de FE-038 y FE-039, y los
    /// tests para comprobar las transiciones.
    pub(crate) fn operaciones(&self) -> &Operaciones {
        &self.operaciones
    }

    /// El diseñador: el modelo del core y lo que se ha hecho con él. FE-044.
    pub(crate) fn diseniador(&self) -> &Diseniador {
        &self.diseniador
    }

    /// El diseñador, para quien tenga que cambiarlo.
    ///
    /// Existe y es público dentro del crate porque el canvas necesita leer el modelo para
    /// pintarlo mientras dibuja. Lo que cambia el modelo no pasa por aquí: eso va por
    /// [`Self::pedir`], que es el único camino que tiene un gesto del diseñador.
    pub(crate) fn diseniador_mut(&mut self) -> &mut Diseniador {
        &mut self.diseniador
    }

    /// Pone un modelo en el diseñador y deja el diseñador vacío de todo lo demás.
    ///
    /// Es lo que llama quien abre el diseñador (FE-059 y FE-063), que es quien sabe qué
    /// formulario se está abriendo.
    pub fn abrir_diseniador(&mut self, modelo: crate::generation::DesignerModel) {
        self.diseniador = Diseniador::con_modelo(modelo);
    }

    /// El único camino por el que un gesto del diseñador llega al modelo. FE-048, FE-052.
    ///
    /// Es hermano de [`Self::emitir`] y va a un sitio distinto: `emitir` deja el comando
    /// para el core y lo ejecuta `avanzar`, porque el core puede tardar; aquí el modelo está
    /// en memoria y aplicarlo es inmediato. Por eso este no espera y el otro sí: guardar un
    /// gesto un frame más solo pondría el modelo un frame por detrás de lo que ve el
    /// usuario.
    ///
    /// Que haya un solo sitio es lo que evita que el modelo se cambie desde el canvas y desde
    /// las propiedades por separado: si cada uno tuviera el suyo, bastaría con que uno se
    /// olvidara de actualizar para que el canvas y el modelo dejaran de cuadrar.
    pub fn pedir(&mut self, gesto: GestoDelDiseniador) -> bool {
        self.diseniador.aplicar(gesto)
    }

    /// Dibuja la ventana y deja escritas las peticiones que se hayan hecho en ella.
    pub fn dibujar(&mut self, ui: &mut egui::Ui) {
        ventana(ui, self);
    }

    /// Lo que el usuario ha pedido desde la ventana y nadie ha ejecutado todavía.
    pub fn peticiones(&self) -> &[Command] {
        &self.peticiones
    }

    /// Lo que pasa entre un frame y el siguiente. FE-043.
    ///
    /// Aquí es donde la ventana ejecuta lo que se le ha pedido y recoge lo que el core ha
    /// terminado, y las dos cosas se hacen sin esperar: por eso se llama a
    /// [`Operaciones::recoger`], que es un `try_recv`, y por eso arrancar una compilación o
    /// una ejecución devuelve en cuanto el trabajo está en marcha. Si esta función esperara
    /// por un resultado, la ventana dejaría de atender el teclado y de redibujarse durante
    /// toda la compilación, que es exactamente lo que FE-043 prohíbe.
    ///
    /// Recoger va antes que ejecutar a propósito. Al revés, una parada se vería como
    /// terminada en el mismo frame en que se pide, y `Deteniendo` —el estado que dice que
    /// se ha pedido pero el sistema aún no ha confirmado— no se vería nunca.
    ///
    /// Al final pide otro repintado si queda algo en marcha, porque egui solo vuelve a
    /// dibujar cuando le llega algo: sin esa petición, una compilación que terminase en
    /// segundo plano se quedaría sin recoger hasta que el usuario tocase algo.
    pub fn avanzar(&mut self, contexto: &egui::Context) {
        self.operaciones.recoger();
        self.ejecutar_peticiones();

        if self.operaciones.esta_ocupada() {
            contexto.request_repaint_after(CADA_CUANTO_MIRA);
        }
    }

    /// Ejecuta lo que se ha pedido desde la ventana y deja la lista vacía.
    ///
    /// Solo ejecutan aquí los comandos que la ventana puede resolver con lo que tiene, que
    /// son los tres de compilar, ejecutar y detener. Los demás se pierden, y se pierden a
    /// propósito: no hay todavía quien los ejecute, así que guardarlos sería acumular
    /// peticiones de cosas que no van a pasar. Cuando el core los ejecute, será una rama más
    /// de este `match` y no un sitio nuevo al que `emitir` tenga que escribir.
    ///
    /// Vaciar la lista aunque no todos los comandos se hayan ejecutado es lo que impide que
    /// una pulsación se cuente para siempre: cada clic se ejecuta una vez, en el frame
    /// siguiente al que se pulsó.
    fn ejecutar_peticiones(&mut self) {
        for comando in std::mem::take(&mut self.peticiones) {
            match comando {
                Command::Build => self.compilar(),
                Command::Run => self.ejecutar(),
                Command::Stop => self.detener(),
                _ => {}
            }
        }
    }

    /// Compila el proyecto abierto.
    ///
    /// Sin proyecto o sin herramienta no hay nada que compilar, y se dice cuál de las dos
    /// cosas falta en lugar de dejar el botón sin efecto: un Compilar que no hace nada
    /// parece un IDE roto, y el usuario no tiene forma de saber qué le falta.
    fn compilar(&mut self) {
        let Some(proyecto) = self.proyecto.clone() else {
            self.estado.set_status(SIN_PROYECTO);
            return;
        };
        let Some(proveedor) = self.proveedor.clone() else {
            self.estado.set_status(SIN_HERRAMIENTA);
            return;
        };
        if !self.avisa_de_que_falta_la_herramienta(proveedor.as_ref()) {
            return;
        }

        self.operaciones.compilar(proveedor, proyecto);
        self.estado.set_status(COMPILANDO);
    }

    /// Ejecuta el proyecto abierto.
    ///
    /// Si la llamada no se puede preparar no se lanza nada, así que la ventana no se pone a
    /// decir que se está ejecutando: lo que hay en ese caso es un proceso que no existe, y
    /// decirlo sería enseñar al usuario algo que no está corriendo.
    fn ejecutar(&mut self) {
        let Some(proyecto) = self.proyecto.clone() else {
            self.estado.set_status(SIN_PROYECTO);
            return;
        };
        let Some(proveedor) = self.proveedor.clone() else {
            self.estado.set_status(SIN_HERRAMIENTA);
            return;
        };
        if !self.avisa_de_que_falta_la_herramienta(proveedor.as_ref()) {
            return;
        }

        match self.operaciones.ejecutar(proveedor.as_ref(), &proyecto) {
            Ok(()) => self.estado.set_status(EJECUTANDO),
            Err(error) => self.estado.set_status(error.to_string()),
        }
    }

    /// Avisa de que la herramienta no está, y dice si se puede seguir. FE-056.
    ///
    /// Se pregunta al proveedor y no se mira el resultado de una compilación fallida porque
    /// lo que se quiere saber es si falta la herramienta o si falló el código del usuario, y
    /// eso solo lo responde quien la busca. Preguntando antes, el aviso sale con el mensaje
    /// del propio proveedor —que es quien sabe cómo se busca— y sin haber lanzado un proceso
    /// que no iba a existir.
    ///
    /// Devuelve si se puede seguir porque el que llama tiene que poder salir sin repetir
    /// la comprobación, y devolver solo el aviso dejaría al que llama sin forma de saber si
    /// sigue o no.
    fn avisa_de_que_falta_la_herramienta(&mut self, proveedor: &dyn ToolchainProvider) -> bool {
        if proveedor.is_available() {
            return true;
        }

        self.estado.set_status(FALTA_LA_HERRAMIENTA);
        self.estado.abrir_dialogo(Dialogo::FaltaHerramienta {
            mensaje: proveedor.missing_message(),
        });

        false
    }

    /// Detiene lo que se esté ejecutando.
    fn detener(&mut self) {
        match self.operaciones.detener() {
            Ok(()) => self.estado.set_status(DETENIENDO),
            Err(error) => self.estado.set_status(error.to_string()),
        }
    }
}

impl eframe::App for App {
    /// Lo que pasa antes de cada frame, y también con la ventana oculta.
    ///
    /// Es el sitio de `logic` y no el de `ui` porque es el único al que eframe llama aunque
    /// la ventana esté tapada, y porque desde aquí se le da a egui el contexto, que es lo que
    /// hace falta para pedir otro repintado. Si esto estuviera en `ui`, con la ventana
    /// minimizada una compilación que terminase se quedaría sin recoger hasta que el usuario
    /// la volviera a abrir.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.avanzar(ctx);
    }

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
    use std::time::Duration;

    use super::*;
    use crate::frontend::icon::LADO;
    use crate::frontend::operaciones::{EstadoDeBuild, EstadoDeRun};

    /// Un proyecto de mentira: estos tests miran la ventana, no sus archivos.
    fn proyecto_de_prueba() -> crate::project::Project {
        use crate::core::ProjectType;
        use crate::project::{BuildConfiguration, ProjectRelativePath};

        crate::project::Project::new(
            "app",
            "C:\\proyectos\\app",
            ProjectType::JavaSwing,
            BuildConfiguration::new(
                ProjectRelativePath::new(BuildConfiguration::DEFAULT_OUTPUT_DIRECTORY)
                    .expect("el directorio de salida por defecto es valido"),
            ),
        )
        .expect("el proyecto de los tests es valido")
    }

    /// Una herramienta que tarda y además falla, para tener compilaciones en marcha.
    ///
    /// Falla porque estos tests no pueden depender de que haya un compilador instalado, y
    /// tarda porque un trabajo que termina al instante no deja nada en marcha que probar.
    /// Devuelve `Arc` porque es lo que [`App::abrir`] se lleva.
    fn herramienta_que_tarda(espera: Duration) -> Arc<dyn ToolchainProvider> {
        use crate::core::{CoreError, ProjectType};
        use crate::diagnostics::Diagnostic;
        use crate::project::Project;
        use crate::runtime::ProcessOutput;
        use crate::toolchain::Invocation;

        struct Lenta(Duration);

        impl ToolchainProvider for Lenta {
            fn project_type(&self) -> ProjectType {
                ProjectType::JavaSwing
            }

            fn tool(&self) -> &'static str {
                "herramienta de los tests"
            }

            fn is_available(&self) -> bool {
                true
            }

            fn build_invocation(&self, _project: &Project) -> crate::core::CoreResult<Invocation> {
                std::thread::sleep(self.0);

                Err(CoreError::Unsupported("de mentira".to_string()))
            }

            fn run_invocation(&self, _project: &Project) -> crate::core::CoreResult<Invocation> {
                Err(CoreError::Unsupported("de mentira".to_string()))
            }

            fn parse_diagnostics(
                &self,
                _output: &ProcessOutput,
                _root: &std::path::Path,
            ) -> Vec<Diagnostic> {
                Vec::new()
            }
        }

        Arc::new(Lenta(espera))
    }

    /// Una herramienta que no está instalada en el equipo.
    ///
    /// Es la que falta en un equipo sin SDK ni JDK, y la que tiene que producir el aviso de
    /// FE-056. Se distingue de [`herramienta_que_tarda`] en lo único que importa: si está o
    /// no está.
    fn herramienta_que_no_esta() -> Arc<dyn ToolchainProvider> {
        use crate::core::{CoreError, ProjectType};
        use crate::diagnostics::Diagnostic;
        use crate::project::Project;
        use crate::runtime::ProcessOutput;
        use crate::toolchain::Invocation;

        struct Ausente;

        impl ToolchainProvider for Ausente {
            fn project_type(&self) -> ProjectType {
                ProjectType::JavaSwing
            }

            fn tool(&self) -> &'static str {
                "herramienta de los tests"
            }

            fn is_available(&self) -> bool {
                false
            }

            fn build_invocation(&self, _project: &Project) -> crate::core::CoreResult<Invocation> {
                panic!("no se prepara la llamada de una herramienta que no está")
            }

            fn run_invocation(&self, _project: &Project) -> crate::core::CoreResult<Invocation> {
                panic!("no se prepara la llamada de una herramienta que no está")
            }

            fn parse_diagnostics(
                &self,
                _output: &ProcessOutput,
                _root: &std::path::Path,
            ) -> Vec<Diagnostic> {
                let _ = CoreError::Unsupported("no hace falta".to_string());
                Vec::new()
            }
        }

        Arc::new(Ausente)
    }

    /// Un frame con una ventana de tamaño conocido.
    fn entrada_de_prueba() -> eframe::egui::RawInput {
        eframe::egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 800.0),
            )),
            ..eframe::egui::RawInput::default()
        }
    }

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

    /// Una herramienta que no está instalada abre el aviso y no compila. FE-056.
    ///
    /// Lo que se mide es que la ventana diga qué falta y que no lance nada: si compila
    /// primero y avisa después, el usuario ve una compilación que no va a salir nunca y un
    /// error que no es el suyo. Y si no avisa, el fallo le llega como un error de compilación
    /// cualquiera, que es justo lo que RF-16 dice que no tiene que pasar.
    #[test]
    fn a_missing_toolchain_opens_a_notice_instead_of_building() {
        let contexto = eframe::egui::Context::default();
        let mut app = App::new();
        app.abrir(proyecto_de_prueba(), herramienta_que_no_esta());

        app.emitir(Command::Build);
        app.avanzar(&contexto);

        assert_eq!(
            app.operaciones().estado_de_build(),
            EstadoDeBuild::Inactivo,
            "sin herramienta no hay nada que compilar"
        );
        let Some(Dialogo::FaltaHerramienta { mensaje }) = app.state().dialogo().cloned() else {
            panic!("tiene que haber un aviso de que falta la herramienta");
        };
        assert!(
            mensaje.contains("herramienta de los tests"),
            "el aviso tiene que decir qué herramienta falta, y lo dice el proveedor: {mensaje:?}"
        );
        assert!(!mensaje.is_empty(), "un aviso vacío no explica nada");
    }

    /// Ejecutar sin herramienta avisa igual que compilar. FE-056.
    ///
    /// Va en su propio test porque son dos botones distintos y basta con que uno avise para
    /// que el usuario piense que el otro funciona.
    #[test]
    fn a_missing_toolchain_stops_the_run_too() {
        let contexto = eframe::egui::Context::default();
        let mut app = App::new();
        app.abrir(proyecto_de_prueba(), herramienta_que_no_esta());

        app.emitir(Command::Run);
        app.avanzar(&contexto);

        assert_eq!(
            app.operaciones().estado_de_run(),
            EstadoDeRun::Inactivo,
            "sin herramienta no hay nada que ejecutar"
        );
        assert!(
            app.state().hay_dialogo_abierto(),
            "y tiene que decir que falta la herramienta"
        );
    }

    /// La barra de estado también avisa, no solo el diálogo. FE-056.
    ///
    /// El diálogo tapa la ventana y se cierra, así que si el aviso viviera solo ahí se
    /// perdería en cuanto el usuario lo cierra; con el texto en la barra el motivo sigue
    /// estando a la vista mientras se trabaja.
    #[test]
    fn el_aviso_de_que_falta_la_herramienta_tambien_se_queda_en_la_barra() {
        let contexto = eframe::egui::Context::default();
        let mut app = App::new();
        app.abrir(proyecto_de_prueba(), herramienta_que_no_esta());

        app.emitir(Command::Build);
        app.avanzar(&contexto);

        assert_eq!(
            app.state().status(),
            Some(FALTA_LA_HERRAMIENTA),
            "la barra tiene que seguir diciendo por qué no se ha compilado"
        );
    }

    /// Avisar de una herramienta que falta no rompe nada de lo que ya estaba. FE-056.
    ///
    /// Es lo de "y no un panic en UI": el aviso se abre en el camino normal de Compilar y
    /// Ejecutar, con el proyecto abierto y todo lo demás en su sitio. Si ese camino se
    /// quedara sin respuesta, la ventana se caería justo cuando el usuario menos puede.
    #[test]
    fn avisar_de_una_herramienta_que_falta_no_rompe_la_ventana() {
        let contexto = eframe::egui::Context::default();
        let mut app = App::new();
        app.abrir(proyecto_de_prueba(), herramienta_que_no_esta());
        app.state_mut().set_status("antes");

        app.emitir(Command::Build);
        app.emitir(Command::Run);
        app.avanzar(&contexto);

        let mut salida = contexto.run_ui(entrada_de_prueba(), |ui| app.dibujar(ui));
        salida.textures_delta.clear();
        assert!(
            !salida.shapes.is_empty(),
            "con un aviso abierto la ventana se sigue dibujando"
        );
        assert_eq!(
            app.operaciones().estado_de_build(),
            EstadoDeBuild::Inactivo,
            "y no ha quedado ninguna compilación a medias"
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
            "operaciones.rs",
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

    /// Una ventana sin proyecto no dice que está compilando ni que se está ejecutando.
    ///
    /// Los dos estados arrancan en `Inactivo` y no se inventan al abrir MiniIDE: si la
    /// ventana dijera que está compilando antes de que nadie haya pulsado nada, el estado
    /// de la barra de estado no valdría para nada, porque no distinguiría "compilando" de
    /// "no compilando nunca".
    #[test]
    fn a_new_window_is_not_building_or_running() {
        let app = App::new();

        assert_eq!(app.operaciones().estado_de_build(), EstadoDeBuild::Inactivo);
        assert_eq!(app.operaciones().estado_de_run(), EstadoDeRun::Inactivo);
        assert!(
            !app.operaciones().esta_ocupada(),
            "una ventana nueva no tiene nada en marcha"
        );
    }

    /// Compilar sin proyecto dice que no hay proyecto, y no finge que compila.
    ///
    /// El botón Compilar está siempre en la barra (FE-040), así que se puede pulsar sin
    /// nada abierto. Lo que no puede ser es quedarse en `Compilando` para siempre: el
    /// usuario vería una compilación que no existe, y no tendría forma de salir de ahí
    /// hasta que abriera un proyecto.
    #[test]
    fn building_without_a_project_says_so_instead_of_pretending() {
        let contexto = eframe::egui::Context::default();
        let mut app = App::new();

        app.emitir(Command::Build);
        app.avanzar(&contexto);

        assert_eq!(
            app.operaciones().estado_de_build(),
            EstadoDeBuild::Inactivo,
            "sin proyecto no hay nada que compilar, y el estado no puede decir que sí"
        );
        assert_eq!(
            app.state().status(),
            Some(SIN_PROYECTO),
            "el usuario tiene que saber por qué no ha pasado nada"
        );
    }

    /// Compilar pide su comando una sola vez, aunque se pulse tres veces.
    ///
    /// Se ejecutan las peticiones pendientes una vez por frame y se vacían, que es lo que
    /// evita que una pulsación se cuente para siempre. Sin vaciarlo, el mismo clic se
    /// ejecutaría en cada repintado y una compilación empezaría tres veces por un botón.
    #[test]
    fn a_command_is_executed_once_per_press() {
        let contexto = eframe::egui::Context::default();
        let mut app = App::new();

        app.emitir(Command::Build);
        app.avanzar(&contexto);
        let primera = app.state().status().map(str::to_owned);

        app.avanzar(&contexto);
        app.avanzar(&contexto);

        assert_eq!(
            app.state().status(),
            primera.as_deref(),
            "una petición ejecutada no se vuelve a ejecutar en el frame siguiente"
        );
    }

    /// Con una compilación en marcha la ventana sigue atendiendo lo que le llega. FE-043.
    ///
    /// El requisito de FE-043 no se comprueba mirando si la ventana "parece" congelada —
    /// eso no se ve desde un test— sino mirando si sigue atendiendo el teclado mientras hay
    /// trabajo en marcha. Una ventana congelada no es la que no pinta: es la que deja de
    /// procesar los eventos que le llegan, así que lo que se mide es si un atajo seguido de
    /// compilar sigue pidiendo su comando.
    ///
    /// La herramienta tarda a propósito y además falla: lo que importa es que haya una
    /// compilación *en marcha* mientras se pulsa, y que la pulsación llegue igual. Con una
    /// compilación rápida el trabajo ya habría terminado y el test no comprobaría nada.
    #[test]
    fn the_window_keeps_taking_input_while_it_compiles() {
        let contexto = eframe::egui::Context::default();
        let mut app = App::new();
        app.abrir(
            proyecto_de_prueba(),
            herramienta_que_tarda(Duration::from_millis(800)),
        );

        app.emitir(Command::Build);
        app.avanzar(&contexto);

        assert_eq!(
            app.operaciones().estado_de_build(),
            EstadoDeBuild::Compilando,
            "para comprobar que la ventana aguanta hace falta una compilación en marcha"
        );

        let pulsacion = [true, false].into_iter().map(|pressed| egui::Event::Key {
            key: egui::Key::S,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                ..egui::Modifiers::default()
            },
        });
        let entrada = eframe::egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 800.0),
            )),
            events: pulsacion.collect(),
            ..eframe::egui::RawInput::default()
        };
        let mut salida = contexto.run_ui(entrada, |ui| app.dibujar(ui));
        salida.textures_delta.clear();

        assert_eq!(
            app.peticiones(),
            [Command::Save],
            "con una compilación en marcha el atajo de guardar tiene que seguir pidiendo su \
             comando: si no, la ventana está congelada"
        );
        assert_eq!(
            app.operaciones().estado_de_build(),
            EstadoDeBuild::Compilando,
            "y la compilación sigue en marcha: escribir no la deshace ni la acelera"
        );
    }

    /// Con una compilación en marcha, la ventana dice que está compilando. FE-038.
    ///
    /// El estado de compilación sirve de muy poco si el usuario no se entera: un
    /// `Compilando` que nadie ve es un campo de un módulo. Se comprueba el texto de la barra
    /// de estado porque es lo que el usuario lee, y no el estado del módulo porque ese ya lo
    /// comprueban sus propios tests.
    #[test]
    fn the_window_says_that_it_is_compiling_while_it_compiles() {
        let contexto = eframe::egui::Context::default();
        let mut app = App::new();
        app.abrir(
            proyecto_de_prueba(),
            herramienta_que_tarda(Duration::from_millis(800)),
        );

        app.emitir(Command::Build);
        app.avanzar(&contexto);

        assert_eq!(
            app.operaciones().estado_de_build(),
            EstadoDeBuild::Compilando,
            "para comprobar que la ventana lo dice hace falta una compilación en marcha"
        );
        assert_eq!(
            app.state().status(),
            Some(COMPILANDO),
            "la barra de estado tiene que decir que se está compilando"
        );
    }
}
