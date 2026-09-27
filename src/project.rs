use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use crate::core::{CoreError, CoreResult, FrameworkId, LanguageId, ProjectType};

const INVALID_NAME_CHARACTERS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

fn validate_name(name: &str) -> CoreResult<()> {
    if name.trim().is_empty() || name.trim() != name {
        return Err(CoreError::InvalidName(name.to_string()));
    }

    let has_invalid_character = name
        .chars()
        .any(|character| INVALID_NAME_CHARACTERS.contains(&character) || character.is_control());

    if has_invalid_character {
        return Err(CoreError::InvalidName(name.to_string()));
    }

    Ok(())
}

/// Una raiz tiene que existir como ruta y ser absoluta. La comparten la raiz
/// de un proyecto y la de un workspace.
pub(crate) fn validate_root(root: &Path) -> CoreResult<()> {
    if root.as_os_str().is_empty() || !root.is_absolute() {
        return Err(CoreError::InvalidPath(root.display().to_string()));
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRelativePath {
    value: PathBuf,
}

impl ProjectRelativePath {
    pub fn new(value: impl Into<PathBuf>) -> CoreResult<Self> {
        let value = value.into();

        if value.as_os_str().is_empty() {
            return Err(CoreError::InvalidPath(value.display().to_string()));
        }

        if value.is_absolute() {
            return Err(CoreError::InvalidPath(value.display().to_string()));
        }

        let is_inside_project = value
            .components()
            .all(|component| matches!(component, Component::Normal(_)));

        if !is_inside_project {
            return Err(CoreError::InvalidPath(value.display().to_string()));
        }

        Ok(Self { value })
    }

    pub fn as_path(&self) -> &Path {
        &self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectFileKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectFile {
    path: ProjectRelativePath,
    kind: ProjectFileKind,
}

impl ProjectFile {
    pub fn new(path: ProjectRelativePath, kind: ProjectFileKind) -> Self {
        Self { path, kind }
    }

    pub fn path(&self) -> &ProjectRelativePath {
        &self.path
    }

    pub fn kind(&self) -> ProjectFileKind {
        self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildConfiguration {
    output_directory: ProjectRelativePath,
    arguments: Vec<String>,
}

impl BuildConfiguration {
    pub const DEFAULT_OUTPUT_DIRECTORY: &'static str = "bin";

    pub fn new(output_directory: ProjectRelativePath) -> Self {
        Self {
            output_directory,
            arguments: Vec::new(),
        }
    }

    pub fn output_directory(&self) -> &ProjectRelativePath {
        &self.output_directory
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    pub fn add_argument(&mut self, argument: impl Into<String>) {
        self.arguments.push(argument.into());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    name: String,
    root: PathBuf,
    project_type: ProjectType,
    build_configuration: BuildConfiguration,
    files: Vec<ProjectFile>,
}

impl Project {
    pub fn new(
        name: impl Into<String>,
        root: impl Into<PathBuf>,
        project_type: ProjectType,
        build_configuration: BuildConfiguration,
    ) -> CoreResult<Self> {
        let name = name.into();
        let root = root.into();

        validate_name(&name)?;
        validate_root(&root)?;

        Ok(Self {
            name,
            root,
            project_type,
            build_configuration,
            files: Vec::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn project_type(&self) -> ProjectType {
        self.project_type
    }

    pub fn language(&self) -> LanguageId {
        self.project_type.language()
    }

    pub fn framework(&self) -> FrameworkId {
        self.project_type.framework()
    }

    pub fn build_configuration(&self) -> &BuildConfiguration {
        &self.build_configuration
    }

    /// Los argumentos de compilacion se anaden al proyecto, que es donde los
    /// lee despues la toolchain al preparar la llamada.
    pub fn build_configuration_mut(&mut self) -> &mut BuildConfiguration {
        &mut self.build_configuration
    }

    pub fn files(&self) -> &[ProjectFile] {
        &self.files
    }

    pub fn add_file(&mut self, file: ProjectFile) {
        self.files.push(file);
    }

    /// Crea un archivo con `contents` en la ruta `path` de la raiz del proyecto
    /// y lo anade al modelo.
    ///
    /// No sobrescribe nada: si ya hay algo en esa ruta devuelve
    /// `CoreError::AlreadyExists` y no cambia el archivo ni el modelo. Tampoco
    /// crea los directorios intermedios: si el directorio de `path` no existe
    /// devuelve `CoreError::NotFound`. Para eso esta crear directorio.
    ///
    /// La ruta la valida `ProjectRelativePath`, asi que no se puede escribir
    /// fuera de la raiz del proyecto.
    pub fn create_file(
        &mut self,
        path: ProjectRelativePath,
        contents: impl Into<String>,
    ) -> CoreResult<()> {
        let target = self.root.join(path.as_path());
        let parent = target.parent().unwrap_or(&self.root);

        if !parent.is_dir() {
            return Err(CoreError::NotFound(parent.display().to_string()));
        }

        // `OpenOptions::create_new` falla si ya hay algo en la ruta, sin la
        // carrera de comprobar y luego crear.
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .map_err(|error| CoreError::from_io(&target, &error))?;

        file.write_all(contents.into().as_bytes())
            .map_err(|error| CoreError::from_io(&target, &error))?;

        self.files
            .push(ProjectFile::new(path, ProjectFileKind::File));

        Ok(())
    }

    /// Crea el directorio `path` dentro de la raiz del proyecto y lo anade al
    /// modelo, creando tambien los directorios intermedios que falten.
    ///
    /// Si ya hay algo en esa ruta, sea un directorio o un archivo, devuelve
    /// `CoreError::AlreadyExists` y no cambia el modelo. A diferencia de crear
    /// un archivo, aqui crear los niveles intermedios es lo normal: quien pide
    /// `src/forms` quiere las dos carpetas.
    ///
    /// La ruta la valida `ProjectRelativePath`, asi que no se puede crear nada
    /// fuera de la raiz del proyecto.
    pub fn create_directory(&mut self, path: ProjectRelativePath) -> CoreResult<()> {
        let target = self.root.join(path.as_path());

        if target.exists() {
            return Err(CoreError::AlreadyExists(target.display().to_string()));
        }

        fs::create_dir_all(&target).map_err(|error| CoreError::from_io(&target, &error))?;

        self.files
            .push(ProjectFile::new(path, ProjectFileKind::Directory));

        Ok(())
    }

    /// Renombra la entrada `path` a `new_name`, sin moverla de directorio, y
    /// refleja en el modelo la ruta nueva.
    ///
    /// `new_name` es solo el nombre: no puede traer separadores, ser vacio ni
    /// ser `..`, asi que la entrada se queda donde esta. Si algo ya ocupa el
    /// nombre nuevo devuelve `CoreError::AlreadyExists` y no toca el modelo.
    ///
    /// Al renombrar un directorio, en el modelo tambien se renombran las
    /// entradas que estaban dentro de el, porque si no sus rutas apuntarian a
    /// sitios que ya no existen. El modelo solo conoce las entradas que se le
    /// han anadido, asi que para tener el arbol completo hay que volver a
    /// enumerar.
    pub fn rename(
        &mut self,
        path: ProjectRelativePath,
        new_name: impl Into<String>,
    ) -> CoreResult<()> {
        let new_name = new_name.into();
        let renamed = renamed_name(&new_name)?;
        let moved = sibling_of(&path, &renamed)?;
        let target = self.root.join(path.as_path());
        let destination = self.root.join(moved.as_path());

        // En Windows `fs::rename` machaca el destino sin avisar, asi que hay
        // que comprobarlo antes. No hay forma atomica de renombrar solo si el
        // destino no existe, asi que entre la comprobacion y el renombrado
        // queda una ventana: si otro proceso crea el destino en ese momento,
        // se sobrescribe.
        if destination.exists() {
            return Err(CoreError::AlreadyExists(destination.display().to_string()));
        }

        fs::rename(&target, &destination)
            .map_err(|error| CoreError::from_io(&destination, &error))?;

        self.move_in_model(&path, &moved);

        Ok(())
    }

    /// Reescribe en el modelo `path` como `moved`, y tambien las entradas que
    /// estaban dentro de `path`.
    fn move_in_model(&mut self, path: &ProjectRelativePath, moved: &ProjectRelativePath) {
        for file in &mut self.files {
            if file.path().as_path() == path.as_path() {
                *file = ProjectFile::new(moved.clone(), file.kind());
                continue;
            }

            if let Ok(inside) = file.path().as_path().strip_prefix(path.as_path()) {
                let mut rewritten = moved.as_path().to_path_buf();
                rewritten.push(inside);

                if let Ok(rewritten) = ProjectRelativePath::new(rewritten) {
                    *file = ProjectFile::new(rewritten, file.kind());
                }
            }
        }
    }

    /// Elimina la entrada `path` de la raiz del proyecto y la quita del modelo,
    /// con lo que hubiera dentro si era un directorio.
    ///
    /// Un directorio con contenido no se elimina: devuelve el error del sistema
    /// y no toca el modelo. Borrar un arbol entero tiene que ser una decision
    /// conscious, no el comportamiento por defecto de "eliminar".
    ///
    /// La ruta la valida `ProjectRelativePath`, asi que no se puede borrar nada
    /// fuera de la raiz del proyecto, y la raiz no se puede borrar porque no es
    /// una ruta relativa valida.
    pub fn remove(&mut self, path: ProjectRelativePath) -> CoreResult<()> {
        let target = self.root.join(path.as_path());

        if target.is_dir() {
            fs::remove_dir(&target).map_err(|error| CoreError::from_io(&target, &error))?;
        } else {
            fs::remove_file(&target).map_err(|error| CoreError::from_io(&target, &error))?;
        }

        self.drop_from_model(&path);

        Ok(())
    }

    /// Quita del modelo `path` y todo lo que estaba dentro de el.
    ///
    /// `strip_prefix` funciona por componentes y tambien vale para la ruta
    /// exacta, asi que un solo filtro quita la entrada y sus descendientes sin
    /// tocar a los vecinos que comparten prefijo.
    fn drop_from_model(&mut self, path: &ProjectRelativePath) {
        self.files
            .retain(|file| file.path().as_path().strip_prefix(path.as_path()).is_err());
    }

    /// Abre el proyecto cuyo archivo de proyecto es `project_file`.
    ///
    /// La raiz del proyecto es el directorio que contiene ese archivo y el
    /// nombre del proyecto es el de ese directorio, no el del archivo: un
    /// `pom.xml` se llamaria "pom".
    ///
    /// Que se pueda abrir depende de poder identificar la plataforma, y para
    /// C# eso significa que el proyecto diga que usa Windows Forms. Un proyecto
    /// WPF o una libreria de consola se rechazan con `CoreError::Unsupported`
    /// en vez de abrirse como WinForms, porqueMiniIDE acabaria generando
    /// codigo que no encaja. Un `pom.xml` se toma como Java con Swing porque no
    /// hay nada en el que se pueda comprobar.
    ///
    /// No es un lector de archivos de proyecto: no se lee `TargetFramework`,
    /// ni las referencias, ni el SDK. La configuracion de compilacion es la de
    /// defecto. Los archivos se enumerate con `discover_files`.
    pub fn open(project_file: &Path) -> CoreResult<Self> {
        if !project_file.exists() {
            return Err(CoreError::NotFound(project_file.display().to_string()));
        }

        if !project_file.is_file() {
            return Err(CoreError::InvalidPath(project_file.display().to_string()));
        }

        let content = fs::read_to_string(project_file)
            .map_err(|error| CoreError::from_io(project_file, &error))?;

        let project_type = detect_project_type(project_file, &content)?;
        let root = project_file
            .parent()
            .ok_or_else(|| CoreError::InvalidPath(project_file.display().to_string()))?;
        let name = root
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| CoreError::InvalidName(root.display().to_string()))?;

        Self::new(
            name,
            root,
            project_type,
            BuildConfiguration::new(ProjectRelativePath::new(
                BuildConfiguration::DEFAULT_OUTPUT_DIRECTORY,
            )?),
        )
    }

    /// Enumera los archivos y directorios que hay bajo la raiz del proyecto.
    ///
    /// El recorrido es en profundidad y en orden alfabetico, de modo que la
    /// lista es un recorrido pre-orden del arbol y cada directorio aparece
    /// antes que lo que contiene. La raiz no aparece en la lista.
    ///
    /// No filtra nada: tambien aparecen `bin`, `obj` o `.git` si existen. No
    /// muta el proyecto, asi que `files` sigue siendo la lista que se haya
    /// añadido a mano.
    pub fn discover_files(&self) -> CoreResult<Vec<ProjectFile>> {
        let mut found = Vec::new();

        discover_in(&self.root, &self.root, &mut found)?;

        Ok(found)
    }
}

/// Enumera el contenido de `directory` y de sus subdirectorios, guardando las
/// rutas relativas a `root`, que es la raiz del proyecto.
///
/// `directory` tiene que existir y ser un directorio.
fn discover_in(root: &Path, directory: &Path, found: &mut Vec<ProjectFile>) -> CoreResult<()> {
    if !directory.exists() {
        return Err(CoreError::NotFound(directory.display().to_string()));
    }

    if !directory.is_dir() {
        return Err(CoreError::InvalidPath(directory.display().to_string()));
    }

    let entries = fs::read_dir(directory).map_err(|error| CoreError::from_io(directory, &error))?;
    let mut children: Vec<PathBuf> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|error| CoreError::from_io(directory, &error))?;

        children.push(entry.path());
    }

    children.sort();

    for child in children {
        let relative = relative_to(root, &child)?;

        if child.is_dir() {
            found.push(ProjectFile::new(relative, ProjectFileKind::Directory));

            discover_in(root, &child, found)?;
        } else {
            found.push(ProjectFile::new(relative, ProjectFileKind::File));
        }
    }

    Ok(())
}

/// Decide la plataforma de un proyecto a partir de su archivo y su contenido.
///
/// La combinacion se pide a `ProjectType::from_parts` para que la regla de que
/// combinaciones estan soportadas viva en el core y no aqui.
fn detect_project_type(project_file: &Path, content: &str) -> CoreResult<ProjectType> {
    let file_name = project_file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_lowercase();

    if file_name.ends_with(".csproj") {
        if !content.contains("UseWindowsForms") {
            return Err(CoreError::Unsupported(format!(
                "{file_name} no declara que use Windows Forms"
            )));
        }

        return ProjectType::from_parts(LanguageId::CSharp, FrameworkId::WinForms);
    }

    if file_name == "pom.xml" {
        return ProjectType::from_parts(LanguageId::Java, FrameworkId::Swing);
    }

    Err(CoreError::Unsupported(format!(
        "{file_name} no es un archivo de proyecto conocido"
    )))
}

/// Nombre nuevo de una entrada, como ruta relativa de un solo componente.
/// `ProjectRelativePath` ya rechaza vacio, `..` y absolutos; queda rechazar los
/// nombres con separadores, que serian en realidad rutas.
fn renamed_name(new_name: &str) -> CoreResult<ProjectRelativePath> {
    let renamed = ProjectRelativePath::new(new_name)?;

    if renamed.as_path().components().count() != 1 {
        return Err(CoreError::InvalidName(new_name.to_string()));
    }

    Ok(renamed)
}

/// `renamed` puesto en el mismo directorio que `path`.
fn sibling_of(
    path: &ProjectRelativePath,
    renamed: &ProjectRelativePath,
) -> CoreResult<ProjectRelativePath> {
    let mut sibling = path
        .as_path()
        .parent()
        .unwrap_or(Path::new(""))
        .to_path_buf();

    sibling.push(renamed.as_path());

    ProjectRelativePath::new(sibling)
}

/// Ruta de `path` relativa a la raiz del proyecto.
///
/// `path` siempre viene de leer un directorio de la raiz, asi que quitarle el
/// prefijo funciona. Si alguna vez no, se cae en la ruta absoluta, que
/// `ProjectRelativePath` rechaza: falla en vez de inventar una ruta.
fn relative_to(root: &Path, path: &Path) -> CoreResult<ProjectRelativePath> {
    let relative = path.strip_prefix(root).unwrap_or(path);

    ProjectRelativePath::new(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_build_configuration() -> BuildConfiguration {
        BuildConfiguration::new(
            ProjectRelativePath::new(BuildConfiguration::DEFAULT_OUTPUT_DIRECTORY).unwrap(),
        )
    }

    #[test]
    fn project_file_keeps_its_relative_path() {
        let path = ProjectRelativePath::new("src/forms/MainForm.cs").unwrap();
        let file = ProjectFile::new(path.clone(), ProjectFileKind::File);

        assert_eq!(file.path(), &path);
    }

    #[test]
    fn project_file_distinguishes_files_from_directories() {
        let path = ProjectRelativePath::new("src/forms").unwrap();

        let file = ProjectFile::new(path.clone(), ProjectFileKind::File);
        let directory = ProjectFile::new(path, ProjectFileKind::Directory);

        assert_eq!(file.kind(), ProjectFileKind::File);
        assert_eq!(directory.kind(), ProjectFileKind::Directory);
    }

    #[test]
    fn project_file_identity_is_path_plus_kind() {
        let path = ProjectRelativePath::new("src/main.rs").unwrap();

        let file = ProjectFile::new(path.clone(), ProjectFileKind::File);
        let same_file = ProjectFile::new(path.clone(), ProjectFileKind::File);
        let same_path_as_directory = ProjectFile::new(path, ProjectFileKind::Directory);

        assert_eq!(file, same_file);
        assert_ne!(file, same_path_as_directory);
    }

    #[test]
    fn project_stores_its_identity_and_location() {
        let project = Project::new(
            "Demo",
            Path::new("C:/projects/Demo"),
            ProjectType::JavaSwing,
            default_build_configuration(),
        )
        .unwrap();

        assert_eq!(project.name(), "Demo");
        assert_eq!(project.root(), Path::new("C:/projects/Demo"));
        assert_eq!(project.project_type(), ProjectType::JavaSwing);
    }

    #[test]
    fn project_exposes_the_language_and_framework_of_its_type() {
        let project = Project::new(
            "Demo",
            Path::new("C:/projects/Demo"),
            ProjectType::CSharpWinForms,
            default_build_configuration(),
        )
        .unwrap();

        assert_eq!(project.language(), LanguageId::CSharp);
        assert_eq!(project.framework(), FrameworkId::WinForms);
    }

    #[test]
    fn project_stores_the_files_added_to_it() {
        let mut project = Project::new(
            "Demo",
            Path::new("C:/projects/Demo"),
            ProjectType::JavaSwing,
            default_build_configuration(),
        )
        .unwrap();

        project.add_file(ProjectFile::new(
            ProjectRelativePath::new("src").unwrap(),
            ProjectFileKind::Directory,
        ));
        project.add_file(ProjectFile::new(
            ProjectRelativePath::new("src/Main.java").unwrap(),
            ProjectFileKind::File,
        ));

        let paths: Vec<&Path> = project
            .files()
            .iter()
            .map(|file| file.path().as_path())
            .collect();

        assert_eq!(paths, vec![Path::new("src"), Path::new("src/Main.java")]);
    }

    #[test]
    fn build_configuration_exposes_its_output_directory() {
        let output_directory = ProjectRelativePath::new("out").unwrap();
        let configuration = BuildConfiguration::new(output_directory.clone());

        assert_eq!(configuration.output_directory(), &output_directory);
    }

    #[test]
    fn build_configuration_starts_without_extra_arguments() {
        let output_directory = ProjectRelativePath::new("out").unwrap();
        let configuration = BuildConfiguration::new(output_directory);

        assert!(configuration.arguments().is_empty());
    }

    #[test]
    fn build_configuration_collects_extra_arguments_in_order() {
        let output_directory = ProjectRelativePath::new("out").unwrap();
        let mut configuration = BuildConfiguration::new(output_directory);

        configuration.add_argument("-p:Platform=AnyCPU");
        configuration.add_argument(String::from("--release"));

        assert_eq!(
            configuration.arguments(),
            ["-p:Platform=AnyCPU".to_string(), "--release".to_string()]
        );
    }

    #[test]
    fn project_exposes_its_build_configuration() {
        let build_configuration = BuildConfiguration::new(ProjectRelativePath::new("bin").unwrap());
        let project = Project::new(
            "Demo",
            Path::new("C:/projects/Demo"),
            ProjectType::CSharpWinForms,
            build_configuration.clone(),
        )
        .unwrap();

        assert_eq!(project.build_configuration(), &build_configuration);
    }

    #[test]
    fn project_rejects_an_empty_name() {
        let result = Project::new(
            "",
            Path::new("C:/projects/Demo"),
            ProjectType::JavaSwing,
            default_build_configuration(),
        );

        assert_eq!(result, Err(CoreError::InvalidName(String::new())));
    }

    #[test]
    fn project_rejects_a_name_surrounded_by_whitespace() {
        let result = Project::new(
            " Demo ",
            Path::new("C:/projects/Demo"),
            ProjectType::JavaSwing,
            default_build_configuration(),
        );

        assert_eq!(result, Err(CoreError::InvalidName(" Demo ".to_string())));
    }

    #[test]
    fn project_rejects_a_name_with_reserved_characters() {
        let result = Project::new(
            "De/mo",
            Path::new("C:/projects/Demo"),
            ProjectType::JavaSwing,
            default_build_configuration(),
        );

        assert_eq!(result, Err(CoreError::InvalidName("De/mo".to_string())));
    }

    #[test]
    fn a_name_made_only_of_whitespace_is_rejected() {
        let result = Project::new(
            "   ",
            Path::new("C:/projects/Demo"),
            ProjectType::JavaSwing,
            default_build_configuration(),
        );

        assert_eq!(result, Err(CoreError::InvalidName("   ".to_string())));
    }

    #[test]
    fn every_reserved_character_is_rejected_in_a_name() {
        let reserved = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

        for character in reserved {
            let name = format!("De{character}mo");
            let result = Project::new(
                name.clone(),
                Path::new("C:/projects/Demo"),
                ProjectType::JavaSwing,
                default_build_configuration(),
            );

            assert_eq!(result, Err(CoreError::InvalidName(name)));
        }

        for character in ['\n', '\u{0}'] {
            let name = format!("De{character}mo");
            let result = Project::new(
                name.clone(),
                Path::new("C:/projects/Demo"),
                ProjectType::JavaSwing,
                default_build_configuration(),
            );

            assert_eq!(result, Err(CoreError::InvalidName(name)));
        }
    }

    #[test]
    fn project_rejects_an_empty_root() {
        let result = Project::new(
            "Demo",
            Path::new(""),
            ProjectType::JavaSwing,
            default_build_configuration(),
        );

        assert_eq!(result, Err(CoreError::InvalidPath(String::new())));
    }

    #[test]
    fn project_rejects_a_relative_root() {
        let result = Project::new(
            "Demo",
            Path::new("projects/Demo"),
            ProjectType::JavaSwing,
            default_build_configuration(),
        );

        assert_eq!(
            result,
            Err(CoreError::InvalidPath("projects/Demo".to_string()))
        );
    }

    #[test]
    fn accepts_a_nested_relative_path() {
        let path = ProjectRelativePath::new("src/forms/MainForm.cs").unwrap();

        assert_eq!(path.as_path(), Path::new("src/forms/MainForm.cs"));
    }

    #[test]
    fn rejects_empty_path() {
        let result = ProjectRelativePath::new("");

        assert_eq!(result, Err(CoreError::InvalidPath(String::new())));
    }

    #[test]
    fn rejects_absolute_path() {
        let result = ProjectRelativePath::new("C:/projects/demo");

        assert!(matches!(result, Err(CoreError::InvalidPath(_))));
    }

    #[test]
    fn rejects_parent_directory_escape() {
        let result = ProjectRelativePath::new("../outside.txt");

        assert!(matches!(result, Err(CoreError::InvalidPath(_))));
    }

    #[test]
    fn rejects_current_directory_components() {
        let result = ProjectRelativePath::new("./src/main.rs");

        assert!(matches!(result, Err(CoreError::InvalidPath(_))));
    }
}
