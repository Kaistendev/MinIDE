use std::path::{Path, PathBuf};

use crate::core::{CoreError, CoreResult, ProjectType};
use crate::diagnostics::Diagnostic;
use crate::project::Project;
use crate::runtime::ProcessOutput;
use crate::toolchain::{detect, msbuild, Invocation, ToolchainProvider};

/// El SDK de .NET instalado en el sistema.
///
/// Se localiza por su ejecutable en el PATH, sin ejecutarlo: detectar que esta
/// no es lo mismo que comprobar que funciona, y solo lanza el binario el
/// build.
pub struct DotNetSdk;

impl DotNetSdk {
    /// Executable con el que se localiza el SDK.
    pub const EXECUTABLE: &'static str = "dotnet";

    /// Extensiones con las que se busca, sin usar `PATHEXT`.
    pub const EXTENSIONS: &'static [&'static str] = &[".exe", ".cmd", ".bat"];

    /// Ruta del ejecutable del SDK, si esta en el PATH del sistema.
    pub fn detect() -> Option<PathBuf> {
        let path = std::env::var_os("PATH")?;

        Self::detect_in(
            &crate::toolchain::usable_directories(&path),
            Self::EXTENSIONS,
        )
    }

    /// Ruta del ejecutable del SDK buscandolo solo en `directories`.
    ///
    /// Existe para poder probar la deteccion con un directorio controlado, sin
    /// depender del PATH de quien ejecuta los tests.
    pub fn detect_in(directories: &[PathBuf], extensions: &[&str]) -> Option<PathBuf> {
        detect(Self::EXECUTABLE, directories, extensions)
    }
}

/// Toolchain de .NET: compila proyectos de C# con Windows Forms.
pub struct DotNetToolchain;

impl ToolchainProvider for DotNetToolchain {
    fn project_type(&self) -> ProjectType {
        ProjectType::CSharpWinForms
    }

    fn tool(&self) -> &'static str {
        ".NET SDK"
    }

    fn is_available(&self) -> bool {
        DotNetSdk::detect().is_some()
    }

    /// Prepara `dotnet build <archivo de proyecto>` mas los argumentos de la
    /// configuracion, desde la raiz del proyecto.
    ///
    /// El archivo de proyecto se busca en la raiz en vez de dejar que `dotnet`
    /// lo decida, para que la llamada diga que compila y para poder avisar con
    /// claridad cuando no hay ninguno o hay mas de uno.
    fn build_invocation(&self, project: &Project) -> CoreResult<Invocation> {
        if project.project_type() != self.project_type() {
            return Err(CoreError::Unsupported(format!(
                "la toolchain de .NET no compila proyectos de {}",
                project.project_type()
            )));
        }

        let project_file = project_file_in(project)?;

        let mut arguments = vec!["build".to_string(), project_file];
        arguments.extend(project.build_configuration().arguments().iter().cloned());

        Ok(Invocation::new(
            DotNetSdk::EXECUTABLE,
            arguments,
            project.root().to_path_buf(),
        ))
    }

    /// Prepara `dotnet run --no-build`, que lanza la aplicacion ya compilada.
    ///
    /// Se pasa `--no-build` porque compilar es otra operacion, con su propio
    /// resultado y su propio historial de errores.
    fn run_invocation(&self, project: &Project) -> CoreResult<Invocation> {
        if project.project_type() != self.project_type() {
            return Err(CoreError::Unsupported(format!(
                "la toolchain de .NET no ejecuta proyectos de {}",
                project.project_type()
            )));
        }

        Ok(Invocation::new(
            DotNetSdk::EXECUTABLE,
            vec!["run".to_string(), "--no-build".to_string()],
            project.root().to_path_buf(),
        ))
    }

    fn parse_diagnostics(&self, output: &ProcessOutput, root: &Path) -> Vec<Diagnostic> {
        msbuild::parse(output, root)
    }
}

/// Nombre del archivo de proyecto de la raiz, si hay exactamente uno.
fn project_file_in(project: &Project) -> CoreResult<String> {
    let root = project.root();
    let entries = std::fs::read_dir(root).map_err(|error| CoreError::from_io(root, &error))?;
    let mut found: Vec<String> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|error| CoreError::from_io(root, &error))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if name.to_lowercase().ends_with(".csproj") {
            found.push(name.into_owned());
        }
    }

    match found.len() {
        0 => Err(CoreError::NotFound(format!(
            "no hay ningun archivo .csproj en {}",
            root.display()
        ))),
        1 => Ok(found.remove(0)),
        _ => Err(CoreError::Unsupported(format!(
            "{} tiene {} archivos de proyecto y no se puede elegir uno",
            root.display(),
            found.len()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CoreError, CoreResult, ProjectType};
    use crate::project::{BuildConfiguration, Project, ProjectRelativePath};
    use crate::toolchain::Invocation;
    use std::path::PathBuf;

    fn project_in(root: &std::path::Path) -> Project {
        Project::new(
            "App",
            root,
            ProjectType::CSharpWinForms,
            BuildConfiguration::new(ProjectRelativePath::new("bin").unwrap()),
        )
        .unwrap()
    }

    /// Crea un directorio temporal con los archivos indicados.
    fn root_with(name: &str, files: &[&str]) -> PathBuf {
        let root = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("directory");

        for file in files {
            std::fs::write(root.join(file), "<Project />").expect("file");
        }

        root
    }

    #[test]
    fn the_dotnet_sdk_is_looked_for_by_its_executable() {
        assert_eq!(DotNetSdk::EXECUTABLE, "dotnet");
    }

    #[test]
    fn the_dotnet_executable_is_looked_for_with_the_usual_windows_extensions() {
        assert_eq!(DotNetSdk::EXTENSIONS, &[".exe", ".cmd", ".bat"]);
    }

    #[test]
    fn the_dotnet_sdk_is_not_where_it_is_not() {
        let found = DotNetSdk::detect_in(&[PathBuf::from("C:\\no-existe")], &[".exe"]);

        assert_eq!(found, None);
    }

    #[test]
    fn detection_uses_the_shared_executable_search() {
        let found = detect("dotnet", &[PathBuf::from("C:\\no-existe")], &[".exe"]);

        assert_eq!(found, None);
    }

    #[test]
    fn the_toolchain_handles_csharp_winforms_with_the_dotnet_sdk() {
        let toolchain = DotNetToolchain;

        assert_eq!(toolchain.project_type(), ProjectType::CSharpWinForms);
        assert_eq!(toolchain.tool(), ".NET SDK");
    }

    #[test]
    fn the_build_invocation_names_the_project_file() {
        let root = root_with("miniide-t056-invocation", &["App.csproj"]);
        let project = project_in(&root);

        let invocation = DotNetToolchain.build_invocation(&project).unwrap();

        assert_eq!(invocation.program(), "dotnet");
        assert_eq!(invocation.arguments()[0], "build");
        assert!(invocation.arguments().contains(&"App.csproj".to_string()));
        assert_eq!(invocation.working_directory(), root);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_build_invocation_carries_the_configuration_arguments() {
        let root = root_with("miniide-t056-arguments", &["App.csproj"]);
        let mut project = project_in(&root);
        project
            .build_configuration_mut()
            .add_argument("--configuration");

        let invocation = DotNetToolchain.build_invocation(&project).unwrap();

        assert_eq!(
            invocation.arguments(),
            &[
                "build".to_string(),
                "App.csproj".to_string(),
                "--configuration".to_string()
            ]
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_root_without_a_project_file_has_no_build_invocation() {
        let root = root_with("miniide-t056-empty", &[]);
        let project = project_in(&root);

        let result = DotNetToolchain.build_invocation(&project);

        assert!(matches!(result, Err(CoreError::NotFound(_))));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_root_with_several_project_files_has_no_undecided_build_invocation() {
        let root = root_with("miniide-t056-many", &["App.csproj", "Otro.csproj"]);
        let project = project_in(&root);

        let result = DotNetToolchain.build_invocation(&project);

        assert!(matches!(result, Err(CoreError::Unsupported(_))));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_run_invocation_does_not_build_the_project_again() {
        let root = root_with("miniide-t058-run", &["App.csproj"]);
        let project = project_in(&root);

        let invocation = DotNetToolchain.run_invocation(&project).unwrap();

        assert_eq!(invocation.program(), "dotnet");
        assert_eq!(invocation.arguments()[0], "run");
        assert!(
            invocation.arguments().contains(&"--no-build".to_string()),
            "el proyecto ya se compilo: {:?}",
            invocation.arguments()
        );
        assert_eq!(invocation.working_directory(), root);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_toolchain_only_builds_csharp_winforms_projects() {
        let root = root_with("miniide-t056-java", &[]);
        let java = Project::new(
            "App",
            &root,
            ProjectType::JavaSwing,
            BuildConfiguration::new(ProjectRelativePath::new("bin").unwrap()),
        )
        .unwrap();

        let result: CoreResult<Invocation> = DotNetToolchain.build_invocation(&java);

        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&root);
    }
}
