use std::fs;
use std::path::Path;

fn read_repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);

    match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => panic!("could not read {}: {error}", path.display()),
    }
}

#[test]
fn format_config_pins_edition_and_width() {
    let config = read_repo_file("rustfmt.toml");

    assert!(config.contains("edition = \"2021\""));
    assert!(config.contains("max_width = 100"));
}

#[test]
fn lint_config_declares_msrv() {
    let config = read_repo_file("clippy.toml");

    assert!(config.contains("msrv = \"1.75\""));
}

#[test]
fn manifest_declares_the_warning_policy() {
    let manifest = read_repo_file("Cargo.toml");

    for expected in [
        "[lints.rust]",
        "unsafe_code = \"forbid\"",
        "[lints.clippy]",
        "all = \"deny\"",
    ] {
        assert!(
            manifest.contains(expected),
            "Cargo.toml must declare `{expected}`"
        );
    }
}

#[test]
fn quality_policy_is_documented() {
    let policy = read_repo_file("docs/quality.md");

    for expected in ["cargo fmt --check", "cargo clippy", "cargo test", "T-003"] {
        assert!(
            policy.contains(expected),
            "docs/quality.md must mention `{expected}`"
        );
    }
}

/// Toolkits de UI que el MVP no usa y que no pueden aparecer en el manifiesto.
///
/// Son losAlternativos de verdad, no una lista de librerias cualesquiera: el test no
/// prohibe nada mas, solo impide que aparezca una segunda interfaz que pelee por la
/// ventana con la que ya esta decidida.
const OTHER_UI_TOOLKITS: &[&str] = &[
    "iced", "druid", "slint", "tauri", "gtk", "gtk4", "fltk", "wx", "azul", "floem", "xilem",
    "dioxus",
];

/// Si el manifiesto depende de `crate_name`.
fn depends_on(manifest: &str, crate_name: &str) -> bool {
    manifest.lines().map(str::trim).any(|line| {
        line.starts_with(crate_name) && line[crate_name.len()..].trim_start().starts_with('=')
    })
}

/// La segunda mitad de la decision del stack se comprueba donde se puede: en lo que
/// el proyecto depende de verdad.
///
/// Declarar el stack en un documento no basta si nada impide meter otro al lado, y
/// con dos toolkits la ventana la pelea la que se isntale antes. egui/eframe es el
/// stack del MVP (`docs/frontend-plan.md` FD-01 y FD-03) y no se vuelve a abrir.
#[test]
fn the_manifest_does_not_depend_on_another_ui_toolkit() {
    let manifest = read_repo_file("Cargo.toml");

    for toolkit in OTHER_UI_TOOLKITS {
        assert!(
            !depends_on(&manifest, toolkit),
            "el stack de UI del MVP es egui/eframe (`docs/frontend-plan.md` FD-01) y `Cargo.toml` depende de `{toolkit}`"
        );
    }
}

/// El guard tiene que reconocer una dependencia de verdad, y no decir que no por
/// casualidad.
///
/// Sin esto, un test que ya no puede fallar parece lo que no es: un guard que se
/// rompe en silencio deja entrar justo lo que deberia impedir.
#[test]
fn the_ui_toolkit_guard_recognises_a_real_dependency() {
    let manifest = "[dependencies]\niced = \"0.13\"\neframe = \"0.29\"\negui_extras = { version = \"0.29\" }\n";

    assert!(depends_on(manifest, "iced"), "no ve una dependencia normal");
    assert!(!depends_on(manifest, "slint"), "no ve la que no esta");
    assert!(
        !depends_on(manifest, "egui"),
        "el stack elegido no es un toolkit prohibido, y `egui_extras` es parte de el"
    );
    assert!(
        depends_on("[dev-dependencies]\n  slint = \"1\"\n", "slint"),
        "no ve una dependencia de desarrollo, que tambien seria un segundo toolkit"
    );
}

/// La decision del stack tiene que estar en la guia del agente, y no solo en los
/// documentos del frontend.
///
/// `AGENTS.md` es lo que se lee antes de tocar nada, y ya dice que Tauri queda
/// fuera; si no dice que es lo que entra, quien llegue aqui no tiene con que
/// trabajar y acaba reabriendo una decision que `docs/tasks.md` y
/// `docs/frontend-tasks.md` dan por cerrada.
#[test]
fn the_agent_guide_declares_the_frontend_stack() {
    let guide = read_repo_file("AGENTS.md");

    assert!(
        guide.contains("egui") && guide.contains("eframe"),
        "AGENTS.md must declare egui/eframe as the frontend stack, so the decision is not reopened in a task"
    );
}
