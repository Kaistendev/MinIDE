//! T-086: tests de regresion del ciclo de proyecto.
//!
//! Los unitarios de `src/project.rs` y `src/workspace.rs` comprueban metodo por
//! metodo. Estos hacen el ciclo entero sobre el disco: crear, abrir, renombrar y
//! cerrar, y comprueban que el modelo y el disco siguen contandolo mismo.
//!
//! El caso que mas se rompe es el del proyecto recien creado: la plantilla escribe
//! los archivos y devuelve el modelo, y cualquier operacion posterior que se apoye
//! en el modelo en vez de en el disco acaba con un modelo que ya no corresponde con
//! lo que hay escrito. Por eso casi todos los casos vuelven a abrir el proyecto y
//! comparan las dos listas de archivos.

use std::path::{Path, PathBuf};

use miniide::core::{CoreError, ProjectType};
use miniide::project::{BuildConfiguration, Project, ProjectRelativePath};
use miniide::templates::{create_project, DESIGNER_FILE, JAVA_WINDOW_FILE};
use miniide::workspace::Workspace;

/// Un directorio vacio y sucio, porque `create_project` no escribe encima de nada.
fn project_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&root);

    root
}

fn relative(value: &str) -> ProjectRelativePath {
    ProjectRelativePath::new(value).expect("una ruta dentro del proyecto")
}

fn default_configuration() -> BuildConfiguration {
    BuildConfiguration::new(
        ProjectRelativePath::new(BuildConfiguration::DEFAULT_OUTPUT_DIRECTORY)
            .expect("el directorio de salida por defecto"),
    )
}

/// El archivo con el que se abre el proyecto.
///
/// La plantilla nombra el `.csproj` como el directorio, porque el nombre del
/// proyecto sale de ahi. Java usa siempre `pom.xml`.
fn archivo_de_proyecto(tipo: ProjectType, root: &Path) -> PathBuf {
    match tipo {
        ProjectType::CSharpWinForms => root.join(format!(
            "{}.csproj",
            root.file_name()
                .expect("el directorio da nombre")
                .to_string_lossy()
        )),
        ProjectType::JavaSwing => root.join("pom.xml"),
    }
}

/// Las rutas de los archivos del proyecto, en el orden en que las da
/// `discover_files` y con `/` para que la comparacion no dependa de Windows.
fn model_paths(project: &Project) -> Vec<String> {
    project
        .discover_files()
        .expect("se enumeran los archivos")
        .iter()
        .map(|file| file.path().as_path().to_string_lossy().replace('\\', "/"))
        .collect()
}

/// Crea el proyecto de la plantilla y lo vuelve a abrir desde el disco.
///
/// Devolver el proyecto reescrito obliga a que cada caso mire el estado que tiene
/// al abrirse, y no solo el que la plantilla ha dejado en memoria.
fn creado_y_reabierto(tipo: ProjectType, root: &Path) -> Project {
    create_project(tipo, root).expect("proyecto nuevo");

    Project::open(&archivo_de_proyecto(tipo, root)).expect("el proyecto se abre otra vez")
}

fn borrar(root: &Path) {
    let _ = std::fs::remove_dir_all(root);
}

/// Un proyecto de C# con Windows Forms sale de la plantilla con sus archivos.
///
/// La plantilla escribe los archivos y devuelve el modelo. Si el modelo no los
/// supiera, el explorador saldria vacio justo despues de crear el proyecto.
#[test]
fn a_new_csharp_project_knows_the_files_the_template_wrote() {
    let root = project_root("miniide-t086-crear-csharp");
    let project = create_project(ProjectType::CSharpWinForms, &root).expect("proyecto nuevo");

    let del_modelo = model_paths(&project);
    let reabierto = Project::open(&archivo_de_proyecto(ProjectType::CSharpWinForms, &root))
        .expect("se abre otra vez");

    assert_eq!(
        del_modelo,
        model_paths(&reabierto),
        "el modelo de la plantilla y el del disco no coinciden"
    );
    assert!(
        del_modelo.contains(&"Form1.cs".to_string()),
        "{del_modelo:?}"
    );
    assert!(
        del_modelo.contains(&DESIGNER_FILE.to_string()),
        "{del_modelo:?}"
    );

    borrar(&root);
}

/// Lo mismo con el proyecto de Java.
#[test]
fn a_new_java_project_knows_the_files_the_template_wrote() {
    let root = project_root("miniide-t086-crear-java");
    let project = create_project(ProjectType::JavaSwing, &root).expect("proyecto nuevo");

    let del_modelo = model_paths(&project);
    let reabierto = Project::open(&archivo_de_proyecto(ProjectType::JavaSwing, &root))
        .expect("se abre otra vez");

    assert_eq!(
        del_modelo,
        model_paths(&reabierto),
        "el modelo de la plantilla y el del disco no coinciden"
    );
    assert!(
        del_modelo.contains(&JAVA_WINDOW_FILE.to_string()),
        "{del_modelo:?}"
    );

    borrar(&root);
}

/// Crear un proyecto no pisa lo que ya habia en el directorio.
///
/// El error viene antes de escribir nada: si escribiera primero y comprobara
/// despues, un directorio con archivos propios perderia su contenido.
#[test]
fn a_directory_with_something_in_it_is_not_used_for_a_new_project() {
    let root = project_root("miniide-t086-ocupado");
    std::fs::create_dir_all(&root).unwrap();
    let suyo = root.join("notas.txt");
    std::fs::write(&suyo, "no se toca").unwrap();

    let resultado = create_project(ProjectType::CSharpWinForms, &root);

    assert!(
        matches!(resultado, Err(CoreError::AlreadyExists(_))),
        "{resultado:?}"
    );
    assert_eq!(
        std::fs::read_to_string(&suyo).unwrap(),
        "no se toca",
        "el directorio del usuario se queda como estaba"
    );
    assert!(!archivo_de_proyecto(ProjectType::CSharpWinForms, &root).exists());

    borrar(&root);
}

/// Abrir algo que no es un proyecto se dice, no se adivina.
///
/// Un archivo de texto, un `.csproj` sin la marca de Windows Forms y una ruta que
/// no existe: los tres se rechazan en lugar de abrirse como un proyecto que luego
/// no compila.
#[test]
fn something_that_is_not_a_project_is_not_opened() {
    let root = project_root("miniide-t086-no-proyecto");
    std::fs::create_dir_all(&root).unwrap();

    let solo_texto = root.join("notas.txt");
    std::fs::write(&solo_texto, "esto no es un proyecto").unwrap();
    assert!(matches!(
        Project::open(&solo_texto),
        Err(CoreError::Unsupported(_))
    ));

    let sin_marca = root.join("App.csproj");
    std::fs::write(&sin_marca, "<Project></Project>").unwrap();
    assert!(matches!(
        Project::open(&sin_marca),
        Err(CoreError::Unsupported(_))
    ));

    let raiz = archivo_de_proyecto(ProjectType::CSharpWinForms, &root);
    assert!(matches!(Project::open(&raiz), Err(CoreError::NotFound(_))));

    borrar(&root);
}

/// Un proyecto de WPF no se abre como si fuera de Windows Forms.
///
/// Los dos son de C# y los dos tienen un `.csproj`. Si el tipo se decidiera por la
/// extension del archivo de proyecto, MiniIDE abriria un proyecto de WPF y
/// generaria en el codigo de WinForms que no compila.
#[test]
fn a_wpf_project_is_not_taken_for_a_winforms_project() {
    let root = project_root("miniide-t086-wpf");
    std::fs::create_dir_all(&root).unwrap();
    let archivo = root.join("App.csproj");
    std::fs::write(
        &archivo,
        "<Project><PropertyGroup><UseWPF>true</UseWPF></PropertyGroup></Project>",
    )
    .unwrap();

    let resultado = Project::open(&archivo);

    assert!(
        matches!(resultado, Err(CoreError::Unsupported(_))),
        "{resultado:?}"
    );

    borrar(&root);
}

/// Renombrar un archivo que pisa a otro se rechaza.
///
/// En Windows `fs::rename` machaca el destino sin avisar. Como no hay forma atomica
/// de renombrar solo si el destino no existe, el proyecto comprueba antes; si no, el
/// archivo del usuario desaparece sin que MiniIDE haya dicho nada.
#[test]
fn a_rename_that_would_overwrite_another_file_is_refused() {
    let root = project_root("miniide-t086-renombrar");
    let mut project = creado_y_reabierto(ProjectType::CSharpWinForms, &root);

    project
        .create_file(relative("notas.txt"), "el primero")
        .unwrap();
    project
        .create_file(relative("apuntes.txt"), "el segundo")
        .unwrap();

    let resultado = project.rename(relative("notas.txt"), "apuntes.txt");

    assert!(
        matches!(resultado, Err(CoreError::AlreadyExists(_))),
        "{resultado:?}"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("notas.txt")).unwrap(),
        "el primero",
        "el archivo que se renombra sigue en su sitio"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("apuntes.txt")).unwrap(),
        "el segundo",
        "el destino sigue con lo suyo"
    );

    borrar(&root);
}

/// Renombrar un directorio mueve lo que hay dentro y el modelo lo sigue.
///
/// El modelo solo conoce las entradas que se le han anadido: si al renombrar un
/// directorio no se reescribieran las rutas de dentro, el explorador apuntaria a
/// sitios que ya no existen.
#[test]
fn renaming_a_directory_moves_its_contents_and_the_model_follows() {
    let root = project_root("miniide-t086-renombrar-directorio");
    let mut project = creado_y_reabierto(ProjectType::CSharpWinForms, &root);

    project.create_directory(relative("src")).unwrap();
    project
        .create_file(relative("src/Principal.cs"), "codigo")
        .unwrap();
    project.create_directory(relative("recursos")).unwrap();
    project
        .create_file(relative("recursos/logo.png"), "png")
        .unwrap();

    project.rename(relative("src"), "codigo").unwrap();

    assert!(root.join("codigo/Principal.cs").is_file());
    assert!(!root.join("src").exists());
    assert!(root.join("recursos/logo.png").is_file(), "el vecino sigue");

    let reabierto = Project::open(&archivo_de_proyecto(ProjectType::CSharpWinForms, &root))
        .expect("se abre otra vez");
    let archivos = model_paths(&reabierto);

    assert!(
        archivos.contains(&"codigo/Principal.cs".to_string()),
        "{archivos:?}"
    );
    assert!(
        !archivos.contains(&"src/Principal.cs".to_string()),
        "{archivos:?}"
    );
    assert!(
        archivos.contains(&"recursos/logo.png".to_string()),
        "el vecino sigue donde estaba: {archivos:?}"
    );

    borrar(&root);
}

/// Un nombre nuevo con separadores no mueve el archivo de directorio.
///
/// `rename` mueve dentro del mismo directorio. Aceptar un nombre con separador o
/// `..` dejaria sacar un archivo del proyecto con un nombre que en la interfaz ni
/// cabe ni se ve.
#[test]
fn a_new_name_with_a_separator_or_a_parent_is_refused() {
    let root = project_root("miniide-t086-nombre");
    let mut project = creado_y_reabierto(ProjectType::CSharpWinForms, &root);

    for nombre in ["", "..", "sub/otro", "sub\\otro"] {
        let resultado = project.rename(relative("Form1.cs"), nombre);

        assert!(
            resultado.is_err(),
            "un nombre nuevo {nombre:?} tiene que rechazarse, dio {resultado:?}"
        );
    }

    assert!(root.join("Form1.cs").is_file(), "nada se movio");

    borrar(&root);
}

/// Cerrar un proyecto lo devuelve y deja el workspace con su raiz.
///
/// Cerrar no toca el disco: el proyecto sigue ahi y se puede volver a abrir. Si
/// cerrara los archivos, el trabajo sin guardar se perderia sin preguntar.
#[test]
fn closing_a_project_returns_it_and_leaves_the_workspace_with_its_root() {
    let root = project_root("miniide-t086-cerrar");
    let proyecto = creado_y_reabierto(ProjectType::CSharpWinForms, &root);
    let raiz_de_sesion = root
        .parent()
        .expect("el padre del directorio")
        .to_path_buf();
    let mut workspace = Workspace::new(&raiz_de_sesion).expect("workspace");

    workspace.set_active_project(proyecto);

    let cerrado = workspace.close_project().expect("el proyecto se devuelve");

    assert_eq!(
        cerrado.name(),
        root.file_name().unwrap().to_string_lossy().as_ref()
    );
    assert!(workspace.active_project().is_none());
    assert_eq!(workspace.root(), raiz_de_sesion);
    assert!(
        archivo_de_proyecto(ProjectType::CSharpWinForms, &root).is_file(),
        "cerrar no borra nada"
    );

    borrar(&root);
}

/// Cerrar cuando no hay nada que cerrar no inventa un proyecto.
///
/// La interfaz llama a cerrar desde un comando que se puede pulsar siempre, y tiene
/// que poder hacerlo sin mirar antes si hay algo abierto.
#[test]
fn closing_a_workspace_with_no_project_gives_nothing() {
    let mut workspace = Workspace::new(std::env::temp_dir()).expect("workspace");

    assert!(workspace.close_project().is_none());
    assert!(
        workspace.close_project().is_none(),
        "cerrar dos veces tampoco"
    );
}

/// Abrir, cerrar y volver a abrir deja el proyecto como estaba.
///
/// Es el ciclo que hace la sesion al cambiar de proyecto, con los dos proyectos en
/// el disco a la vez.
#[test]
fn a_project_survives_being_opened_closed_and_opened_again() {
    let root_csharp = project_root("miniide-t086-ciclo-csharp");
    let root_java = project_root("miniide-t086-ciclo-java");
    let csharp = creado_y_reabierto(ProjectType::CSharpWinForms, &root_csharp);
    let java = creado_y_reabierto(ProjectType::JavaSwing, &root_java);
    let antes = model_paths(&csharp);
    let mut workspace = Workspace::new(std::env::temp_dir()).expect("workspace");

    workspace.set_active_project(csharp);
    workspace.close_project();
    workspace.set_active_project(java);
    assert_eq!(
        workspace.active_project().unwrap().project_type(),
        ProjectType::JavaSwing
    );
    workspace.close_project();

    workspace.set_active_project(
        Project::open(&archivo_de_proyecto(
            ProjectType::CSharpWinForms,
            &root_csharp,
        ))
        .expect("se vuelve a abrir"),
    );

    let activo = workspace.active_project().expect("proyecto activo");

    assert_eq!(activo.project_type(), ProjectType::CSharpWinForms);
    assert_eq!(model_paths(activo), antes);

    borrar(&root_csharp);
    borrar(&root_java);
}

/// Abrir un proyecto dos veces lleva al mismo sitio.
///
/// `Project::open` no guarda estado, asi que abrir el mismo archivo otra vez tiene
/// que dar un proyecto igual: si no, el editor y el explorador estarian mirando
/// modelos distintos del mismo disco.
#[test]
fn opening_the_same_project_twice_gives_the_same_model() {
    let root = project_root("miniide-t086-dos-veces");
    let primero = creado_y_reabierto(ProjectType::CSharpWinForms, &root);
    let segundo =
        Project::open(&archivo_de_proyecto(ProjectType::CSharpWinForms, &root)).expect("se abre");

    assert_eq!(primero.name(), segundo.name());
    assert_eq!(primero.root(), segundo.root());
    assert_eq!(primero.project_type(), segundo.project_type());
    assert_eq!(model_paths(&primero), model_paths(&segundo));

    borrar(&root);
}

/// Un proyecto abierto en un workspace sustituye al que habia.
///
/// La sesion solo puede tener un proyecto activo: si al abrir otro no se sustituyera
/// el anterior, el editor y el explorador estarian mirando proyectos distintos.
#[test]
fn opening_a_second_project_replaces_the_active_one() {
    let root_csharp = project_root("miniide-t086-dos-csharp");
    let root_java = project_root("miniide-t086-dos-java");
    let mut workspace = Workspace::new(std::env::temp_dir()).expect("workspace");

    workspace.set_active_project(creado_y_reabierto(
        ProjectType::CSharpWinForms,
        &root_csharp,
    ));
    workspace.set_active_project(creado_y_reabierto(ProjectType::JavaSwing, &root_java));

    let activo = workspace.active_project().expect("proyecto activo");

    assert_eq!(activo.project_type(), ProjectType::JavaSwing);
    assert_eq!(activo.language(), activo.project_type().language());

    borrar(&root_csharp);
    borrar(&root_java);
}

/// El ciclo entero de un proyecto deja el disco como estaba.
///
/// Crear, anadir un archivo, renombrarlo, borrarlo y volver a abrir tiene que dar
/// un proyecto sin lo que se anadio. Si el modelo guardara las entradas sin mirar el
/// disco, aqui se veria.
#[test]
fn the_whole_cycle_leaves_the_model_matching_the_disk() {
    let root = project_root("miniide-t086-ciclo-completo");
    let mut project = creado_y_reabierto(ProjectType::CSharpWinForms, &root);
    let antes = model_paths(&project);

    project.create_directory(relative("extra")).unwrap();
    project
        .create_file(relative("extra/uno.cs"), "uno")
        .unwrap();
    project.rename(relative("extra/uno.cs"), "dos.cs").unwrap();
    project.remove(relative("extra/dos.cs")).unwrap();
    project.remove(relative("extra")).unwrap();

    let reabierto = Project::open(&archivo_de_proyecto(ProjectType::CSharpWinForms, &root))
        .expect("se abre otra vez");

    assert_eq!(model_paths(&reabierto), antes);
    assert!(!root.join("extra").exists());

    borrar(&root);
}

/// La raiz de un proyecto tiene que ser absoluta.
///
/// Con una ruta relativa, el mismo proyecto se abriria en un sitio distinto segun
/// desde donde se llame al IDE, y el archivo que se guarda acabaria en otro disco.
#[test]
fn a_relative_root_is_not_a_project_root() {
    let resultado = Project::new(
        "app",
        "proyectos\\app",
        ProjectType::CSharpWinForms,
        default_configuration(),
    );

    assert!(
        matches!(resultado, Err(CoreError::InvalidPath(_))),
        "{resultado:?}"
    );
}

/// Un proyecto se construye en memoria aunque su carpeta no este.
///
/// `Project::new` no toca el disco: da el modelo con lo que se le pasa. Lo que
/// dice si la carpeta existe es enumerar sus archivos, que es cuando se necesita
/// saber. Asi el nucleo no necesita el disco para existir.
#[test]
fn building_a_project_does_not_need_its_root_to_exist() {
    let root = project_root("miniide-t086-no-existe");
    let proyecto = Project::new(
        "app",
        &root,
        ProjectType::CSharpWinForms,
        default_configuration(),
    )
    .expect("el modelo se construye sin disco");

    assert_eq!(proyecto.name(), "app");
    assert_eq!(proyecto.root(), root);
    assert!(!root.exists(), "construir el modelo no crea la carpeta");
    assert!(
        matches!(proyecto.discover_files(), Err(CoreError::NotFound(_))),
        "enumerar si avisa de que la carpeta no esta"
    );
}
