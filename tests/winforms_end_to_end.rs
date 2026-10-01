//! T-065: un proyecto de C# con Windows Forms de principio a fin.
//!
//! Las pruebas de este archivo usan el SDK de .NET de verdad, asi que la que
//! compila y ejecuta va marcada como ignorada. Sin esta prueba, el codigo que
//! genera `WinFormsGenerator` solo se comprobaria contra si mismo, y un error
//! de C# pasaria desapercibido hasta que un usuario genere un formulario.
//!
//! La primera prueba no necesita el SDK y si corre siempre: comprueba que el
//! archivo del disenador en el disco lleva el diseño que dice llevar.
//!
//! Para ejecutar la que compila:
//!
//! ```text
//! cargo test --test winforms_end_to_end -- --ignored
//! ```

use std::path::{Path, PathBuf};
use std::time::Duration;

use miniide::build::build_with;
use miniide::core::ProjectType;
use miniide::document::TextPosition;
use miniide::editor::{Document, DocumentPath, Tab};
use miniide::framework::WinFormsModel;
use miniide::generation::WinFormsGenerator;
use miniide::project::Project;
use miniide::runtime::{self, ProcessRegistry, ProcessState};
use miniide::templates::{create_project, DESIGNER_FILE};
use miniide::toolchain::{DotNetSdk, DotNetToolchain, Invocation, ToolchainProvider};

/// Un directorio vacio y sucio, porque `create_project` no escribe encima de
/// nada.
fn project_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&root);

    root
}

/// Un formulario con los cuatro controles minimos, con sus propiedades puestas.
///
/// El panel se oculta y se desactiva para que el codigo generado lleve banderas de
/// verdad: si el generador las escribiera entrecomilladas, `Visible = "false"` no
/// compilaria y este flujo lo diria en el sitio.
fn a_form() -> WinFormsModel {
    let mut form = WinFormsModel::new("MainForm", "Formulario de prueba", 640, 480);

    form.add_control("titleLabel", "Label", 12, 12, 200, 20)
        .unwrap();
    form.set_text("titleLabel", "MiniIDE").unwrap();
    form.add_control("nameTextBox", "TextBox", 12, 40, 200, 24)
        .unwrap();
    form.add_control("rootPanel", "Panel", 8, 70, 400, 200)
        .unwrap();
    form.set_visible("rootPanel", false).unwrap();
    form.set_enabled("rootPanel", false).unwrap();
    form.add_control("okButton", "Button", 312, 280, 96, 30)
        .unwrap();
    form.set_text("okButton", "Aceptar").unwrap();

    form
}

/// Abre el archivo del disenador en una pestana, le anade una linea al final y lo
/// guarda.
///
/// Es el paso de editar del flujo de T-089: el diseno se aplica despues sobre el
/// archivo que el usuario ha tocado, no sobre el que salio de la plantilla.
fn edit_the_designer_file(path: &Path) {
    let source = std::fs::read_to_string(path).expect("el archivo del disenador de la plantilla");
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

/// Crea el proyecto de la plantilla, edita el archivo del disenador y le escribe el
/// diseño de `form`: crear → editar → diseñar.
fn created_with_design(name: &str) -> (Project, PathBuf) {
    let root = project_root(name);
    let project = create_project(ProjectType::CSharpWinForms, &root).expect("proyecto nuevo");
    let designer = root.join(DESIGNER_FILE);

    edit_the_designer_file(&designer);

    let source = std::fs::read_to_string(&designer).expect("el archivo del disenador ya editado");
    let generated = WinFormsGenerator
        .apply_to(a_form().model(), &source)
        .expect("el diseño se escribe en la zona");
    std::fs::write(&designer, generated).expect("se guarda el diseño");

    (project, designer)
}

/// Si esta instalado el runtime que hace falta para ejecutar Windows Forms.
///
/// El SDK compila para `net8.0-windows` aunque el runtime de Windows Forms de
/// esa version no este: sin el, el proyecto compila y la aplicacion no llega a
/// arrancar. Es una diferencia entre tener el SDK y poder ejecutar, y el test
/// tiene que distinguir las dos cosas.
fn windows_forms_runtime_is_installed() -> bool {
    let invocation = Invocation::new(
        DotNetSdk::EXECUTABLE,
        vec!["--list-runtimes".to_string()],
        std::env::temp_dir(),
    );

    match runtime::run(&invocation) {
        Ok(output) => output
            .standard_output()
            .contains("Microsoft.WindowsDesktop.App"),
        Err(error) => {
            eprintln!("no se puede preguntar por los runtimes de .NET: {error}");
            false
        }
    }
}

#[test]
fn a_generated_form_reaches_the_designer_file_on_disk() {
    let (_project, designer) = created_with_design("miniide-t065-escribe");

    let generated = std::fs::read_to_string(&designer).expect("el archivo del disenador");

    for expected in [
        "private System.Windows.Forms.Label titleLabel;",
        "private System.Windows.Forms.TextBox nameTextBox;",
        "private System.Windows.Forms.Panel rootPanel;",
        "private System.Windows.Forms.Button okButton;",
        r#"this.okButton.Text = "Aceptar";"#,
        r#"this.Text = "Formulario de prueba";"#,
    ] {
        assert!(
            generated.contains(expected),
            "falta en el archivo: {expected}"
        );
    }
}

#[test]
fn a_regenerated_form_keeps_the_code_the_user_wrote() {
    let (project, designer) = created_with_design("miniide-t065-protegido");

    // Codigo del usuario en `Form1.cs`, el archivo que el disenador no toca.
    let form = project.root().join("Form1.cs");
    let mut source = std::fs::read_to_string(&form).expect("Form1.cs");
    source.push_str("\n// un comentario del usuario\n");
    std::fs::write(&form, source).expect("se guarda Form1.cs");

    // Y un segundo paso de generación, por si el archivo ya tenía región.
    let current = std::fs::read_to_string(&designer).expect("el archivo del disenador");
    let regenerated = WinFormsGenerator
        .apply_to(a_form().model(), &current)
        .expect("se regenera");
    std::fs::write(&designer, regenerated).expect("se guarda");

    let user_code = std::fs::read_to_string(&form).expect("Form1.cs");

    assert!(user_code.contains("// un comentario del usuario"));
    assert!(std::fs::read_to_string(&designer)
        .expect("el archivo del disenador")
        .contains("okButton"));

    let _ = std::fs::remove_dir_all(project.root());
}

#[test]
#[ignore = "compila y ejecuta de verdad: necesita el SDK de .NET y su runtime de Windows Forms"]
fn a_generated_winforms_project_compiles_runs_and_can_be_stopped() {
    if DotNetSdk::detect().is_none() {
        eprintln!("se salta: no hay SDK de .NET en el PATH");

        return;
    }

    let (project, _designer) = created_with_design("miniide-t065-completo");

    // El proyecto se deja en su sitio a proposito. Borrarlo casi nunca cuela:
    // el proceso que MiniIDE acaba de parar sigue un momento con los ficheros
    // abiertos. Y cuando la compilacion falla, poder abrir el C# generado vale
    // mas que un temporal limpio.
    eprintln!("proyecto de la prueba: {}", project.root().display());

    // 1. Compila.
    let build = build_with(&DotNetToolchain, &project).expect("dotnet build se ejecuta");

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
    if !windows_forms_runtime_is_installed() {
        eprintln!(
            "el proyecto compila, pero se salta la ejecución: no hay runtime \
             Microsoft.WindowsDesktop.App instalado para esta versión del TFM"
        );

        return;
    }

    let mut registry = ProcessRegistry::new();
    let id = registry
        .spawn(
            &DotNetToolchain
                .run_invocation(&project)
                .expect("dotnet run"),
        )
        .expect("la aplicación arranca");

    std::thread::sleep(Duration::from_secs(5));

    let state = registry
        .get_mut(id)
        .expect("el proceso sigue registrado")
        .state();

    assert_eq!(
        state,
        ProcessState::Running,
        "la aplicación de Windows Forms no se queda viva"
    );

    // 3. Se detiene sin cerrar el IDE.
    registry.stop(id).expect("la aplicación se detiene");

    assert_eq!(
        registry.get_mut(id).expect("sigue registrado").state(),
        ProcessState::Stopped
    );
}

/// El flujo de este archivo compila y ejecuta con `dotnet`, asi que necesita un
/// proyecto de C#. Un proyecto de Java no tiene su archivo de proyecto, y por eso
/// este flujo no se puede aplicar a el.
#[test]
fn a_java_project_is_not_a_csharp_project() {
    let root = project_root("miniide-t065-java");

    let project = create_project(ProjectType::JavaSwing, &root).expect("proyecto de Java");

    assert!(root.join("pom.xml").is_file());
    assert!(
        !root.join("App.csproj").exists(),
        "un proyecto de Java no puede traer un archivo de proyecto de C#"
    );
    let _ = std::fs::remove_dir_all(project.root());
}
