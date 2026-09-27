use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;

use crate::core::{CoreError, CoreResult};
use crate::diagnostics::{Diagnostic, DiagnosticLevel};
use crate::project::Project;
use crate::runtime::{run, ProcessOutput, ProcessState, RunResult};
use crate::toolchain::ToolchainProvider;

/// Lo que dejo una compilacion: si fue bien y la salida del proceso.
///
/// La salida es la misma estructura que devuelve una ejecucion, sin copiarse sus
/// tres datos. Que la compilacion haya ido bien no se guarda: se deduce del codigo
/// de salida del proceso, que es donde esta la verdad.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildResult {
    output: ProcessOutput,
    diagnostics: Vec<Diagnostic>,
}

impl BuildResult {
    /// Normaliza la salida de una compilacion.
    ///
    /// Va bien si el proceso termino con codigo 0. Sin codigo de salida no se
    /// puede afirmar nada, y entonces no fue bien.
    pub fn new(output: ProcessOutput, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            output,
            diagnostics,
        }
    }

    pub fn succeeded(&self) -> bool {
        self.output.exit_code() == Some(0)
    }

    /// La salida del proceso de compilacion, con su codigo, su stdout y su stderr.
    pub fn output(&self) -> &ProcessOutput {
        &self.output
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

/// Compila `project` con el proveedor dado y devuelve el resultado.
///
/// El proveedor prepara la llamada, la ejecuta `runtime` y lee su salida para
/// dejar diagnosticos. Es sincrono: no bloquea todavia, y por eso tampoco se
/// puede usar en el hilo de la interfaz sin dejar la UI congelada mientras
/// compila.
///
/// Sin la herramienta no hay error: el build falla con un diagnostico que dice que
/// falta. Un `Err` obligaria a la interfaz a inventarse el mensaje, y lo que
/// tiene que explicar es la falta de la herramienta, que es cosa de la toolchain.
///
/// Lo que si es un error es lo que el usuario puede arreglar en su proyecto, como
/// un proyecto sin fuentes: eso llega como `CoreError`.
pub fn build_with(provider: &dyn ToolchainProvider, project: &Project) -> CoreResult<BuildResult> {
    if !provider.is_available() {
        return Ok(missing_toolchain(provider));
    }

    let invocation = provider.build_invocation(project)?;

    let output = run(&invocation)?;
    let diagnostics = provider.parse_diagnostics(&output, project.root());

    Ok(BuildResult::new(output, diagnostics))
}

/// El resultado de intentar compilar sin la herramienta: sin codigo de salida,
/// porque no llego a ejecutarse nada, y con un error que lo dice.
fn missing_toolchain(provider: &dyn ToolchainProvider) -> BuildResult {
    let diagnostic = Diagnostic::new(DiagnosticLevel::Error, provider.missing_message(), None);

    BuildResult::new(ProcessOutput::empty(), vec![diagnostic])
}

/// Compila `project` sin esperar, y devuelve por donde llega el resultado.
///
/// `build_with` bloquea mientras compila, asi que llamarlo desde el hilo de la
/// interfaz dejaria la ventana congelada hasta que la compilacion acabase. Aqui el
/// trabajo se pone en otro hilo y se devuelve solo el canal por el que llega:
/// quien lo pide sigue dando vueltas, y el resultado aparece cuando el proceso
/// termina.
///
/// El resultado no se pierde si se llega tarde: el canal lo guarda hasta que
/// alguien lo pida con `recv`.
///
/// El proveedor y el proyecto se llevan en propiedad porque el trabajo se hace en
/// otro hilo, y quien lo llama solo puede seguir con lo que le devuelve.
pub fn start_build(
    provider: Arc<dyn ToolchainProvider>,
    project: Project,
) -> Receiver<CoreResult<BuildResult>> {
    start(move || build_with(&*provider, &project))
}

/// Ejecuta `project` sin esperar, y devuelve por donde llega el resultado.
///
/// Hace falta igual que `start_build`: una aplicacion tarda lo que tarda, y quien
/// la lanza no puede quedarse esperando a que termine.
///
/// El resultado es el de la ejecucion completa, con su salida. El estado que sale
/// aqui es el de un proceso que ya termino; el de uno vivo lo lleva
/// `ProcessRegistry`, que es donde se lo sigue mientras corre.
pub fn start_run(
    provider: Arc<dyn ToolchainProvider>,
    project: Project,
) -> Receiver<CoreResult<RunResult>> {
    start(move || {
        let invocation = provider.run_invocation(&project)?;
        let output = run(&invocation)?;

        let state = match output.exit_code() {
            Some(0) => ProcessState::Exited,
            _ => ProcessState::Failed,
        };

        Ok(RunResult::new(state, output, None))
    })
}

/// Pone `work` en otro hilo y devuelve el canal por el que llega su resultado.
///
/// Es lo que tienen en comun las dos operaciones: las dos son preparar una
/// llamada, esperar a que el proceso termine y mirar su salida, y las dos tardan
/// lo que tarde el proceso. El que espera, espera en su hilo, y no en el del
/// interfaz.
///
/// Es tambien el sitio donde el trabajo se puede romper de verdad, y donde se
/// recoge: leer y escribir archivos, lanzar procesos y entender lo que dice una
/// toolchain son cosas que tocan el disco y el sistema, y un fallo que nadie
/// previó ahi se llevaria por delante el hilo entero. Sin recogerlo, el canal se
/// quedaria sin enviar y quien espera se encontraria un fallo sin resultado en vez
/// de un error que enseñar.
///
/// El panic se recoge aqui y no mas adentro porque este es el limite: a partir de
/// aqui el trabajo es de otro hilo y a partir de aqui se devuelve siempre un
/// resultado. Un error de los que el codigo sabe que pueden pasar no pasa por aqui:
/// lo devuelve el propio trabajo, y se queda como es.
fn start<T, F>(work: F) -> Receiver<CoreResult<T>>
where
    F: FnOnce() -> CoreResult<T> + Send + 'static,
    T: Send + 'static,
{
    let (sender, receiver) = mpsc::channel();

    std::thread::spawn(move || {
        // `AssertUnwindSafe` porque el trabajo se lleva el proyecto y la toolchain en
        // propiedad y no los comparte con nadie mas: si se rompe a mitad, lo que
        // queden a medias se va con el hilo y no lo va a mirar ninguna otra parte.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
            .unwrap_or_else(|panic| Err(CoreError::from_panic(panic)));

        // Si quien pidio el resultado ya no lo quiere, no hay nada que hacer con
        // el: el canal se queda sin receptor y el envio se cae solo.
        let _ = sender.send(result);
    });

    receiver
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CoreError, ProjectType};
    use crate::diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticLocation};
    use crate::document::TextPosition;
    use crate::project::{BuildConfiguration, Project, ProjectRelativePath};
    use crate::runtime::ProcessState;
    use crate::toolchain::Invocation;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::thread::{self, ThreadId};

    /// Una toolchain que no esta instalada, como la de un equipo sin JDK.
    ///
    /// Preparar la llamada revienta a proposito: si el IDE compiles con una
    /// herramienta que no esta, el problema no es la llamada, y el test tiene que
    /// notarlo.
    struct SinHerramienta;

    impl ToolchainProvider for SinHerramienta {
        fn project_type(&self) -> ProjectType {
            ProjectType::JavaSwing
        }

        fn tool(&self) -> &'static str {
            "JDK"
        }

        fn is_available(&self) -> bool {
            false
        }

        fn build_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
            panic!("no se prepara la llamada de una herramienta que no esta")
        }

        fn run_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
            panic!("no se prepara la llamada de una herramienta que no esta")
        }

        fn parse_diagnostics(&self, _output: &ProcessOutput, _root: &Path) -> Vec<Diagnostic> {
            Vec::new()
        }
    }

    /// Un proveedor que no lanza nada: falla al preparar la llamada, asi que el
    /// test no necesita ninguna herramienta instalada ni ningun proceso.
    ///
    /// Ademas apunta en que hilo lo han llamado, que es justo lo que hay que
    /// comprobar.
    struct AnotaHilo {
        donde: Mutex<Option<ThreadId>>,
    }

    impl AnotaHilo {
        fn nuevo() -> Self {
            Self {
                donde: Mutex::new(None),
            }
        }

        fn hilo_de_la_llamada(&self) -> Option<ThreadId> {
            *self.donde.lock().expect("el candado")
        }

        /// Espera a que el trabajo llegue al proveedor, y devuelve el hilo en el que
        /// llego. Se espera en vez de mirar de inmediato porque el trabajo va en otro
        /// hilo, y ese hilo no se pone a la vez que quien lo lanza.
        fn esperar_al_hilo(&self) -> Option<ThreadId> {
            for _ in 0..200 {
                if let Some(donde) = self.hilo_de_la_llamada() {
                    return Some(donde);
                }

                std::thread::sleep(std::time::Duration::from_millis(25));
            }

            None
        }
    }

    impl ToolchainProvider for AnotaHilo {
        fn project_type(&self) -> ProjectType {
            ProjectType::JavaSwing
        }

        fn tool(&self) -> &'static str {
            "herramienta de mentira"
        }

        fn is_available(&self) -> bool {
            true
        }

        fn build_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
            *self.donde.lock().expect("el candado") = Some(thread::current().id());

            Err(CoreError::Unsupported(
                "sin herramientas de prueba".to_string(),
            ))
        }

        fn run_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
            Err(CoreError::Unsupported(
                "sin herramientas de prueba".to_string(),
            ))
        }

        fn parse_diagnostics(&self, _output: &ProcessOutput, _root: &Path) -> Vec<Diagnostic> {
            Vec::new()
        }
    }

    /// Un proveedor cuya compilacion tarda lo que tarda el proceso que lanza.
    struct CompilacionLenta {
        segundos: u32,
    }

    impl ToolchainProvider for CompilacionLenta {
        fn project_type(&self) -> ProjectType {
            ProjectType::JavaSwing
        }

        fn tool(&self) -> &'static str {
            "compilacion lenta"
        }

        fn is_available(&self) -> bool {
            true
        }

        fn build_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
            Ok(Invocation::new(
                "cmd",
                vec![
                    "/C".to_string(),
                    format!("ping -n {} 127.0.0.1 >NUL", self.segundos),
                ],
                std::env::temp_dir(),
            ))
        }

        fn run_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
            self.build_invocation(
                &Project::new(
                    "App",
                    std::env::temp_dir(),
                    ProjectType::JavaSwing,
                    BuildConfiguration::new(ProjectRelativePath::new("bin").unwrap()),
                )
                .expect("proyecto"),
            )
        }

        fn parse_diagnostics(&self, _output: &ProcessOutput, _root: &Path) -> Vec<Diagnostic> {
            Vec::new()
        }
    }

    /// Una toolchain que revienta por dentro, como revienta el codigo cuando hay un
    /// fallo que nadie previó.
    ///
    /// Es un panic y no un error: un error se puede devolver y quien lo llama lo sabe
    /// manejar, mientras que un panic se lleva por delante el hilo que lo ha tenido.
    /// Preparar la llamada es donde mas cosas se tocan (el proyecto, las rutas, los
    /// argumentos), y por eso revienta aqui.
    struct FallaPorDentro;

    impl ToolchainProvider for FallaPorDentro {
        fn project_type(&self) -> ProjectType {
            ProjectType::JavaSwing
        }

        fn tool(&self) -> &'static str {
            "toolchain que revienta"
        }

        fn is_available(&self) -> bool {
            true
        }

        fn build_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
            panic!("la toolchain ha fallado por dentro")
        }

        fn run_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
            panic!("la toolchain ha fallado por dentro")
        }

        fn parse_diagnostics(&self, _output: &ProcessOutput, _root: &Path) -> Vec<Diagnostic> {
            Vec::new()
        }
    }

    /// Un proyecto de Java real en un directorio temporal, para que se pueda
    /// comprobar que un build fallido no lo toca.
    fn java_project(name: &str) -> (Project, PathBuf) {
        let root = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).expect("directorio");
        std::fs::write(root.join("src/Main.java"), "public class Main { }\n").expect("fuente");
        std::fs::write(root.join("pom.xml"), "<project />").expect("archivo de proyecto");

        let project = Project::new(
            "App",
            &root,
            ProjectType::JavaSwing,
            BuildConfiguration::new(ProjectRelativePath::new("bin").unwrap()),
        )
        .expect("proyecto");

        (project, root)
    }

    /// Compilar sin la herramienta da un diagnostico y no un error: el IDE puede
    /// ensenarlo y seguir funcionando.
    #[test]
    fn a_build_without_its_toolchain_fails_with_a_clear_diagnostic() {
        let (project, root) = java_project("miniide-t081-sin-herramienta");

        let result = build_with(&SinHerramienta, &project).expect("el build responde");

        assert!(!result.succeeded());
        assert_eq!(result.output().exit_code(), None, "no llego a ejecutarse");
        assert_eq!(result.diagnostics().len(), 1, "{:?}", result.diagnostics());
        assert_eq!(result.diagnostics()[0].level(), DiagnosticLevel::Error);
        let message = result.diagnostics()[0].message();
        assert!(message.contains("JDK"), "{message}");
        assert!(message.contains("Instalalo"), "{message}");

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Un diagnostico sin posicion, como los que no vienen de un archivo del
    /// proyecto: la falta de herramienta no tiene archivo.
    #[test]
    fn the_diagnostic_of_a_missing_toolchain_has_no_position() {
        let (project, root) = java_project("miniide-t081-sin-posicion");

        let result = build_with(&SinHerramienta, &project).expect("el build responde");

        assert_eq!(result.diagnostics()[0].location(), None);

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Lo que se puede hacer despues de un build que no ha podido empezar: el
    /// proyecto sigue ahi, se puede volver a abrir y reintentar sin que nada
    /// cambie.
    #[test]
    fn the_ide_keeps_working_after_a_build_without_its_toolchain() {
        let (project, root) = java_project("miniide-t081-sigue-vivo");

        let first = build_with(&SinHerramienta, &project).expect("el build responde");
        let second = build_with(&SinHerramienta, &project).expect("el build responde");

        assert!(!first.succeeded());
        assert_eq!(first, second, "reintentar no cambia lo que se ve");

        let reopened = Project::open(&root.join("pom.xml")).expect("el proyecto se abre");

        assert_eq!(reopened.root(), project.root());
        assert!(root.join("src/Main.java").is_file());

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Compilar empieza a hacerlo otro, no quien lo pidio: si el trabajo saliera
    /// en el hilo de la interfaz, la ventana se congelaria hasta que acabara.
    #[test]
    fn a_build_does_not_run_in_the_thread_that_asks_for_it() {
        let provider = Arc::new(AnotaHilo::nuevo());
        let (project, root) = java_project("miniide-t082-hilo");

        let _receiver = start_build(provider.clone(), project);

        let worker = provider
            .esperar_al_hilo()
            .expect("el proveedor recibe la llamada");

        assert_ne!(
            worker,
            thread::current().id(),
            "el build se ha hecho en el hilo de quien lo pidio"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// El resultado no se pierde por llegar tarde: el canal lo guarda hasta que
    /// alguien lo pida.
    #[test]
    fn the_result_of_a_build_waits_to_be_asked_for() {
        let (project, root) = java_project("miniide-t082-espera");

        let receiver = start_build(Arc::new(AnotaHilo::nuevo()), project);

        std::thread::sleep(std::time::Duration::from_millis(200));

        let result = receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("llego el resultado");

        assert!(matches!(result, Err(CoreError::Unsupported(_))));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Una compilacion que tarda no puede tener al que la pidio esperando: eso es
    /// justo lo que hacia que la ventana se quedara congelada.
    #[test]
    #[ignore = "launches cmd.exe"]
    fn a_slow_build_does_not_freeze_whoever_asks_for_it() {
        let (project, root) = java_project("miniide-t082-lenta");
        let slow = Arc::new(CompilacionLenta { segundos: 5 });

        let beginning = std::time::Instant::now();
        let receiver = start_build(slow, project);
        let waiting = beginning.elapsed();

        assert!(
            waiting < std::time::Duration::from_secs(2),
            "pedir el build tardo {waiting:?}: la interfaz habria estado ese rato esperando"
        );

        // No se mira cuanto tardo el proceso en total: lo que importa es que pedir el
        // build no espero a que terminara, y eso ya lo dice el primer `assert`. Si
        // `start_build` bloqueara, ese rato seria el del proceso entero y fallaria.
        let result = receiver
            .recv_timeout(std::time::Duration::from_secs(30))
            .expect("llego el resultado")
            .expect("la compilacion de prueba va bien");

        assert!(result.succeeded());
        assert_eq!(result.output().exit_code(), Some(0));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Lanzar una aplicacion larga tampoco puede dejar al que la lanza esperando.
    #[test]
    #[ignore = "launches cmd.exe"]
    fn launching_a_long_application_does_not_freeze_whoever_asks_for_it() {
        let (project, root) = java_project("miniide-t082-ejecuta");
        let slow = Arc::new(CompilacionLenta { segundos: 5 });

        let beginning = std::time::Instant::now();
        let receiver = start_run(slow, project);
        let waiting = beginning.elapsed();

        assert!(
            waiting < std::time::Duration::from_secs(2),
            "pedir la ejecucion tardo {waiting:?}: la interfaz habria estado ese rato esperando"
        );

        let result = receiver
            .recv_timeout(std::time::Duration::from_secs(30))
            .expect("llego el resultado")
            .expect("la ejecucion de prueba va bien");

        assert_eq!(result.state(), ProcessState::Exited);
        assert_eq!(result.output().exit_code(), Some(0));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Un fallo que MiniIDE no previó no puede cerrar el IDE. Como el trabajo va en
    /// un hilo aparte, lo que sin mas se lleva por delante es ese hilo, y entonces
    /// quien espera se queda sin canal: un fallo interno se comeria la compilacion
    /// entera sin dejar ni un error que enseñar.
    ///
    /// Lo que tiene que pasar es que el fallo vuelva por el mismo camino que
    /// cualquier otro resultado, como un error mas, para que la interfaz pueda
    /// seguir funcionando y contarlo.
    #[test]
    fn a_build_that_fails_in_an_unexpected_way_does_not_close_the_ide() {
        let (project, root) = java_project("miniide-t084-build-revienta");

        let receiver = start_build(Arc::new(FallaPorDentro), project);

        let received = receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("el canal sigue vivo y entrega algo");

        let error = match received {
            Err(error) => error,
            Ok(_) => panic!("una toolchain que revienta no puede devolver un resultado"),
        };

        assert!(
            matches!(error, CoreError::Internal(_)),
            "un fallo inesperado se clasifica como interno, y no como otro error: {error:?}"
        );
        assert!(
            error
                .message()
                .contains("la toolchain ha fallado por dentro"),
            "el fallo tiene que decir que paso: {}",
            error.message()
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Lo mismo al ejecutar: un fallo inesperado al lanzar la aplicacion es un
    /// error, no un hilo que se cae en silencio.
    #[test]
    fn a_run_that_fails_in_an_unexpected_way_does_not_close_the_ide() {
        let (project, root) = java_project("miniide-t084-run-revienta");

        let receiver = start_run(Arc::new(FallaPorDentro), project);

        let received = receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("el canal sigue vivo y entrega algo");

        assert!(
            matches!(received, Err(CoreError::Internal(_))),
            "un fallo inesperado se clasifica como interno: {received:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Un fallo de verdad, de los que el codigo sabe que pueden pasar, no se
    /// disfraza de fallo interno: quien lo recibe tiene que poder distinguir uno de
    /// otro, porque se cuentan y se corrigen distinto.
    #[test]
    fn an_expected_failure_is_not_reported_as_an_internal_one() {
        let (project, root) = java_project("miniide-t084-build-normal");

        let receiver = start_build(Arc::new(AnotaHilo::nuevo()), project);

        let received = receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("llego el resultado");

        assert!(
            matches!(received, Err(CoreError::Unsupported(_))),
            "un error de la toolchain sigue siendo suyo: {received:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// El panel de diagnosticos se monta con lo que devolvio la compilacion, sin
    /// que nadie tenga que ir a buscarlos de otra parte.
    #[test]
    fn the_diagnostics_of_a_compilation_can_be_shown_in_a_panel() {
        use crate::diagnostics::DiagnosticsPanel;

        let result = BuildResult::new(
            ProcessOutput::new(Some(1), "", "Build FAILED."),
            vec![
                Diagnostic::new(DiagnosticLevel::Warning, "CS0219: sin usar", None),
                Diagnostic::new(DiagnosticLevel::Error, "CS1002: ; expected", None),
            ],
        );

        let panel = DiagnosticsPanel::new(result.diagnostics());

        assert_eq!(panel.groups().len(), 2);
        assert_eq!(panel.groups()[0].level(), DiagnosticLevel::Error);
        assert_eq!(panel.groups()[1].level(), DiagnosticLevel::Warning);
        assert_eq!(panel.len(), 2);
    }

    #[test]
    fn a_build_result_carries_the_output_of_the_process() {
        let output = ProcessOutput::new(Some(0), "compilando", "");
        let result = BuildResult::new(output.clone(), Vec::new());

        assert_eq!(result.output(), &output);
    }

    /// La misma estructura para los dos: el que muestra la salida de una
    /// compilacion y el que muestra la de una ejecucion leen igual, sin que ninguno
    /// tenga su propia copia de stdout, stderr y codigo de salida.
    #[test]
    fn the_build_and_the_run_deliver_the_output_in_the_same_structure() {
        use crate::runtime::{ProcessState, RunResult};

        let output = ProcessOutput::new(Some(0), "compilando", "un aviso");

        let build = BuildResult::new(output.clone(), Vec::new());
        let run = RunResult::new(ProcessState::Exited, output.clone(), None);

        assert_eq!(build.output(), run.output());
        assert_eq!(build.output(), &output);
    }

    #[test]
    fn build_result_reports_a_successful_build() {
        let result = BuildResult::new(ProcessOutput::new(Some(0), "", ""), Vec::new());

        assert!(result.succeeded());
        assert_eq!(result.output().exit_code(), Some(0));
        assert_eq!(result.output().standard_output(), "");
        assert_eq!(result.output().standard_error(), "");
        assert!(result.diagnostics().is_empty());
    }

    #[test]
    fn build_result_reports_a_failed_build() {
        let result = BuildResult::new(
            ProcessOutput::new(Some(1), "Restaurando paquetes...", "Build FAILED."),
            Vec::new(),
        );

        assert!(!result.succeeded());
        assert_eq!(result.output().exit_code(), Some(1));
        assert_eq!(result.output().standard_output(), "Restaurando paquetes...");
        assert_eq!(result.output().standard_error(), "Build FAILED.");
    }

    #[test]
    fn build_result_reports_a_build_that_never_ran() {
        let diagnostic = Diagnostic::new(
            DiagnosticLevel::Error,
            "No se encontro el SDK de .NET.",
            None,
        );
        let result = BuildResult::new(ProcessOutput::empty(), vec![diagnostic]);

        assert!(!result.succeeded());
        assert_eq!(result.output().exit_code(), None);
        assert_eq!(result.diagnostics().len(), 1);
        assert_eq!(result.diagnostics()[0].level(), DiagnosticLevel::Error);
    }

    #[test]
    fn build_result_keeps_its_diagnostics_in_order() {
        let error = Diagnostic::new(
            DiagnosticLevel::Error,
            "CS1002: ; expected",
            Some(DiagnosticLocation::new(
                ProjectRelativePath::new("src/Program.cs").unwrap(),
                TextPosition::new(11, 4),
            )),
        );
        let warning = Diagnostic::new(
            DiagnosticLevel::Warning,
            "CS0219: variable asignada sin usar",
            None,
        );

        let result = BuildResult::new(ProcessOutput::new(Some(1), "", ""), vec![error, warning]);

        assert_eq!(result.diagnostics().len(), 2);
        assert_eq!(result.diagnostics()[0].level(), DiagnosticLevel::Error);
        assert_eq!(result.diagnostics()[1].level(), DiagnosticLevel::Warning);
        assert_eq!(
            result.diagnostics()[0].location().unwrap().file().as_path(),
            Path::new("src/Program.cs")
        );
        assert_eq!(result.diagnostics()[1].location(), None);
    }

    #[test]
    fn a_process_that_finished_with_zero_is_a_successful_build() {
        let result = BuildResult::new(ProcessOutput::new(Some(0), "compilando", ""), Vec::new());

        assert!(result.succeeded());
        assert_eq!(result.output().exit_code(), Some(0));
        assert_eq!(result.output().standard_output(), "compilando");
    }

    #[test]
    fn a_process_that_failed_is_not_a_successful_build() {
        let result = BuildResult::new(ProcessOutput::new(Some(1), "", "error MSB"), Vec::new());

        assert!(!result.succeeded());
        assert_eq!(result.output().standard_error(), "error MSB");
    }

    #[test]
    fn a_process_without_exit_code_is_not_a_successful_build() {
        let result = BuildResult::new(ProcessOutput::new(None, "", ""), Vec::new());

        assert!(
            !result.succeeded(),
            "sin codigo de salida no se puede afirmar"
        );
    }

    #[test]
    fn the_diagnostics_of_the_toolchain_reach_the_build_result() {
        let parsed = Diagnostic::new(DiagnosticLevel::Error, "CS1002: falta punto y coma", None);

        let result = BuildResult::new(ProcessOutput::new(Some(1), "", ""), vec![parsed]);

        assert_eq!(result.diagnostics().len(), 1);
        assert_eq!(result.diagnostics()[0].level(), DiagnosticLevel::Error);
    }
}
