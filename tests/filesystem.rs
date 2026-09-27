//! Descubrimiento, creacion, renombrado, borrado, apertura de proyecto y
//! guardado de documentos sobre el sistema de archivos real.
//!
//! Cada test trabaja en su propio directorio temporal bajo el del sistema y lo
//! borra al terminar, sin tocar nada fuera de ahi.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use miniide::core::{CoreError, FrameworkId, LanguageId, ProjectType};
use miniide::document::TextPosition;
use miniide::editor::{Document, DocumentPath, Tab};
use miniide::project::{
    BuildConfiguration, Project, ProjectFile, ProjectFileKind, ProjectRelativePath,
};
use miniide::templates::create_project;
use miniide::toolchain::DotNetSdk;
use miniide::workspace::Workspace;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Directorio temporal exclusivo de este test, ya creado.
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(name: &str) -> Self {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "miniide-t030-{name}-{}-{unique}",
            std::process::id()
        ));

        fs::create_dir_all(&root).expect("temporary directory");

        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    /// Crea un archivo con su contenido, creando los directorios intermedios.
    fn file(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent directory");
        }

        fs::write(path, content).expect("file written");
    }

    /// Crea un directorio, con sus padres.
    fn dir(&self, relative: &str) {
        fs::create_dir_all(self.root.join(relative)).expect("directory created");
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn project_at(root: &Path) -> Project {
    Project::new(
        "app",
        root,
        ProjectType::CSharpWinForms,
        BuildConfiguration::new(ProjectRelativePath::new("bin").unwrap()),
    )
    .expect("valid project")
}

fn relative_paths(project: &Project) -> Vec<String> {
    project
        .discover_files()
        .expect("files discovered")
        .iter()
        .map(|file| {
            file.path()
                .as_path()
                .to_str()
                .expect("utf-8 path")
                .replace('\\', "/")
        })
        .collect()
}

fn relative(value: &str) -> ProjectRelativePath {
    ProjectRelativePath::new(value).expect("path inside the project")
}

fn model_paths(project: &Project) -> Vec<String> {
    project
        .files()
        .iter()
        .map(|file| {
            file.path()
                .as_path()
                .to_str()
                .expect("utf-8 path")
                .replace('\\', "/")
        })
        .collect()
}

fn kind_of(project: &Project, relative: &str) -> ProjectFileKind {
    project
        .discover_files()
        .expect("files discovered")
        .into_iter()
        .find(|file| {
            file.path()
                .as_path()
                .to_str()
                .expect("utf-8 path")
                .replace('\\', "/")
                == relative
        })
        .unwrap_or_else(|| panic!("{relative} should have been discovered"))
        .kind()
}

#[test]
fn an_empty_directory_has_no_files() {
    let tree = TempTree::new("empty");
    let project = project_at(tree.root());

    assert_eq!(project.discover_files().unwrap(), vec![]);
}

#[test]
fn the_files_of_the_root_are_discovered() {
    let tree = TempTree::new("root-files");
    tree.file("app.csproj", "<Project />");
    tree.file("Program.cs", "class Program {}");
    let project = project_at(tree.root());

    let found = relative_paths(&project);

    assert!(found.contains(&"app.csproj".to_string()), "{found:?}");
    assert!(found.contains(&"Program.cs".to_string()), "{found:?}");
}

#[test]
fn the_directories_of_the_root_are_discovered() {
    let tree = TempTree::new("root-dirs");
    tree.dir("bin");
    tree.dir("obj");
    let project = project_at(tree.root());

    assert_eq!(kind_of(&project, "bin"), ProjectFileKind::Directory);
    assert_eq!(kind_of(&project, "obj"), ProjectFileKind::Directory);
}

#[test]
fn the_files_inside_the_directories_are_discovered() {
    let tree = TempTree::new("nested");
    tree.file("app.csproj", "<Project />");
    tree.file("src/Program.cs", "class Program {}");
    tree.file("src/forms/MainForm.cs", "class MainForm {}");
    let project = project_at(tree.root());

    let found = relative_paths(&project);

    assert!(found.contains(&"src/Program.cs".to_string()), "{found:?}");
    assert!(
        found.contains(&"src/forms/MainForm.cs".to_string()),
        "{found:?}"
    );
}

#[test]
fn the_directories_inside_the_directories_are_discovered() {
    let tree = TempTree::new("nested-dirs");
    tree.dir("src/forms");
    tree.file("src/forms/MainForm.cs", "class MainForm {}");
    let project = project_at(tree.root());

    assert_eq!(kind_of(&project, "src"), ProjectFileKind::Directory);
    assert_eq!(kind_of(&project, "src/forms"), ProjectFileKind::Directory);
    assert_eq!(
        kind_of(&project, "src/forms/MainForm.cs"),
        ProjectFileKind::File
    );
}

#[test]
fn a_directory_comes_before_the_files_it_contains() {
    let tree = TempTree::new("order");
    tree.file("src/Program.cs", "class Program {}");
    tree.dir("src");
    let project = project_at(tree.root());

    let found = relative_paths(&project);

    assert_eq!(found, vec!["src", "src/Program.cs"]);
}

#[test]
fn the_root_itself_is_not_discovered() {
    let tree = TempTree::new("no-root");
    tree.file("app.csproj", "<Project />");
    let project = project_at(tree.root());

    let found = relative_paths(&project);

    assert!(!found.iter().any(|path| path.is_empty()), "{found:?}");
    assert_eq!(found, vec!["app.csproj"]);
}

#[test]
fn discovering_files_twice_gives_the_same_result() {
    let tree = TempTree::new("twice");
    tree.file("app.csproj", "<Project />");
    tree.file("src/Program.cs", "class Program {}");
    let project = project_at(tree.root());

    assert_eq!(relative_paths(&project), relative_paths(&project));
}

#[test]
fn discovering_files_does_not_change_the_project() {
    let tree = TempTree::new("no-change");
    tree.file("app.csproj", "<Project />");
    let project = project_at(tree.root());

    project.discover_files().unwrap();

    assert_eq!(project.files(), &[]);
}

#[test]
fn a_root_that_does_not_exist_is_reported_as_not_found() {
    let tree = TempTree::new("missing");
    let missing = tree.root().join("no-existe");
    let project = project_at(&missing);

    let result = project.discover_files();

    assert!(matches!(result, Err(CoreError::NotFound(_))));
}

#[test]
fn a_root_that_is_a_file_is_rejected() {
    let tree = TempTree::new("root-is-file");
    tree.file("app.csproj", "<Project />");
    let project = project_at(&tree.root().join("app.csproj"));

    let result = project.discover_files();

    assert!(matches!(result, Err(CoreError::InvalidPath(_))));
}

#[test]
fn a_file_is_created_in_the_project_root() {
    let tree = TempTree::new("create-root");
    let mut project = project_at(tree.root());

    project
        .create_file(relative("app.csproj"), "<Project />")
        .unwrap();

    assert_eq!(
        fs::read_to_string(tree.root().join("app.csproj")).unwrap(),
        "<Project />"
    );
    assert_eq!(model_paths(&project), vec!["app.csproj"]);
    assert_eq!(project.files()[0].kind(), ProjectFileKind::File);
}

#[test]
fn a_file_is_created_inside_an_existing_directory() {
    let tree = TempTree::new("create-nested");
    tree.dir("src");
    let mut project = project_at(tree.root());

    project
        .create_file(relative("src/Program.cs"), "class Program {}")
        .unwrap();

    assert_eq!(
        fs::read_to_string(tree.root().join("src").join("Program.cs")).unwrap(),
        "class Program {}"
    );
    assert_eq!(model_paths(&project), vec!["src/Program.cs"]);
}

#[test]
fn an_empty_file_can_be_created() {
    let tree = TempTree::new("create-empty");
    let mut project = project_at(tree.root());

    project.create_file(relative("vacio.txt"), "").unwrap();

    assert_eq!(
        fs::read_to_string(tree.root().join("vacio.txt")).unwrap(),
        ""
    );
    assert_eq!(model_paths(&project), vec!["vacio.txt"]);
}

#[test]
fn the_content_written_is_the_content_given() {
    let tree = TempTree::new("create-content");
    let mut project = project_at(tree.root());
    let content = "using System;\r\n\r\nclass MainForm {}\n";

    project
        .create_file(relative("MainForm.cs"), content)
        .unwrap();

    assert_eq!(
        fs::read_to_string(tree.root().join("MainForm.cs")).unwrap(),
        content
    );
}

#[test]
fn the_model_keeps_every_created_file() {
    let tree = TempTree::new("create-many");
    let mut project = project_at(tree.root());

    project.create_file(relative("app.csproj"), "").unwrap();
    project.create_file(relative("nombres.txt"), "").unwrap();

    assert_eq!(model_paths(&project), vec!["app.csproj", "nombres.txt"]);
}

#[test]
fn a_created_file_is_discovered_afterwards() {
    let tree = TempTree::new("create-then-discover");
    let mut project = project_at(tree.root());

    project.create_file(relative("app.csproj"), "").unwrap();

    assert_eq!(relative_paths(&project), vec!["app.csproj"]);
}

#[test]
fn a_file_cannot_be_created_over_an_existing_one() {
    let tree = TempTree::new("create-existing");
    tree.file("app.csproj", "<Project />");
    let mut project = project_at(tree.root());

    let result = project.create_file(relative("app.csproj"), "otro contenido");

    assert!(matches!(result, Err(CoreError::AlreadyExists(_))));
    assert_eq!(
        fs::read_to_string(tree.root().join("app.csproj")).unwrap(),
        "<Project />"
    );
}

#[test]
fn a_file_cannot_be_created_when_its_directory_does_not_exist() {
    let tree = TempTree::new("create-missing-dir");
    let mut project = project_at(tree.root());

    let result = project.create_file(relative("src/Program.cs"), "class Program {}");

    assert!(matches!(result, Err(CoreError::NotFound(_))));
}

#[test]
fn the_model_is_not_changed_when_the_creation_fails() {
    let tree = TempTree::new("create-failed-model");
    tree.file("app.csproj", "<Project />");
    let mut project = project_at(tree.root());

    let _ = project.create_file(relative("app.csproj"), "otro");
    let _ = project.create_file(relative("src/Program.cs"), "class Program {}");

    assert_eq!(project.files(), &[]);
}

#[test]
fn a_directory_is_created_in_the_project_root() {
    let tree = TempTree::new("mkdir-root");
    let mut project = project_at(tree.root());

    project.create_directory(relative("bin")).unwrap();

    assert!(tree.root().join("bin").is_dir());
    assert_eq!(model_paths(&project), vec!["bin"]);
    assert_eq!(project.files()[0].kind(), ProjectFileKind::Directory);
}

#[test]
fn a_directory_is_created_inside_an_existing_directory() {
    let tree = TempTree::new("mkdir-nested");
    let mut project = project_at(tree.root());
    project.create_directory(relative("src")).unwrap();

    project.create_directory(relative("src/forms")).unwrap();

    assert!(tree.root().join("src").join("forms").is_dir());
    assert_eq!(model_paths(&project), vec!["src", "src/forms"]);
}

#[test]
fn the_intermediate_directories_of_a_directory_are_created() {
    let tree = TempTree::new("mkdir-intermediate");
    let mut project = project_at(tree.root());

    project
        .create_directory(relative("src/forms/controls"))
        .unwrap();

    assert!(tree.root().join("src").is_dir());
    assert!(tree.root().join("src").join("forms").is_dir());
    assert!(tree
        .root()
        .join("src")
        .join("forms")
        .join("controls")
        .is_dir());
}

#[test]
fn a_created_directory_is_discovered_afterwards() {
    let tree = TempTree::new("mkdir-then-discover");
    let mut project = project_at(tree.root());

    project.create_directory(relative("bin")).unwrap();

    assert_eq!(relative_paths(&project), vec!["bin"]);
}

#[test]
fn a_created_directory_can_hold_a_file_afterwards() {
    let tree = TempTree::new("mkdir-then-file");
    let mut project = project_at(tree.root());
    project.create_directory(relative("src")).unwrap();

    project
        .create_file(relative("src/Program.cs"), "class Program {}")
        .unwrap();

    assert_eq!(
        fs::read_to_string(tree.root().join("src").join("Program.cs")).unwrap(),
        "class Program {}"
    );
    assert_eq!(model_paths(&project), vec!["src", "src/Program.cs"]);
}

#[test]
fn a_directory_cannot_be_created_over_an_existing_directory() {
    let tree = TempTree::new("mkdir-existing");
    let mut project = project_at(tree.root());
    project.create_directory(relative("bin")).unwrap();

    let result = project.create_directory(relative("bin"));

    assert!(matches!(result, Err(CoreError::AlreadyExists(_))));
}

#[test]
fn a_directory_cannot_be_created_over_an_existing_file() {
    let tree = TempTree::new("mkdir-over-file");
    tree.file("app.csproj", "<Project />");
    let mut project = project_at(tree.root());

    let result = project.create_directory(relative("app.csproj"));

    assert!(matches!(result, Err(CoreError::AlreadyExists(_))));
    assert!(tree.root().join("app.csproj").is_file());
}

#[test]
fn the_model_is_not_changed_when_the_directory_creation_fails() {
    let tree = TempTree::new("mkdir-failed-model");
    tree.file("app.csproj", "<Project />");
    let mut project = project_at(tree.root());

    let result = project.create_directory(relative("app.csproj"));

    assert!(matches!(result, Err(CoreError::AlreadyExists(_))));
    assert_eq!(project.files(), &[]);
}

#[test]
fn a_file_is_renamed_on_disk() {
    let tree = TempTree::new("rename-file");
    tree.file("viejo.txt", "contenido");
    let mut project = project_at(tree.root());

    project.rename(relative("viejo.txt"), "nuevo.txt").unwrap();

    assert!(!tree.root().join("viejo.txt").exists());
    assert_eq!(
        fs::read_to_string(tree.root().join("nuevo.txt")).unwrap(),
        "contenido"
    );
}

#[test]
fn a_file_is_renamed_inside_its_own_directory() {
    let tree = TempTree::new("rename-in-dir");
    tree.file("src/viejo.txt", "contenido");
    let mut project = project_at(tree.root());

    project
        .rename(relative("src/viejo.txt"), "nuevo.txt")
        .unwrap();

    assert!(tree.root().join("src").join("nuevo.txt").is_file());
    assert!(!tree.root().join("src").join("viejo.txt").exists());
}

#[test]
fn the_model_reflects_the_new_path_of_a_renamed_file() {
    let tree = TempTree::new("rename-model");
    let mut project = project_at(tree.root());
    project
        .create_file(relative("viejo.txt"), "contenido")
        .unwrap();

    project.rename(relative("viejo.txt"), "nuevo.txt").unwrap();

    assert_eq!(model_paths(&project), vec!["nuevo.txt"]);
    assert_eq!(project.files()[0].kind(), ProjectFileKind::File);
}

#[test]
fn a_renamed_directory_keeps_its_kind_in_the_model() {
    let tree = TempTree::new("rename-dir-kind");
    let mut project = project_at(tree.root());
    project.create_directory(relative("src")).unwrap();

    project.rename(relative("src"), "app").unwrap();

    assert_eq!(project.files()[0].kind(), ProjectFileKind::Directory);
    assert!(tree.root().join("app").is_dir());
    assert!(!tree.root().join("src").exists());
}

#[test]
fn the_paths_inside_a_renamed_directory_follow_in_the_model() {
    let tree = TempTree::new("rename-dir-model");
    let mut project = project_at(tree.root());
    project.create_directory(relative("src")).unwrap();
    project
        .create_file(relative("src/Program.cs"), "class Program {}")
        .unwrap();

    project.rename(relative("src"), "app").unwrap();

    assert_eq!(model_paths(&project), vec!["app", "app/Program.cs"]);
}

#[test]
fn the_files_of_a_renamed_directory_are_renamed_on_disk() {
    let tree = TempTree::new("rename-dir-disk");
    let mut project = project_at(tree.root());
    project.create_directory(relative("src")).unwrap();
    project
        .create_file(relative("src/Program.cs"), "class Program {}")
        .unwrap();

    project.rename(relative("src"), "app").unwrap();

    assert!(tree.root().join("app").join("Program.cs").is_file());
    assert!(!tree.root().join("src").exists());
}

#[test]
fn a_renamed_directory_is_discovered_afterwards() {
    let tree = TempTree::new("rename-then-discover");
    let mut project = project_at(tree.root());
    project.create_directory(relative("src")).unwrap();
    project
        .create_file(relative("src/Program.cs"), "class Program {}")
        .unwrap();

    project.rename(relative("src"), "app").unwrap();

    assert_eq!(relative_paths(&project), vec!["app", "app/Program.cs"]);
}

#[test]
fn a_new_name_with_a_separator_is_rejected() {
    let tree = TempTree::new("rename-separator");
    tree.file("viejo.txt", "contenido");
    let mut project = project_at(tree.root());

    let result = project.rename(relative("viejo.txt"), "carpeta/nuevo.txt");

    assert!(matches!(result, Err(CoreError::InvalidName(_))));
    assert!(tree.root().join("viejo.txt").is_file());
    assert!(!tree.root().join("carpeta").exists());
}

#[test]
fn an_empty_new_name_is_rejected() {
    let tree = TempTree::new("rename-empty");
    tree.file("viejo.txt", "contenido");
    let mut project = project_at(tree.root());

    let result = project.rename(relative("viejo.txt"), "");

    assert!(matches!(result, Err(CoreError::InvalidPath(_))));
    assert!(tree.root().join("viejo.txt").is_file());
}

#[test]
fn a_new_name_cannot_be_a_parent_directory() {
    let tree = TempTree::new("rename-parent");
    tree.file("viejo.txt", "contenido");
    let mut project = project_at(tree.root());

    let result = project.rename(relative("viejo.txt"), "..");

    assert!(matches!(result, Err(CoreError::InvalidPath(_))));
    assert!(tree.root().join("viejo.txt").is_file());
}

#[test]
fn a_file_cannot_be_renamed_over_an_existing_file() {
    let tree = TempTree::new("rename-over");
    tree.file("viejo.txt", "propio");
    tree.file("nuevo.txt", "ajeno");
    let mut project = project_at(tree.root());

    let result = project.rename(relative("viejo.txt"), "nuevo.txt");

    assert!(matches!(result, Err(CoreError::AlreadyExists(_))));
    assert_eq!(
        fs::read_to_string(tree.root().join("nuevo.txt")).unwrap(),
        "ajeno"
    );
}

#[test]
fn renaming_a_path_that_does_not_exist_is_reported() {
    let tree = TempTree::new("rename-missing");
    let mut project = project_at(tree.root());

    let result = project.rename(relative("no-existe.txt"), "nuevo.txt");

    assert!(matches!(result, Err(CoreError::NotFound(_))));
}

#[test]
fn the_model_is_not_changed_when_the_rename_fails() {
    let tree = TempTree::new("rename-failed-model");
    tree.file("viejo.txt", "propio");
    tree.file("nuevo.txt", "ajeno");
    let mut project = project_at(tree.root());
    project.add_file(ProjectFile::new(
        relative("viejo.txt"),
        ProjectFileKind::File,
    ));

    let result = project.rename(relative("viejo.txt"), "nuevo.txt");

    assert!(matches!(result, Err(CoreError::AlreadyExists(_))));
    assert_eq!(model_paths(&project), vec!["viejo.txt"]);
}

#[test]
fn a_file_is_removed_from_disk() {
    let tree = TempTree::new("remove-file");
    tree.file("viejo.txt", "contenido");
    let mut project = project_at(tree.root());

    project.remove(relative("viejo.txt")).unwrap();

    assert!(!tree.root().join("viejo.txt").exists());
}

const WINFORMS_CSPROJ: &str = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>WinExe</OutputType>
    <UseWindowsForms>true</UseWindowsForms>
    <TargetFramework>net8.0-windows</TargetFramework>
  </PropertyGroup>
</Project>
"#;

#[test]
fn a_csharp_winforms_project_is_opened() {
    let tree = TempTree::new("open-cs");
    tree.file("app/app.csproj", WINFORMS_CSPROJ);

    let project = Project::open(&tree.root().join("app").join("app.csproj")).unwrap();

    assert_eq!(project.name(), "app");
    assert_eq!(project.root(), tree.root().join("app"));
    assert_eq!(project.project_type(), ProjectType::CSharpWinForms);
    assert_eq!(project.language(), LanguageId::CSharp);
    assert_eq!(project.framework(), FrameworkId::WinForms);
}

#[test]
fn a_java_swing_project_is_opened() {
    let tree = TempTree::new("open-java");
    tree.file(
        "app/pom.xml",
        "<project><artifactId>app</artifactId></project>",
    );

    let project = Project::open(&tree.root().join("app").join("pom.xml")).unwrap();

    assert_eq!(project.name(), "app");
    assert_eq!(project.root(), tree.root().join("app"));
    assert_eq!(project.project_type(), ProjectType::JavaSwing);
    assert_eq!(project.language(), LanguageId::Java);
    assert_eq!(project.framework(), FrameworkId::Swing);
}

#[test]
fn the_project_name_is_the_directory_and_not_the_project_file() {
    let tree = TempTree::new("open-name");
    tree.file("InterfazPrincipal/pom.xml", "<project />");

    let project = Project::open(&tree.root().join("InterfazPrincipal").join("pom.xml")).unwrap();

    assert_eq!(project.name(), "InterfazPrincipal");
}

#[test]
fn a_wpf_project_is_not_mistaken_for_winforms() {
    let tree = TempTree::new("open-wpf");
    tree.file(
        "app/app.csproj",
        "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><UseWPF>true</UseWPF></PropertyGroup></Project>",
    );

    let result = Project::open(&tree.root().join("app").join("app.csproj"));

    assert!(matches!(result, Err(CoreError::Unsupported(_))));
}

#[test]
fn a_csharp_project_without_the_windows_forms_marker_is_not_supported() {
    let tree = TempTree::new("open-cs-plain");
    tree.file(
        "app/app.csproj",
        "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup></Project>",
    );

    let result = Project::open(&tree.root().join("app").join("app.csproj"));

    assert!(matches!(result, Err(CoreError::Unsupported(_))));
}

#[test]
fn a_file_that_is_not_a_project_file_is_not_supported() {
    let tree = TempTree::new("open-not-project");
    tree.file("notas.txt", "esto no es un proyecto");

    let result = Project::open(&tree.root().join("notas.txt"));

    assert!(matches!(result, Err(CoreError::Unsupported(_))));
}

#[test]
fn a_project_file_that_does_not_exist_is_reported() {
    let tree = TempTree::new("open-missing");

    let result = Project::open(&tree.root().join("no-existe.csproj"));

    assert!(matches!(result, Err(CoreError::NotFound(_))));
}

#[test]
fn a_directory_cannot_be_opened_as_a_project() {
    let tree = TempTree::new("open-dir");
    tree.dir("app");

    let result = Project::open(&tree.root().join("app"));

    assert!(matches!(result, Err(CoreError::InvalidPath(_))));
}

#[test]
fn an_opened_project_can_enumerate_its_files() {
    let tree = TempTree::new("open-discover");
    tree.file("app/app.csproj", WINFORMS_CSPROJ);
    tree.file("app/src/Program.cs", "class Program {}");

    let project = Project::open(&tree.root().join("app").join("app.csproj")).unwrap();

    assert_eq!(
        relative_paths(&project),
        vec!["app.csproj", "src", "src/Program.cs"]
    );
}

#[test]
fn the_workspace_activates_an_opened_project() {
    let tree = TempTree::new("open-activate");
    tree.file("app/app.csproj", WINFORMS_CSPROJ);
    let mut workspace = Workspace::new(tree.root()).unwrap();

    workspace
        .set_active_project(Project::open(&tree.root().join("app").join("app.csproj")).unwrap());

    assert_eq!(workspace.active_project().unwrap().name(), "app");
    assert_eq!(
        workspace.active_project().unwrap().root(),
        tree.root().join("app")
    );
}

#[test]
fn closing_the_project_leaves_the_workspace_with_its_root() {
    let tree = TempTree::new("open-close");
    tree.file("app/app.csproj", WINFORMS_CSPROJ);
    let mut workspace = Workspace::new(tree.root()).unwrap();
    workspace
        .set_active_project(Project::open(&tree.root().join("app").join("app.csproj")).unwrap());

    let closed = workspace.close_project();

    assert_eq!(closed.unwrap().name(), "app");
    assert_eq!(workspace.active_project(), None);
    assert_eq!(workspace.root(), tree.root());
}

#[test]
fn closing_a_workspace_without_a_project_gives_nothing() {
    let tree = TempTree::new("close-empty");
    let mut workspace = Workspace::new(tree.root()).unwrap();

    assert!(workspace.close_project().is_none());
    assert_eq!(workspace.active_project(), None);
}

/// Ruta de un archivo dentro del arbol temporal.
fn file_path(tree: &TempTree, relative: &str) -> DocumentPath {
    DocumentPath::new(tree.root().join(relative)).expect("document path")
}

#[test]
fn a_document_is_written_to_its_path() {
    let tree = TempTree::new("save-write");
    let mut tab = Tab::new(file_path(&tree, "notes.txt"), Document::new("uno"));
    tab.document_mut()
        .insert(TextPosition::new(0, 3), "!")
        .unwrap();

    tab.save().unwrap();

    assert_eq!(
        fs::read_to_string(tree.root().join("notes.txt")).unwrap(),
        "uno!"
    );
}

#[test]
fn the_saved_content_is_exactly_the_document_content() {
    let tree = TempTree::new("save-exact");
    let contenido = "using System;\r\n\r\nclass MainForm\r\n{\r\n}\n// sin cerrar\náéí\n";
    let mut tab = Tab::new(file_path(&tree, "MainForm.cs"), Document::new(contenido));

    tab.save().unwrap();

    let written = fs::read(tree.root().join("MainForm.cs")).unwrap();

    assert_eq!(written, contenido.as_bytes());
}

#[test]
fn saving_clears_the_modified_state() {
    let tree = TempTree::new("save-clears");
    let mut tab = Tab::new(file_path(&tree, "notes.txt"), Document::new("uno"));
    tab.document_mut()
        .insert(TextPosition::new(0, 3), "!")
        .unwrap();
    assert!(tab.is_modified());

    tab.save().unwrap();

    assert!(!tab.is_modified());
}

#[test]
fn a_document_that_was_not_modified_can_be_saved() {
    let tree = TempTree::new("save-unmodified");
    let mut tab = Tab::new(file_path(&tree, "notes.txt"), Document::new("uno"));

    tab.save().unwrap();

    assert_eq!(
        fs::read_to_string(tree.root().join("notes.txt")).unwrap(),
        "uno"
    );
    assert!(!tab.is_modified());
}

#[test]
fn a_new_document_is_created_by_saving_it() {
    let tree = TempTree::new("save-create");
    let mut tab = Tab::new(file_path(&tree, "nuevo.txt"), Document::new("hola"));

    tab.save().unwrap();

    assert_eq!(
        fs::read_to_string(tree.root().join("nuevo.txt")).unwrap(),
        "hola"
    );
}

#[test]
fn saving_twice_writes_the_latest_content() {
    let tree = TempTree::new("save-twice");
    let mut tab = Tab::new(file_path(&tree, "notes.txt"), Document::new("uno"));
    tab.save().unwrap();
    tab.document_mut()
        .insert(TextPosition::new(0, 3), "?")
        .unwrap();

    tab.save().unwrap();

    assert_eq!(
        fs::read_to_string(tree.root().join("notes.txt")).unwrap(),
        "uno?"
    );
    assert!(!tab.is_modified());
}

#[test]
fn undoing_back_to_the_saved_content_leaves_the_document_unmodified() {
    let tree = TempTree::new("save-undo");
    let mut tab = Tab::new(file_path(&tree, "notes.txt"), Document::new("uno"));
    tab.document_mut()
        .insert(TextPosition::new(0, 3), "!")
        .unwrap();
    tab.save().unwrap();
    tab.document_mut()
        .insert(TextPosition::new(0, 4), "?")
        .unwrap();
    assert!(tab.is_modified());

    assert!(tab.document_mut().undo());

    assert!(!tab.is_modified());
    assert_eq!(tab.document().buffer().text(), "uno!");
}

#[test]
fn a_failed_save_keeps_the_document_modified() {
    let tree = TempTree::new("save-failed");
    let mut tab = Tab::new(
        DocumentPath::new(tree.root().join("no-existe").join("notes.txt")).unwrap(),
        Document::new("uno"),
    );
    tab.document_mut()
        .insert(TextPosition::new(0, 3), "!")
        .unwrap();

    let result = tab.save();

    assert!(matches!(result, Err(CoreError::NotFound(_))));
    assert!(tab.is_modified());
    assert_eq!(tab.document().buffer().text(), "uno!");
}

#[test]
fn a_created_file_can_be_renamed_and_then_removed() {
    let tree = TempTree::new("lifecycle-entry");
    let mut project = project_at(tree.root());
    project
        .create_file(relative("viejo.txt"), "contenido")
        .unwrap();
    project.rename(relative("viejo.txt"), "nuevo.txt").unwrap();
    assert!(tree.root().join("nuevo.txt").is_file());

    project.remove(relative("nuevo.txt")).unwrap();

    assert!(!tree.root().join("nuevo.txt").exists());
    assert_eq!(project.files(), &[]);
    assert_eq!(relative_paths(&project), Vec::<String>::new());
}

#[test]
fn the_model_and_the_disk_agree_after_a_sequence_of_operations() {
    let tree = TempTree::new("lifecycle-agree");
    let mut project = project_at(tree.root());
    project.create_directory(relative("src")).unwrap();
    project.create_file(relative("app.csproj"), "").unwrap();
    project
        .create_file(relative("src/Program.cs"), "class Program {}")
        .unwrap();
    project.rename(relative("src"), "app").unwrap();

    let mut in_model = model_paths(&project);
    let mut on_disk = relative_paths(&project);
    in_model.sort();
    on_disk.sort();

    assert_eq!(in_model, on_disk);
    assert_eq!(on_disk, vec!["app", "app.csproj", "app/Program.cs"]);
}

#[test]
fn a_document_of_the_project_is_saved_inside_the_project() {
    let tree = TempTree::new("lifecycle-save");
    tree.file("app/app.csproj", WINFORMS_CSPROJ);
    tree.file("app/notes.txt", "uno");
    let project = Project::open(&tree.root().join("app").join("app.csproj")).unwrap();
    let mut tab = Tab::new(
        DocumentPath::new(project.root().join("notes.txt")).unwrap(),
        Document::new("uno"),
    );
    tab.document_mut()
        .insert(TextPosition::new(0, 3), "!")
        .unwrap();

    tab.save().unwrap();

    assert_eq!(
        fs::read_to_string(project.root().join("notes.txt")).unwrap(),
        "uno!"
    );
    assert!(!tab.is_modified());
    assert!(relative_paths(&project).contains(&"notes.txt".to_string()));
}

#[test]
fn a_project_can_be_opened_again_after_being_closed() {
    let tree = TempTree::new("lifecycle-reopen");
    tree.file("app/app.csproj", WINFORMS_CSPROJ);
    let mut workspace = Workspace::new(tree.root()).unwrap();
    let project_file = tree.root().join("app").join("app.csproj");

    workspace.set_active_project(Project::open(&project_file).unwrap());
    let closed = workspace.close_project().unwrap();

    workspace.set_active_project(Project::open(&project_file).unwrap());

    assert_eq!(closed.name(), "app");
    assert_eq!(workspace.active_project().unwrap().name(), "app");
}

#[test]
fn a_file_that_the_model_does_not_know_can_be_renamed() {
    let tree = TempTree::new("lifecycle-unknown");
    tree.file("ajeno.txt", "contenido");
    let mut project = project_at(tree.root());

    project.rename(relative("ajeno.txt"), "propio.txt").unwrap();

    assert!(tree.root().join("propio.txt").is_file());
    assert_eq!(project.files(), &[]);
    assert_eq!(relative_paths(&project), vec!["propio.txt"]);
}

#[test]
fn an_empty_directory_is_removed_from_disk() {
    let tree = TempTree::new("remove-dir");
    let mut project = project_at(tree.root());
    project.create_directory(relative("bin")).unwrap();

    project.remove(relative("bin")).unwrap();

    assert!(!tree.root().join("bin").exists());
    assert_eq!(project.files(), &[]);
}

#[test]
fn the_entries_inside_a_removed_directory_are_gone_from_the_model() {
    let tree = TempTree::new("remove-dir-model");
    let mut project = project_at(tree.root());
    project.create_directory(relative("src")).unwrap();
    project
        .create_file(relative("src/Program.cs"), "class Program {}")
        .unwrap();
    // El archivo desaparece por fuera, asi que el directorio queda vacio pero el
    // modelo sigue diciendo que esta ahi.
    fs::remove_file(tree.root().join("src").join("Program.cs")).unwrap();

    project.remove(relative("src")).unwrap();

    assert_eq!(project.files(), &[]);
}

#[test]
fn a_directory_with_files_in_it_is_not_removed() {
    let tree = TempTree::new("remove-dir-full");
    let mut project = project_at(tree.root());
    project.create_directory(relative("src")).unwrap();
    project
        .create_file(relative("src/Program.cs"), "class Program {}")
        .unwrap();

    let result = project.remove(relative("src"));

    assert!(matches!(result, Err(CoreError::Io(_))));
    assert!(tree.root().join("src").join("Program.cs").is_file());
    assert_eq!(model_paths(&project), vec!["src", "src/Program.cs"]);
}

#[test]
fn a_removed_file_is_not_discovered_afterwards() {
    let tree = TempTree::new("remove-then-discover");
    let mut project = project_at(tree.root());
    project
        .create_file(relative("viejo.txt"), "contenido")
        .unwrap();
    project.create_file(relative("otro.txt"), "otro").unwrap();

    project.remove(relative("viejo.txt")).unwrap();

    assert_eq!(relative_paths(&project), vec!["otro.txt"]);
}

#[test]
fn removing_a_path_that_does_not_exist_is_reported() {
    let tree = TempTree::new("remove-missing");
    let mut project = project_at(tree.root());

    let result = project.remove(relative("no-existe.txt"));

    assert!(matches!(result, Err(CoreError::NotFound(_))));
}

#[test]
fn the_model_is_not_changed_when_the_removal_fails() {
    let tree = TempTree::new("remove-failed-model");
    let mut project = project_at(tree.root());
    project
        .create_file(relative("viejo.txt"), "contenido")
        .unwrap();

    let result = project.remove(relative("no-existe.txt"));

    assert!(matches!(result, Err(CoreError::NotFound(_))));
    assert_eq!(model_paths(&project), vec!["viejo.txt"]);
}

#[test]
fn a_sibling_with_a_similar_name_is_kept_when_removing() {
    let tree = TempTree::new("remove-sibling");
    let mut project = project_at(tree.root());
    project
        .create_file(relative("Program.cs"), "class Program {}")
        .unwrap();
    project
        .create_file(relative("Program.cs.bak"), "copia")
        .unwrap();

    project.remove(relative("Program.cs")).unwrap();

    assert_eq!(model_paths(&project), vec!["Program.cs.bak"]);
    assert!(tree.root().join("Program.cs.bak").is_file());
}

#[test]
fn a_dotnet_executable_in_a_directory_is_detected() {
    let tree = TempTree::new("dotnet-detect");
    tree.file("dotnet.exe", "no es un ejecutable real");

    let found = DotNetSdk::detect_in(&[tree.root().to_path_buf()], DotNetSdk::EXTENSIONS);

    assert_eq!(found, Some(tree.root().join("dotnet.exe")));
}

#[test]
fn a_dotnet_executable_with_another_extension_is_also_detected() {
    let tree = TempTree::new("dotnet-detect-cmd");
    tree.file("dotnet.cmd", "no es un ejecutable real");

    let found = DotNetSdk::detect_in(&[tree.root().to_path_buf()], DotNetSdk::EXTENSIONS);

    assert_eq!(found, Some(tree.root().join("dotnet.cmd")));
}

#[test]
fn a_directory_without_the_dotnet_executable_does_not_have_the_sdk() {
    let tree = TempTree::new("dotnet-detect-none");
    tree.file("java.exe", "tampoco");

    let found = DotNetSdk::detect_in(&[tree.root().to_path_buf()], DotNetSdk::EXTENSIONS);

    assert_eq!(found, None);
}

#[test]
fn the_first_directory_with_the_dotnet_executable_wins() {
    let first = TempTree::new("dotnet-first");
    let second = TempTree::new("dotnet-second");
    first.file("dotnet.exe", "primero");
    second.file("dotnet.exe", "segundo");

    let found = DotNetSdk::detect_in(
        &[first.root().to_path_buf(), second.root().to_path_buf()],
        DotNetSdk::EXTENSIONS,
    );

    assert_eq!(found, Some(first.root().join("dotnet.exe")));
}

#[test]
#[ignore = "requires .NET SDK"]
fn the_dotnet_sdk_is_available_on_this_system() {
    let found = DotNetSdk::detect();

    assert!(
        found.is_some(),
        "no se ha encontrado dotnet.exe en el PATH de este sistema"
    );
}

#[test]
#[ignore = "requires .NET SDK"]
fn a_generated_csharp_project_compiles() {
    use miniide::build::build_with;
    use miniide::toolchain::DotNetToolchain;

    let tree = TempTree::new("build-csharp");
    let root = tree.root().join("App");
    let created = create_project(ProjectType::CSharpWinForms, &root).unwrap();

    let result = build_with(&DotNetToolchain, &created).unwrap();

    assert!(
        result.succeeded(),
        "la compilacion fallo ({}):\n{}\n{}",
        result.exit_code().unwrap_or(-1),
        result.standard_output(),
        result.standard_error()
    );
}

#[test]
#[ignore = "requires .NET SDK"]
fn a_compilation_error_becomes_a_diagnostic_with_its_position() {
    use miniide::build::build_with;
    use miniide::diagnostics::DiagnosticLevel;
    use miniide::toolchain::DotNetToolchain;

    let tree = TempTree::new("build-diagnostics");
    let root = tree.root().join("App");
    let created = create_project(ProjectType::CSharpWinForms, &root).unwrap();
    // Se rompe el formulario en una linea conocida.
    fs::write(
        root.join("Form1.cs"),
        "using System.Windows.Forms;\n\nnamespace App;\n\npublic partial class Form1 : Form\n{\n    public Form1()\n    {\n        this.noExisteNada();\n    }\n}\n",
    )
    .expect("broken form written");

    let result = build_with(&DotNetToolchain, &created).unwrap();

    assert!(!result.succeeded(), "el proyecto roto no deberia compilar");

    let error = result
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.level() == DiagnosticLevel::Error)
        .unwrap_or_else(|| {
            panic!(
                "no se parseo ningun error:\n{}\n{}",
                result.standard_output(),
                result.standard_error()
            )
        });

    let location = error
        .location()
        .expect("el error de compilacion tiene archivo y linea");

    assert_eq!(location.file().as_path(), Path::new("Form1.cs"));
    // `this.noExisteNada();` esta en la linea 9 de 1 en uno.
    assert_eq!(location.position().line(), 8);
}

fn project_directory(tree: &TempTree, name: &str) -> PathBuf {
    tree.root().join(name)
}

#[test]
fn a_new_csharp_project_creates_its_directory_and_its_files() {
    let tree = TempTree::new("template-files");
    let root = project_directory(&tree, "App");

    let project = create_project(ProjectType::CSharpWinForms, &root).unwrap();

    assert!(root.is_dir());
    assert_eq!(project.name(), "App");
    assert_eq!(project.root(), root);
    for file in ["App.csproj", "Program.cs", "Form1.cs", "Form1.Designer.cs"] {
        assert!(root.join(file).is_file(), "falta {file}");
    }
}

#[test]
fn the_created_project_declares_that_it_uses_windows_forms() {
    let tree = TempTree::new("template-marker");
    let root = project_directory(&tree, "App");
    create_project(ProjectType::CSharpWinForms, &root).unwrap();

    let csproj = fs::read_to_string(root.join("App.csproj")).unwrap();

    assert!(csproj.contains("UseWindowsForms"), "{csproj}");
}

#[test]
fn the_files_of_a_new_project_agree_on_its_name() {
    let tree = TempTree::new("template-name");
    let root = project_directory(&tree, "JuegoDePrueba");

    create_project(ProjectType::CSharpWinForms, &root).unwrap();

    for file in ["Program.cs", "Form1.cs", "Form1.Designer.cs"] {
        let content = fs::read_to_string(root.join(file)).unwrap();

        assert!(
            content.contains("namespace JuegoDePrueba;"),
            "{file} no lleva el namespace del proyecto: {content}"
        );
    }
    assert!(root.join("JuegoDePrueba.csproj").is_file());
}

#[test]
fn the_created_project_can_be_opened_again() {
    let tree = TempTree::new("template-reopen");
    let root = project_directory(&tree, "App");
    create_project(ProjectType::CSharpWinForms, &root).unwrap();

    let project = Project::open(&root.join("App.csproj")).unwrap();

    assert_eq!(project.name(), "App");
    assert_eq!(project.root(), root);
    assert_eq!(project.project_type(), ProjectType::CSharpWinForms);
}

#[test]
fn the_created_project_is_discovered_with_all_its_files() {
    let tree = TempTree::new("template-discover");
    let root = project_directory(&tree, "App");
    let project = create_project(ProjectType::CSharpWinForms, &root).unwrap();

    let mut found = relative_paths(&project);
    found.sort();

    assert_eq!(
        found,
        vec!["App.csproj", "Form1.Designer.cs", "Form1.cs", "Program.cs"]
    );
}

#[test]
fn a_directory_that_is_not_empty_is_not_used_for_a_new_project() {
    let tree = TempTree::new("template-not-empty");
    let root = project_directory(&tree, "App");
    tree.file("App/ya-existe.txt", "contenido");

    let result = create_project(ProjectType::CSharpWinForms, &root);

    assert!(matches!(result, Err(CoreError::AlreadyExists(_))));
    assert_eq!(
        fs::read_to_string(root.join("ya-existe.txt")).unwrap(),
        "contenido"
    );
    assert!(!root.join("App.csproj").exists());
}

#[test]
fn a_root_that_is_a_file_is_not_used_for_a_new_project() {
    let tree = TempTree::new("template-root-file");
    let root = project_directory(&tree, "App");
    tree.file("App", "esto es un archivo");

    let result = create_project(ProjectType::CSharpWinForms, &root);

    assert!(matches!(result, Err(CoreError::InvalidPath(_))));
    assert_eq!(fs::read_to_string(&root).unwrap(), "esto es un archivo");
}

#[test]
fn a_relative_root_is_not_used_for_a_new_project() {
    let result = create_project(ProjectType::CSharpWinForms, Path::new("App"));

    assert!(matches!(result, Err(CoreError::InvalidPath(_))));
}

#[test]
fn a_java_project_cannot_be_created_yet() {
    let tree = TempTree::new("template-java");
    let root = project_directory(&tree, "App");

    let result = create_project(ProjectType::JavaSwing, &root);

    assert!(matches!(result, Err(CoreError::Unsupported(_))));
    assert!(!root.exists());
}

/// Mata un proceso y sus hijos, para no dejar ninguno vivo al terminar.
fn kill_process_tree(system_id: u32) {
    let _ = std::process::Command::new("taskkill")
        .args(["/PID", &system_id.to_string(), "/T", "/F"])
        .output();
}

#[test]
#[ignore = "requires .NET SDK"]
fn a_compiled_csharp_project_can_be_launched_and_stays_registered() {
    use miniide::build::build_with;
    use miniide::runtime::ProcessRegistry;
    use miniide::toolchain::{DotNetToolchain, ToolchainProvider};

    let tree = TempTree::new("run-csharp");
    let root = tree.root().join("App");
    let project = create_project(ProjectType::CSharpWinForms, &root).unwrap();

    let built = build_with(&DotNetToolchain, &project).unwrap();
    assert!(built.succeeded(), "{}", built.standard_output());

    let invocation = DotNetToolchain.run_invocation(&project).unwrap();
    let mut registry = ProcessRegistry::new();
    let id = registry.spawn(&invocation).unwrap();
    let system_id = registry.get(id).unwrap().system_id();

    // Parar el proceso no puede cerrar MiniIDE, y el proceso se queda
    // registrado con su estado.
    let stopped = registry.stop(id);
    kill_process_tree(system_id);

    stopped.unwrap();
    assert_eq!(registry.len(), 1, "el proceso sigue registrado tras parar");
}
