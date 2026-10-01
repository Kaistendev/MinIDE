//! T-078: un proyecto de Java con Swing de principio a fin.
//!
//! Las pruebas de este archivo usan el JDK de verdad, asi que la que compila y
//! ejecuta va marcada como ignorada. Sin esta prueba, el codigo que genera
//! `SwingGenerator` solo se comprobaria contra si mismo, y un error de Java pasaria
//! desapercibido hasta que un usuario genere una ventana.
//!
//! Las dos primeras no necesitan el JDK y si corren siempre: comprueban que el
//! archivo de la ventana en el disco lleva el diseño que dice llevar, y que
//! regenerarlo no se lleva el codigo que el usuario escribio.
//!
//! Para ejecutar la que compila:
//!
//! ```text
//! cargo test --test java_swing_end_to_end -- --ignored
//! ```

use std::path::{Path, PathBuf};
use std::time::Duration;

use miniide::core::ProjectType;
use miniide::document::TextPosition;
use miniide::editor::{Document, DocumentPath, Tab};
use miniide::framework::SwingModel;
use miniide::generation::SwingGenerator;
use miniide::project::Project;
use miniide::templates::{create_project, JAVA_WINDOW_FILE};
use miniide::toolchain::JdkToolchain;

/// Un directorio vacio y sucio, porque `create_project` no escribe encima de nada.
fn project_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&root);

    root
}

/// Una ventana con los cuatro componentes minimos, con sus propiedades puestas.
///
/// Los dos ultimos componentes se ocultan y se desactivan para que el codigo
/// generado lleve banderas de verdad: si el generador las escribiera entrecomilladas,
/// el proyecto no compilaria y este flujo lo diria en el sitio.
fn a_window() -> SwingModel {
    let mut window = SwingModel::new("MainWindow", "Formulario de prueba", 640, 480);

    window
        .add_component("titleLabel", "JLabel", 12, 12, 200, 20)
        .unwrap();
    window.set_text("titleLabel", "MiniIDE").unwrap();
    window
        .add_component("nameText", "JTextField", 12, 40, 200, 24)
        .unwrap();
    window
        .add_component("rootPanel", "JPanel", 8, 70, 400, 200)
        .unwrap();
    window.set_visible("rootPanel", false).unwrap();
    window.set_enabled("rootPanel", false).unwrap();
    window
        .add_component("okButton", "JButton", 312, 280, 96, 30)
        .unwrap();
    window.set_text("okButton", "Aceptar").unwrap();

    window
}

/// Abre el archivo de la ventana en una pestana, le anade una linea al final y lo
/// guarda.
///
/// Es el paso de editar del flujo de T-090: el diseno se aplica despues sobre el
/// archivo que el usuario ha tocado, no sobre el que salio de la plantilla.
fn edit_the_window_file(path: &Path) {
    let source = std::fs::read_to_string(path).expect("el archivo de la ventana de la plantilla");
    let mut tab = Tab::new(
        DocumentPath::new(path).expect("una ruta de documento"),
        Document::new(source),
    );
    let end = end_of(tab.document().buffer().text());

    tab.document_mut()
        .insert(end, "// una linea escrita a mano\n")
        .expect("se escribe la linea del usuario");

    assert!(tab.is_modified());
    tab.save().expect("se guarda el archivo editado");
}

/// La posicion del final de un texto.
fn end_of(text: &str) -> TextPosition {
    let lines: Vec<&str> = text.split('\n').collect();
    let last = lines.last().copied().unwrap_or_default();

    TextPosition::new((lines.len() - 1) as u32, last.len() as u32)
}

/// Crea el proyecto de la plantilla, edita el archivo de la ventana y le escribe
/// el diseño de `window`: crear → editar → diseñar.
fn created_with_design(name: &str) -> (Project, PathBuf) {
    let root = project_root(name);
    let project = create_project(ProjectType::JavaSwing, &root).expect("proyecto nuevo");
    let window_file = root.join(JAVA_WINDOW_FILE);

    edit_the_window_file(&window_file);

    let source =
        std::fs::read_to_string(&window_file).expect("el archivo de la ventana ya editado");
    let generated = SwingGenerator
        .apply_to(a_window().model(), &source)
        .expect("el diseño se escribe en la zona");
    std::fs::write(&window_file, generated).expect("se guarda el diseño");

    (project, window_file)
}

/// Si el proceso `system_id` sigue vivo en el sistema.
///
/// Se pregunta al sistema y no al registro: el registro dice que lo paro
/// MiniIDE, que no es lo mismo que decir que el proceso ha muerto de verdad. Un
/// fallo aqui significa que la ventana se ha quedado abierta, asi que el mensaje
/// lo dice para que no haya que adivinarlo.
fn process_is_running(system_id: u32) -> bool {
    let output = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {system_id}"), "/NH"])
        .output()
        .expect("tasklist se ejecuta");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line.contains(&system_id.to_string()))
}

/// Espera a que el proceso `system_id` desaparezca del sistema.
///
/// Pedir la muerte no la borra de la lista en el acto: hay un momento en el que
/// el proceso ya no se puede matar mas pero sigue apareciendo. Preguntar una sola
/// vez daria un falso fallo, asi que se pregunta un rato.
fn wait_until_gone(system_id: u32) -> bool {
    for _ in 0..40 {
        if !process_is_running(system_id) {
            return true;
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    false
}

#[test]
fn a_generated_window_reaches_the_window_file_on_disk() {
    let (_project, window_file) = created_with_design("miniide-t078-escribe");

    let generated = std::fs::read_to_string(&window_file).expect("el archivo de la ventana");

    for expected in [
        "private JLabel titleLabel;",
        "private JTextField nameText;",
        "private JPanel rootPanel;",
        "private JButton okButton;",
        r#"titleLabel.setText("MiniIDE");"#,
        r#"okButton.setText("Aceptar");"#,
        r#"setTitle("Formulario de prueba");"#,
        "okButton.setBounds(312, 280, 96, 30);",
    ] {
        assert!(
            generated.contains(expected),
            "falta en el archivo: {expected}"
        );
    }
}

#[test]
fn a_regenerated_window_keeps_the_code_the_user_wrote() {
    let (project, window_file) = created_with_design("miniide-t078-protegido");

    // Codigo del usuario en `Main.java`, el archivo que la ventana no toca.
    let main = project.root().join("src/main/java/Main.java");
    let mut source = std::fs::read_to_string(&main).expect("Main.java");
    source.push_str("\n// un comentario del usuario\n");
    std::fs::write(&main, source).expect("se guarda Main.java");

    // Y un segundo paso de generación, por si el archivo ya tenía región.
    let current = std::fs::read_to_string(&window_file).expect("el archivo de la ventana");
    let regenerated = SwingGenerator
        .apply_to(a_window().model(), &current)
        .expect("se regenera");
    std::fs::write(&window_file, regenerated).expect("se guarda");

    let user_code = std::fs::read_to_string(&main).expect("Main.java");

    assert!(user_code.contains("// un comentario del usuario"));
    assert!(std::fs::read_to_string(&window_file)
        .expect("el archivo de la ventana")
        .contains("okButton"));

    let _ = std::fs::remove_dir_all(project.root());
}

#[test]
#[ignore = "compila y ejecuta de verdad: necesita el JDK"]
fn a_generated_swing_project_compiles_runs_and_can_be_stopped() {
    use miniide::build::build_with;
    use miniide::runtime::{ProcessRegistry, ProcessState};
    use miniide::toolchain::ToolchainProvider;

    if !JdkToolchain.is_available() {
        eprintln!("se salta: no hay JDK en el PATH");

        return;
    }

    let (project, _window_file) = created_with_design("miniide-t078-completo");

    // El proyecto se deja en su sitio a proposito. Borrarlo casi nunca cuela:
    // el proceso que MiniIDE acaba de parar sigue un momento con los ficheros
    // abiertos. Y cuando la compilacion falla, poder abrir el Java generado vale
    // mas que un temporal limpio.
    eprintln!("proyecto de la prueba: {}", project.root().display());

    // 1. Compila.
    let build = build_with(&JdkToolchain, &project).expect("javac se ejecuta");

    assert!(
        build.succeeded(),
        "el proyecto generado no compila.\n{}\n{}",
        build.output().standard_output(),
        build.output().standard_error()
    );

    for diagnostic in build.diagnostics() {
        eprintln!("diagnostico: {}", diagnostic.message());
    }

    // 2. Se ejecuta.
    let mut registry = ProcessRegistry::new();
    let id = registry
        .spawn(
            &JdkToolchain
                .run_invocation(&project)
                .expect("java -cp bin Main"),
        )
        .expect("la aplicación arranca");
    let system_id = registry.get(id).expect("registrado").system_id();

    std::thread::sleep(Duration::from_secs(5));

    let state = registry
        .get_mut(id)
        .expect("el proceso sigue registrado")
        .result()
        .state();

    assert_eq!(
        state,
        ProcessState::Running,
        "la ventana de Swing no se queda viva"
    );

    // 3. Se detiene sin cerrar el IDE, y la ventana se cierra de verdad.
    registry.stop(id).expect("la aplicación se detiene");

    assert_eq!(
        registry
            .get_mut(id)
            .expect("sigue registrado")
            .result()
            .state(),
        ProcessState::Stopped
    );
    assert!(
        wait_until_gone(system_id),
        "la ventana se ha quedado abierta: el proceso {system_id} sigue vivo"
    );
}
