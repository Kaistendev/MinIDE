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

    assert!(config.contains("msrv = \"1.95\""));
}

/// La version minima que declara el lint y la que declara el crate tienen que ser la
/// misma.
///
/// Clippy usa `msrv` para no sugerir APIs mas nuevas de las que se puede usar, asi
/// que si las dos cifras se separan, el lint deja de proteger justo lo que dice
/// proteger. Addadir el stack de la interfaz subio el minimo a `1.95` y las cuatro
/// menciones de la cifra hubo que cambiarlas a mano.
#[test]
fn the_lint_msrv_matches_the_crate_rust_version() {
    let clippy = read_repo_file("clippy.toml");
    let manifest = read_repo_file("Cargo.toml");

    let lint = declared(&clippy, "msrv = ");
    let manifest_version = declared(&manifest, "rust-version = ");

    assert_eq!(
        lint, manifest_version,
        "clippy.toml y Cargo.toml tienen que declarar la misma version minima"
    );
}

/// El valor de una entrada `clave = valor` del primer lugar donde aparece.
fn declared(content: &str, prefix: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let line = line.trim();

        line.starts_with(prefix)
            .then(|| line[prefix.len()..].trim_matches('"').to_string())
    })
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

/// El nombre y la razon escrita de cada dependencia declarada.
///
/// La razon son las lineas de comentario que hay justo encima de la dependencia, que
/// es donde se explica para que esta.
fn declared_dependencies(manifest: &str) -> Vec<(String, String)> {
    let mut declaradas = Vec::new();
    let mut seccion = String::new();
    let mut comentario: Vec<String> = Vec::new();

    for line in manifest.lines() {
        let linea = line.trim();

        if linea.starts_with('[') {
            seccion = linea.to_string();
            comentario.clear();
            continue;
        }

        if linea.starts_with('#') {
            comentario.push(linea.to_string());
            continue;
        }

        if linea.is_empty() {
            continue;
        }

        if seccion.contains("dependencies") {
            if let Some((clave, _)) = linea.split_once('=') {
                declaradas.push((clave.trim().to_string(), comentario.join(" ")));
            }
        }

        comentario.clear();
    }

    declaradas
}

/// T-091: cada dependencia declarada se usa en el crate y lleva su razon escrita.
///
/// Una dependencia que nadie usa es peso muerto que se compila en cada build; y una
/// dependencia sin la razon escrita al lado es la que acaba creciendo sin que nadie
/// sepa para que estaba.
#[test]
fn every_dependency_is_used_and_explained() {
    let manifest = read_repo_file("Cargo.toml");
    let fuentes = crate_sources();

    for (nombre, razon) in declared_dependencies(&manifest) {
        assert!(
            !razon.is_empty(),
            "`{nombre}` no lleva la razon escrita encima en Cargo.toml"
        );
        assert!(
            razon.contains(&nombre),
            "la razon de `{nombre}` no lo nombra, asi que no se sabe a cual se refiere: {razon:?}"
        );
        assert!(
            contains_identifier(&fuentes, &nombre),
            "`{nombre}` esta declarada en Cargo.toml y no se usa en el crate"
        );
    }
}

/// El guard tiene que reconocer una dependencia de verdad y no confundir los
/// comentarios con declaraciones.
#[test]
fn the_dependency_guard_reads_the_manifest() {
    let manifest = "\
[dependencies]
# Para la ventana.
eframe = { version = \"0.36\" }
image = \"0.25\"

[dev-dependencies]
# Para las pruebas.
serde = \"1\"
";

    let declaradas = declared_dependencies(manifest);

    assert_eq!(
        declaradas,
        vec![
            ("eframe".to_string(), "# Para la ventana.".to_string()),
            ("image".to_string(), String::new()),
            ("serde".to_string(), "# Para las pruebas.".to_string()),
        ]
    );
}

/// Todos los `.rs` del crate: `src/`, `tests/` y el `build.rs`.
fn crate_sources() -> String {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut todo = String::new();

    for carpeta in ["src", "tests"] {
        collect_rs(&raiz.join(carpeta), &mut todo);
    }

    if let Ok(build) = fs::read_to_string(raiz.join("build.rs")) {
        todo.push_str(&build);
    }

    todo
}

/// Anade a `todo` el texto de cada `.rs` que haya bajo `dir`, recursivamente.
fn collect_rs(dir: &std::path::Path, todo: &mut String) {
    let Ok(entradas) = fs::read_dir(dir) else {
        return;
    };

    for entrada in entradas.flatten() {
        let ruta = entrada.path();

        if ruta.is_dir() {
            collect_rs(&ruta, todo);
        } else if ruta.extension().is_some_and(|extension| extension == "rs") {
            if let Ok(contenido) = fs::read_to_string(&ruta) {
                todo.push_str(&contenido);
            }
        }
    }
}

/// Si `name` aparece como un identificador suelto en `text`.
///
/// Partir por lo que no es un identificador evita que `image` cuente por aparecer
/// dentro de `imagebar` o de una palabra cualquiera.
fn contains_identifier(text: &str, name: &str) -> bool {
    text.split(|caracter: char| !caracter.is_alphanumeric() && caracter != '_')
        .any(|token| token == name)
}
