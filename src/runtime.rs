use std::fmt;
use std::process::Command;

use crate::core::{CoreError, CoreResult};
use crate::toolchain::Invocation;

/// Lo que dejo un proceso externo al terminar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    pub exit_code: Option<i32>,
    pub standard_output: String,
    pub standard_error: String,
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

    Ok(ProcessOutput {
        exit_code: output.status.code(),
        standard_output: String::from_utf8_lossy(&output.stdout).into_owned(),
        standard_error: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunResult {
    state: ProcessState,
    exit_code: Option<i32>,
    standard_output: String,
    standard_error: String,
    error: Option<String>,
}

impl RunResult {
    pub fn new(
        state: ProcessState,
        exit_code: Option<i32>,
        standard_output: impl Into<String>,
        standard_error: impl Into<String>,
        error: Option<String>,
    ) -> Self {
        Self {
            state,
            exit_code,
            standard_output: standard_output.into(),
            standard_error: standard_error.into(),
            error,
        }
    }

    pub fn state(&self) -> ProcessState {
        self.state
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    pub fn standard_output(&self) -> &str {
        &self.standard_output
    }

    pub fn standard_error(&self) -> &str {
        &self.standard_error
    }

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

        RunResult::new(state, exit_code, String::new(), String::new(), None)
    }

    /// Mata el proceso y lo marca como parado.
    ///
    /// Devuelve si estaba vivo. Un proceso que ya habia terminado no se marca
    /// como parado: no lo ha parado MiniIDE, se paro solo.
    fn kill(&mut self) -> CoreResult<bool> {
        if self.state() != ProcessState::Running {
            return Ok(false);
        }

        match self.child.kill() {
            Ok(()) => {
                self.stopped = true;
                Ok(true)
            }
            // El proceso se termino entre la pregunta y el intento de matarlo.
            Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => Ok(false),
            Err(error) => Err(CoreError::from_io(
                self.id.value().to_string().as_ref(),
                &error,
            )),
        }
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

/// Los procesos que MiniIDE ha lanzado y sigue teniendolocalizados.
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
    /// El proceso se queda registrado, marked como `Stopped`. Si ya habia
    /// terminado, no se hace nada y no se cambia su estado: no lo ha parado
    /// MiniIDE. Si el identificador no existe, es un `NotFound`.
    pub fn stop(&mut self, id: ProcessId) -> CoreResult<()> {
        let process = self
            .get_mut(id)
            .ok_or_else(|| CoreError::NotFound(format!("no hay proceso {id}")))?;

        process.kill()?;

        Ok(())
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
    use std::path::PathBuf;

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
        assert_eq!(result.exit_code(), None);
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
        assert_eq!(registry.get_mut(id).unwrap().result().exit_code(), Some(3));
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
    fn process_output_can_be_read() {
        let output = ProcessOutput {
            exit_code: Some(0),
            standard_output: "salida".to_string(),
            standard_error: String::new(),
        };

        assert_eq!(output.exit_code, Some(0));
        assert_eq!(output.standard_output, "salida");
    }

    #[test]
    fn a_running_process_has_no_exit_code_yet() {
        let result = RunResult::new(
            ProcessState::Running,
            None,
            String::new(),
            String::new(),
            None,
        );

        assert_eq!(result.state(), ProcessState::Running);
        assert_eq!(result.exit_code(), None);
        assert_eq!(result.standard_output(), "");
        assert_eq!(result.standard_error(), "");
        assert_eq!(result.error(), None);
    }

    #[test]
    fn a_finished_process_keeps_its_output_and_exit_code() {
        let result = RunResult::new(
            ProcessState::Exited,
            Some(0),
            "Hola desde la aplicacion",
            String::new(),
            None,
        );

        assert_eq!(result.state(), ProcessState::Exited);
        assert_eq!(result.exit_code(), Some(0));
        assert_eq!(result.standard_output(), "Hola desde la aplicacion");
    }

    #[test]
    fn a_stopped_process_is_not_reported_as_failed() {
        let stopped = RunResult::new(
            ProcessState::Stopped,
            None,
            String::new(),
            String::new(),
            None,
        );
        let failed = RunResult::new(
            ProcessState::Failed,
            None,
            String::new(),
            String::new(),
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
            None,
            String::new(),
            String::new(),
            Some("No se encontro el ejecutable.".to_string()),
        );

        assert_eq!(result.state(), ProcessState::Failed);
        assert_eq!(result.exit_code(), None);
        assert_eq!(result.standard_error(), "");
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
