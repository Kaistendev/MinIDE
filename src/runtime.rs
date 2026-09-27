use std::fmt;
use std::process::{Command, Stdio};

use crate::core::{CoreError, CoreResult};
use crate::toolchain::Invocation;

/// Lo que dejo un proceso externo al terminar.
///
/// Es la estructura comun de la salida de un proceso: la construccion y la
/// compilacion la entregan tal cual, sin copiarse sus tres datos. Quien necesite
/// el codigo de salida o el texto lee de aqui, y anadir algo nuevo a la salida de
/// un proceso es cosa de este sitio y no de cada resultado que la muestra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    exit_code: Option<i32>,
    standard_output: String,
    standard_error: String,
}

impl ProcessOutput {
    /// La salida de un proceso que termino. Sin codigo de salida cuando el
    /// sistema no lo dio, que es distinto de que terminara con un error.
    pub fn new(
        exit_code: Option<i32>,
        standard_output: impl Into<String>,
        standard_error: impl Into<String>,
    ) -> Self {
        Self {
            exit_code,
            standard_output: standard_output.into(),
            standard_error: standard_error.into(),
        }
    }

    /// La salida de un proceso que no se ha ejecutado todavia, o cuya salida no se
    /// ha recogido.
    pub fn empty() -> Self {
        Self::new(None, "", "")
    }

    /// Codigo con el que termino, si el sistema lo dio.
    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    pub fn standard_output(&self) -> &str {
        &self.standard_output
    }

    pub fn standard_error(&self) -> &str {
        &self.standard_error
    }
}

/// Ejecuta `invocation` y espera a que termine, capturando stdout y stderr.
///
/// Es sincrono y por tanto bloquea mientras el proceso corre. La ejecucion sin
/// bloqueo es la tarea de procesos sin bloquear; mientras tanto, quien llame
/// tiene que hacerlo fuera del hilo de la interfaz.
///
/// Si el proceso no se puede lanzar, el error dice que programa era.
pub fn run(invocation: &Invocation) -> CoreResult<ProcessOutput> {
    let output = Command::new(invocation.program())
        .args(invocation.arguments())
        .current_dir(invocation.working_directory())
        .output()
        .map_err(|error| CoreError::from_io(invocation.program().as_ref(), &error))?;

    Ok(ProcessOutput::new(
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessState {
    Running,
    Exited,
    Stopped,
    Failed,
}

impl ProcessState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessState::Running => "running",
            ProcessState::Exited => "exited",
            ProcessState::Stopped => "stopped",
            ProcessState::Failed => "failed",
        }
    }
}

impl fmt::Display for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Lo que quedo de una ejecucion: como termino y la salida del proceso.
///
/// El estado es de la ejecucion (viva, terminada, parada) y la salida es la del
/// proceso, que es la misma estructura que devuelve una compilacion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunResult {
    state: ProcessState,
    output: ProcessOutput,
    error: Option<String>,
}

impl RunResult {
    pub fn new(state: ProcessState, output: ProcessOutput, error: Option<String>) -> Self {
        Self {
            state,
            output,
            error,
        }
    }

    pub fn state(&self) -> ProcessState {
        self.state
    }

    /// La salida del proceso, con su codigo de salida, su stdout y su stderr.
    pub fn output(&self) -> &ProcessOutput {
        &self.output
    }

    /// Por que no se pudo ejecutar, si no se pudo.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

/// Identificador de un proceso registrado.
///
/// No es el PID del sistema: es un numero del registro, para no depender de
/// reutilizar ids del sistema operativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(u64);

impl ProcessId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Un proceso lanzado y todavia vivo, o que ya termino.
#[derive(Debug)]
pub struct RunningProcess {
    id: ProcessId,
    child: std::process::Child,
    stopped: bool,
}

impl RunningProcess {
    fn new(id: ProcessId, child: std::process::Child) -> Self {
        Self {
            id,
            child,
            stopped: false,
        }
    }

    pub fn id(&self) -> ProcessId {
        self.id
    }

    /// Identificador del proceso en el sistema, por si hay que actuar sobre el
    /// desde fuera.
    pub fn system_id(&self) -> u32 {
        self.child.id()
    }

    /// Estado real del proceso, consultado al sistema.
    ///
    /// Preguntar al sistema cambia al proceso, asi que hace falta `&mut`. Se
    /// prefiere una verdad siempre correcta a un estado guardado que se queda
    /// viejo.
    pub fn state(&mut self) -> ProcessState {
        let waited = match self.child.try_wait() {
            Ok(None) => Waited::Running,
            Ok(Some(status)) => Waited::Finished(status.code()),
            Err(_) => Waited::Unknown,
        };

        state_from(self.stopped, waited)
    }

    /// Lo que se sabe del proceso ahora mismo.
    ///
    /// La salida se recoge cuando el proceso termina, no mientras corre: leerla
    /// en caliente exige hilo, y es la tarea de procesos sin bloquear.
    pub fn result(&mut self) -> RunResult {
        let state = self.state();
        let exit_code = match self.child.try_wait() {
            Ok(Some(status)) => status.code(),
            _ => None,
        };

        RunResult::new(state, ProcessOutput::new(exit_code, "", ""), None)
    }

    /// Para el proceso y todo lo que tenga debajo, y lo marca como parado.
    ///
    /// Devuelve si estaba vivo. Un proceso que ya habia terminado no se marca como
    /// parado: no lo ha parado MiniIDE, se paro solo. Y uno que ya estaba parado no
    /// se vuelve a parar, que parar dos veces no puede ser un error.
    fn stop(&mut self) -> CoreResult<bool> {
        if self.state() != ProcessState::Running {
            return Ok(false);
        }

        if stop_process_tree(self.child.id()) {
            self.reap_stopped();

            return Ok(true);
        }

        // Sin arbol al que llegar, al menos el proceso que se lanzo desde aqui.
        match self.child.kill() {
            Ok(()) => {
                self.reap_stopped();
                Ok(true)
            }
            Err(error) => {
                // Si ya no esta vivo, lo que fallo es que se termino justo entre la
                // pregunta y el intento. Eso no es un fallo: se acabo solo.
                if self.state() != ProcessState::Running {
                    return Ok(false);
                }

                Err(CoreError::from_io(
                    self.id.value().to_string().as_ref(),
                    &error,
                ))
            }
        }
    }

    /// Espera a que el proceso este de verdad terminado, y lo marca como parado.
    fn reap_stopped(&mut self) {
        // Sin recogerlo, el sistema se queda con el proceso a medias y preguntarle
        // despues seguiria dando la sensacion de que esta vivo.
        let _ = self.child.wait();

        self.stopped = true;
    }
}

/// Para el proceso `system_id` y todo lo que tenga debajo.
///
/// En Windows el proceso que lanza MiniIDE no siempre es el que hay que parar: un
/// lanzador como `cmd /C` o el `javapath` de Oracle dejan debajo el proceso de
/// verdad, y parar solo el lanzador dejaria la compilacion o la aplicacion
/// corriendo mientras el IDE da por parado.
///
/// Se usa `taskkill` porque la biblioteca estandar solo sabe terminar el proceso
/// que lanzo, no los que lanzo ese, y en Windows no hay otra forma de llegar a
/// ellos sin anadir una dependencia.
///
/// Cuesta unos milisegundos y su salida se tira: parar un proceso no es algo que
/// deba llenar el panel de salida del IDE.
///
/// Devuelve si el arbol ha dejado de estar. Que no se pueda no es un error: quien
/// llama tiene el proceso a mano y puede intentar pararlo directamente.
fn stop_process_tree(system_id: u32) -> bool {
    let stopped = Command::new("taskkill")
        .args(["/PID", &system_id.to_string(), "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match stopped {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

/// Lo que contesta el sistema al preguntar por un proceso.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Waited {
    /// Sigue vivo.
    Running,
    /// Ha terminado, con su codigo de salida si el sistema lo tiene.
    Finished(Option<i32>),
    /// No se ha podido preguntar.
    Unknown,
}

/// El estado de un proceso a partir de lo que se sabe de el.
///
/// Lo que se ha parado desde MiniIDE sale como `Stopped` y no como `Failed`:
/// parar no es un fallo.
fn state_from(stopped: bool, waited: Waited) -> ProcessState {
    if stopped {
        return ProcessState::Stopped;
    }

    match waited {
        Waited::Running | Waited::Unknown => ProcessState::Running,
        Waited::Finished(Some(0)) => ProcessState::Exited,
        Waited::Finished(_) => ProcessState::Failed,
    }
}

/// Los procesos que MiniIDE ha lanzado y sigue teniendo localizados.
///
/// Existe para que un proceso se pueda volver a encontrar: sin el, un proceso
/// lanzado se pierde en el sistema y no se puede consultar ni detener.
#[derive(Debug, Default)]
pub struct ProcessRegistry {
    next_id: u64,
    processes: Vec<RunningProcess>,
}

impl ProcessRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Lanza `invocation` sin esperar a que termine y registra el proceso.
    ///
    /// No bloquea: devuelve en cuanto el proceso esta en marcha. Si no se puede
    /// lanzar, no se registra nada.
    pub fn spawn(&mut self, invocation: &Invocation) -> CoreResult<ProcessId> {
        let child = Command::new(invocation.program())
            .args(invocation.arguments())
            .current_dir(invocation.working_directory())
            .spawn()
            .map_err(|error| CoreError::from_io(invocation.program().as_ref(), &error))?;

        self.next_id += 1;
        let id = ProcessId::new(self.next_id);

        self.processes.push(RunningProcess::new(id, child));

        Ok(id)
    }

    pub fn get(&self, id: ProcessId) -> Option<&RunningProcess> {
        self.processes.iter().find(|process| process.id() == id)
    }

    /// Para preguntar el estado, que necesita comprobar el proceso real.
    pub fn get_mut(&mut self, id: ProcessId) -> Option<&mut RunningProcess> {
        self.processes.iter_mut().find(|process| process.id() == id)
    }

    /// Identificadores de los procesos registrados, en orden de lanzamiento.
    pub fn ids(&self) -> Vec<ProcessId> {
        self.processes.iter().map(|process| process.id()).collect()
    }

    /// Detiene el proceso `id` sin cerrar MiniIDE.
    ///
    /// Se para el proceso y todo lo que haya lanzado, no solo el proceso que se
    /// registro: el que de verdad esta corriendo suele estar por debajo de un
    /// lanzador, y dejar ese vivo haria que el IDE crease que la ha parado sin que
    /// sea verdad.
    ///
    /// El proceso se queda registrado, marcado como `Stopped`. Si ya habia
    /// terminado, no se hace nada y no se cambia su estado: no lo ha parado
    /// MiniIDE. Si el identificador no existe, es un `NotFound`.
    pub fn stop(&mut self, id: ProcessId) -> CoreResult<()> {
        let process = self
            .get_mut(id)
            .ok_or_else(|| CoreError::NotFound(format!("no hay proceso {id}")))?;

        process.stop()?;

        Ok(())
    }

    /// Si hay algun proceso vivo todavia.
    ///
    /// Es lo que le dice al IDE si puede volver a estar inactivo: mientras algo siga
    /// corriendo hay trabajo en marcha, y en cuanto se para no queda nada activo.
    /// Preguntar por un proceso es preguntarselo al sistema, y preguntar cambia al
    /// proceso, asi que hace falta `&mut`.
    pub fn is_active(&mut self) -> bool {
        self.processes
            .iter_mut()
            .any(|process| process.state() == ProcessState::Running)
    }

    /// Se olvida del proceso `id` y devuelve si estaba registrado.
    ///
    /// El registro guarda lo que se puede consultar o parar, no un historial: un
    /// proceso del que ya no se va a volver a saber nada no tiene por que seguir
    /// ocupando sitio, y compilar y ejecutar muchas veces no puede hacer que el
    /// registro crezca sin fin.
    pub fn forget(&mut self, id: ProcessId) -> bool {
        let before = self.processes.len();

        self.processes.retain(|process| process.id() != id);

        self.processes.len() != before
    }

    pub fn len(&self) -> usize {
        self.processes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toolchain::Invocation;
    use std::path::{Path, PathBuf};

    fn invocation(program: &str, arguments: &[&str]) -> Invocation {
        Invocation::new(
            program,
            arguments
                .iter()
                .map(|argument| argument.to_string())
                .collect(),
            std::env::temp_dir(),
        )
    }

    /// Una llamada que termina enseguida, para probar el registro sin dejar nada.
    fn quick_invocation() -> Invocation {
        invocation("cmd", &["/C", "exit 0"])
    }

    /// La salida de un proceso se lee por metodos y no por campos sueltos: es la
    /// estructura que build y run entregan, y anadirle un dato tiene que ser cosa
    /// del proceso y no de cada sitio que lo muestra.
    #[test]
    fn the_output_of_a_process_is_read_through_its_accessors() {
        let process_output = ProcessOutput::new(Some(3), "compilando", "un aviso");

        assert_eq!(process_output.exit_code(), Some(3));
        assert_eq!(process_output.standard_output(), "compilando");
        assert_eq!(process_output.standard_error(), "un aviso");
    }

    #[test]
    fn a_process_that_said_nothing_has_empty_output() {
        let process_output = ProcessOutput::empty();

        assert_eq!(process_output.exit_code(), None);
        assert_eq!(process_output.standard_output(), "");
        assert_eq!(process_output.standard_error(), "");
    }

    #[test]
    fn a_run_result_carries_the_output_of_the_process() {
        let process_output = ProcessOutput::new(Some(0), "Hola", "");
        let result = RunResult::new(ProcessState::Exited, process_output.clone(), None);

        assert_eq!(result.output(), &process_output);
    }

    /// Espera a que un proceso termine, con un tope de tiempo.
    fn wait_for_exit(registry: &mut ProcessRegistry, id: ProcessId) -> ProcessState {
        for _ in 0..100 {
            let state = registry.get_mut(id).expect("proceso registrado").state();

            if state != ProcessState::Running {
                return state;
            }

            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        ProcessState::Running
    }

    #[test]
    fn a_spawned_process_is_registered() {
        let mut registry = ProcessRegistry::new();

        let id = registry.spawn(&quick_invocation()).unwrap();

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.get(id).is_some());
    }

    #[test]
    fn each_process_gets_its_own_id() {
        let mut registry = ProcessRegistry::new();

        let first = registry.spawn(&quick_invocation()).unwrap();
        let second = registry.spawn(&quick_invocation()).unwrap();

        assert_ne!(first, second);
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn a_process_that_was_never_spawned_is_not_found() {
        let registry = ProcessRegistry::new();

        assert!(registry.get(ProcessId::new(999)).is_none());
    }

    #[test]
    fn a_new_registry_has_no_processes() {
        let registry = ProcessRegistry::new();

        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.ids().is_empty());
    }

    #[test]
    fn a_spawned_process_starts_running_without_an_exit_code() {
        let mut registry = ProcessRegistry::new();
        let id = registry.spawn(&quick_invocation()).unwrap();

        let result = registry.get_mut(id).unwrap().result();

        assert_eq!(result.state(), ProcessState::Running);
        assert_eq!(result.output().exit_code(), None);
    }

    #[test]
    fn a_spawned_process_is_identified_by_its_id() {
        let mut registry = ProcessRegistry::new();
        let id = registry.spawn(&quick_invocation()).unwrap();

        assert_eq!(registry.get(id).unwrap().id(), id);
    }

    #[test]
    fn a_process_that_cannot_be_launched_is_not_registered() {
        let mut registry = ProcessRegistry::new();

        let result = registry.spawn(&invocation("miniide-no-existe-este-programa", &[]));

        assert!(matches!(result, Err(CoreError::NotFound(_))));
        assert!(registry.is_empty(), "un fallo no deja proceso registrado");
    }

    #[test]
    #[ignore = "launches cmd.exe"]
    fn a_process_that_fails_is_reported_as_failed_with_its_code() {
        let mut registry = ProcessRegistry::new();
        let id = registry
            .spawn(&invocation("cmd", &["/C", "exit 3"]))
            .unwrap();

        let state = wait_for_exit(&mut registry, id);

        assert_eq!(state, ProcessState::Failed);
        assert_eq!(
            registry.get_mut(id).unwrap().result().output().exit_code(),
            Some(3)
        );
    }

    #[test]
    #[ignore = "launches cmd.exe"]
    fn the_registry_forgets_nothing_that_is_still_reachable() {
        let mut registry = ProcessRegistry::new();
        let first = registry.spawn(&quick_invocation()).unwrap();
        let second = registry.spawn(&quick_invocation()).unwrap();

        wait_for_exit(&mut registry, first);

        assert_eq!(registry.ids().len(), 2);
        assert!(registry.get(first).is_some());
        assert!(registry.get(second).is_some());
    }

    #[test]
    fn a_process_stopped_by_miniide_is_reported_as_stopped() {
        assert_eq!(state_from(true, Waited::Running), ProcessState::Stopped);
        assert_eq!(
            state_from(true, Waited::Finished(Some(1))),
            ProcessState::Stopped
        );
        assert_eq!(state_from(true, Waited::Unknown), ProcessState::Stopped);
    }

    #[test]
    fn a_process_that_finished_with_zero_is_reported_as_exited() {
        assert_eq!(
            state_from(false, Waited::Finished(Some(0))),
            ProcessState::Exited
        );
    }

    #[test]
    fn a_process_that_finished_with_an_error_is_reported_as_failed() {
        assert_eq!(
            state_from(false, Waited::Finished(Some(1))),
            ProcessState::Failed
        );
        assert_eq!(
            state_from(false, Waited::Finished(Some(-1))),
            ProcessState::Failed
        );
    }

    #[test]
    fn a_process_that_was_terminated_without_a_code_is_reported_as_failed() {
        assert_eq!(
            state_from(false, Waited::Finished(None)),
            ProcessState::Failed
        );
    }

    #[test]
    fn a_process_with_no_exit_code_yet_is_running() {
        assert_eq!(state_from(false, Waited::Running), ProcessState::Running);
    }

    #[test]
    fn not_being_able_to_ask_does_not_mean_the_process_is_dead() {
        assert_eq!(state_from(false, Waited::Unknown), ProcessState::Running);
    }

    #[test]
    fn stopping_a_process_that_was_never_spawned_is_reported() {
        let mut registry = ProcessRegistry::new();

        let result = registry.stop(ProcessId::new(999));

        assert!(matches!(result, Err(CoreError::NotFound(_))));
    }

    #[test]
    #[ignore = "launches cmd.exe"]
    fn a_registered_process_can_be_stopped_and_is_then_reported_as_stopped() {
        let mut registry = ProcessRegistry::new();
        let id = registry
            .spawn(&invocation("cmd", &["/C", "ping -n 60 127.0.0.1 >NUL"]))
            .unwrap();

        registry.stop(id).unwrap();

        assert_eq!(registry.get_mut(id).unwrap().state(), ProcessState::Stopped);
        assert_eq!(registry.len(), 1, "el proceso sigue registrado");
    }

    #[test]
    #[ignore = "launches cmd.exe"]
    fn stopping_a_process_twice_does_not_fail() {
        let mut registry = ProcessRegistry::new();
        let id = registry
            .spawn(&invocation("cmd", &["/C", "ping -n 60 127.0.0.1 >NUL"]))
            .unwrap();
        registry.stop(id).unwrap();

        let result = registry.stop(id);

        assert!(result.is_ok(), "detener dos veces no puede ser un error");
        assert_eq!(registry.get_mut(id).unwrap().state(), ProcessState::Stopped);
    }

    #[test]
    #[ignore = "launches cmd.exe"]
    fn stopping_a_process_that_already_finished_leaves_it_as_it_was() {
        let mut registry = ProcessRegistry::new();
        let id = registry.spawn(&quick_invocation()).unwrap();
        wait_for_exit(&mut registry, id);

        registry.stop(id).unwrap();

        assert_eq!(
            registry.get_mut(id).unwrap().state(),
            ProcessState::Exited,
            "un proceso que ya habia terminado no se ha parado"
        );
    }

    /// El IDE tiene que poder volver a estar inactivo: mientras algo corre hay algo
    /// activo, y en cuanto se para no queda nada en marcha. Es lo que permite a la
    /// interfaz volver a estar quieta.
    #[test]
    #[ignore = "launches cmd.exe"]
    fn stopping_a_process_leaves_the_ide_with_nothing_active() {
        let mut registry = ProcessRegistry::new();
        let id = registry
            .spawn(&invocation("cmd", &["/C", "ping -n 60 127.0.0.1 >NUL"]))
            .unwrap();

        assert!(registry.is_active(), "un proceso vivo es trabajo en marcha");

        registry.stop(id).unwrap();

        assert!(
            !registry.is_active(),
            "tras parar no puede quedar nada en marcha"
        );
        assert_eq!(registry.get_mut(id).unwrap().state(), ProcessState::Stopped);
    }

    /// Un proceso que se acaba solo tambien deja al IDE inactivo, sin que nadie lo
    /// haya parado: no por eso sigue habiendo trabajo en marcha.
    #[test]
    #[ignore = "launches cmd.exe"]
    fn a_process_that_ends_on_its_own_leaves_the_ide_with_nothing_active() {
        let mut registry = ProcessRegistry::new();
        let id = registry.spawn(&quick_invocation()).unwrap();
        wait_for_exit(&mut registry, id);

        assert!(
            !registry.is_active(),
            "un proceso terminado no esta en marcha"
        );
    }

    /// Parar dos veces no puede ser un error, y ademas no puede dejar al IDE con
    /// algo en marcha que ya no esta.
    #[test]
    #[ignore = "launches cmd.exe"]
    fn stopping_a_process_twice_leaves_the_ide_with_nothing_active() {
        let mut registry = ProcessRegistry::new();
        let id = registry
            .spawn(&invocation("cmd", &["/C", "ping -n 60 127.0.0.1 >NUL"]))
            .unwrap();

        registry.stop(id).unwrap();
        registry.stop(id).unwrap();

        assert!(!registry.is_active());
        assert_eq!(registry.get_mut(id).unwrap().state(), ProcessState::Stopped);
    }

    /// El proceso que hay que parar no siempre es el que lanzo MiniIDE: `cmd /C` y
    /// el `javapath` de Oracle dejan debajo el proceso de verdad. Si solo se para el
    /// lanzador, la aplicacion sigue corriendo y el IDE cree que la ha parado: el
    /// estado vuelve a inactivo y la ventana de verdad sigue abierta.
    #[test]
    #[ignore = "launches cmd.exe"]
    fn stopping_a_process_also_stops_what_it_launched() {
        let file = std::env::temp_dir().join("miniide-t083-vivo.txt");
        let hijo = writing_script("miniide-t083-hijo.bat", &file);
        let lanzador = launching_script("miniide-t083-lanzador.bat", &hijo);
        let _ = std::fs::remove_file(&file);

        let mut registry = ProcessRegistry::new();
        let id = registry
            .spawn(&invocation("cmd", &["/C", lanzador.to_str().unwrap()]))
            .unwrap();

        assert!(
            wait_until_the_file_has_lines(&file, 2),
            "el proceso de debajo deberia estar escribiendo"
        );

        registry.stop(id).unwrap();

        // El proceso de debajo escribe cada dos segundos, asi que tres segundos sin
        // cambios y otros tres mas igual quiere decir que ya no escribe. Si solo se
        // paro el lanzador, el archivo seguiria creciendo.
        std::thread::sleep(std::time::Duration::from_secs(3));
        let al_parar = file_size(&file);
        std::thread::sleep(std::time::Duration::from_secs(3));

        assert_eq!(
            file_size(&file),
            al_parar,
            "el proceso de debajo sigue vivo: solo se ha parado el lanzador"
        );

        let _ = std::fs::remove_file(&file);
        let _ = std::fs::remove_file(&hijo);
        let _ = std::fs::remove_file(&lanzador);
    }

    /// Un proceso que ya no esta en marcha se puede olvidar: el registro guarda lo
    /// que el IDE puede consultar o parar, y lo que ya termino no sirve para nada.
    #[test]
    fn a_process_can_be_forgotten_once_it_is_no_longer_needed() {
        let mut registry = ProcessRegistry::new();
        let id = registry.spawn(&quick_invocation()).unwrap();

        let forgotten = registry.forget(id);

        assert!(forgotten, "un proceso registrado se puede olvidar");
        assert!(registry.get(id).is_none());
        assert!(registry.is_empty());
    }

    #[test]
    fn a_process_that_was_never_spawned_cannot_be_forgotten() {
        let mut registry = ProcessRegistry::new();

        assert!(
            !registry.forget(ProcessId::new(999)),
            "olvidar algo que no esta no puede decir que si"
        );
        assert!(registry.is_empty());
    }

    /// Escribe un script que deja constancia de que sigue vivo: anade una linea a
    /// `file` cada dos segundos mientras dure.
    ///
    /// Es la forma de saber si un proceso continua vivo sin mirar al sistema: si el
    /// archivo deja de crecer, el proceso ha terminado. Se usa en vez de contar los
    /// procesos del sistema porque esos pueden ser de otros, y un test no puede dar
    /// por suyo lo que ve de otros.
    ///
    /// Se corta sola a los 30 intentos, para que un test que falle no deje nada
    /// dando vueltas toda la tarde.
    fn writing_script(name: &str, file: &Path) -> PathBuf {
        let script = std::env::temp_dir().join(name);
        // `%%i` y no `%i` porque esto es un `.bat`: en un script el `%` se escribe
        // duplicado, y con un solo `%` cmd empareja los signos de porcentaje y deja
        // la linea sin sentido.
        let contents = format!(
            "@for /L %%i in (1,1,30) do @(echo x>>\"{}\" & ping -n 2 127.0.0.1 >NUL)\r\n",
            file.display()
        );

        std::fs::write(&script, contents).expect("se puede escribir el script");

        script
    }

    /// Escribe un script que lanza `hijo` en un proceso aparte y se queda esperando.
    ///
    /// Lo de lanzar en un proceso aparte es justo lo que se quiere comprobar: el
    /// proceso registrado es el lanzador, y el que de verdad hay que parar es el que
    /// este deja debajo.
    fn launching_script(name: &str, hijo: &Path) -> PathBuf {
        let script = std::env::temp_dir().join(name);
        // Las comillas van duplicadas porque es la forma de que `cmd /C` entienda
        // una ruta con espacios sin comerse el principio del camino.
        let contents = format!("@cmd /C \"\"{}\"\"\r\n", hijo.display());

        std::fs::write(&script, contents).expect("se puede escribir el script");

        script
    }

    /// Cuanto ocupa `file`, o cero si todavia no existe.
    fn file_size(file: &Path) -> u64 {
        std::fs::metadata(file).map(|file| file.len()).unwrap_or(0)
    }

    /// Espera a que `file` tenga al menos `lineas` lineas, con un tope de tiempo.
    fn wait_until_the_file_has_lines(file: &Path, lineas: usize) -> bool {
        for _ in 0..100 {
            if file_size(file) >= lineas as u64 * 2 {
                return true;
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        false
    }

    #[test]
    fn a_command_that_cannot_be_launched_reports_the_program() {
        let result = run(&invocation("miniide-no-existe-este-programa", &[]));

        assert!(matches!(result, Err(CoreError::NotFound(_))));
    }

    #[test]
    #[ignore = "launches cmd.exe"]
    fn a_finished_command_keeps_its_output_and_its_exit_code() {
        let output = run(&invocation(
            "cmd",
            &["/C", "echo salida && echo error 1>&2"],
        ))
        .unwrap();

        assert_eq!(output.exit_code, Some(0));
        assert!(output.standard_output.contains("salida"), "{output:?}");
        assert!(output.standard_error.contains("error"), "{output:?}");
    }

    #[test]
    #[ignore = "launches cmd.exe"]
    fn a_command_that_fails_keeps_its_exit_code() {
        let output = run(&invocation("cmd", &["/C", "exit 3"])).unwrap();

        assert_eq!(output.exit_code, Some(3));
    }

    #[test]
    #[ignore = "launches cmd.exe"]
    fn a_command_runs_in_its_working_directory() {
        let temp = std::env::temp_dir();
        let call = Invocation::new("cmd", vec!["/C".to_string(), "cd".to_string()], &temp);

        let output = run(&call).unwrap();

        let current = output.standard_output.to_lowercase();

        assert!(current.contains("temp"), "{output:?}");
    }

    #[test]
    fn a_working_directory_that_does_not_exist_is_reported() {
        let call = Invocation::new(
            "cmd",
            vec!["/C".to_string(), "echo".to_string()],
            PathBuf::from("C:\\miniide-no-existe-este-directorio"),
        );

        let result = run(&call);

        assert!(
            result.is_err(),
            "un directorio de trabajo que no existe no puede lanzar el proceso"
        );
    }

    #[test]
    fn a_running_process_has_no_exit_code_yet() {
        let result = RunResult::new(ProcessState::Running, ProcessOutput::empty(), None);

        assert_eq!(result.state(), ProcessState::Running);
        assert_eq!(result.output().exit_code(), None);
        assert_eq!(result.output().standard_output(), "");
        assert_eq!(result.output().standard_error(), "");
        assert_eq!(result.error(), None);
    }

    #[test]
    fn a_finished_process_keeps_its_output_and_exit_code() {
        let result = RunResult::new(
            ProcessState::Exited,
            ProcessOutput::new(Some(0), "Hola desde la aplicacion", ""),
            None,
        );

        assert_eq!(result.state(), ProcessState::Exited);
        assert_eq!(result.output().exit_code(), Some(0));
        assert_eq!(
            result.output().standard_output(),
            "Hola desde la aplicacion"
        );
    }

    #[test]
    fn a_stopped_process_is_not_reported_as_failed() {
        let stopped = RunResult::new(ProcessState::Stopped, ProcessOutput::empty(), None);
        let failed = RunResult::new(
            ProcessState::Failed,
            ProcessOutput::empty(),
            Some("No se encontro el ejecutable.".to_string()),
        );

        assert_ne!(stopped.state(), ProcessState::Failed);
        assert_eq!(stopped.error(), None);
        assert_eq!(failed.error(), Some("No se encontro el ejecutable."));
    }

    #[test]
    fn a_process_that_could_not_start_keeps_the_reason() {
        let result = RunResult::new(
            ProcessState::Failed,
            ProcessOutput::empty(),
            Some("No se encontro el ejecutable.".to_string()),
        );

        assert_eq!(result.state(), ProcessState::Failed);
        assert_eq!(result.output().exit_code(), None);
        assert_eq!(result.output().standard_error(), "");
        assert_eq!(result.error(), Some("No se encontro el ejecutable."));
    }

    #[test]
    fn process_state_exposes_a_stable_label() {
        assert_eq!(ProcessState::Running.as_str(), "running");
        assert_eq!(ProcessState::Exited.as_str(), "exited");
        assert_eq!(ProcessState::Stopped.as_str(), "stopped");
        assert_eq!(ProcessState::Failed.as_str(), "failed");
        assert_eq!(ProcessState::Stopped.to_string(), "stopped");
    }
}
