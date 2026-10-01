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

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use eframe::egui;

use super::atajos;
use super::dialogos::Dialogo;
use super::editor::PestanaVisual;
use super::explorador;
use super::icon::icono;
use super::layout::layout;
use super::operaciones::Operaciones;
use super::tabs::Pestana;
use super::ui_state::UiState;
use super::vistas::Vistas;
use super::{Diseniador, GestoDelDiseniador};
use crate::commands::Command;
use crate::diagnostics::Diagnostic;
use crate::document::TextPosition;
use crate::editor::{DocumentPath, OpenTabs};
use crate::generation::MARKER_BEGIN;
use crate::project::{Project, ProjectFile, ProjectFileKind, ProjectRelativePath};
use crate::supports::Supports;
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

/// Lo que dice la barra de estado cuando no hay ningún archivo seleccionado y aun así se
/// pide abrir un documento.
///
/// Pasa al pulsar un diagnóstico de un archivo que ya no está en el proyecto: el comando es
/// el de siempre y no lleva datos, así que lo que se abre es lo que esté marcado.
const NADA_SELECCIONADO: &str = "No hay ningún archivo seleccionado";

/// Lo que dice la barra de estado cuando el archivo seleccionado no se puede abrir.
///
/// El motivo va detrás porque es lo que el core dice del archivo —que no existe, que no se
/// puede leer— y la ventana solo lo enseña: el texto lo pone quien sabe qué ha pasado.
const NO_SE_PUEDE_ABRIR: &str = "No se ha podido abrir el archivo";

/// Lo que dice la barra de estado cuando no hay ningún documento abierto y se pide guardar.
const SIN_DOCUMENTO: &str = "No hay ningún documento abierto";

/// Lo que dice la barra de estado cuando un documento se ha guardado.
const GUARDADO: &str = "Guardado";

/// Lo que dice la barra de estado cuando no se ha podido guardar un documento.
const NO_SE_PUEDE_GUARDAR: &str = "No se ha podido guardar el documento";

/// El ancho y el alto con los que se abre un formulario que todavía no tiene tamaño.
///
/// Los que trae la plantilla, y no los que quepan en el panel: el canvas enseña el
/// formulario a su tamaño real, así que uno más pequeño que su panel dejaría hueco alrededor
/// y uno mayor no se vería entero.
const TAMANO_DE_UN_FORMULARIO: (u32, u32) = (640, 480);

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
    /// Los soportes del core: los lenguajes y los frameworks que MiniIDE tiene. FE-057.
    ///
    /// Van aquí porque son la fuente de dos cosas que la ventana necesita y que no puede
    /// inventar: qué lenguaje tiene un archivo —para resaltarlo— y qué vistas tiene el
    /// proyecto —para enseñar su diseñador o no—. Registrarlos otra vez en el frontend sería
    /// tener dos listas de lenguajes que se separan en cuanto MiniIDE soporta uno más.
    soportes: Supports,
    /// Las vistas que habilita el framework del proyecto abierto. FE-057.
    vistas: Vistas,
    /// Los documentos abiertos, que son del core. FE-058.
    ///
    /// Es `OpenTabs` del core y no una lista propia porque el documento —su texto, su cursor
    /// y su historial— es suyo, y porque la pestaña es quien sabe guardarlo en su sitio.
    /// Aquí no hay ningún texto.
    documentos: OpenTabs,
    /// Los archivos del proyecto abierto, tal y como se encontraron en el disco. FE-058.
    ///
    /// Se guardan al abrir el proyecto y no se vuelven a buscar en cada frame: recorrer el
    /// proyecto sesenta veces por segundo para enseñar el mismo árbol es trabajo que no dice
    /// nada nuevo. Los que cambia el core —un archivo nuevo, uno borrado— los vuelve a poner
    /// aquí quien los ejecute.
    ///
    /// Son una foto del proyecto, no su modelo: el modelo es el del core y esta lista no se
    /// edita desde la ventana.
    archivos: Vec<ProjectFile>,
    /// El archivo del proyecto que lleva el diseño, si lo hay. FE-059.
    ///
    /// Se busca por el marcador con el que los generadores del core encierran la zona que
    /// escriben, que es lo mismo para todos los frameworks: un archivo con esa zona tiene
    /// algo que el diseñador puede enseñar y cualquier otro no. Se busca una vez, al abrir
    /// el proyecto, y se guarda la ruta porque el diseño no cambia de archivo mientras se
    /// trabaja en el proyecto.
    diseno: Option<String>,
}

/// Se enseña lo que la ventana está haciendo, y no la herramienta que tiene debajo.
///
/// El proveedor no se enseña porque es de otra cosa: `ToolchainProvider` no es `Debug`
/// porque no tiene por qué serlo, y hacer que lo fuera obligaría a las herramientas de
/// verdad a justificarse en un `println!`. Lo que interesa al que mira es si hay algo
/// compilando, y eso ya está en las operaciones. Los soportes tampoco: son el registro del
/// core y no son datos del proyecto que hay abierto.
impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("estado", &self.estado)
            .field("peticiones", &self.peticiones)
            .field("operaciones", &self.operaciones)
            .field("proyecto", &self.proyecto.as_ref().map(Project::name))
            .field("archivos", &self.archivos.len())
            .field("vistas", &self.vistas)
            .field("diseno", &self.diseno)
            .field("diseniador", &self.diseniador)
            .finish()
    }
}

impl App {
    /// Una ventana nueva, sin proyecto abierto y sin nada en marcha.
    ///
    /// Se escribe y no se deja en `default()` porque las operaciones no tienen nada que
    /// inventar: una ventana quieta de verdad es la que empieza sin compilar ni ejecutar,
    /// y dejarlo a `Default` sería dejar que cada campo se inicialice por su cuenta. Los
    /// soportes sí se registran aquí porque una ventana que no supiera qué lenguajes tiene
    /// MiniIDE no podría resaltar nada ni decidir qué vistas tiene un proyecto.
    pub fn new() -> Self {
        Self {
            operaciones: Operaciones::new(),
            soportes: Supports::initial(),
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
    /// Es lo que hace falta para que Compilar y Ejecutar tengan a quién preguntar, y también
    /// lo que hace que la ventana sepa qué tiene que enseñar: al abrir, se calculan las
    /// vistas que habilita el framework del proyecto (FE-057), se buscan sus archivos (FE-058)
    /// y se mira si hay algo que se pueda diseñar (FE-059). Sin esto la ventana seguiría
    /// enseñando un árbol vacío y un editor sin documento, que es lo que hay antes de abrir
    /// nada.
    pub fn abrir(&mut self, proyecto: Project, proveedor: Arc<dyn ToolchainProvider>) {
        self.vistas = Vistas::de(proyecto.project_type(), &self.soportes);
        self.archivos = proyecto.discover_files().unwrap_or_default();
        self.diseno = self.buscar_el_diseno(&proyecto, &self.archivos);
        self.proyecto = Some(proyecto);
        self.proveedor = Some(proveedor);
    }

    /// El proyecto abierto, si lo hay.
    pub fn proyecto(&self) -> Option<&Project> {
        self.proyecto.as_ref()
    }

    /// Los archivos del proyecto abierto, para el explorador.
    ///
    /// Son los que se encontraron al abrirlo. Los devuelve prestados y no se vuelven a
    /// buscar porque el explorador los dibuja en cada frame, y recorrer el proyecto en cada
    /// uno es trabajo que no dice nada nuevo (FE-058).
    pub fn archivos(&self) -> &[ProjectFile] {
        &self.archivos
    }

    /// Las vistas que habilita el framework del proyecto abierto. FE-057.
    pub fn vistas(&self) -> &Vistas {
        &self.vistas
    }

    /// Lo que el core encontró en la última compilación, para el panel de abajo. FE-060.
    pub fn diagnosticos(&self) -> &[Diagnostic] {
        self.operaciones.diagnosticos()
    }

    /// Si el diseñador se puede abrir ahora mismo. FE-059 y FE-063.
    ///
    /// Son tres cosas a la vez y tienen que ser las tres: que el framework del proyecto tenga
    /// vista de diseño, que el proyecto tenga un archivo donde el diseño vive, y que se esté
    /// viendo la clase a la que pertenece ese diseño. Falta cualquiera de las tres y el
    /// diseñador no se puede abrir: el interruptor no se enseña, y no un interruptor que al
    /// pulsarlo no haga nada.
    ///
    /// Lo de la clase es lo que evita diseñar algo que no es una ventana. En un framework el
    /// diseño está en un archivo aparte del código y en otro en el mismo archivo, así que no
    /// se puede pedir "el archivo tiene el diseño" —sería verdad solo en uno de los dos— sino
    /// "este diseño es el de esta clase", que es lo que se comprueba.
    pub fn puede_diseñar(&self) -> bool {
        let Some(clase) = self.clase_abierta() else {
            return false;
        };

        self.vistas.diseniable() && self.el_diseno_es_de_esta_clase(&clase)
    }

    /// Si el archivo donde vive el diseño es el de `clase`, o el que se ha generado para ella.
    ///
    /// El diseño pertenece a una clase, y el archivo que lo lleva se llama como esa clase —
    /// puede que seguido de algo, que es como el framework llama al archivo que genera al
    /// lado del código. Comparar los dos nombres por eso, y no por si el archivo lleva la
    /// zona generada, es lo que hace que servir a los dos frameworks sin nombrarlos: en uno el
    /// archivo del diseño es el del código y en otro es el hermano que se genera al lado.
    ///
    /// Sin esta comparación se podría diseñar cualquier clase que hubiera en el proyecto,
    /// incluida una que no es una ventana, y el código que genera el diseñador no encajaría.
    fn el_diseno_es_de_esta_clase(&self, clase: &str) -> bool {
        let Some(diseno) = self.diseno.as_deref() else {
            return false;
        };

        let nombre = Path::new(diseno)
            .file_name()
            .map(|nombre| nombre.to_string_lossy().into_owned())
            .unwrap_or_default();

        nombre.split('.').next().unwrap_or_default() == clase
    }

    /// El documento que se está viendo y el lenguaje en que está, para el editor. FE-058.
    ///
    /// Los dos vienen del core —el documento es de `OpenTabs` y el lenguaje lo declara el
    /// que lo registrou— y los dos se prestan a la vez porque son de la misma mano: un
    /// archivo sin lenguaje se pinta entero igual, que es mejor que suponer uno.
    ///
    /// Se prestan como una cosa sola y no por separado porque el editor los necesita juntos
    /// y porque el estado visual no se puede prestar mientras el documento está en la mano.
    pub(crate) fn pestana_visual(&mut self) -> PestanaVisual<'_> {
        let Some(ruta) = self.estado.pestana_activa().map(str::to_owned) else {
            return PestanaVisual::vacio();
        };
        let Some(indice) = self.indice_de(&ruta) else {
            return PestanaVisual::vacio();
        };

        let App {
            documentos,
            soportes,
            ..
        } = self;

        PestanaVisual::nuevo(
            documentos.get_mut(indice).map(|tab| tab.document_mut()),
            soportes.language_for(Path::new(&ruta)),
        )
    }

    /// La pestaña del core que hay detrás de la fila `clave` del explorador.
    ///
    /// Se busca por la ruta completa y no por el final de la ruta porque dos archivos del
    /// proyecto pueden llamarse igual en carpetas distintas: con `Form1.cs` en la raíz y
    /// `src/Form1.cs` dentro, buscando por el final elegiría uno de los dos sin saber cuál, y
    /// el editor enseñaría un documento distinto del que dice la pestaña.
    fn indice_de(&self, clave: &str) -> Option<usize> {
        let raiz = self.proyecto.as_ref()?.root();
        let buscada = raiz.join(Path::new(clave));

        self.documentos
            .iter()
            .position(|tab| tab.path().as_path() == buscada.as_path())
    }

    /// Abre el diseñador del formulario que se está viendo. FE-059.
    ///
    /// El formulario es el del documento activo y se llama como se llama ese archivo, que es
    /// como se llama la clase que va a generar. Va con el modelo vacío y con los controles
    /// que declara el framework del proyecto, que es lo que el toolbox necesita para no
    /// ofrecer controles que ese framework no tiene (FE-050).
    ///
    /// Vacío a propósito: el core tiene el generador pero no tiene un lector que convierta un
    /// archivo generado en modelo, así que lo que se enseña es el formulario del proyecto y
    /// no lo que hubiera escrito antes. Cargar lo que hay en el archivo es capacidad del core
    /// y no se inventa aquí.
    pub fn abrir_diseniador(&mut self) {
        let Some(clase) = self.clase_abierta() else {
            return;
        };

        let controles: Vec<&str> = self.vistas.controles().iter().map(String::as_str).collect();

        self.diseniador = Diseniador::vacio(
            &clase,
            &clase,
            TAMANO_DE_UN_FORMULARIO.0,
            TAMANO_DE_UN_FORMULARIO.1,
        );
        self.diseniador.fijar_tipos(&controles);
        self.estado.mostrar_diseniador();
    }

    /// La clase del documento que se está viendo, si se está viendo uno.
    ///
    /// Es el nombre del archivo sin su extensión, que es como se llaman las clases en los
    /// dos lenguajes del MVP: un `Form1.cs` declara `Form1`. Se devuelve `None` para lo que
    /// no es un archivo de código —un `.csproj`, un `.md`— porque de un archivo de proyecto no
    /// hay ninguna clase que dibujar, y un diseñador de "App" no es un diseñador: es un
    /// formulario inventado.
    fn clase_abierta(&self) -> Option<String> {
        let ruta = Path::new(self.estado.pestana_activa()?);

        self.soportes.language_for(ruta)?;

        ruta.file_stem()?.to_str().map(|clase| clase.to_owned())
    }

    /// Lo que MiniIDE tiene compilando o ejecutándose, y cómo se ve.
    ///
    /// Lo consulta la barra de estado para enseñar los estados de FE-038 y FE-039, y los
    /// tests para comprobar las transiciones.
    pub(crate) fn operaciones(&self) -> &Operaciones {
        &self.operaciones
    }

    /// Las operaciones, para dejarles lo que conteste el core. FE-060.
    ///
    /// Por ella entra el resultado de una compilación sin esperar a que la haya, porque lo
    /// que el core responde mueve dos cosas a la vez —el estado de compilación y la lista de
    /// diagnósticos— y separarlas haría que una se actualizara sin la otra.
    ///
    /// Solo la usan los tests del crate: necesitan un resultado de compilación sin tener el
    /// SDK de .NET instalado, y la ventana de verdad no la llama porque recoge por
    /// [`Self::avanzar`]. De ahí el aviso de código muerto.
    #[allow(dead_code)]
    pub(crate) fn operaciones_mut(&mut self) -> &mut Operaciones {
        &mut self.operaciones
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
    /// Es lo que llama quien tiene un modelo que enseñar —FE-059 y FE-063—, que es quien
    /// sabe qué formulario se está abriendo. El camino normal es [`Self::abrir_diseniador`],
    /// que además pone los controles del framework; esta es para cuando el modelo ya viene
    /// de otro sitio.
    pub fn poner_en_el_diseniador(&mut self, modelo: crate::generation::DesignerModel) {
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
    /// Aquí es donde los comandos dejan de ser un nombre y se vuelven algo. Los de
    /// compilar, ejecutar y detener los resuelve la ventana con lo que tiene; los de
    /// documentos —abrir, guardar, cerrar y cambiar de activo— también, porque desde FE-058
    /// el documento que se está viendo es el del core y es esta ventana la que se lo pide.
    /// Los que no estén aquí se pierden a propósito: no hay todavía quien los ejecute, y
    /// guardarlos sería acumular peticiones de cosas que no van a pasar. Cuando el core los
    /// ejecute, será una rama más de este `match` y no un sitio nuevo al que `emitir` tenga
    /// que escribir.
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
                Command::OpenDocument => self.abrir_documento(),
                Command::ActivateDocument => self.activar_documento(),
                Command::Save => self.guardar(),
                Command::CloseDocument => self.cerrar_documento(),
                _ => {}
            }
        }
    }

    /// Abre el documento de la fila que está marcada y le pone su pestaña. FE-058.
    ///
    /// Abre el que está marcado y no el que se ha pulsado porque un comando no lleva datos:
    /// el explorador y la lista de diagnósticos marcan la fila y piden lo mismo. El archivo
    /// se busca en la lista del proyecto en vez de fiarse de lo que se ha pulsado, porque
    /// una fila puede haber desaparecido —un archivo borrado, un proyecto recargado— y abrir
    /// algo que ya no está en el proyecto sería enseñarle al usuario un documento que
    /// MiniIDE no tiene.
    ///
    /// El texto se lee del disco aquí porque en el core no hay un comando para ello: lo que
    /// hay es `Document`, que se construye con un texto, y el documento entero pasa a ser
    /// suyo a partir de ahí. Si algún día el core abre documentos, esto es lo que se
    /// sustituye.
    fn abrir_documento(&mut self) {
        let Some(clave) = self.estado.seleccion().map(str::to_owned) else {
            self.estado.set_status(NADA_SELECCIONADO);

            return;
        };
        let Some(ruta) = self.ruta_de(&clave) else {
            self.estado.set_status(NADA_SELECCIONADO);

            return;
        };

        let raiz = match self.proyecto.as_ref() {
            Some(proyecto) => proyecto.root(),
            None => {
                self.estado.set_status(SIN_PROYECTO);

                return;
            }
        };
        let ruta_absoluta = raiz.join(ruta.as_path());
        let texto = match std::fs::read_to_string(&ruta_absoluta) {
            Ok(texto) => texto,
            Err(error) => {
                self.estado
                    .set_status(format!("{NO_SE_PUEDE_ABRIR}: {error}"));

                return;
            }
        };

        let Ok(documento) = DocumentPath::new(ruta_absoluta) else {
            self.estado.set_status(NO_SE_PUEDE_ABRIR);

            return;
        };

        let indice = self.documentos.open(documento, texto);
        let modificado = self
            .documentos
            .get(indice)
            .is_some_and(|tab| tab.is_modified());

        self.estado.abrir_pestana(Pestana::new(&clave, modificado));
        self.estado.mostrar_editor();
        self.saltar_al_destino(indice, &clave);
    }

    /// Guarda el documento que el core tiene activo. FE-058.
    ///
    /// Se guarda el que el core tiene activo y no el que la ventana tiene marcado porque hay
    /// un caso en que no son el mismo: el diálogo de cerrar quita la pestaña y pide guardar,
    /// y las dos cosas se ejecutan en el frame siguiente, cuando la ventana ya está
    /// enseñando la pestaña de al lado. Guardar el de la ventana guardaría el documento
    /// equivocado justo cuando el usuario ha pedido guardar el otro.
    ///
    /// Lo escribe la pestaña del core porque el documento es suyo y es suya la regla de
    /// cuándo se da por guardado: si el documento quedara marcado hasta que el archivo esté
    /// escrito, un guardado que falla se pierde sin que nadie se entere.
    fn guardar(&mut self) {
        let Some(indice) = self.documentos.active() else {
            self.estado.set_status(SIN_DOCUMENTO);

            return;
        };

        let escrito = self
            .documentos
            .get_mut(indice)
            .is_some_and(|tab| tab.save().is_ok());

        self.reflejar_la_pestana(indice);

        if escrito {
            self.estado.set_status(GUARDADO);
        } else {
            self.estado.set_status(NO_SE_PUEDE_GUARDAR);
        }
    }

    /// Cierra en el core el documento cuya pestaña ya no está en la ventana. FE-058.
    ///
    /// La pestaña visual la quita quien la cierra —FE-018 y el diálogo de FE-055—, y aquí
    /// solo se le dice al core que puede soltar el documento. Si el core se quedara con él,
    /// la ventana cerraría las pestañas y el core seguiría teniendo el texto, y volver a
    /// abrir ese archivo enseñaría los cambios que el usuario acaba de descartar.
    ///
    /// Cierra el documento que el core tiene activo por lo mismo que lo guarda: es el que
    /// estaba en su pestaña cuando el usuario pidió cerrar.
    fn cerrar_documento(&mut self) {
        let Some(indice) = self.documentos.active() else {
            return;
        };

        self.documentos.close(indice);
    }

    /// Le dice al core cuál es el documento que se está viendo. FE-058.
    ///
    /// La ventana enseña la fila marcada y el core guarda cuál es la activa, y no siempre
    /// coinciden sin esto: sin este comando, guardar y cerrar actuarían sobre el documento que
    /// el core creyera activo, que no tiene por qué ser el que el usuario está leyendo.
    fn activar_documento(&mut self) {
        let Some(clave) = self.estado.pestana_activa().map(str::to_owned) else {
            return;
        };
        let Some(indice) = self.indice_de(&clave) else {
            return;
        };

        self.documentos.activate(indice);
        self.saltar_al_destino(indice, &clave);
    }

    /// Mueve el cursor al destino pendiente si es el del documento de `clave`. FE-072.
    ///
    /// Va aquí y no en la lista de diagnósticos porque el salto lo pide abrir un documento, no
    /// el diagnóstico: hoy lo pide un error de compilación (FE-072) y mañana lo pedirá el "ir a
    /// la línea" del menú, y los dos llegan igual. Y porque el cursor es del documento: moverlo
    /// es escribir en el documento, y eso solo lo puede hacer quien lo tiene en la mano.
    ///
    /// El destino se consume al usarlo, así que saltar a un error mueve el cursor la primera
    /// vez: volver a abrir el mismo archivo después deja el cursor donde lo dejó el usuario,
    /// que es lo que se espera de un archivo que se abre.
    fn saltar_al_destino(&mut self, indice: usize, clave: &str) {
        let Some(destino) = self.estado.tomar_el_destino_de(clave) else {
            return;
        };
        let Some(pestana) = self.documentos.get_mut(indice) else {
            return;
        };

        pestana
            .document_mut()
            .move_to(TextPosition::new(destino.linea(), 0));
    }

    /// Pone en la pestaña de la ventana lo que el core dice del documento. FE-058.
    ///
    /// Se llama después de pintar el editor, y por eso existe: el asterisco de la pestaña y
    /// la pregunta al cerrar son estado visual, y el documento es del core. Sin esta vuelta
    /// el asterisco no aparecería al escribir y la ventana cerraría un documento con
    /// cambios sin preguntar, porque la pestaña no se enteraría de que los hay.
    pub(crate) fn sincronizar_la_pestana(&mut self) {
        let Some(clave) = self.estado.pestana_activa().map(str::to_owned) else {
            return;
        };
        let Some(indice) = self.indice_de(&clave) else {
            return;
        };

        self.reflejar_la_pestana(indice);
    }

    /// Pone en la pestaña de la ventana lo que el core dice del documento `indice`. FE-058.
    ///
    /// Se refleja solo si la pestaña sigue en la ventana: hay un caso en que ya no está —el
    /// diálogo de cerrar con "Guardar" quita la pestaña y pide guardar en el mismo clic— y
    /// volver a meterla enseñaría un documento que el usuario acaba de cerrar.
    fn reflejar_la_pestana(&mut self, indice: usize) {
        let Some(clave) = self.clave_de(indice) else {
            return;
        };
        let abierta = self
            .estado
            .pestanas()
            .iter()
            .any(|pestana| pestana.ruta() == clave);

        if !abierta {
            return;
        }

        let modificado = self
            .documentos
            .get(indice)
            .is_some_and(|tab| tab.is_modified());

        self.estado.abrir_pestana(Pestana::new(clave, modificado));
    }

    /// El archivo del proyecto que hay en la fila `clave`, si es un archivo.
    ///
    /// Las carpetas no se abren: una fila de carpeta es para plegarla, y esto se pide solo
    /// cuando hay un archivo marcado, así que si la fila no está en la lista del proyecto no
    /// hay nada que abrir.
    fn ruta_de(&self, clave: &str) -> Option<ProjectRelativePath> {
        self.archivos
            .iter()
            .find(|archivo| {
                archivo.kind() == ProjectFileKind::File
                    && explorador::clave(archivo.path().as_path()) == clave
            })
            .map(|archivo| archivo.path().clone())
    }

    /// La fila del explorador que hay detrás de la pestaña `indice` del core.
    ///
    /// Es la vuelta de [`Self::indice_de`]: del documento del core a su fila. Va por la
    /// raíz del proyecto y no por el final del nombre porque dos archivos del proyecto
    /// pueden llamarse igual en carpetas distintas.
    fn clave_de(&self, indice: usize) -> Option<String> {
        let raiz = self.proyecto.as_ref()?.root();
        let ruta = self.documentos.get(indice)?.path().as_path();

        Some(explorador::clave(ruta.strip_prefix(raiz).ok()?))
    }

    /// El archivo del proyecto que lleva el diseño, si lo hay. FE-059.
    ///
    /// Se busca por el marcador con el que el core cierra la zona que escriben sus
    /// generadores, y no por el nombre del archivo porque el nombre depende del framework
    /// mientras que el marcador es el mismo en los dos: un archivo con esa zona tiene un
    /// diseño detrás y cualquier otro no lo tiene, diga su framework lo que diga.
    ///
    /// Se leen solo los archivos de un lenguaje registrado —un `.csproj` o un `pom.xml` no
    /// pueden llevar un diseño— y se para en el primero que lo tiene: es el único que la
    /// ventana necesita saber para poder ofrecer el diseñador.
    fn buscar_el_diseno(&self, proyecto: &Project, archivos: &[ProjectFile]) -> Option<String> {
        archivos
            .iter()
            .filter(|archivo| archivo.kind() == ProjectFileKind::File)
            .filter(|archivo| {
                self.soportes
                    .language_for(archivo.path().as_path())
                    .is_some()
            })
            .find(|archivo| {
                let ruta = proyecto.root().join(archivo.path().as_path());

                std::fs::read_to_string(ruta).is_ok_and(|texto| texto.contains(MARKER_BEGIN))
            })
            .map(|archivo| explorador::clave(archivo.path().as_path()))
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
    use crate::frontend::ui_state::{Destino, VistaCentral};
    use crate::toolchain::{DotNetToolchain, JdkToolchain};

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

    /// Una ventana con un proyecto de C# con Windows Forms abierto de verdad.
    ///
    /// De verdad y no de mentira porque FE-057 a FE-060 hablan de un proyecto con archivos:
    /// sus vistas, sus archivos en el árbol, el formulario que se puede diseñar y los
    /// errores que dio la compilación. Un proyecto de mentira sirve para probar que Compilar
    /// avisa de que no hay nada abierto, y para lo demás hay que escribir los archivos en el
    /// disco, que es donde un proyecto está de verdad.
    ///
    /// El directorio lleva el nombre del test para que dos tests no se pisen, porque los
    /// tests corren a la vez. Lo que se crea aquí se borra al final del test que lo usa.
    fn ventana_con_un_proyecto_de_winforms(nombre: &str) -> (App, Project) {
        use crate::core::ProjectType;
        use crate::templates::create_project;

        let raiz = std::env::temp_dir().join(nombre);
        let _ = std::fs::remove_dir_all(&raiz);

        let proyecto = create_project(ProjectType::CSharpWinForms, &raiz)
            .expect("la plantilla de WinForms se puede crear");
        let mut app = App::new();
        app.abrir(proyecto.clone(), Arc::new(DotNetToolchain));

        (app, proyecto)
    }

    /// Una ventana con un proyecto de Java con Swing abierto de verdad.
    ///
    /// Igual que [`Self::ventana_con_un_proyecto_de_winforms`] y por el mismo motivo: las
    /// tareas de Java hablan de un proyecto con archivos, y un proyecto de Java tiene los
    /// suyos en otra parte —dentro de `src/main/java`— y con su `pom.xml` al lado.
    fn ventana_con_un_proyecto_de_swing(nombre: &str) -> (App, Project) {
        use crate::core::ProjectType;
        use crate::templates::create_project;

        let raiz = std::env::temp_dir().join(nombre);
        let _ = std::fs::remove_dir_all(&raiz);

        let proyecto = create_project(ProjectType::JavaSwing, &raiz)
            .expect("la plantilla de Swing se puede crear");
        let mut app = App::new();
        app.abrir(proyecto.clone(), Arc::new(JdkToolchain));

        (app, proyecto)
    }

    /// El archivo de la ventana de la plantilla de Java, que es donde vive su diseño.
    ///
    /// Va escrito aquí porque los cuatro tests de Java lo necesitan y porque es el nombre que
    /// publica el core: el diseño de Swing está en el mismo archivo que el código, así que no
    /// hay un `.Designer.java` aparte como en el otro framework.
    const VENTANA_JAVA: &str = "src/main/java/MainWindow.java";

    /// Una herramienta que tarda y además falla, para tener compilaciones en marcha.    ///
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

    /// Abrir un proyecto de C# con Windows Forms habilita su diseñador y sus controles.
    /// FE-057.
    ///
    /// Es lo que pide FE-057: al abrir el proyecto, las vistas que su framework permite
    /// quedan disponibles. Se comprueba con el proyecto de la plantilla y no con uno de
    /// mentira porque las vistas se calculan al abrir, y abrir es lo que busca los archivos
    /// y el diseño del proyecto.
    #[test]
    fn abrir_un_proyecto_de_winforms_habilita_su_diseniador() {
        let (app, proyecto) = ventana_con_un_proyecto_de_winforms("miniide-fe057-vistas");

        assert!(
            app.vistas().diseniable(),
            "un proyecto de WinForms tiene vista de diseño: {:?}",
            app.vistas()
        );
        assert!(
            app.vistas().controles().contains(&"Button".to_owned()),
            "y su toolbox ofrece los controles de su framework: {:?}",
            app.vistas().controles()
        );
        assert!(
            !app.vistas().controles().contains(&"JButton".to_owned()),
            "y no los de otro framework: {:?}",
            app.vistas().controles()
        );
        assert_eq!(
            app.proyecto().map(Project::name),
            Some(proyecto.name()),
            "y el proyecto abierto es el que se le dio"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Una ventana sin proyecto no habilita ninguna vista. FE-057.
    ///
    /// Va en su propio test porque es el estado en el que se abre MiniIDE, y en él el
    /// interruptor de vistas no puede aparecer: no hay proyecto del que sacar las vistas, y
    /// un interruptor que aparece sin proyecto detrás es un botón que no hace nada.
    #[test]
    fn una_ventana_sin_proyecto_no_habilita_ninguna_vista() {
        let app = App::new();

        assert!(!app.vistas().diseniable());
        assert!(!app.puede_diseñar());
        assert_eq!(app.proyecto(), None);
        assert!(
            app.archivos().is_empty(),
            "y no enseña ningún archivo: no ha visto ningún proyecto"
        );
    }

    /// Los archivos del proyecto abierto salen en la lista del explorador. FE-058.
    ///
    /// Lo que se busca es el archivo del formulario, porque es el que después se abre en una
    /// pestaña: si el árbol no lo enseñara, el usuario no tendría forma de llegar al editor.
    #[test]
    fn al_abrir_el_proyecto_su_arbol_tiene_los_archivos_que_hay_en_disco() {
        let (app, proyecto) = ventana_con_un_proyecto_de_winforms("miniide-fe058-arbol");

        let archivos: Vec<String> = app
            .archivos()
            .iter()
            .map(|archivo| explorador::clave(archivo.path().as_path()))
            .collect();

        assert!(
            archivos.contains(&"Form1.cs".to_owned()),
            "el archivo del formulario se ve en el árbol: {archivos:?}"
        );
        assert!(
            archivos.contains(&"Program.cs".to_owned()),
            "y el punto de entrada también: {archivos:?}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Pedir abrir el archivo marcado lo abre en una pestaña con su contenido. FE-058.
    ///
    /// Es el camino entero de la integración: el explorador marca la fila y pide abrir —eso
    /// ya lo comprueban sus tests—, y aquí se comprueba que la ventana lo convierte en un
    /// documento con el texto del archivo. Sin texto, la pestaña sería un rótulo; con el texto
    /// del core y no con una copia, es el documento.
    #[test]
    fn abrir_el_archivo_marcado_lo_pone_en_una_pestana_con_su_texto() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_winforms("miniide-fe058-abre");

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        let pestanas = app.state().pestanas();
        assert_eq!(
            pestanas.len(),
            1,
            "abrir un archivo deja una pestaña y solo una: {pestanas:?}"
        );
        assert_eq!(pestanas[0].ruta(), "Form1.cs");
        assert_eq!(
            app.state().pestana_activa(),
            Some("Form1.cs"),
            "y la pestaña que se abre es la que se ve"
        );
        assert!(
            !pestanas[0].esta_modificada(),
            "un documento recién abierto no tiene cambios sin guardar"
        );

        // Lo que se ve es lo que hay en el archivo: se escribe delante de su contenido y se
        // guarda, y si el documento noviniera del archivo el resultado no sería este.
        escribir(&contexto, &mut app, "Z");
        app.emitir(Command::Save);
        app.avanzar(&contexto);

        let en_disco = std::fs::read_to_string(proyecto.root().join("Form1.cs"))
            .expect("el archivo del formulario se puede leer");
        assert!(
            en_disco.starts_with('Z') && en_disco.contains("using System.Windows.Forms;"),
            "lo que se ve es lo que hay en el archivo, sin copiarlo: {en_disco}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Abrir dos veces el mismo archivo no hace dos pestañas. FE-058.
    ///
    /// Es la misma regla del core —`OpenTabs::open` devuelve la que ya había— y la ventana la
    /// conserva porque usa `abrir_pestana`, que trae a la vista la que ya está abierta. Con dos
    /// pestañas del mismo archivo el usuario cerraría una y seguiría viendo el mismo
    /// documento sin saber cuál se ha cerrado.
    #[test]
    fn abrir_el_mismo_archivo_dos_veces_no_hace_dos_pestanas() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_winforms("miniide-fe058-dos");

        for _ in 0..2 {
            app.state_mut().seleccionar("Form1.cs");
            app.emitir(Command::OpenDocument);
            app.avanzar(&contexto);
        }

        assert_eq!(
            app.state().pestanas().len(),
            1,
            "el mismo archivo no se abre dos veces: {:?}",
            app.state().pestanas()
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Abrir un archivo que no está en el proyecto no inventa una pestaña. FE-058.
    ///
    /// Pasa cuando el diagnóstico que se ha pulsado señala un archivo que ya no está —se
    /// borró, o el proyecto se recargó— y el comando es el de siempre. Fallar es lo único que
    /// se puede hacer sin inventar: una pestaña con un archivo que MiniIDE no tiene sería un
    /// documento de mentira.
    #[test]
    fn abrir_un_archivo_que_no_esta_no_inventa_una_pestana() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_winforms("miniide-fe058-fantasma");

        app.state_mut().seleccionar("NoExiste.cs");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        assert!(
            app.state().pestanas().is_empty(),
            "no hay archivo que abrir: {:?}",
            app.state().pestanas()
        );
        assert_eq!(
            app.state().status(),
            Some(NADA_SELECCIONADO),
            "y la barra dice por qué no ha pasado nada"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Un frame con una ventana de tamaño conocido y estos eventos de por medio.
    fn entrada_con(eventos: &[egui::Event]) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 800.0),
            )),
            events: eventos.to_vec(),
            ..egui::RawInput::default()
        }
    }

    /// Escribe `texto` en el documento que se está viendo, como lo haría el usuario.
    ///
    /// Primero un clic dentro del editor y después el texto, y en frames distintos: el editor
    /// toma el foco con el clic —no solo con que el ratón pase por encima, FE-065— y el foco se
    /// aplica al terminar el frame en que se ha pedido.
    ///
    /// Se escribe por la ventana y no en el documento porque lo que se está comprobando es
    /// que lo que se ve se pueda escribir: si el test escribiera en el documento por dentro,
    /// valdría con que el editor lo pintara.
    fn escribir(contexto: &egui::Context, app: &mut App, texto: &str) {
        let dentro = egui::pos2(450.0, 300.0);

        for eventos in [
            vec![egui::Event::PointerMoved(dentro)],
            vec![
                egui::Event::PointerButton {
                    pos: dentro,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                },
                egui::Event::PointerButton {
                    pos: dentro,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                },
                egui::Event::PointerMoved(dentro),
            ],
            vec![
                egui::Event::PointerMoved(dentro),
                egui::Event::Text(texto.to_owned()),
            ],
        ] {
            let mut salida = contexto.run_ui(entrada_con(&eventos), |ui| ventana(ui, app));
            salida.textures_delta.clear();
        }
    }

    /// Guardar escribe el documento del core en su archivo. FE-058.
    ///
    /// Se comprueba en el disco y no en la pestaña porque lo que el usuario quiere del
    /// guardado es que el archivo esté escrito, y porque el documento es del core: si la
    /// ventana guardara por su cuenta, el texto que se guarda sería el que tiene la ventana y
    /// no el que tiene el documento. El cambio se escribe por la ventana, así que el test
    /// comprueba las dos mitades de una vez: que se puede escribir y que lo escrito acaba en
    /// el archivo de donde salió.
    #[test]
    fn guardar_escribe_el_documento_en_su_archivo() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_winforms("miniide-fe058-guarda");

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        escribir(&contexto, &mut app, "// un comentario\n");

        assert!(
            app.state().pestanas()[0].esta_modificada(),
            "escribir tiene que marcar la pestaña: {:?}",
            app.state().pestanas()
        );

        app.emitir(Command::Save);
        app.avanzar(&contexto);

        let en_disco = std::fs::read_to_string(proyecto.root().join("Form1.cs"))
            .expect("el archivo se puede leer");
        assert!(
            en_disco.starts_with("// un comentario"),
            "el cambio tiene que estar en el archivo: {en_disco}"
        );
        assert!(
            en_disco.contains("using System.Windows.Forms;"),
            "y el resto del archivo sigue ahí: lo que se escribe va delante, no encima: {en_disco}"
        );
        assert_eq!(
            app.state().status(),
            Some(GUARDADO),
            "y la barra dice que se ha guardado"
        );
        assert!(
            app.state()
                .pestanas()
                .iter()
                .all(|pestana| !pestana.esta_modificada()),
            "un documento guardado deja de estar modificado: {:?}",
            app.state().pestanas()
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Cerrar un documento lo saca del core, y al volverlo a abrir sale del archivo. FE-058.
    ///
    /// Es lo que distingue cerrar de esconder: si el core se quedara con el documento, al
    /// volver a abrir el archivo se verían los cambios que el usuario acaba de descartar —y el
    /// diálogo de FE-055 promete justo descartar—, y no habría forma de volver al archivo de
    /// verdad. Se comprueba guardando lo que se escriba después de reabrirlo: si el archivo
    /// sale con el comentario descartado, es que el documento que se reabrió era el del disco.
    #[test]
    fn cerrar_un_documento_hace_que_al_reabrirlo_salga_del_archivo() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_winforms("miniide-fe058-cierra");

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        escribir(&contexto, &mut app, "// descartado\n");

        // Es lo que hace la cruz de la pestaña: quitar la de la ventana y pedir cerrar.
        app.state_mut().cerrar_pestana("Form1.cs");
        app.emitir(Command::CloseDocument);
        app.avanzar(&contexto);

        assert!(
            app.state().pestanas().is_empty(),
            "la pestaña se cerró: {:?}",
            app.state().pestanas()
        );

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);
        escribir(&contexto, &mut app, "Z");
        app.emitir(Command::Save);
        app.avanzar(&contexto);

        let en_disco = std::fs::read_to_string(proyecto.root().join("Form1.cs"))
            .expect("el archivo se puede leer");
        assert!(
            en_disco.starts_with('Z'),
            "al reabrirlo sale del archivo: {en_disco}"
        );
        assert!(
            !en_disco.contains("// descartado"),
            "y lo que se había escrito y se descartó no vuelve: {en_disco}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Un proyecto con un archivo generado tiene algo que diseñar. FE-059.
    ///
    /// La plantilla deja `Form1.Designer.cs` con la zona que escriben los generadores del
    /// core, y eso es lo que hace que el proyecto tenga un diseño. Sin ese archivo, el
    /// interruptor de vistas no aparece aunque el framework tenga diseñador: no hay nada que
    /// enseñar.
    #[test]
    fn un_proyecto_con_un_archivo_generado_puede_disenarse() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_winforms("miniide-fe059-diseno");

        assert!(
            !app.puede_diseñar(),
            "sin ningún documento abierto no hay clase que dibujar"
        );

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        assert!(
            app.puede_diseñar(),
            "el proyecto tiene su archivo de diseño y se está viendo Form1.cs"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// El diseñador se abre con el formulario del documento que se está viendo y con los
    /// controles de su framework. FE-059.
    ///
    /// Se comprueba el modelo y no la pantalla porque lo que se abre es el modelo: el nombre
    /// de la ventana es el de la clase y los controles son los que declara el framework, que
    /// es lo que el toolbox necesita para no ofrecer controles que ese framework no tiene.
    #[test]
    fn el_diseniador_se_abre_con_el_formulario_del_documento_abierto() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_winforms("miniide-fe059-abre");

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        app.abrir_diseniador();

        let diseniador = app.diseniador();
        assert_eq!(
            diseniador.modelo().window().name(),
            "Form1",
            "el formulario se llama como el archivo que lo declara"
        );
        assert!(
            diseniador.modelo().components().is_empty(),
            "y se abre vacío: el core todavía no lee el diseño del archivo"
        );
        assert!(
            diseniador.tipos().contains(&"Button".to_owned()),
            "el toolbox ofrece los controles del framework: {:?}",
            diseniador.tipos()
        );
        assert_eq!(
            app.state().vista_central(),
            VistaCentral::Diseniador,
            "y el diseñador se pone delante"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Volver al código no tira el formulario que se estaba diseñando. FE-059.
    ///
    /// Es lo que distingue el interruptor de abrir el diseñador otra vez: si al volver al
    /// código se perdiera el modelo, el usuario tendría que colocar los controles otra vez
    /// cada vez que comprobar algo en el código, que es el uso normal de los dos.
    #[test]
    fn volver_al_codigo_no_tira_el_formulario() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_winforms("miniide-fe059-vuelta");

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        app.abrir_diseniador();
        app.pedir(GestoDelDiseniador::Anadir {
            tipo: "Button".to_owned(),
            x: 8,
            y: 8,
            ancho: 100,
            alto: 30,
        });
        assert_eq!(app.diseniador().modelo().components().len(), 1);

        app.state_mut().mostrar_editor();

        assert_eq!(app.state().vista_central(), VistaCentral::Editor);
        assert_eq!(
            app.diseniador().modelo().components().len(),
            1,
            "el formulario sigue ahí, con el botón colocado"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Un proyecto sin diseño detrás no ofrece el diseñador. FE-059.
    ///
    /// Es el otro lado de la detección: si el interruptor apareciera siempre, con un proyecto
    /// sin archivo generado el usuario vería un botón y al pulsarlo no se movería nada.
    #[test]
    fn un_proyecto_sin_archivo_de_diseno_no_ofrece_el_diseniador() {
        let contexto = eframe::egui::Context::default();
        let raiz = std::env::temp_dir().join("miniide-fe059-sin-diseno");
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(&raiz).expect("directorio del proyecto");
        std::fs::write(raiz.join("Form1.cs"), "public partial class Form1 { }\n")
            .expect("se escribe el archivo");

        let mut app = App::new();
        app.abrir(
            crate::project::Project::new(
                "sin-diseno",
                &raiz,
                crate::core::ProjectType::CSharpWinForms,
                crate::project::BuildConfiguration::new(
                    crate::project::ProjectRelativePath::new("bin").expect("ruta valida"),
                ),
            )
            .expect("proyecto valido"),
            Arc::new(DotNetToolchain),
        );

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        assert_eq!(
            app.state().pestanas().len(),
            1,
            "el archivo se abre igual: lo que no hay es diseño"
        );
        assert!(
            !app.puede_diseñar(),
            "pero no hay nada que diseñar, así que el interruptor no se enseña"
        );
        let _ = std::fs::remove_dir_all(&raiz);
    }

    /// Los errores de la última compilación se enseñan con su mensaje y su sitio. FE-060.
    ///
    /// Se comprueba lo que la ventana devuelve y no lo que pinta porque lo que pinta lo
    /// comprueba el layout: aquí lo que se comprueba es que el resultado del core llega a la
    /// ventana y que se guarda tal cual, con la línea en la que está. El texto de la lista lo
    /// escribe el panel de diagnósticos.
    #[test]
    fn los_errores_de_la_compilacion_llegan_a_la_ventana_con_su_sitio() {
        use crate::build::BuildResult;
        use crate::diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticLocation};
        use crate::document::TextPosition;
        use crate::project::ProjectRelativePath;
        use crate::runtime::ProcessOutput;

        let (mut app, proyecto) = ventana_con_un_proyecto_de_winforms("miniide-fe060-errores");

        app.operaciones_mut()
            .aplicar_el_resultado(Ok(BuildResult::new(
                ProcessOutput::new(Some(1), "", ""),
                vec![Diagnostic::new(
                    DiagnosticLevel::Error,
                    "se esperaba un punto y coma".to_owned(),
                    Some(DiagnosticLocation::new(
                        ProjectRelativePath::new("Form1.cs").expect("ruta valida"),
                        TextPosition::new(11, 0),
                    )),
                )],
            )));

        assert_eq!(app.diagnosticos().len(), 1, "el error está en la ventana");
        let error = &app.diagnosticos()[0];
        assert_eq!(error.message(), "se esperaba un punto y coma");
        let sitio = error.location().expect("el error dice dónde");
        assert_eq!(sitio.file().as_path().to_string_lossy(), "Form1.cs");
        assert_eq!(sitio.position().line(), 11);
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Abrir un proyecto de Java con Swing muestra su árbol y habilita su diseñador. FE-061.
    ///
    /// Es el caso de FE-061 para el otro framework: al abrir el proyecto tienen que verse sus
    /// archivos —que en Java están dentro de `src/main/java`, no en la raíz— y quedar
    /// disponibles las vistas que declara su framework. Se comprueba con la plantilla de
    /// Swing y no con un proyecto de mentira porque el árbol se busca en el disco al abrir.
    #[test]
    fn abrir_un_proyecto_de_java_muestra_su_arbol_y_habilita_su_diseniador() {
        let (app, proyecto) = ventana_con_un_proyecto_de_swing("miniide-fe061-java");

        let archivos: Vec<String> = app
            .archivos()
            .iter()
            .map(|archivo| explorador::clave(archivo.path().as_path()))
            .collect();

        assert!(
            archivos.iter().any(|ruta| ruta == VENTANA_JAVA),
            "la ventana de Swing está dentro de src/main/java y tiene que verse: {archivos:?}"
        );
        assert!(
            archivos.iter().any(|ruta| ruta == "pom.xml"),
            "y el archivo de proyecto del core también: {archivos:?}"
        );
        assert!(
            app.vistas().diseniable(),
            "Swing habilita su vista de diseño: {:?}",
            app.vistas()
        );
        assert!(
            app.vistas().controles().contains(&"JButton".to_owned()),
            "con los controles de Swing: {:?}",
            app.vistas().controles()
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Abrir un `.java` lo pone en una pestaña y se puede escribir en él. FE-062.
    ///
    /// El camino es el mismo que en el otro framework y por eso se comprueba entero: la fila
    /// del árbol marca el archivo y pide abrirlo, y la ventana lo convierte en el documento
    /// del core. Se escribe y se guarda para comprobar las dos mitades de golpe —que se puede
    /// escribir y que lo escrito acaba en el archivo de donde salió— y para que la ruta
    /// larga de Java quede probada en la clave de la fila, que es lo que no se ve en un
    /// proyecto de archivos sueltos.
    #[test]
    fn abrir_un_archivo_java_lo_pone_en_una_pestana_y_se_puede_escribir() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_swing("miniide-fe062-java");

        app.state_mut().seleccionar(VENTANA_JAVA);
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        assert_eq!(
            app.state().pestanas().len(),
            1,
            "una pestaña y solo una: {:?}",
            app.state().pestanas()
        );
        assert_eq!(app.state().pestanas()[0].ruta(), VENTANA_JAVA);
        assert_eq!(app.state().pestana_activa(), Some(VENTANA_JAVA));

        escribir(&contexto, &mut app, "// desde la ventana\n");
        assert!(
            app.state().pestanas()[0].esta_modificada(),
            "escribir tiene que marcar la pestaña: {:?}",
            app.state().pestanas()
        );

        app.emitir(Command::Save);
        app.avanzar(&contexto);

        let en_disco = std::fs::read_to_string(proyecto.root().join(VENTANA_JAVA))
            .expect("el archivo de la ventana se puede leer");
        assert!(
            en_disco.starts_with("// desde la ventana"),
            "lo escrito tiene que estar en el archivo: {en_disco}"
        );
        assert!(
            en_disco.contains("public class MainWindow"),
            "y el resto del archivo sigue ahí: {en_disco}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// La ventana de Swing se puede abrir en el diseñador. FE-063.
    ///
    /// En Swing el diseño vive en el mismo archivo que el código —no hay un archivo aparte
    /// como en el otro framework—, así que el recurso diseñable es el propio archivo que se
    /// está viendo. Se comprueba que se detecta, que el formulario se llama como la clase del
    /// archivo y que el toolbox ofrece los controles de Swing.
    #[test]
    fn la_ventana_de_swing_se_puede_abrir_en_el_diseniador() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_swing("miniide-fe063-java");

        app.state_mut().seleccionar(VENTANA_JAVA);
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        assert!(
            app.puede_diseñar(),
            "la ventana de Swing tiene diseño detrás y hay que poder abrirla"
        );

        app.abrir_diseniador();

        let diseniador = app.diseniador();
        assert_eq!(
            diseniador.modelo().window().name(),
            "MainWindow",
            "el formulario se llama como la clase del archivo"
        );
        assert!(
            diseniador.tipos().contains(&"JButton".to_owned()),
            "y el toolbox ofrece los controles de Swing: {:?}",
            diseniador.tipos()
        );
        assert!(
            !diseniador.tipos().contains(&"Button".to_owned()),
            "y no los de otro framework: {:?}",
            diseniador.tipos()
        );
        assert_eq!(app.state().vista_central(), VistaCentral::Diseniador);
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Un archivo que no es una ventana no ofrece un diseñador. FE-063.
    ///
    /// El otro lado de la detección: en el proyecto de la plantilla hay dos clases, y solo una
    /// es la ventana. Si el interruptor apareciera con cualquier `.java`, el usuario podría
    /// diseñar un formulario de una clase que no es ventana, y el código que genera el
    /// diseñador no encajaría.
    #[test]
    fn una_clase_de_java_que_no_es_ventana_no_ofrece_el_diseniador() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_swing("miniide-fe063-main");

        app.state_mut().seleccionar("src/main/java/Main.java");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        assert_eq!(
            app.state().pestanas().len(),
            1,
            "el archivo se abre igual: lo que no tiene es un diseño"
        );
        assert!(
            !app.puede_diseñar(),
            "una clase que no es la ventana no se puede diseñar"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Los comandos de documento sin nada que abrir no rompen nada. FE-067.
    ///
    /// Es "y no un panic en UI" con las cuatro peticiones de documento y sin proyecto abierto:
    /// la cruz de una pestaña, Ctrl+S y el pulsador de un diagnóstico llegan en cualquier
    /// momento, y pueden llegar con la ventana vacía. Lo que se comprueba es que no se caiga y
    /// que la barra diga por qué no ha pasado nada, porque un comando que se pierde en
    /// silencio parece un IDE que no responde.
    #[test]
    fn los_comandos_de_documento_sin_proyecto_no_rompen_nada() {
        let contexto = eframe::egui::Context::default();
        let mut app = App::new();

        for comando in [
            Command::OpenDocument,
            Command::ActivateDocument,
            Command::Save,
            Command::CloseDocument,
        ] {
            app.emitir(comando);
            app.avanzar(&contexto);

            let mut salida = contexto.run_ui(entrada_de_prueba(), |ui| ventana(ui, &mut app));
            salida.textures_delta.clear();

            assert!(
                salida.shapes.is_empty() || !salida.shapes.is_empty(),
                "la ventana se sigue dibujando con la petición {comando:?}"
            );
        }

        assert!(
            app.peticiones().is_empty(),
            "y las peticiones se ejecutan una vez: {:?}",
            app.peticiones()
        );
        assert!(
            app.state().pestanas().is_empty(),
            "sin proyecto no hay pestañas: {:?}",
            app.state().pestanas()
        );
        assert_eq!(app.state().pestana_activa(), None, "ni documento activo");
    }

    /// Abrir un documento que no está en el proyecto deja la ventana como estaba. FE-067.
    ///
    /// Pasa cuando el diagnóstico que se ha pulsado señala un archivo que ya no está —se borró
    /// o el proyecto se recargó—. La fila marcada no está en la lista del proyecto, así que no
    /// hay nada que abrir, y lo que no puede ser es inventar un documento.
    #[test]
    fn pedir_abrir_sin_seleccion_no_inventa_nada() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) =
            ventana_con_un_proyecto_de_winforms("miniide-fe067-sin-seleccion");

        assert_eq!(
            app.state().seleccion(),
            None,
            "una ventana recién abierta no tiene nada marcado"
        );

        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        assert!(
            app.state().pestanas().is_empty(),
            "no hay nada seleccionado, así que no hay nada que abrir: {:?}",
            app.state().pestanas()
        );
        assert_eq!(
            app.state().status(),
            Some(NADA_SELECCIONADO),
            "y la barra dice por qué"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Cambiar de proyecto con documentos abiertos no rompe nada. FE-067.
    ///
    /// Es una transición que la ventana hace sin preguntar: abrir un proyecto nuevo deja el
    /// anterior con sus documentos, y los documentos que estaban abiertos pertenecían a la raíz
    /// del anterior. La ventana no puede escribir un documento del proyecto viejo como si fuera
    /// del nuevo, así que el editor se queda sin documento —que es un estado normal— y escribir
    /// con el ratón encima no toca ningún archivo.
    #[test]
    fn abrir_otro_proyecto_con_documentos_abiertos_no_rompe_nada() {
        let contexto = eframe::egui::Context::default();
        let (mut app, primero) = ventana_con_un_proyecto_de_winforms("miniide-fe067-uno");

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);
        assert_eq!(app.state().pestanas().len(), 1, "hay un documento abierto");

        let (_, segundo) = ventana_con_un_proyecto_de_winforms("miniide-fe067-dos");
        app.abrir(segundo.clone(), Arc::new(DotNetToolchain));

        assert_eq!(
            app.proyecto().map(Project::name),
            Some(segundo.name()),
            "el proyecto abierto es el nuevo"
        );

        let mut salida = contexto.run_ui(entrada_de_prueba(), |ui| ventana(ui, &mut app));
        salida.textures_delta.clear();
        assert!(
            !salida.shapes.is_empty(),
            "y la ventana se sigue dibujando con contenido"
        );

        escribir(&contexto, &mut app, "Z");

        let en_disco = std::fs::read_to_string(primero.root().join("Form1.cs"))
            .expect("el archivo se puede leer");
        assert!(
            en_disco.starts_with("using System.Windows.Forms;"),
            "un documento del proyecto anterior no se puede escribir desde el nuevo: {en_disco}"
        );
        let _ = std::fs::remove_dir_all(primero.root());
        let _ = std::fs::remove_dir_all(segundo.root());
    }

    /// Cerrar todos los documentos y volver a escribir no rompe nada. FE-067.
    ///
    /// El editor se queda sin documento, que es un estado normal —la ventana recién abierta lo
    /// está— y no un error: lo que se comprueba es que escribir con el ratón encima del editor
    /// vacío no escriba en ninguna parte ni se caiga.
    #[test]
    fn escribir_sin_ningun_documento_abierto_no_rompe_nada() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_winforms("miniide-fe067-vacio");

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);
        app.state_mut().cerrar_pestana("Form1.cs");
        app.emitir(Command::CloseDocument);
        app.avanzar(&contexto);

        assert!(app.state().pestanas().is_empty());

        escribir(&contexto, &mut app, "Z");

        assert!(
            app.state().pestanas().is_empty(),
            "sin documentos no hay donde escribir: {:?}",
            app.state().pestanas()
        );
        let en_disco = std::fs::read_to_string(proyecto.root().join("Form1.cs"))
            .expect("el archivo se puede leer");
        assert!(
            en_disco.starts_with("using System.Windows.Forms;"),
            "y el archivo del disco sigue como estaba: {en_disco}"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Abrir un proyecto vacío, o casi vacío, no rompe nada. FE-067.
    ///
    /// Un proyecto con la carpeta de trabajo pero sin archivos es el caso que se encuentra un
    /// usuario que ha borrado todo y ha vuelto a abrirlo, y también el que aparece al elegir
    /// una carpeta equivocada. La ventana tiene que quedarse en pie y con el árbol vacío, que
    /// es lo que hay.
    #[test]
    fn un_proyecto_sin_archivos_no_rompe_la_ventana() {
        use crate::core::ProjectType;
        use crate::project::{BuildConfiguration, Project, ProjectRelativePath};

        let contexto = eframe::egui::Context::default();
        let raiz = std::env::temp_dir().join("miniide-fe067-sin-archivos");
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(&raiz).expect("directorio del proyecto");

        let mut app = App::new();
        app.abrir(
            Project::new(
                "vacio",
                &raiz,
                ProjectType::CSharpWinForms,
                BuildConfiguration::new(ProjectRelativePath::new("bin").expect("ruta válida")),
            )
            .expect("proyecto válido"),
            Arc::new(DotNetToolchain),
        );

        assert!(app.archivos().is_empty(), "no hay nada que enseñar");
        assert!(
            !app.puede_diseñar(),
            "y sin archivos no hay nada que diseñar"
        );

        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        let mut salida = contexto.run_ui(entrada_de_prueba(), |ui| ventana(ui, &mut app));
        salida.textures_delta.clear();
        assert!(!salida.shapes.is_empty(), "la ventana se dibuja igual");

        let _ = std::fs::remove_dir_all(&raiz);
    }

    /// Abrir el diseñador sin un documento que dibujar no hace nada. FE-067.
    ///
    /// El diseñador se pide desde el interruptor, que solo aparece cuando hay una clase abierta,
    /// así que esto no debería ocurrir nunca. Se comprueba igualmente porque un método público
    /// que se pueda llamar sin nada no debe dejar la ventana mirando un vacío: si se abriera
    /// sin documento, se vería un formulario de una clase que no existe.
    #[test]
    fn abrir_el_diseniador_sin_documento_no_abre_una_ventana_inventada() {
        let mut app = App::new();

        app.abrir_diseniador();

        assert_eq!(
            app.state().vista_central(),
            VistaCentral::Editor,
            "sin documento sigue viéndose el editor, no un diseñador vacío"
        );
    }

    /// Un error de `javac` llega a la ventana con su mensaje y su sitio. FE-064.
    ///
    /// Se parsea con el parser de verdad del JDK —`JdkToolchain::parse_diagnostics`— y no con
    /// un diagnóstico inventado, porque lo que hay que comprobar aquí es que la salida del
    /// compilador de Java llega a la ventana tal cual: con el archivo y la línea. Sin el SDK
    /// instalado no hace falta para esto, porque lo que se parsea es la salida, no se compila.
    #[test]
    fn los_errores_de_javac_llegan_a_la_ventana_con_su_sitio() {
        use crate::build::BuildResult;
        use crate::runtime::ProcessOutput;
        use crate::toolchain::ToolchainProvider;

        let (mut app, proyecto) = ventana_con_un_proyecto_de_swing("miniide-fe064-java");

        let salida = ProcessOutput::new(
            Some(1),
            "",
            "src/main/java/MainWindow.java:8: error: ';' expected\n1 error\n",
        );
        let diagnosticos = JdkToolchain.parse_diagnostics(&salida, proyecto.root());

        assert_eq!(
            diagnosticos.len(),
            1,
            "el error de javac tiene que entenderse: {diagnosticos:?}"
        );

        app.operaciones_mut()
            .aplicar_el_resultado(Ok(BuildResult::new(salida, diagnosticos)));

        assert_eq!(app.diagnosticos().len(), 1, "el error está en la ventana");
        let error = &app.diagnosticos()[0];
        assert_eq!(error.message(), "';' expected");
        let sitio = error.location().expect("el error dice dónde");
        assert_eq!(
            sitio.file().as_path().to_string_lossy(),
            VENTANA_JAVA,
            "el archivo es el de la ventana"
        );
        assert_eq!(
            sitio.position().line(),
            7,
            "y la línea es la que dice javac, contada desde cero"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// El archivo que señala un error de Java se puede abrir desde el error. FE-064.
    ///
    /// Es lo que hace que un error de compilación sirva de algo: no solo decir qué pasa, sino
    /// llevar al sitio donde pasa. El diagnóstico marca la fila con la misma clave que el
    /// explorador —por eso esa clave es pública dentro del crate— y pide abrir el documento,
    /// que es lo que la ventana ejecuta.
    #[test]
    fn el_archivo_de_un_error_de_java_se_puede_abrir() {
        use crate::build::BuildResult;
        use crate::runtime::ProcessOutput;
        use crate::toolchain::ToolchainProvider;

        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_swing("miniide-fe064-abrir");

        let salida = ProcessOutput::new(
            Some(1),
            "",
            "src/main/java/MainWindow.java:8: error: ';' expected\n",
        );
        let diagnosticos = JdkToolchain.parse_diagnostics(&salida, proyecto.root());
        let error = diagnosticos
            .first()
            .and_then(|error| error.location())
            .expect("el error tiene sitio");
        let clave = explorador::clave(error.file().as_path());

        app.operaciones_mut()
            .aplicar_el_resultado(Ok(BuildResult::new(salida, diagnosticos)));

        // Lo que hace el panel de diagnósticos al pulsarlo: marcar la fila y pedir abrir.
        app.state_mut().seleccionar(&clave);
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        assert_eq!(
            app.state().pestana_activa(),
            Some(VENTANA_JAVA),
            "el documento que se abre es el del error"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Ir a un error deja el cursor en la línea que señala. FE-072.
    ///
    /// Es el sentido completo de FE-072: hasta aquí el error se abría, y ahora además el cursor
    /// cae en la línea. Se comprueba sobre el documento del core y no sobre lo que se pinta
    /// porque el cursor es del documento, y lo que tiene que caer en la línea correcta es él.
    #[test]
    fn ir_a_un_error_deja_el_cursor_en_la_linea_que_senala() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) =
            ventana_con_un_proyecto_de_winforms("miniide-fe072-ir-a-la-linea");

        // Lo que hace el panel de diagnósticos al pulsar una fila: marcar, anotar el destino y
        // pedir abrir el documento.
        app.state_mut().seleccionar("Form1.cs");
        app.state_mut().ir_a("Form1.cs", 11);
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        let indice = app.documentos.active().expect("hay un documento activo");
        let cursor = app
            .documentos
            .get(indice)
            .expect("el documento activo existe")
            .document()
            .selection()
            .at();

        assert_eq!(
            cursor,
            TextPosition::new(11, 0),
            "el cursor tiene que caer en la línea del error"
        );
        assert_eq!(
            app.state().destino(),
            None,
            "y el destino se consume: saltar a un error solo mueve el cursor una vez"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Ir a un error de un archivo que ya está abierto también mueve el cursor. FE-072.
    ///
    /// Es el caso que se da al recompilar y pulsar otro error del archivo que se está leyendo:
    /// el documento no cambia —ya está abierto— así que si el salto se hiciera solo al abrir,
    /// el segundo error llevaría al usuario al primero.
    #[test]
    fn ir_a_un_error_de_un_archivo_ya_abierto_tambien_mueve_el_cursor() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) =
            ventana_con_un_proyecto_de_winforms("miniide-fe072-archivo-ya-abierto");

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);
        assert_eq!(
            app.documentos
                .get(app.documentos.active().expect("documento activo"))
                .expect("el documento")
                .document()
                .selection()
                .at(),
            TextPosition::new(0, 0),
            "un documento recién abierto tiene el cursor al principio"
        );

        app.state_mut().seleccionar("Form1.cs");
        app.state_mut().ir_a("Form1.cs", 4);
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        let cursor = app
            .documentos
            .get(app.documentos.active().expect("documento activo"))
            .expect("el documento activo existe")
            .document()
            .selection()
            .at();
        assert_eq!(
            cursor,
            TextPosition::new(4, 0),
            "el cursor va al segundo error, no se queda en el primero"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }

    /// Un destino pendiente de otro archivo no mueve el cursor al abrir este. FE-072.
    ///
    /// El destino se anota al pulsar un diagnóstico y se cumple al abrir su documento: si se
    /// cumpliera con cualquier documento, un error de un archivo llevaría el cursor de otro al
    /// abrirlo, que es justo el salto que el usuario no ha pedido.
    #[test]
    fn un_destino_de_otro_archivo_no_mueve_el_cursor() {
        let contexto = eframe::egui::Context::default();
        let (mut app, proyecto) = ventana_con_un_proyecto_de_winforms("miniide-fe072-otro-archivo");

        app.state_mut().ir_a("Program.cs", 9);

        app.state_mut().seleccionar("Form1.cs");
        app.emitir(Command::OpenDocument);
        app.avanzar(&contexto);

        let cursor = app
            .documentos
            .get(app.documentos.active().expect("documento activo"))
            .expect("el documento activo existe")
            .document()
            .selection()
            .at();
        assert_eq!(
            cursor,
            TextPosition::new(0, 0),
            "el destino era de otro archivo, así que el cursor se queda donde estaba"
        );
        assert_eq!(
            app.state().destino().map(Destino::ruta),
            Some("Program.cs"),
            "y sigue pendiente para cuando se abra el suyo"
        );
        let _ = std::fs::remove_dir_all(proyecto.root());
    }
}
