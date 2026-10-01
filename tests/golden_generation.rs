//! T-087 y T-088: la salida de los generadores de codigo comparada con una
//! referencia versionada.
//!
//! Los tests de `src/generation/` comprueban partes del codigo escrito. Estos
//! comprueban el archivo entero de la zona: cualquier cambio en el orden de las
//! lineas, en el formato de un numero o en una cadena escapada rompe el test, que
//! es justo lo que se busca cuando la salida cambia sin querer.
//!
//! Las referencias estan en `tests/golden/<framework>/<caso>.txt` y son texto
//! plano. Se actualizan solo cuando el cambio del generador es intencionado, y el
//! `git diff` de la referencia se revisa como parte de la tarea que lo cambia.
//!
//! `.gitattributes` fija `eol=lf` para `tests/golden/**`: con `autocrlf` de
//! Windows el archivo volveria del repositorio con `CRLF` y la comparacion
//! fallaria sin que hubiera cambiado el generador.

use std::fs;
use std::path::PathBuf;

use miniide::framework::{SwingModel, WinFormsModel};
use miniide::generation::{CodeGenerator, SwingGenerator, WinFormsGenerator};

/// La referencia golden de un caso, sin el salto de linea que el archivo lleva al
/// final: el generador no lo escribe y la comparacion es exacta.
fn referencia(framework: &str, caso: &str) -> String {
    let ruta = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(framework)
        .join(format!("{caso}.txt"));

    let contenido = fs::read_to_string(&ruta).unwrap_or_else(|error| {
        panic!("no se puede leer la referencia {}: {error}", ruta.display())
    });

    contenido
        .strip_suffix('\n')
        .unwrap_or(&contenido)
        .to_string()
}

/// Comprueba que lo generado sea exactamente lo que dice la referencia.
fn comprobar(framework: &str, caso: &str, generado: &str) {
    assert_eq!(
        generado,
        referencia(framework, caso),
        "la salida de {framework} no coincide con tests/golden/{framework}/{caso}.txt; \
         si el cambio es intencionado, actualiza la referencia y revisa el diff"
    );
}

/// Una ventana de WinForms con los cuatro controles minimos y su texto.
fn formulario_con_controles() -> WinFormsModel {
    let mut form = WinFormsModel::new("MainForm", "Formulario principal", 640, 480);

    form.add_control("titleLabel", "Label", 12, 12, 200, 20)
        .unwrap();
    form.set_text("titleLabel", "MiniIDE").unwrap();
    form.add_control("nameTextBox", "TextBox", 12, 40, 200, 24)
        .unwrap();
    form.add_control("rootPanel", "Panel", 8, 70, 400, 200)
        .unwrap();
    form.add_control("okButton", "Button", 312, 280, 96, 30)
        .unwrap();
    form.set_text("okButton", "Aceptar").unwrap();

    form
}

/// Un formulario con las propiedades que el usuario ha tocado a mano.
fn formulario_con_propiedades() -> WinFormsModel {
    let mut form = WinFormsModel::new("MainForm", "Formulario principal", 640, 480);

    form.add_control("class", "Label", 0, 0, 100, 20).unwrap();
    form.set_visible("class", false).unwrap();
    form.add_control("details", "TextBox", 0, 24, 100, 20)
        .unwrap();
    form.set_text("details", "Diga \"algo\"").unwrap();
    form.set_enabled("details", false).unwrap();

    form
}

#[test]
fn a_winforms_window_with_no_controls_matches_its_reference() {
    let form = WinFormsModel::new("MainForm", "Formulario principal", 640, 480);
    let code = WinFormsGenerator.generate(form.model()).unwrap();

    comprobar("winforms", "ventana_vacia", code.region().content());
}

#[test]
fn a_winforms_form_with_controls_matches_its_reference() {
    let code = WinFormsGenerator
        .generate(formulario_con_controles().model())
        .unwrap();

    comprobar(
        "winforms",
        "formulario_con_controles",
        code.region().content(),
    );
}

#[test]
fn a_winforms_form_with_hand_touched_properties_matches_its_reference() {
    let code = WinFormsGenerator
        .generate(formulario_con_propiedades().model())
        .unwrap();

    comprobar("winforms", "propiedades", code.region().content());
}

/// Una ventana de Swing con los cuatro componentes minimos y su texto.
fn ventana_con_componentes() -> SwingModel {
    let mut window = SwingModel::new("MainWindow", "Ventana principal", 640, 480);

    window
        .add_component("titleLabel", "JLabel", 12, 12, 200, 20)
        .unwrap();
    window.set_text("titleLabel", "MiniIDE").unwrap();
    window
        .add_component("nameField", "JTextField", 12, 40, 200, 24)
        .unwrap();
    window
        .add_component("rootPanel", "JPanel", 8, 70, 400, 200)
        .unwrap();
    window
        .add_component("okButton", "JButton", 312, 280, 96, 30)
        .unwrap();
    window.set_text("okButton", "Aceptar").unwrap();

    window
}

/// Una ventana con las propiedades que el usuario ha tocado a mano.
fn ventana_con_propiedades() -> SwingModel {
    let mut window = SwingModel::new("MainWindow", "Ventana principal", 640, 480);

    window
        .add_component("final", "JLabel", 0, 0, 100, 20)
        .unwrap();
    window.set_visible("final", false).unwrap();
    window
        .add_component("details", "JTextField", 0, 24, 100, 20)
        .unwrap();
    window.set_text("details", "Diga \"algo\"").unwrap();
    window.set_enabled("details", false).unwrap();

    window
}

#[test]
fn a_swing_window_with_no_components_matches_its_reference() {
    let window = SwingModel::new("MainWindow", "Ventana principal", 640, 480);
    let code = SwingGenerator.generate(window.model()).unwrap();

    comprobar("swing", "ventana_vacia", code.region().content());
}

#[test]
fn a_swing_window_with_components_matches_its_reference() {
    let code = SwingGenerator
        .generate(ventana_con_componentes().model())
        .unwrap();

    comprobar("swing", "ventana_con_componentes", code.region().content());
}

#[test]
fn a_swing_window_with_hand_touched_properties_matches_its_reference() {
    let code = SwingGenerator
        .generate(ventana_con_propiedades().model())
        .unwrap();

    comprobar("swing", "propiedades", code.region().content());
}
