use std::fmt;
use std::path::{Path, PathBuf};

use crate::core::{CoreError, CoreResult, ProjectType};
use crate::diagnostics::Diagnostic;
use crate::project::{Project, ProjectFileKind};
use crate::runtime::{self, ProcessOutput};
use crate::toolchain::{detect, javac, usable_directories, Invocation, ToolchainProvider};

/// Argumento con el que `javac` dice donde dejar las clases.
const OUTPUT_ARGUMENT: &str = "-d";

/// Executable con el que se ejecuta un proyecto de Java. Es el de la JVM, que en
/// un JDK va junto al compilador.
const RUN_EXECUTABLE: &str = "java";

/// Argumento con el que la JVM dice donde buscar las clases.
const CLASSPATH_ARGUMENT: &str = "-cp";

/// La firma que la JVM busca para poder ejecutar una clase.
const MAIN_METHOD: &str = "public static void main";

/// ExtENSION de los fuentes que compila `javac`.
const SOURCE_EXTENSION: &str = "java";

/// Argumento con el que un ejecutable de la JVM dice donde esta instalado.
const SETTINGS_ARGUMENT: &str = "-XshowSettings:properties";

/// Propiedad de esas propiedades que dice cual es el JDK de verdad.
const HOME_PROPERTY: &str = "java.home";

/// Extension de los ejecutables. MiniIDE es de Windows y por eso no la busca en
/// un `PATHEXT` que tambien trae extensiones que aqui no significan nada.
const EXECUTABLE_EXTENSION: &str = ".exe";

/// El JDK instalado en el sistema.
///
/// Se localiza por su compilador, `javac`, y no por `java`: un runtime de Java
/// trae `java` pero no compila, y MiniIDE necesita compilar para poder ejecutar
/// lo que genera el disenador.
pub struct Jdk;

impl Jdk {
    /// Executable con el que se localiza el JDK.
    pub const EXECUTABLE: &'static str = "javac";

    /// Extensiones con las que se busca, sin usar `PATHEXT`.
    pub const EXTENSIONS: &'static [&'static str] = &[".exe", ".cmd", ".bat"];

    /// Argumento con el que el compilador dice que version es.
    pub const VERSION_ARGUMENT: &'static str = "-version";

    /// Ruta del compilador del JDK, si esta en el PATH del sistema.
    pub fn detect() -> Option<PathBuf> {
        let path = std::env::var_os("PATH")?;

        Self::detect_in(&usable_directories(&path), Self::EXTENSIONS)
    }

    /// Ruta del compilador buscandolo solo en `directories`.
    ///
    /// Existe para poder probar la deteccion con un directorio controlado, sin
    /// depender del PATH de quien ejecuta los tests.
    pub fn detect_in(directories: &[PathBuf], extensions: &[&str]) -> Option<PathBuf> {
        detect(Self::EXECUTABLE, directories, extensions)
    }

    /// La llamada que pregunta la version al compilador de `location`.
    ///
    /// Se llama a la ruta completa y no a `javac`, para que la version sea la del
    /// JDK que se ha encontrado y no la de otro que aparezca antes en el PATH.
    pub fn version_invocation(location: &Path) -> Invocation {
        let working_directory = location
            .parent()
            .filter(|directory| directory.is_dir())
            .map_or_else(std::env::temp_dir, Path::to_path_buf);

        Invocation::new(
            location.to_string_lossy().into_owned(),
            vec![Self::VERSION_ARGUMENT.to_string()],
            working_directory,
        )
    }

    /// Estado del JDK del sistema.
    pub fn status() -> JdkStatus {
        match std::env::var_os("PATH") {
            Some(path) => Self::status_in(&usable_directories(&path)),
            None => JdkStatus::missing(),
        }
    }

    /// Estado del JDK buscandolo solo en `directories`.
    ///
    /// Es sincrono: preguntar la version lanza el compilador y espera a que
    /// termine. Quien lo llame desde la interfaz tiene que hacerlo fuera del
    /// hilo de la interfaz.
    pub fn status_in(directories: &[PathBuf]) -> JdkStatus {
        let Some(location) = Self::detect_in(directories, Self::EXTENSIONS) else {
            return JdkStatus::missing();
        };

        let version = runtime::run(&Self::version_invocation(&location))
            .ok()
            .and_then(|output| Self::version_of(&output));

        JdkStatus::available(location, version)
    }

    /// La version que dice la salida de `javac -version`, si se puede leer.
    ///
    /// El formato es `javac <version>`. Se lee stdout y, si no hay nada ahi,
    /// stderr: las versiones antiguas escribian en stderr la linea de la
    /// version. Una salida que no nombra al compilador no da version, en vez de
    /// inventarse una con lo que haya venga.
    fn version_of(output: &ProcessOutput) -> Option<String> {
        [&output.standard_output(), &output.standard_error()]
            .into_iter()
            .find_map(|stream| Self::version_line(stream))
    }

    /// La version de la primera linea de `stream` que hable del compilador.
    fn version_line(stream: &str) -> Option<String> {
        stream.lines().find_map(|line| {
            let mut words = line.split_whitespace();
            let tool = words.next()?;

            if !tool.eq_ignore_ascii_case(Self::EXECUTABLE) {
                return None;
            }

            let version = words.next()?;

            Some(version.trim_matches('"').to_string())
        })
    }

    /// Lo que se le dice al usuario cuando no hay JDK, y el texto del estado que
    /// falta.
    pub fn missing_message() -> String {
        format!(
            "no hay JDK instalado: no se ha encontrado {} en el PATH. \
             Instala un JDK y anade su carpeta bin al PATH.",
            Self::EXECUTABLE
        )
    }

    /// El ejecutable de `tool` (`java` o `javac`) del JDK de verdad, o el nombre
    /// del PATH si no se puede saber cual es el JDK de verdad.
    ///
    /// En el PATH puede no estar el JDK, sino un lanzador: el que instala Oracle
    /// en `Common Files\...\javapath` no es la JVM ni el compilador, es un
    /// programa que los pone en marcha como proceso hijo y se queda esperando.
    ///
    /// Importa porque MiniIDE registra el proceso que lanza y despues lo para:
    /// con el lanzador, pararlo dejaba la aplicacion viva, porque el proceso que
    /// tiene la ventana es el hijo. Y como el lanzador decide que JDK usa, y no
    /// siempre es el mismo entre `java` y `javac`, compilar y ejecutar se
    /// resuelven los dos en el mismo JDK.
    ///
    /// Se pregunta al lanzador de la JVM y no al del compilador, porque es el de
    /// la JVM el que responde con su `java.home`; preguntar al otro dria siempre
    /// que no se sabe el JDK.
    ///
    /// Cuesta un arranque de JVM por llamada, y no se guarda lo que dice: un JDK
    /// que se cambia se tiene que notar en la siguiente compilacion.
    pub fn executable(tool: &str) -> String {
        Self::executable_from(Path::new(RUN_EXECUTABLE), tool)
    }

    /// El ejecutable de `tool` en el JDK que dice `launcher`, y el nombre a
    /// secas si `launcher` no dice nada o su JDK no trae ese ejecutable.
    fn executable_from(launcher: &Path, tool: &str) -> String {
        Self::executable_in(Self::home_of(launcher).as_deref(), tool)
    }

    /// El ejecutable de `tool` en `home`, o el nombre a secas si no se sabe el
    /// home o si ahi no esta ese ejecutable.
    ///
    /// Sin home, o sin el ejecutable, se devuelve el nombre: es lo que hacia el
    /// PATH resolver, y es mejor lanzar el lanzador que no lanzar nada.
    fn executable_in(home: Option<&Path>, tool: &str) -> String {
        let executable = home
            .map(|home| {
                home.join("bin")
                    .join(format!("{tool}{EXECUTABLE_EXTENSION}"))
            })
            .filter(|path| path.is_file());

        match executable {
            Some(path) => path.to_string_lossy().into_owned(),
            None => tool.to_string(),
        }
    }

    /// El directorio del JDK que dice `launcher`, si lo dice y si existe.
    fn home_of(launcher: &Path) -> Option<PathBuf> {
        let working_directory = launcher
            .parent()
            .filter(|directory| directory.is_dir())
            .map_or_else(std::env::temp_dir, Path::to_path_buf);

        let invocation = Invocation::new(
            launcher.to_string_lossy().into_owned(),
            vec![
                SETTINGS_ARGUMENT.to_string(),
                Jdk::VERSION_ARGUMENT.to_string(),
            ],
            working_directory,
        );

        let output = runtime::run(&invocation).ok()?;

        // La properties van a stderr en unas versiones y a stdout en otras, asi
        // que se mira en las dos antes de concluir que no las hay.
        let home = [&output.standard_error(), &output.standard_output()]
            .into_iter()
            .find_map(|stream| home_in(stream).map(str::to_string))?;

        let home = PathBuf::from(home);

        home.is_dir().then_some(home)
    }
}

/// El valor de `java.home` en la salida de un `-XshowSettings:properties`.
///
/// Son lineas de la forma `    java.home = C:\Java\jdk-21`, con la propiedad
/// sangrada y el valor detras de un `=`.
fn home_in(text: &str) -> Option<&str> {
    text.lines().find_map(|line| {
        let (property, value) = line.trim().split_once('=')?;
        let value = value.trim();

        if property.trim() == HOME_PROPERTY && !value.is_empty() {
            Some(value)
        } else {
            None
        }
    })
}

/// El JDK encontrado, con su ubicacion y su version, o el hecho de que no esta.
///
/// Es lo que el IDE necesita para decidir si puede compilar y para explicar al
/// usuario por que no puede.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JdkStatus {
    /// El JDK esta en `location`, con la `version` si se ha podido leer.
    Available {
        location: PathBuf,
        version: Option<String>,
    },
    /// No hay JDK. El motivo no depende de la maquina: falta el compilador.
    Missing,
}

impl JdkStatus {
    /// El JDK esta en `location`. `version` es `None` si no se ha podido leer,
    /// que no es motivo para decir que el JDK no esta.
    pub fn available(location: PathBuf, version: Option<String>) -> Self {
        Self::Available { location, version }
    }

    /// No hay JDK.
    pub fn missing() -> Self {
        Self::Missing
    }

    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available { .. })
    }

    /// Donde esta el compilador, si hay JDK.
    pub fn location(&self) -> Option<&Path> {
        match self {
            Self::Available { location, .. } => Some(location),
            Self::Missing => None,
        }
    }

    /// La version del JDK, si se ha podido preguntar.
    pub fn version(&self) -> Option<&str> {
        match self {
            Self::Available { version, .. } => version.as_deref(),
            Self::Missing => None,
        }
    }
}

impl fmt::Display for JdkStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Available { location, version } => match version {
                Some(version) => write!(f, "JDK {version} en {}", location.display()),
                None => write!(f, "JDK en {} (version no leida)", location.display()),
            },
            Self::Missing => f.write_str(&Jdk::missing_message()),
        }
    }
}

/// Toolchain de Java: compila proyectos de Java con Swing.
pub struct JdkToolchain;

impl ToolchainProvider for JdkToolchain {
    fn project_type(&self) -> ProjectType {
        ProjectType::JavaSwing
    }

    fn tool(&self) -> &'static str {
        "JDK"
    }

    fn is_available(&self) -> bool {
        Jdk::detect().is_some()
    }

    /// El JDK se busca por su compilador, asi que el mensaje lo dice: tener Java no
    /// es tener JDK, y quien lee el error tiene que saber donde se mira.
    fn missing_message(&self) -> String {
        Jdk::missing_message()
    }

    /// Prepara `javac -d <salida> <argumentos> <fuentes>`, desde la raiz del
    /// proyecto.
    ///
    /// `javac` no compila "el proyecto": compila los archivos que se le pasan, y
    /// `-d` dice donde deja las clases. Por eso los fuentes se buscan en el arbol
    /// del proyecto y no en un archivo de proyecto.
    fn build_invocation(&self, project: &Project) -> CoreResult<Invocation> {
        if project.project_type() != self.project_type() {
            return Err(CoreError::Unsupported(format!(
                "la toolchain del JDK no compila proyectos de {}",
                project.project_type()
            )));
        }

        let mut arguments = vec![
            OUTPUT_ARGUMENT.to_string(),
            project
                .build_configuration()
                .output_directory()
                .as_path()
                .to_string_lossy()
                .into_owned(),
        ];
        arguments.extend(project.build_configuration().arguments().iter().cloned());
        arguments.extend(java_sources(project)?);

        Ok(Invocation::new(
            Jdk::executable(Jdk::EXECUTABLE),
            arguments,
            project.root().to_path_buf(),
        ))
    }

    /// Prepara `java -cp <salida> <clase principal>`, desde la raiz del proyecto.
    ///
    /// Las clases se buscan en la carpeta de salida de la compilacion, y la clase
    /// principal se busca en los fuentes del proyecto.
    fn run_invocation(&self, project: &Project) -> CoreResult<Invocation> {
        if project.project_type() != self.project_type() {
            return Err(CoreError::Unsupported(format!(
                "la toolchain del JDK no ejecuta proyectos de {}",
                project.project_type()
            )));
        }

        let sources = java_sources(project)?;
        let main_class = main_class_in(project, &sources)?;

        Ok(Invocation::new(
            Jdk::executable(RUN_EXECUTABLE),
            vec![
                CLASSPATH_ARGUMENT.to_string(),
                project
                    .build_configuration()
                    .output_directory()
                    .as_path()
                    .to_string_lossy()
                    .into_owned(),
                main_class,
            ],
            project.root().to_path_buf(),
        ))
    }

    /// Los errores de `javac` se convierten en `Diagnostic`.
    ///
    /// El parseo es el del compilador de Java, en `javac`, para que la toolchain
    /// no sepa el formato de la salida de su herramienta.
    fn parse_diagnostics(&self, output: &ProcessOutput, root: &Path) -> Vec<Diagnostic> {
        javac::parse(output, root)
    }
}

/// Los fuentes de Java del proyecto, relativos a su raiz y en orden.
///
/// Java no tiene un archivo de proyecto que diga donde estan los fuentes, asi que
/// se compila todo lo que hay en el arbol del proyecto. El directorio de salida no
/// se excluye: `javac` solo mira los archivos que se le pasan.
///
/// Se devuelven con `/` como separador porque es el separador de las rutas
/// relativas del proyecto, y `javac` lo acepta en Windows.
fn java_sources(project: &Project) -> CoreResult<Vec<String>> {
    let mut sources: Vec<String> = Vec::new();

    for file in project.discover_files()? {
        let path = file.path().as_path();

        if file.kind() != ProjectFileKind::File {
            continue;
        }

        let is_source = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case(SOURCE_EXTENSION));

        if is_source {
            sources.push(relative_with_slashes(path));
        }
    }

    if sources.is_empty() {
        return Err(CoreError::NotFound(format!(
            "no hay ningun archivo .java en {}",
            project.root().display()
        )));
    }

    Ok(sources)
}

/// `path` con `/` entre sus componentes.
fn relative_with_slashes(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<String>>()
        .join("/")
}

/// La clase que se ejecuta, buscada entre los fuentes del proyecto.
///
/// Java no tiene un archivo de proyecto que diga cual es la clase principal, y
/// MiniIDE tampoco se lo inventa: se busca el `main` que la JVM pide.
///
/// El nombre de la clase es el del archivo que la contiene, porque una clase
/// publica tiene que estar en un archivo con su nombre, y delante va el paquete
/// si el fuente lo declara: la JVM recibe el nombre completo.
///
/// Si hay mas de una clase con `main` no se elige ninguna: un proyecto con dos
/// puntos de entrada necesita que se le diga cual es.
fn main_class_in(project: &Project, sources: &[String]) -> CoreResult<String> {
    let mut found: Vec<String> = Vec::new();

    for source in sources {
        let path = project.root().join(source);
        let content =
            std::fs::read_to_string(&path).map_err(|error| CoreError::from_io(&path, &error))?;

        if !content.contains(MAIN_METHOD) {
            continue;
        }

        found.push(class_name_of(source, &content));
    }

    match found.len() {
        0 => Err(CoreError::NotFound(format!(
            "ningun fuente de {} declara {MAIN_METHOD}",
            project.root().display()
        ))),
        1 => Ok(found.remove(0)),
        _ => Err(CoreError::Unsupported(format!(
            "{} tiene {} clases que declaran {MAIN_METHOD} y no se puede elegir una",
            project.root().display(),
            found.len()
        ))),
    }
}

/// La clase que declara el fuente `source`, con su paquete si lo tiene.
///
/// El paquete se lee del propio fuente y no del directorio: MiniIDE deja los
/// fuentes donde quiera el usuario, y la carpeta no tiene por que ser el paquete.
fn class_name_of(source: &str, content: &str) -> String {
    let class = Path::new(source)
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();

    match package_of(content) {
        Some(package) => format!("{package}.{class}"),
        None => class,
    }
}

/// El paquete que declara `content`, si declara alguno.
fn package_of(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let rest = line
            .trim()
            .strip_prefix("package ")?
            .trim()
            .strip_suffix(';')?;

        if rest.is_empty() {
            None
        } else {
            Some(rest.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CoreError, CoreResult, ProjectType};
    use crate::diagnostics::DiagnosticLevel;
    use crate::project::{BuildConfiguration, Project, ProjectRelativePath};
    use crate::toolchain::{Invocation, JdkStatus, ToolchainProvider};
    use std::path::{Path, PathBuf};

    /// Crea un directorio temporal con los archivos indicados, creando los
    /// directorios intermedios de cada ruta.
    fn directory_with(name: &str, files: &[&str]) -> PathBuf {
        let root = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("directory");

        for file in files {
            let path = root.join(file);

            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("directorio");
            }

            std::fs::write(path, "").expect("file");
        }

        root
    }

    /// Un proyecto de Java con Swing, como el que crea la plantilla.
    fn java_project_in(root: &Path) -> Project {
        Project::new(
            "App",
            root,
            ProjectType::JavaSwing,
            BuildConfiguration::new(ProjectRelativePath::new("bin").unwrap()),
        )
        .unwrap()
    }

    /// Un directorio con los fuentes indicados, con el contenido que se le pida.
    fn directory_with_sources(name: &str, sources: &[(&str, &str)]) -> PathBuf {
        let root = directory_with(name, &[]);

        for (path, content) in sources {
            let target = root.join(path);

            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).expect("directorio");
            }

            std::fs::write(target, content).expect("file");
        }

        root
    }

    const WITH_MAIN: &str =
        "public class Main {\n    public static void main(String[] args) {\n    }\n}\n";

    const WITHOUT_MAIN: &str = "public class MainWindow {\n    public void abrir() {\n    }\n}\n";

    fn output(standard_output: &str, standard_error: &str) -> ProcessOutput {
        ProcessOutput::new(
            Some(0),
            standard_output.to_string(),
            standard_error.to_string(),
        )
    }

    #[test]
    fn the_jdk_is_looked_for_by_its_compiler() {
        assert_eq!(Jdk::EXECUTABLE, "javac");
    }

    #[test]
    fn the_java_executable_is_looked_for_with_the_usual_windows_extensions() {
        assert_eq!(Jdk::EXTENSIONS, &[".exe", ".cmd", ".bat"]);
    }

    #[test]
    fn a_compiler_that_is_not_in_the_search_path_is_not_found() {
        let found = Jdk::detect_in(&[PathBuf::from("C:\\no-existe")], &[".exe"]);

        assert_eq!(found, None);
    }

    #[test]
    fn the_compiler_of_the_first_directory_of_the_search_path_is_the_one_detected() {
        let first = directory_with("miniide-t066-orden-1", &["javac.exe"]);
        let second = directory_with("miniide-t066-orden-2", &["javac.exe"]);

        let found = Jdk::detect_in(&[first.clone(), second.clone()], Jdk::EXTENSIONS);

        assert_eq!(found, Some(first.join("javac.exe")));
        let _ = std::fs::remove_dir_all(&first);
        let _ = std::fs::remove_dir_all(&second);
    }

    #[test]
    fn a_search_path_with_no_compiler_reports_a_missing_jdk() {
        let empty = directory_with("miniide-t066-vacio", &["otro.txt"]);

        let status = Jdk::status_in(std::slice::from_ref(&empty));

        assert!(!status.is_available());
        assert_eq!(status.location(), None);
        assert_eq!(status.version(), None);
        let _ = std::fs::remove_dir_all(&empty);
    }

    #[test]
    fn a_missing_jdk_says_which_tool_is_missing_and_where_it_is_looked_for() {
        let message = JdkStatus::missing().to_string();

        assert!(message.contains("javac"), "{message}");
        assert!(message.contains("PATH"), "{message}");
    }

    #[test]
    fn the_version_is_asked_to_the_compiler_that_was_found() {
        let directory = directory_with("miniide-t066-version", &["javac.exe"]);
        let location = directory.join("javac.exe");

        let invocation = Jdk::version_invocation(&location);

        assert_eq!(invocation.program(), location.to_string_lossy());
        assert_eq!(invocation.arguments(), &["-version".to_string()]);
        assert_eq!(invocation.working_directory(), directory);
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn the_version_of_the_jdk_is_read_from_the_output_of_the_compiler() {
        let version = Jdk::version_of(&output("javac 21.0.1\n", ""));

        assert_eq!(version, Some("21.0.1".to_string()));
    }

    #[test]
    fn a_version_written_to_stderr_is_still_the_version_of_the_jdk() {
        let version = Jdk::version_of(&output("", "javac 1.8.0_202\n"));

        assert_eq!(version, Some("1.8.0_202".to_string()));
    }

    #[test]
    fn a_quoted_version_is_read_without_the_quotes() {
        let version = Jdk::version_of(&output("javac \"21.0.1+12\"\n", ""));

        assert_eq!(version, Some("21.0.1+12".to_string()));
    }

    #[test]
    fn output_that_does_not_name_the_compiler_gives_no_version() {
        assert_eq!(Jdk::version_of(&output("", "")), None);
        assert_eq!(
            Jdk::version_of(&output("error: unrecognized option\n", "")),
            None
        );
        assert_eq!(Jdk::version_of(&output("javac\n", "")), None);
    }

    #[test]
    fn an_available_jdk_reports_its_location_and_its_version() {
        let status = JdkStatus::available(
            PathBuf::from("C:\\jdk\\bin\\javac.exe"),
            Some("21".to_string()),
        );

        assert!(status.is_available());
        assert_eq!(
            status.location(),
            Some(Path::new("C:\\jdk\\bin\\javac.exe"))
        );
        assert_eq!(status.version(), Some("21"));
    }

    #[test]
    fn a_jdk_whose_version_cannot_be_read_is_still_available() {
        let status = JdkStatus::available(PathBuf::from("C:\\jdk\\bin\\javac.exe"), None);

        assert!(status.is_available());
        assert_eq!(status.version(), None);
    }

    #[test]
    fn an_available_jdk_shows_its_version_and_its_location() {
        let status = JdkStatus::available(
            PathBuf::from("C:\\jdk\\bin\\javac.exe"),
            Some("21.0.1".to_string()),
        );

        let text = status.to_string();

        assert!(text.contains("21.0.1"), "{text}");
        assert!(text.contains("javac.exe"), "{text}");
    }

    #[test]
    fn the_toolchain_handles_java_swing_with_the_jdk() {
        let toolchain = JdkToolchain;

        assert_eq!(toolchain.project_type(), ProjectType::JavaSwing);
        assert_eq!(toolchain.tool(), "JDK");
    }

    /// El JDK se busca por su compilador, asi que el mensaje lo dice: tener Java no
    /// es tener JDK, y quien lee el error tiene que saber que mira.
    #[test]
    fn the_java_toolchain_says_where_it_looks_for_the_jdk() {
        let message = JdkToolchain.missing_message();

        assert!(message.contains("javac"), "{message}");
        assert!(message.contains("PATH"), "{message}");
    }

    /// `javac` compila lo que se le dice, y `-d` dice donde deja las clases. Sin
    /// las dos cosas el proyecto no compila o compila en un sitio que MiniIDE no
    /// conoce.
    #[test]
    fn the_build_invocation_compiles_every_source_into_the_output_directory() {
        let root = directory_with(
            "miniide-t069-compila",
            &["src/main/java/Main.java", "src/main/java/MainWindow.java"],
        );
        let project = java_project_in(&root);

        let invocation = JdkToolchain.build_invocation(&project).unwrap();

        assert!(
            is_the_tool(invocation.program(), "javac"),
            "el build no llama al compilador: {}",
            invocation.program()
        );
        assert_eq!(
            invocation.arguments(),
            &[
                "-d".to_string(),
                "bin".to_string(),
                "src/main/java/Main.java".to_string(),
                "src/main/java/MainWindow.java".to_string(),
            ]
        );
        assert_eq!(invocation.working_directory(), root);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// El directorio de salida es el del proyecto, no uno fijo: cambiarlo en la
    /// configuracion tiene que cambiar la llamada.
    #[test]
    fn the_build_invocation_compiles_into_the_output_directory_of_the_project() {
        let root = directory_with("miniide-t069-salida", &["src/Main.java"]);
        let mut project = Project::new(
            "App",
            &root,
            ProjectType::JavaSwing,
            BuildConfiguration::new(ProjectRelativePath::new("out/classes").unwrap()),
        )
        .unwrap();
        project.build_configuration_mut().add_argument("-Xlint:all");

        let invocation = JdkToolchain.build_invocation(&project).unwrap();

        assert_eq!(
            invocation.arguments(),
            &[
                "-d".to_string(),
                "out/classes".to_string(),
                "-Xlint:all".to_string(),
                "src/Main.java".to_string(),
            ]
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Un proyecto de Java no tiene archivo de proyecto que diga donde estan sus
    /// fuentes, asi que se compila todo lo que hay en el arbol.
    #[test]
    fn every_java_source_of_the_project_is_compiled() {
        let root = directory_with(
            "miniide-t069-arbol",
            &[
                "Main.java",
                "src/Main.java",
                "src/vista/Panel.java",
                "notas.txt",
                "src/Form1.cs",
            ],
        );
        let project = java_project_in(&root);

        let invocation = JdkToolchain.build_invocation(&project).unwrap();

        for expected in ["Main.java", "src/Main.java", "src/vista/Panel.java"] {
            assert!(
                invocation
                    .arguments()
                    .iter()
                    .any(|argument| argument == expected),
                "no se compila {expected}: {:?}",
                invocation.arguments()
            );
        }
        for unexpected in ["notas.txt", "src/Form1.cs"] {
            assert!(
                !invocation
                    .arguments()
                    .iter()
                    .any(|argument| argument == unexpected),
                "se compila {unexpected}, que no es un fuente de Java"
            );
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Compilar un proyecto sin fuentes no puede ser un exito silencioso: no hay
    /// nada que compilar y el usuario tiene que saberlo.
    #[test]
    fn a_java_project_without_sources_has_no_build_invocation() {
        let root = directory_with("miniide-t069-vacio", &["notas.txt"]);
        let project = java_project_in(&root);

        let result = JdkToolchain.build_invocation(&project);

        assert!(matches!(result, Err(CoreError::NotFound(_))));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_toolchain_only_builds_java_swing_projects() {
        let root = directory_with("miniide-t069-csharp", &["Program.cs"]);
        let csharp = Project::new(
            "App",
            &root,
            ProjectType::CSharpWinForms,
            BuildConfiguration::new(ProjectRelativePath::new("bin").unwrap()),
        )
        .unwrap();

        let result: CoreResult<Invocation> = JdkToolchain.build_invocation(&csharp);

        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// La aplicacion se ejecuta con la JVM, con las clases de la carpeta de
    /// salida y la clase que declara el `main`.
    #[test]
    fn the_run_invocation_executes_the_main_class_of_the_project() {
        let root = directory_with_sources(
            "miniide-t071-ejecuta",
            &[
                ("src/main/java/Main.java", WITH_MAIN),
                ("src/main/java/MainWindow.java", WITHOUT_MAIN),
            ],
        );
        let project = java_project_in(&root);

        let invocation = JdkToolchain.run_invocation(&project).unwrap();

        assert!(
            is_the_tool(invocation.program(), "java"),
            "el run no llama a la JVM: {}",
            invocation.program()
        );
        assert_eq!(
            invocation.arguments(),
            &["-cp".to_string(), "bin".to_string(), "Main".to_string()]
        );
        assert_eq!(invocation.working_directory(), root);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Las clases se buscan donde las dejo la compilacion, no en la carpeta
    /// actual: si el proyecto compila en otro sitio, se ejecuta desde ese sitio.
    #[test]
    fn the_run_invocation_uses_the_output_directory_of_the_project() {
        let root = directory_with_sources("miniide-t071-salida", &[("src/Main.java", WITH_MAIN)]);
        let project = Project::new(
            "App",
            &root,
            ProjectType::JavaSwing,
            BuildConfiguration::new(ProjectRelativePath::new("out/classes").unwrap()),
        )
        .unwrap();

        let invocation = JdkToolchain.run_invocation(&project).unwrap();

        assert_eq!(
            invocation.arguments(),
            &[
                "-cp".to_string(),
                "out/classes".to_string(),
                "Main".to_string()
            ]
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Java no tiene archivo de proyecto que diga cual es la clase principal, asi
    /// que se busca en los fuentes el `main` que pide la JVM.
    #[test]
    fn the_main_class_is_the_file_that_declares_the_main_method() {
        let root = directory_with_sources(
            "miniide-t071-busca",
            &[
                ("src/vista/Main.java", WITH_MAIN),
                ("src/MainWindow.java", WITHOUT_MAIN),
            ],
        );
        let project = java_project_in(&root);

        let invocation = JdkToolchain.run_invocation(&project).unwrap();

        assert_eq!(invocation.arguments().last().unwrap(), "Main");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Una clase de un paquete se ejecuta con su nombre completo, o la JVM no la
    /// encuentra.
    #[test]
    fn the_main_class_of_a_source_in_a_package_is_qualified() {
        let root = directory_with_sources(
            "miniide-t071-paquete",
            &[(
                "src/vista/Panel.java",
                "package vista;\n\npublic class Panel {\n    public static void main(String[] args) {\n    }\n}\n",
            )],
        );
        let project = java_project_in(&root);

        let invocation = JdkToolchain.run_invocation(&project).unwrap();

        assert_eq!(invocation.arguments().last().unwrap(), "vista.Panel");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Un proyecto sin `main` no se puede ejecutar, y hay que decirlo en vez de
    /// preparar una llamada que no va a funcionar.
    #[test]
    fn a_project_without_a_main_method_has_no_run_invocation() {
        let root =
            directory_with_sources("miniide-t071-sin-main", &[("src/Main.java", WITHOUT_MAIN)]);
        let project = java_project_in(&root);

        let result = JdkToolchain.run_invocation(&project);

        assert!(matches!(result, Err(CoreError::NotFound(_))));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Con dos clases ejecutables no se elige una: se dice que hay dos.
    #[test]
    fn a_project_with_two_main_classes_has_no_undecided_run_invocation() {
        let root = directory_with_sources(
            "miniide-t071-dos-main",
            &[("src/Main.java", WITH_MAIN), ("src/Otro.java", WITH_MAIN)],
        );
        let project = java_project_in(&root);

        let result = JdkToolchain.run_invocation(&project);

        assert!(matches!(result, Err(CoreError::Unsupported(_))));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_toolchain_only_runs_java_swing_projects() {
        let root = directory_with("miniide-t071-csharp", &["Program.cs"]);
        let csharp = Project::new(
            "App",
            &root,
            ProjectType::CSharpWinForms,
            BuildConfiguration::new(ProjectRelativePath::new("bin").unwrap()),
        )
        .unwrap();

        let result: CoreResult<Invocation> = JdkToolchain.run_invocation(&csharp);

        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// La toolchain entrega al core los errores de `javac` ya convertidos en
    /// diagnosticos, con su archivo y su linea.
    #[test]
    fn the_errors_of_javac_become_diagnostics_of_the_toolchain() {
        let root = directory_with("miniide-t070-toolchain", &["src/Main.java"]);
        let output = output("", "src\\Main.java:2: error: ';' expected\n1 error\n");

        let diagnostics = JdkToolchain.parse_diagnostics(&output, &root);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].level(), DiagnosticLevel::Error);
        assert_eq!(diagnostics[0].message(), "';' expected");
        let location = diagnostics[0].location().expect("con posicion");
        assert_eq!(
            location.file().as_path(),
            Path::new("src").join("Main.java")
        );
        assert_eq!(location.position().line(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Si `program` es esa herramienta del JDK, esta-named o resuelta en el JDK.
    fn is_the_tool(program: &str, tool: &str) -> bool {
        program == tool
            || Path::new(program)
                .file_stem()
                .is_some_and(|stem| stem.to_string_lossy() == tool)
    }

    /// Un directorio que hace de JDK, con sus ejecutables en `bin`.
    fn fake_jdk(name: &str, tools: &[&str]) -> PathBuf {
        let home = directory_with(name, &[]);
        std::fs::create_dir_all(home.join("bin")).expect("bin");

        for tool in tools {
            std::fs::write(
                home.join("bin")
                    .join(format!("{tool}{EXECUTABLE_EXTENSION}")),
                "",
            )
            .expect("ejecutable");
        }

        home
    }

    /// Un lanzador de mentira que dice cual es su `java.home`.
    ///
    /// Es un `.cmd` porque un ejecutable de verdad no se puede escribir aqui, y
    /// con el se prueba toda la cadena sin depender del JDK de la maquina.
    fn fake_launcher(name: &str, home: &Path) -> PathBuf {
        let launcher = directory_with(name, &[]).join("java.cmd");
        let properties = format!("@echo off\r\necho     java.home = {}\r\n", home.display());

        std::fs::write(&launcher, properties).expect("lanzador");

        launcher
    }

    /// La cadena entera: el lanzador dice su JDK, y de ahi sale el ejecutable.
    #[test]
    fn the_executable_comes_from_the_jdk_that_the_launcher_names() {
        let home = fake_jdk("miniide-t072-cadena", &["java", "javac"]);
        let launcher = fake_launcher("miniide-t072-lanzador", &home);

        for tool in ["java", "javac"] {
            let expected = home
                .join("bin")
                .join(format!("{tool}{EXECUTABLE_EXTENSION}"));

            assert_eq!(
                Jdk::executable_from(&launcher, tool),
                expected.to_string_lossy().into_owned()
            );
        }

        let _ = std::fs::remove_dir_all(&home);
        let _ = std::fs::remove_dir_all(launcher.parent().expect("directorio"));
    }

    #[test]
    fn the_home_of_the_jdk_is_read_from_the_settings_of_the_jvm() {
        // La salida real de `java -XshowSettings:properties -version`.
        let settings = "Property settings:\n    java.class.version = 65.0\n    java.home = C:\\Program Files\\Java\\jdk-21\n    java.io.tmpdir = C:\\Temp\\\n    java.version = 21.0.7\n";

        assert_eq!(home_in(settings), Some("C:\\Program Files\\Java\\jdk-21"));
    }

    #[test]
    fn output_without_a_java_home_gives_no_home() {
        for output in [
            "",
            "openjdk version \"21.0.7\"\n",
            "    java.class.version = 65.0\n",
            "    java.home =\n",
        ] {
            assert_eq!(home_in(output), None, "{output}");
        }
    }

    /// El ejecutable sale del `bin` del JDK, y no de un sitio inventado.
    #[test]
    fn the_executable_of_a_tool_comes_from_the_bin_of_its_jdk() {
        let home = fake_jdk("miniide-t072-jdk", &["java", "javac"]);

        for tool in ["java", "javac"] {
            let expected = home
                .join("bin")
                .join(format!("{tool}{EXECUTABLE_EXTENSION}"));

            assert_eq!(
                Jdk::executable_in(Some(&home), tool),
                expected.to_string_lossy().into_owned()
            );
        }
        let _ = std::fs::remove_dir_all(&home);
    }
    /// Un JDK puede no traer uno de los dos ejecutables, y entonces se usa el
    /// nombre del PATH: es mejor lanzar el lanzador que no lanzar nada.
    #[test]
    fn a_jdk_without_that_executable_falls_back_to_the_name_of_the_path() {
        let home = fake_jdk("miniide-t072-sin-javac", &["java"]);

        assert_eq!(
            Jdk::executable_in(Some(&home), "java"),
            home.join("bin")
                .join(format!("java{EXECUTABLE_EXTENSION}"))
                .to_string_lossy()
                .into_owned()
        );
        assert_eq!(Jdk::executable_in(Some(&home), "javac"), "javac");
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn without_a_home_the_name_of_the_path_is_used() {
        for tool in ["java", "javac"] {
            assert_eq!(Jdk::executable_in(None, tool), tool);
        }
    }

    /// Un home que no existe no vale: se devuelve el nombre en vez de una ruta que
    /// no se puede lanzar.
    #[test]
    fn a_home_that_is_not_there_is_not_used() {
        let missing = std::env::temp_dir().join("miniide-t072-no-existe");

        assert_eq!(Jdk::executable_in(Some(&missing), "java"), "java");
    }

    /// Sin lanzador al que preguntar no se puede saber el JDK, y el proceso no se
    /// lanza en absoluto.
    #[test]
    fn a_launcher_that_cannot_be_asked_leaves_the_name_of_the_path() {
        for tool in ["java", "javac"] {
            assert_eq!(
                Jdk::executable_from(Path::new("C:\\miniide-no-existe.exe"), tool),
                tool
            );
        }
    }

    #[test]
    #[ignore = "requires JDK: pregunta al lanzador de la JVM del PATH cual es su JDK"]
    fn the_executable_of_the_jdk_of_this_machine_is_the_one_of_its_home() {
        let home = Jdk::home_of(Path::new(RUN_EXECUTABLE))
            .expect("el lanzador de la JVM dice cual es su JDK");

        for tool in ["java", "javac"] {
            let executable = Jdk::executable(tool);

            assert_eq!(
                Path::new(&executable),
                home.join("bin")
                    .join(format!("{tool}{EXECUTABLE_EXTENSION}")),
                "{tool} no se resuelve en el JDK del lanzador"
            );
        }
    }
}
