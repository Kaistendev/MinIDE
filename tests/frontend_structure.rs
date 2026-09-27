use std::fs;
use std::path::{Path, PathBuf};

use miniide::frontend::{App, UiState};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);

    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()))
}

/// Archivos de Rust que hay bajo `src/`, con su ruta relativa al repositorio.
///
/// Se devuelven ordenados para que un fallo muestre siempre la misma lista.
fn source_files() -> Vec<String> {
    fn visit(directory: &Path, root: &Path, found: &mut Vec<String>) {
        let entries = fs::read_dir(directory)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", directory.display()));

        for entry in entries {
            let path = entry.expect("readable directory entry").path();

            if path.is_dir() {
                visit(&path, root, found);
                continue;
            }

            if path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }

            let relative = path.strip_prefix(root).expect("path inside src");

            found.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }

    let src = repo_root().join("src");
    let mut found = Vec::new();

    visit(&src, &src, &mut found);
    found.sort();

    found
}

/// El frontend tiene su propia carpeta y su propio modulo raiz.
///
/// La carpeta es lo que separa la interfaz del core en el codigo. Con todo colgado
/// de `src/` en archivos sueltos, una dependencia entre la UI y el core se cuela sin
/// que se note, y la regla de que la UI manda y el core obedece se queda en el
/// papel.
#[test]
fn the_frontend_lives_in_its_own_folder() {
    let folder = repo_root().join("src").join("frontend");

    assert!(folder.is_dir(), "src/frontend tiene que ser una carpeta");
    assert!(
        folder.join("mod.rs").is_file(),
        "src/frontend tiene que tener su modulo raiz"
    );
}

/// El estado de la interfaz es del frontend y no del core.
///
/// Es la regla de que la UI no se guarda el estado del dominio: mientras el estado
/// visual viva entre los modulos del core, la frontera queda difuminada y ya no se
/// sabe que es de quien. Aqui solo se comprueba donde vive; lo que lleva dentro lo
/// decide `docs/frontend-tasks.md` FE-004.
#[test]
fn the_ui_state_lives_in_the_frontend_and_not_in_the_core() {
    assert!(
        !repo_root().join("src").join("ui.rs").exists(),
        "el estado de la interfaz no puede seguir en la raiz del core"
    );
    assert!(
        source_files()
            .iter()
            .any(|file| file.starts_with("frontend/")),
        "el estado de la interfaz tiene que estar dentro de src/frontend"
    );
}

/// El frontend se abre desde fuera del core.
///
/// Que este archivo compile ya es la comprobacion: `miniide::frontend` solo existe si
/// el modulo raiz del frontend es publico, y sin el no habria por donde entrar a la
/// interfaz sin meterse en el core.
#[test]
fn the_frontend_is_reachable_from_outside_the_core() {
    let mut state = UiState::new();
    state.set_status("Compilando...");

    let mut app = App::new();
    app.state_mut().set_status("Compilando...");

    assert_eq!(state.status(), Some("Compilando..."));
    assert_eq!(app.state().status(), Some("Compilando..."));
}

/// La dependencia va de la interfaz al core y nunca al reves.
///
/// El core no conoce la interfaz: la interfaz consume el core y le emite comandos, y
/// en cuanto el core llama a la interfaz la regla se ha dado la vuelta. Aqui se
/// comprueba que no hay ningun modulo del core que la use.
///
/// El binario queda fuera del recuento, y no por caso: `src/main.rs` es lo que
/// ejecuta el usuario, y su trabajo es precisamente abrir la interfaz. Lo que no
/// puede es abrirla desde el core.
#[test]
fn the_core_does_not_depend_on_the_frontend() {
    let uses: Vec<String> = source_files()
        .iter()
        .filter(|file| !file.starts_with("frontend/") && file.as_str() != "main.rs")
        .filter(|file| {
            let content = read(&format!("src/{file}"));

            content.contains("crate::frontend") || content.contains("miniide::frontend")
        })
        .cloned()
        .collect();

    assert!(
        uses.is_empty(),
        "el core no puede usar el frontend, y estos archivos lo hacen: {uses:?}"
    );
}

/// La aplicacion entra por el frontend.
///
/// El binario es el unico sitio de fuera de la carpeta del frontend que puede usar la
/// interfaz, porque es lo que ejecuta el usuario. Comprobarlo evita que la ventana se
/// llegue a abrir desde el core en algun sitio, que es justo lo que la regla prohibe.
#[test]
fn the_application_enters_the_interface_through_the_frontend() {
    let main = read("src/main.rs");

    assert!(
        main.contains("miniide::frontend::run()"),
        "src/main.rs tiene que abrir la interfaz a traves del frontend"
    );
}
