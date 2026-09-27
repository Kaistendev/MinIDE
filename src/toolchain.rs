use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::core::{CoreResult, ProjectType};
use crate::diagnostics::Diagnostic;
use crate::project::Project;
use crate::runtime::ProcessOutput;

mod dotnet;
mod javac;
mod jdk;
mod msbuild;

pub use dotnet::{DotNetSdk, DotNetToolchain};
pub use jdk::{Jdk, JdkStatus, JdkToolchain};

/// Directorios de `path`, sin entradas vacias.
///
/// En Windows una entrada vacia en el PATH significa el directorio actual, y
/// buscar ahi seria buscar en el sitio desde el que se lanza MiniIDE. No se
/// sigue.
pub fn usable_directories(path: &OsStr) -> Vec<PathBuf> {
    std::env::split_paths(path)
        .filter(|directory| !directory.as_os_str().is_empty())
        .collect()
}

/// Nombres que se prueban para `name`: el tal cual y con cada extension.
pub fn candidate_names(name: &str, extensions: &[&str]) -> Vec<String> {
    let mut candidates = vec![name.to_string()];

    candidates.extend(
        extensions
            .iter()
            .map(|extension| format!("{name}{extension}")),
    );

    candidates
}

/// Primer ejecutable que existe en `directories`, probando cada nombre en cada
/// directorio en orden.
///
/// No seusa `PATHEXT`: ahi estan extensiones como `.PY` o `.JS`, y un
/// `dotnet.py` no es el SDK de .NET.
pub fn detect(name: &str, directories: &[PathBuf], extensions: &[&str]) -> Option<PathBuf> {
    let candidates = candidate_names(name, extensions);

    for directory in directories {
        for candidate in &candidates {
            let path = directory.join(candidate);

            if path.is_file() {
                return Some(path);
            }
        }
    }

    None
}

/// Una llamada a una herramienta externa, preparada y todavia no ejecutada.
///
/// Es lo que entrega el proveedor de toolchain. Quien la ejecuta es la capa de
/// procesos, que ya sabe lanzar sin bloquear, capturar stdout y stderr y
/// detener el proceso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    program: String,
    arguments: Vec<String>,
    working_directory: PathBuf,
}

impl Invocation {
    /// `program` es el ejecutable, `arguments` sus argumentos y
    /// `working_directory` el directorio desde el que lanzarlo, que para las
    /// herramientas de compilacion es el directorio del proyecto y no el actual.
    pub fn new(
        program: impl Into<String>,
        arguments: Vec<String>,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            program: program.into(),
            arguments,
            working_directory: working_directory.into(),
        }
    }

    /// Ejecutable que se va a lanzar.
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Argumentos, en el orden en que se pasan.
    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    /// Directorio desde el que se lanza.
    pub fn working_directory(&self) -> &Path {
        &self.working_directory
    }
}

/// Lo que el core puede pedir a una toolchain sin conocer comandos concretos.
///
/// El proveedor prepara la llamada; no la ejecuta. Asi el core no depende de
/// como se compila cada plataforma, y el lancement, la captura de salida y la
/// parada del proceso quedan en un solo sitio.
///
/// Exige `Send` y `Sync` porque una toolchain acaba usandose desde el hilo que
/// compila y desde el que lanza el proceso, que no son el mismo. Sin eso no
/// podria ir en un `Arc` ni pasar a un hilo de trabajo, y el trabajo largo
/// quedaria atado al hilo de la interfaz.
pub trait ToolchainProvider: Send + Sync {
    /// Proyecto que compila y ejecuta este proveedor. Es lo que permite elegir
    /// proveedor a partir del proyecto.
    fn project_type(&self) -> ProjectType;

    /// Herramienta externa que necesita, tal y como se le llama al usuario.
    fn tool(&self) -> &'static str;

    /// Si la herramienta esta disponible en el sistema.
    fn is_available(&self) -> bool;

    /// Lo que se le dice al usuario cuando la herramienta no esta.
    ///
    /// Por defecto basta con el nombre de la herramienta, que ya dice lo que
    /// falta. Una toolchain que sepa mas de como se busca —que es el caso de la
    /// del JDK, que necesita el compilador en el PATH— lo dice aqui en vez de
    /// dejar que el core lo adivine.
    fn missing_message(&self) -> String {
        format!(
            "no se ha encontrado {} en este equipo. Instalalo y vuelve a compilar.",
            self.tool()
        )
    }

    /// Prepara la llamada que compila `project`, con la configuracion de
    /// compilacion que tenga el proyecto.
    ///
    /// Es para un proyecto de `project_type`; de eso se encarga el core al
    /// elegir proveedor, asi que no se comprueba aqui.
    fn build_invocation(&self, project: &Project) -> CoreResult<Invocation>;

    /// Prepara la llamada que ejecuta `project`.
    ///
    /// Es para un proyecto de `project_type`, con la misma salvedad que
    /// `build_invocation`.
    fn run_invocation(&self, project: &Project) -> CoreResult<Invocation>;

    /// Traduce la salida de la herramienta en diagnosticos del core.
    ///
    /// Cada toolchain sabe leer su propia salida, y por eso esto no es un
    /// `match` con el lenguaje en el core. Los archivos se relativity a la raiz
    /// del proyecto; los que no estan dentro salen sin posicion en vez de
    /// inventarse una ruta.
    fn parse_diagnostics(&self, output: &ProcessOutput, root: &Path) -> Vec<Diagnostic>;
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::core::{CoreError, CoreResult, ProjectType};
    use crate::project::{BuildConfiguration, Project, ProjectRelativePath};
    use crate::toolchain::{Invocation, ToolchainProvider};

    struct FakeToolchain {
        project_type: ProjectType,
        tool: &'static str,
        available: bool,
    }

    impl FakeToolchain {
        fn dotnet() -> Self {
            Self {
                project_type: ProjectType::CSharpWinForms,
                tool: ".NET SDK",
                available: true,
            }
        }

        fn jdk() -> Self {
            Self {
                project_type: ProjectType::JavaSwing,
                tool: "JDK",
                available: false,
            }
        }
    }

    impl ToolchainProvider for FakeToolchain {
        fn project_type(&self) -> ProjectType {
            self.project_type
        }

        fn tool(&self) -> &'static str {
            self.tool
        }

        fn is_available(&self) -> bool {
            self.available
        }

        fn build_invocation(&self, project: &Project) -> CoreResult<Invocation> {
            Ok(Invocation::new(
                "dotnet",
                vec!["build".to_string(), project.name().to_string()],
                project.root().to_path_buf(),
            ))
        }

        fn run_invocation(&self, project: &Project) -> CoreResult<Invocation> {
            Ok(Invocation::new(
                "java",
                vec!["-jar".to_string(), "app.jar".to_string()],
                project.root().to_path_buf(),
            ))
        }

        fn parse_diagnostics(&self, _output: &ProcessOutput, _root: &Path) -> Vec<Diagnostic> {
            Vec::new()
        }
    }

    fn project() -> Project {
        Project::new(
            "app",
            "C:\\proyectos\\app",
            ProjectType::CSharpWinForms,
            BuildConfiguration::new(ProjectRelativePath::new("bin").unwrap()),
        )
        .unwrap()
    }

    #[test]
    fn the_provider_declares_the_project_it_handles() {
        assert_eq!(
            FakeToolchain::dotnet().project_type(),
            ProjectType::CSharpWinForms
        );
        assert_eq!(FakeToolchain::jdk().project_type(), ProjectType::JavaSwing);
    }

    #[test]
    fn the_provider_declares_the_tool_it_needs() {
        assert_eq!(FakeToolchain::dotnet().tool(), ".NET SDK");
        assert_eq!(FakeToolchain::jdk().tool(), "JDK");
    }

    #[test]
    fn the_availability_of_the_tool_comes_from_the_provider() {
        assert!(FakeToolchain::dotnet().is_available());
        assert!(!FakeToolchain::jdk().is_available());
    }

    /// El mensaje por defecto tiene que decir que falta y que hacer: con solo el
    /// nombre de la herramienta no hay forma de arreglarlo.
    #[test]
    fn a_toolchain_that_is_missing_says_what_to_do() {
        let dotnet = FakeToolchain::dotnet();

        let message = dotnet.missing_message();

        assert!(message.contains(".NET SDK"), "{message}");
        assert!(message.contains("Instalalo"), "{message}");
    }

    #[test]
    fn a_build_invocation_carries_the_program_arguments_and_directory() {
        let invocation = FakeToolchain::dotnet()
            .build_invocation(&project())
            .unwrap();

        assert_eq!(invocation.program(), "dotnet");
        assert_eq!(
            invocation.arguments(),
            &["build".to_string(), "app".to_string()]
        );
        assert_eq!(
            invocation.working_directory(),
            Path::new("C:\\proyectos\\app")
        );
    }

    #[test]
    fn a_run_invocation_carries_the_program_arguments_and_directory() {
        let invocation = FakeToolchain::jdk().run_invocation(&project()).unwrap();

        assert_eq!(invocation.program(), "java");
        assert_eq!(
            invocation.arguments(),
            &["-jar".to_string(), "app.jar".to_string()]
        );
        assert_eq!(
            invocation.working_directory(),
            Path::new("C:\\proyectos\\app")
        );
    }

    #[test]
    fn an_invocation_can_have_no_arguments() {
        let invocation = Invocation::new("app.exe", Vec::new(), PathBuf::from("C:\\app"));

        assert_eq!(invocation.arguments(), &[] as &[String]);
    }

    #[test]
    fn several_toolchains_can_be_queried_through_the_same_interface() {
        let toolchains: [&dyn ToolchainProvider; 2] =
            [&FakeToolchain::dotnet(), &FakeToolchain::jdk()];

        let types: Vec<ProjectType> = toolchains.iter().map(|t| t.project_type()).collect();
        let tools: Vec<&str> = toolchains.iter().map(|t| t.tool()).collect();
        let availability: Vec<bool> = toolchains.iter().map(|t| t.is_available()).collect();

        assert_eq!(
            types,
            vec![ProjectType::CSharpWinForms, ProjectType::JavaSwing]
        );
        assert_eq!(tools, vec![".NET SDK", "JDK"]);
        assert_eq!(availability, vec![true, false]);
    }

    #[test]
    fn a_provider_can_refuse_to_prepare_an_invocation() {
        struct Picky;

        impl ToolchainProvider for Picky {
            fn project_type(&self) -> ProjectType {
                ProjectType::CSharpWinForms
            }

            fn tool(&self) -> &'static str {
                ".NET SDK"
            }

            fn is_available(&self) -> bool {
                true
            }

            fn build_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
                Err(CoreError::Unsupported("falta el SDK".to_string()))
            }

            fn run_invocation(&self, _project: &Project) -> CoreResult<Invocation> {
                Err(CoreError::Unsupported("falta el SDK".to_string()))
            }

            fn parse_diagnostics(&self, _output: &ProcessOutput, _root: &Path) -> Vec<Diagnostic> {
                Vec::new()
            }
        }

        let result = Picky.build_invocation(&project());

        assert!(matches!(result, Err(CoreError::Unsupported(_))));
    }

    #[test]
    fn the_search_path_is_split_into_directories() {
        let directories = usable_directories(OsStr::new("C:\\a;C:\\b"));

        assert_eq!(
            directories,
            vec![PathBuf::from("C:\\a"), PathBuf::from("C:\\b")]
        );
    }

    #[test]
    fn an_empty_entry_in_the_search_path_is_ignored() {
        let directories = usable_directories(OsStr::new("C:\\a;;C:\\b;"));

        assert_eq!(
            directories,
            vec![PathBuf::from("C:\\a"), PathBuf::from("C:\\b")]
        );
    }

    #[test]
    fn a_search_path_with_nothing_usable_gives_no_directories() {
        assert!(usable_directories(OsStr::new("")).is_empty());
        assert!(usable_directories(OsStr::new(";;")).is_empty());
    }

    #[test]
    fn an_executable_is_looked_for_with_and_without_each_extension() {
        let candidates = candidate_names("dotnet", &[".exe", ".cmd"]);

        assert_eq!(candidates, vec!["dotnet", "dotnet.exe", "dotnet.cmd"]);
    }

    #[test]
    fn an_executable_that_is_not_there_is_not_found() {
        let found = detect("dotnet", &[PathBuf::from("C:\\no-existe")], &[".exe"]);

        assert_eq!(found, None);
    }
}
