//! Los estados de compilación y ejecución, y el trabajo que hay detrás.
//!
//! Los estados que ve el usuario mientras compila y se ejecuta son los de FE-038 y FE-039,
//! y el trabajo que los mueve es el de FE-043. Los dos son la misma cosa: una ventana que
//! ha pedido compilar y todavía no sabe cómo ha ido.
//!
//! Los estados no son la verdad. La verdad es la que contesta el core —el código de salida
//! del proceso— y la que sigue viva en el sistema —el proceso en marcha—. Este módulo solo
//! traduce una cosa en la otra para que haya algo que enseñar, y por eso se actualiza al
//! recoger lo que el core devuelve y no al pulsar el usuario: un estado que se pusiera
//! "compilando" al Compilar y se quedara ahí para siempre sería una ventana que miente.
//!
//! El trabajo no se espera: `start_build` lo pone en otro hilo y deja el canal por el que
//! llega su respuesta, y `recoger` la recoge con `try_recv`, que vuelve al instante tanto
//! si ya ha llegado algo como si todavía no. Un `recv` aquí dejaría la ventana congelada
//! mientras compila, que es lo que T-098 y AGENTS.md §14 prohíben.
//!
//! Lo que sí se guarda es si hay algo en marcha, porque eso es lo que le dice a la ventana
//! que tiene que volver a mirar el canal: un resultado que llega a un canal que nadie mira
//! se pierde igual que si no hubiera llegado.

use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::Arc;

use crate::build::{start_build, BuildResult};
use crate::core::CoreResult;
use crate::project::Project;
use crate::runtime::{ProcessId, ProcessRegistry, ProcessState};
use crate::toolchain::ToolchainProvider;

/// En qué está la compilación. FE-038.
///
/// Son los cuatro estados que la ventana tiene que poder enseñar: no compilando, compilando,
/// compilada bien y compilada mal. Los dos últimos no son lo mismo que "no compila": una
/// compilación que ha salido bien y una que ha salido mal son las dos formas de acabar, y
/// quien no las distingue necesita un estado más para decir "antes de esto".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EstadoDeBuild {
    /// No hay ninguna compilación en marcha ni ninguna que haya terminado.
    #[default]
    Inactivo,
    /// Hay una compilación en marcha.
    Compilando,
    /// La última compilación terminó con código de salida cero.
    Exitosa,
    /// La última compilación no terminó bien, o no se pudo llegar a compilar.
    Fallida,
}

impl EstadoDeBuild {
    /// Lo que dice la ventana.
    ///
    /// Son palabras y no los nombres de las variantes porque esto lo lee alguien: al lado
    /// de "Compilación", "Correcta" dice lo mismo que `Exitosa` y se entiende mejor.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inactivo => "Inactivo",
            Self::Compilando => "Compilando",
            Self::Exitosa => "Correcta",
            Self::Fallida => "Fallida",
        }
    }

    /// Si hay una compilación sin terminar.
    ///
    /// Es lo que le dice a la ventana que tiene trabajo en marcha, y por eso solo mira la
    /// compilación: estar compilando es lo único que puede terminar sola.
    pub fn esta_trabajando(self) -> bool {
        matches!(self, Self::Compilando)
    }

    /// El estado que corresponde a lo que conteste el core.
    ///
    /// Decide `BuildResult::succeeded`, que mira el código de salida del proceso, y no
    /// esta función: si va bien o mal es del core, y decidirlo aquí sería tener dos
    /// verdades sobre lo mismo y que se separen el día que el core cambie.
    ///
    /// Un error es una compilación fallida y no un estado aparte. El motivo —que falte la
    /// herramienta, que el proyecto no tenga fuentes— lo dicen el `CoreError` o el
    /// diagnóstico que vienen con él, y ese es el sitio donde se explica lo que pasó.
    fn desde(resultado: CoreResult<BuildResult>) -> Self {
        match resultado {
            Ok(resultado) if resultado.succeeded() => Self::Exitosa,
            _ => Self::Fallida,
        }
    }
}

/// En qué está la ejecución. FE-039.
///
/// Son los cuatro estados que la ventana tiene que poder enseñar: no hay nada corriendo,
/// hay algo corriendo, se está parando y se ha terminado. `Terminado` no distingue entre
/// salir bien y salir mal porque para la ventana es lo mismo: el proceso ya no está, y lo
/// que saliera mal se dice en la salida y en los diagnósticos, no cambiando el nombre del
/// estado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EstadoDeRun {
    /// No hay ninguna ejecución en marcha.
    #[default]
    Inactivo,
    /// Hay un proceso vivo.
    Ejecutando,
    /// Se ha pedido pararlo y el sistema todavía no ha confirmado que se paró.
    Deteniendo,
    /// El proceso ya no está, salga como salga.
    Terminado,
}

impl EstadoDeRun {
    /// Lo que dice la ventana.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inactivo => "Inactivo",
            Self::Ejecutando => "Ejecutando",
            Self::Deteniendo => "Deteniendo",
            Self::Terminado => "Terminado",
        }
    }

    /// Si hay algo que todavía puede cambiar sin que nadie pulse nada.
    ///
    /// `Deteniendo` cuenta porque el proceso todavía está vivo y se está acabando de
    /// parar: si no contara, la ventana dejaría de mirarlo en el momento de pedir la parada
    /// y se quedaría diciendo "Deteniendo" para siempre si el proceso no llegara a morir.
    pub fn esta_trabajando(self) -> bool {
        matches!(self, Self::Ejecutando | Self::Deteniendo)
    }
}

impl From<ProcessState> for EstadoDeRun {
    /// El estado de la ventana según lo que diga el proceso real.
    ///
    /// Es una traducción y no una decisión. El proceso es quien sabe si sigue vivo, y a
    /// `From` se le pregunta una vez: si está vivo se está ejecutando, y si no, ha
    /// terminado. Por eso los tres estados finales se traducen al mismo: `Exited`,
    /// `Stopped` y `Failed` son tres maneras de que un proceso ya no esté ahí, y aquí no
    /// se distingue porque esa diferencia la enseña la salida, no la barra de estado.
    fn from(estado: ProcessState) -> Self {
        match estado {
            ProcessState::Running => Self::Ejecutando,
            ProcessState::Exited | ProcessState::Stopped | ProcessState::Failed => Self::Terminado,
        }
    }
}

/// Lo que MiniIDE tiene en marcha, y cómo se ve.
///
/// Se divide en dos porque son dos cosas distintas: los estados son lo que se enseña y se
/// cambian desde aquí, y el trabajo en marcha son los canales y los procesos, que solo se
/// tocan cuando el core contesta. Los canales se guardan porque el resultado se pierde si
/// nadie los mira: un canal sin leer deja su mensaje dentro y no vuelve a avisar.
#[derive(Debug, Default)]
pub struct Operaciones {
    build: EstadoDeBuild,
    run: EstadoDeRun,
    /// Por dónde llega la compilación en marcha, si hay alguna.
    ///
    /// Es `Option` y no un canal siempre presente porque "no hay nada en marcha" y "hay algo
    /// que no ha terminado" son estados distintos, y con un solo canal vacío no se sabe
    /// cuál de los dos se está mirando.
    build_en_curso: Option<Receiver<CoreResult<BuildResult>>>,
    /// El proceso que se está ejecutando, si lo hay.
    ///
    /// Se guarda el identificador y no el proceso porque el registro es el que sabe cuál es
    /// cuál, y tener aquí una copia sería tener dos listas de procesos que se separan en
    /// cuanto se lanza algo por otro camino.
    proceso: Option<ProcessId>,
    procesos: ProcessRegistry,
}

impl Operaciones {
    pub fn new() -> Self {
        Self::default()
    }

    /// En qué está la compilación.
    pub fn estado_de_build(&self) -> EstadoDeBuild {
        self.build
    }

    /// En qué está la ejecución.
    pub fn estado_de_run(&self) -> EstadoDeRun {
        self.run
    }

    /// Si hay trabajo sin terminar.
    ///
    /// Es lo que decide si la ventana tiene que seguir mirando: mientras haya algo en
    /// marcha el resultado puede llegar en cualquier momento, y cuando no lo hay no hay nada
    /// que esperar y la ventana puede quedarse quieta.
    pub fn esta_ocupada(&self) -> bool {
        self.build.esta_trabajando() || self.run.esta_trabajando()
    }

    /// Compila `proyecto` en segundo plano y deja la ventana en `Compilando`.
    ///
    /// Vuelve en cuanto el trabajo está en marcha, no cuando ha terminado: por eso el
    /// resultado llega por el canal y no de vuelta. Quien llama sigue dando vueltas, y eso
    /// es lo que hace que la ventana no se congele.
    pub fn compilar(&mut self, proveedor: Arc<dyn ToolchainProvider>, proyecto: Project) {
        self.build_en_curso = Some(start_build(proveedor, proyecto));
        self.build = EstadoDeBuild::Compilando;
    }

    /// Ejecuta `proyecto` y deja la ventana en `Ejecutando`.
    ///
    /// Se lanza con el registro de procesos y no con `start_run` porque el registro es lo
    /// único que sabe qué proceso hay vivo y lo único que lo puede parar: un proceso
    /// lanzado dentro de un hilo del que solo se guarda el canal deja de ser localizable en
    /// cuanto ese hilo termina, y FE-042 depende de poder pararlo.
    ///
    /// Lo que se pierde es la salida mientras corre, y eso sigue siendo de T-098: el
    /// registro recoge lo que el proceso dejó al terminar, no lo que va diciendo.
    pub fn ejecutar(
        &mut self,
        proveedor: &dyn ToolchainProvider,
        proyecto: &Project,
    ) -> CoreResult<()> {
        let invocacion = proveedor.run_invocation(proyecto)?;
        let proceso = self.procesos.spawn(&invocacion)?;

        self.proceso = Some(proceso);
        self.run = EstadoDeRun::Ejecutando;

        Ok(())
    }

    /// Detiene lo que se esté ejecutando.
    ///
    /// Pone `Deteniendo` y no `Terminado` porque aquí se acaba de *pedir* la parada, y lo
    /// que confirma que se ha parado es el sistema, que contesta al siguiente `recoger`. Si
    /// se pusiera `Terminado` al pedirla, el usuario vería que ha parado un proceso que
    /// todavía está corriendo, y ese proceso puede tener archivos abiertos.
    pub fn detener(&mut self) -> CoreResult<()> {
        let Some(proceso) = self.proceso else {
            return Ok(());
        };

        self.procesos.stop(proceso)?;
        self.run = EstadoDeRun::Deteniendo;

        Ok(())
    }

    /// Recoge lo que haya terminado y actualiza los estados. No espera.
    ///
    /// Es `try_recv` y no `recv` porque esta función se llama en cada frame: con `recv` la
    /// ventana se quedaría parada dentro de ella hasta que acabara la compilación, que es
    /// decir, toda la razón de FE-043.
    ///
    /// Se llama antes de ejecutar lo que se haya pedido, a propósito. Si se llamara después,
    /// una parada se vería como `Terminado` en el mismo frame en que se pide, y `Deteniendo`
    /// no se vería nunca.
    pub fn recoger(&mut self) {
        self.recoger_build();
        self.recoger_run();
    }

    /// Actualiza el estado de compilación con lo que conteste el core, si ya ha contestado.
    fn recoger_build(&mut self) {
        let respuesta = self.build_en_curso.as_ref().map(Receiver::try_recv);

        match respuesta {
            Some(Ok(resultado)) => {
                self.build = EstadoDeBuild::desde(resultado);
                self.build_en_curso = None;
            }
            // El hilo que compilaba se ha ido sin dejar resultado. No debería pasar: el
            // core recoge hasta el panic y siempre manda algo. Se trata como compilación
            // fallida y no se espera más, porque un canal sin emisor no va a traer nada
            // nunca y dejarlo ahí sería quedarse mirando para siempre.
            Some(Err(TryRecvError::Disconnected)) => {
                self.build = EstadoDeBuild::Fallida;
                self.build_en_curso = None;
            }
            // Todavía no ha terminado. No es un error: es la razón de que este módulo
            // exista, y la ventana tiene que poder seguir dibujando mientras tanto.
            Some(Err(TryRecvError::Empty)) | None => {}
        }
    }

    /// Actualiza el estado de ejecución con lo que diga el proceso de verdad.
    ///
    /// Solo mira el proceso cuando ya hay alguno: mientras no se ha lanzado nada, el estado
    /// que hay es el que se puso al lanzarlo, y preguntarle al registro por un proceso que
    /// no existe no dice nada. Lo que hace es mirar si el que había sigue vivo y, si no,
    /// darlo por terminado y olvidarse de él, que si no el registro crecería con cada
    /// ejecución que se lanzara.
    fn recoger_run(&mut self) {
        let Some(id) = self.proceso else {
            return;
        };

        let Some(proceso) = self.procesos.get_mut(id) else {
            self.proceso = None;
            return;
        };

        if proceso.state() != ProcessState::Running {
            self.procesos.forget(id);
            self.proceso = None;
            self.run = EstadoDeRun::Terminado;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::thread::ThreadId;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::core::{CoreError, ProjectType};
    use crate::diagnostics::Diagnostic;
    use crate::project::{BuildConfiguration, ProjectRelativePath};
    use crate::runtime::ProcessOutput;
    use crate::toolchain::Invocation;

    /// Un proyecto cualquiera: estos tests no leen sus archivos.
    fn proyecto() -> Project {
        Project::new(
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

    /// Una herramienta que no hace falta instalar para probar estos estados.
    ///
    /// Falla al preparar la llamada a propósito, así que ni hace falta el JDK ni hace falta
    /// lanzar nada: lo que se está probando es qué estado deja la ventana cuando el core
    /// contesta, y contesta igual de rápido con un error que con un código de salida. El
    /// retardo es lo que permite tener una compilación *en marcha* durante un test, que es
    /// lo que necesita FE-043 para comprobar que la ventana no se para a esperarla.
    struct Falsa {
        espera: Duration,
        donde: Mutex<Option<ThreadId>>,
    }

    impl Falsa {
        fn nueva(espera: Duration) -> Arc<Self> {
            Arc::new(Self {
                espera,
                donde: Mutex::new(None),
            })
        }

        fn hilo_de_la_llamada(&self) -> Option<ThreadId> {
            *self.donde.lock().expect("el candado")
        }
    }

    impl ToolchainProvider for Falsa {
        fn project_type(&self) -> ProjectType {
            ProjectType::JavaSwing
        }

        fn tool(&self) -> &'static str {
            "herramienta de los tests"
        }

        fn is_available(&self) -> bool {
            true
        }

        fn build_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
            *self.donde.lock().expect("el candado") = Some(std::thread::current().id());
            std::thread::sleep(self.espera);

            Err(CoreError::Unsupported(
                "los tests no compilan de verdad".to_string(),
            ))
        }

        fn run_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
            Err(CoreError::Unsupported(
                "los tests no ejecutan de verdad".to_string(),
            ))
        }

        fn parse_diagnostics(&self, _output: &ProcessOutput, _root: &Path) -> Vec<Diagnostic> {
            Vec::new()
        }
    }

    /// Una herramienta que lanza un proceso que tarda.
    ///
    /// Solo se usa en los tests ignorados, porque lanzar un proceso de verdad depende de
    /// que haya uno que lanzar y de que el sistema lo mate, y eso no puede ser un test que
    /// se ejecute en cada `cargo test`.
    struct ProcesoLento;

    impl ToolchainProvider for ProcesoLento {
        fn project_type(&self) -> ProjectType {
            ProjectType::JavaSwing
        }

        fn tool(&self) -> &'static str {
            "proceso de los tests"
        }

        fn is_available(&self) -> bool {
            true
        }

        fn build_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
            Err(CoreError::Unsupported("no hace falta".to_string()))
        }

        fn run_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
            Ok(Invocation::new(
                "cmd",
                vec!["/C".to_string(), "ping -n 20 127.0.0.1 >NUL".to_string()],
                PathBuf::from("C:\\"),
            ))
        }

        fn parse_diagnostics(&self, _output: &ProcessOutput, _root: &Path) -> Vec<Diagnostic> {
            Vec::new()
        }
    }

    /// Espera a que `operaciones` se quede sin trabajo en marcha, y dice si lo consiguió.
    ///
    /// Se espera en vez de mirar una vez porque el trabajo va en otro hilo, y ese hilo no se
    /// pone a la vez que quien lo lanza. Si no llega nunca, el test falla en vez de quedarse
    /// con el estado a medias.
    fn esperar_a_que_termine(operaciones: &mut Operaciones) -> bool {
        for _ in 0..300 {
            operaciones.recoger();

            if !operaciones.esta_ocupada() {
                return true;
            }

            std::thread::sleep(Duration::from_millis(10));
        }

        false
    }

    /// Una ventana nueva está quieta en las dos cosas.
    ///
    /// Es el punto de partida de FE-038 y FE-039: si los dos estados arrancaran con algo
    /// puesto, la ventana mentiría sobre lo que está haciendo antes de que nadie le haya
    /// pedido nada.
    #[test]
    fn una_ventana_nueva_no_compila_ni_ejecuta() {
        let operaciones = Operaciones::new();

        assert_eq!(operaciones.estado_de_build(), EstadoDeBuild::Inactivo);
        assert_eq!(operaciones.estado_de_run(), EstadoDeRun::Inactivo);
        assert!(
            !operaciones.esta_ocupada(),
            "sin trabajo en marcha la ventana no tiene que mirar ningún canal"
        );
    }

    /// Los dos estados dicen en palabras en qué están.
    ///
    /// La barra de estado enseña estas palabras y no los nombres de las variantes, así que
    /// si una se queda vacía aparece un hueco que parece que MiniIDE no ha terminado de
    /// dibujarse.
    #[test]
    fn los_estados_dicen_en_que_estan() {
        for (estado, texto) in [
            (EstadoDeBuild::Inactivo, "Inactivo"),
            (EstadoDeBuild::Compilando, "Compilando"),
            (EstadoDeBuild::Exitosa, "Correcta"),
            (EstadoDeBuild::Fallida, "Fallida"),
        ] {
            assert_eq!(
                estado.as_str(),
                texto,
                "el estado de compilación tiene que decir {texto:?}"
            );
        }

        for (estado, texto) in [
            (EstadoDeRun::Inactivo, "Inactivo"),
            (EstadoDeRun::Ejecutando, "Ejecutando"),
            (EstadoDeRun::Deteniendo, "Deteniendo"),
            (EstadoDeRun::Terminado, "Terminado"),
        ] {
            assert_eq!(
                estado.as_str(),
                texto,
                "el estado de ejecución tiene que decir {texto:?}"
            );
        }
    }

    /// Compilar pone el estado en `Compilando` antes de que exista el resultado.
    ///
    /// Es lo que ve el usuario al pulsar, y va antes que el resultado a propósito: si el
    /// estado apareciera solo cuando la compilación hubiera terminado, la ventana no diría
    /// nada durante todo el rato que tarda, que es justo cuando más hace falta saber que
    /// está compilando.
    #[test]
    fn compilar_ponte_compilando_y_no_espera_al_resultado() {
        let mut operaciones = Operaciones::new();

        let antes = Instant::now();
        operaciones.compilar(Falsa::nueva(Duration::from_millis(600)), proyecto());
        let esperado = antes.elapsed();

        assert_eq!(
            operaciones.estado_de_build(),
            EstadoDeBuild::Compilando,
            "al pedir compilar hay que estar compilando, no esperando"
        );
        assert!(
            operaciones.esta_ocupada(),
            "con una compilación en marcha hay trabajo que recoger"
        );
        assert!(
            esperado < Duration::from_millis(400),
            "pedir compilar no puede tardar lo que tarda compilar: tardó {esperado:?}"
        );
    }

    /// Recoger lo que todavía no ha llegado no cambia nada y no se queda esperando.
    ///
    /// Esta es la mitad de FE-043 que se puede comprobar sin ventana: si `recoger` esperara
    /// por un resultado que no ha llegado, la ventana se quedaría quieta durante toda la
    /// compilación. Que un `recoger` con el canal vacío vuelva, y además deje el estado
    /// como estaba, es lo que dice que se puede seguir dibujando.
    #[test]
    fn recoger_no_espera_a_lo_que_no_ha_llegado() {
        let mut operaciones = Operaciones::new();
        operaciones.compilar(Falsa::nueva(Duration::from_millis(600)), proyecto());

        let antes = Instant::now();
        operaciones.recoger();
        let esperado = antes.elapsed();

        assert_eq!(
            operaciones.estado_de_build(),
            EstadoDeBuild::Compilando,
            "recoger no decide nada: solo recoge lo que ya ha llegado"
        );
        assert!(
            esperado < Duration::from_millis(200),
            "recoger tiene que volver al instante y tardó {esperado:?}"
        );
    }

    /// El trabajo no se hace en el hilo de quien lo pide. FE-043.
    ///
    /// Es lo que separa "la ventana sigue viva mientras compila" de "la ventana espera a que
    /// compile": las dos se ven igual si solo se mira el resultado, y solo se distinguen en el
    /// hilo. Si el core volviera a compilar en el hilo de quien llama, FE-043 dejaría de
    /// cumplirse sin que nada en esta ventana cambiara, así que el hilo se comprueba aquí y
    /// no en el código que lo usa.
    #[test]
    fn el_trabajo_no_se_hace_en_el_hilo_de_quien_lo_pide() {
        let herramienta = Falsa::nueva(Duration::from_millis(0));
        let ventana = std::thread::current().id();

        let mut operaciones = Operaciones::new();
        operaciones.compilar(herramienta.clone(), proyecto());

        assert!(
            esperar_a_que_termine(&mut operaciones),
            "el trabajo tiene que terminar: el core contesta siempre"
        );

        let trabajo = herramienta
            .hilo_de_la_llamada()
            .expect("el proveedor tiene que preparar la llamada");
        assert_ne!(
            trabajo, ventana,
            "compilar en el hilo que lo pide deja la ventana congelada mientras compila"
        );
    }

    /// El estado de compilación sigue al core. FE-038.
    ///
    /// Se miran las dos formas de acabar porque una ventana que solo distinguiera
    /// "compilando" de "ya está" dejaría al usuario sin saber si su código compila. El caso
    /// bueno no se puede tener sin lanzar un compilador de verdad, así que se comprueba la
    /// traducción directamente sobre el `BuildResult` que devuelve el core; el camino entero
    /// por `start_build` está en `una_compilacion_que_termina_deja_de_ocupar_la_ventana`.
    #[test]
    fn el_estado_de_compilacion_sigue_al_core() {
        let buena = BuildResult::new(ProcessOutput::new(Some(0), "", ""), Vec::new());
        let mala = BuildResult::new(ProcessOutput::new(Some(1), "", ""), Vec::new());

        assert_eq!(
            EstadoDeBuild::desde(Ok(buena)),
            EstadoDeBuild::Exitosa,
            "con código de salida cero la compilación ha ido bien"
        );
        assert_eq!(
            EstadoDeBuild::desde(Ok(mala)),
            EstadoDeBuild::Fallida,
            "con un código de salida que no es cero la compilación ha fallado"
        );
        assert_eq!(
            EstadoDeBuild::desde(Ok(BuildResult::new(ProcessOutput::empty(), Vec::new()))),
            EstadoDeBuild::Fallida,
            "sin código de salida no se puede decir que haya ido bien, así que no se dice"
        );
        assert_eq!(
            EstadoDeBuild::desde(Err(CoreError::Unsupported("nada".to_string()))),
            EstadoDeBuild::Fallida,
            "una compilación que ni se pudo preparar es una compilación fallida"
        );
    }

    /// Cuando el core contesta, la ventana deja de estar compilando y deja de estar ocupada.
    /// FE-038 y FE-043.
    ///
    /// El camino entero: se pide compilar, la ventana se pone a mirar el canal, llega la
    /// respuesta del core y la ventana se queda quieta. Lo que se comprueba es que el
    /// resultado se recoge —que no se pierda en un canal que nadie lee— y que la ventana
    /// vuelve a quedar libre, porque si siguiera ocupada después de recogerlo seguiría
    /// pidiendo repintados para siempre.
    #[test]
    fn una_compilacion_que_termina_deja_de_ocupar_la_ventana() {
        let mut operaciones = Operaciones::new();
        operaciones.compilar(Falsa::nueva(Duration::from_millis(0)), proyecto());

        assert!(
            esperar_a_que_termine(&mut operaciones),
            "la compilación tiene que terminar: el core contesta siempre y aquí no hay \
             ningún motivo para que no llegue"
        );

        assert_eq!(
            operaciones.estado_de_build(),
            EstadoDeBuild::Fallida,
            "esta herramienta de prueba no compila, y el estado tiene que decirlo"
        );
        assert!(
            !operaciones.esta_ocupada(),
            "recogido el resultado ya no hay nada que esperar"
        );
    }

    /// El estado de ejecución sale del proceso real, no de lo que se pulsó. FE-039.
    ///
    /// El proceso es quien sabe si sigue vivo, y por eso el estado se traduce desde
    /// `ProcessState`. La traducción tiene que ser total y sin sorpresas: un proceso que ha
    /// terminado —porque saliera bien, porque saliera mal o porque MiniIDE lo paró— ya no
    /// está corriendo, y la ventana no puede seguir diciendo que sí.
    #[test]
    fn el_estado_de_ejecucion_sigue_al_proceso_real() {
        assert_eq!(
            EstadoDeRun::from(ProcessState::Running),
            EstadoDeRun::Ejecutando,
            "un proceso vivo es una ejecución en marcha"
        );

        for (proceso, nombre) in [
            (ProcessState::Exited, "salió bien"),
            (ProcessState::Stopped, "lo paró MiniIDE"),
            (ProcessState::Failed, "salió mal"),
        ] {
            assert_eq!(
                EstadoDeRun::from(proceso),
                EstadoDeRun::Terminado,
                "un proceso que {nombre} ya no está, y la ventana no puede decir que sí"
            );
        }
    }

    /// Un proceso que se está parando sigue contando como trabajo en marcha. FE-039.
    ///
    /// Es lo que hace que la ventana llegue a `Terminado`: si `Deteniendo` no contara,
    /// `esta_ocupada` sería falsa en cuanto se pide la parada, la ventana dejaría de mirar
    /// el proceso y se quedaría diciendo "Deteniendo" para siempre aunque ya estuviera
    /// parado. Va en su propio test porque es el estado que más se confunde con "ya no hay
    /// nada que hacer".
    #[test]
    fn deteniendo_sigue_siendo_trabajo_en_marcha() {
        assert!(
            EstadoDeRun::Deteniendo.esta_trabajando(),
            "mientras se para hay algo que puede cambiar sin que nadie pulse nada"
        );
        assert!(
            !EstadoDeRun::Terminado.esta_trabajando(),
            "un proceso que ya no está no hace falta seguir esperándolo"
        );
        assert!(
            !EstadoDeRun::Inactivo.esta_trabajando(),
            "nunca se ha ejecutado nada y no hay nada que esperar"
        );
    }

    /// Pedir ejecutar sin poder preparar la llamada no deja la ventana diciendo que se ejecuta.
    ///
    /// Es el mismo criterio que en la compilación: si no se ha lanzado nada no se puede decir
    /// que haya algo corriendo, porque el usuario lo leería como que hay un proceso vivo
    /// que puede tener sus archivos abiertos.
    #[test]
    fn ejecutar_sin_poder_preparar_la_llamada_no_dice_que_se_ejecuta() {
        let mut operaciones = Operaciones::new();

        let resultado = operaciones.ejecutar(&*Falsa::nueva(Duration::from_millis(0)), &proyecto());

        assert!(
            resultado.is_err(),
            "esta herramienta de prueba no sabe ejecutar, y tiene que fallar"
        );
        assert_eq!(
            operaciones.estado_de_run(),
            EstadoDeRun::Inactivo,
            "no se ha lanzado nada, así que no hay nada ejecutándose"
        );
        assert!(
            !operaciones.esta_ocupada(),
            "sin proceso no hay trabajo que recoger"
        );
    }

    /// Parar sin nada corriendo no rompe nada.
    ///
    /// El botón Detener está siempre en la barra (FE-042), así que se puede pulsar sin haber
    /// ejecutado nada, y entonces tiene que no pasar nada: un error por parar sin proceso
    /// sería un fallo del IDE en un botón que el usuario no ha usado mal.
    #[test]
    fn detener_sin_ejecutar_nada_no_falla() {
        let mut operaciones = Operaciones::new();

        operaciones
            .detener()
            .expect("parar sin nada corriendo no tiene nada que fallar");

        assert_eq!(
            operaciones.estado_de_run(),
            EstadoDeRun::Inactivo,
            "no había nada corriendo y no hay nada que decir que se ha parado"
        );
    }

    /// El proceso de verdad manda en el estado de ejecución, de principio a fin. FE-039.
    ///
    /// Este sí lanza un proceso de verdad, y por eso va marcado como ignorado: `ping`
    /// contra uno mismo tarda unos segundos y depende de que el sistema lo mate, así que no
    /// puede ser un test que se ejecute en cada `cargo test`. Se ejecuta aparte con
    /// `cargo test -- --ignored`, que es lo que dice `docs/testing.md`.
    #[test]
    #[ignore = "lanza un proceso de verdad"]
    fn la_ejecucion_corre_y_se_detiene_sobre_el_proceso_real() {
        let mut operaciones = Operaciones::new();
        operaciones
            .ejecutar(&ProcesoLento, &proyecto())
            .expect("un proceso de prueba se puede lanzar");

        assert_eq!(
            operaciones.estado_de_run(),
            EstadoDeRun::Ejecutando,
            "un proceso vivo es una ejecución en marcha"
        );

        operaciones
            .detener()
            .expect("parar un proceso vivo se puede parar");

        assert_eq!(
            operaciones.estado_de_run(),
            EstadoDeRun::Deteniendo,
            "pedir la parada no es lo mismo que haber parado"
        );

        assert!(
            esperar_a_que_termine(&mut operaciones),
            "un proceso parado termina: el sistema lo mata con lo que tenga debajo"
        );

        assert_eq!(
            operaciones.estado_de_run(),
            EstadoDeRun::Terminado,
            "el proceso ya no está, y la ventana tiene que decirlo"
        );
    }
}
