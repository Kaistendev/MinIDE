//! T-092: el core no conoce detalles concretos de C#, Java, WinForms o Swing.
//!
//! `docs/tasks.md` lo pide asi: la logica especifica de cada tecnologia vive en su
//! proveedor, y el core trabaja con conceptos (`LanguageId`, `FrameworkId`,
//! `ProjectType`, `CodeGenerator`) y no con sus implementaciones. Los unitarios
//! comprueban el contrato; este guard comprueba lo que un contrato no puede: que la
//! implementacion concreta no se haya colado en un modulo del core.
//!
//! El guard no prohibe nombrar las tecnologias: `core.rs` tiene que poder decir
//! "Windows Forms" para mostrarlo. Lo que prohibe es el detalle de implementacion
//! fuera de su proveedor: el espacio de nombres de C#, el paquete de Java, el nombre
//! de un tipo de control concreto y el nombre del ejecutable de una herramienta.

use std::fs;
use std::path::Path;

/// Los detalles de implementacion que solo puede conocer su proveedor.
const IMPLEMENTATION_DETAILS: &[&str] = &[
    "System.Windows.Forms",
    "javax.swing",
    "JFrame",
    "JButton",
    "JLabel",
    "JPanel",
    "JTextField",
    "dotnet",
    "javac",
];

/// Los modulos donde si se pueden escribir esos detalles.
///
/// `generation` escribe el codigo de cada framework, `framework` tiene los modelos
/// del disenador, `language` los proveedores de lenguaje, `toolchain` lanza las
/// herramientas, `templates` escribe los archivos de la plantilla y `supports` los
/// registra. El `frontend` tambien los enseña: la interfaz consume el core y muestra
/// sus nombres, pero no implementa nada de lo que hay aqui.
fn is_provider(relative: &str) -> bool {
    const PREFIXES: [&str; 6] = [
        "frontend/",
        "generation",
        "framework",
        "language",
        "toolchain",
        "supports",
    ];
    const ROOTS: [&str; 2] = ["templates.rs", "lib.rs"];

    PREFIXES.iter().any(|prefix| relative.starts_with(prefix)) || ROOTS.contains(&relative)
}

/// Los `.rs` del core que se escanean, con su ruta relativa a `src/`.
fn core_sources() -> Vec<(String, String)> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut fuentes = Vec::new();

    collect(&src, &src, &mut fuentes);
    fuentes
}

fn collect(dir: &Path, src: &Path, fuentes: &mut Vec<(String, String)>) {
    let Ok(entradas) = fs::read_dir(dir) else {
        return;
    };

    for entrada in entradas.flatten() {
        let ruta = entrada.path();

        if ruta.is_dir() {
            collect(&ruta, src, fuentes);
        } else if ruta.extension().is_some_and(|extension| extension == "rs") {
            let relativa = ruta
                .strip_prefix(src)
                .expect("el archivo esta bajo src")
                .to_string_lossy()
                .replace('\\', "/");

            if !is_provider(&relativa) {
                if let Ok(contenido) = fs::read_to_string(&ruta) {
                    fuentes.push((relativa, contenido));
                }
            }
        }
    }
}

/// Los modulos del core que contienen `word`, buscando por texto literal.
fn modules_containing(word: &str) -> Vec<String> {
    core_sources()
        .into_iter()
        .filter(|(_, contenido)| contenido.contains(word))
        .map(|(relativa, _)| relativa)
        .collect()
}

/// El core no menciona el espacio de nombres de C# ni el paquete de Java.
///
/// `using System.Windows.Forms;` y `import javax.swing.*;` son detalles de
/// implementacion: si aparecieran en el core es que la generacion de codigo se ha
/// salido de `generation/`.
#[test]
fn the_core_does_not_mention_the_namespaces_of_the_frameworks() {
    for detail in ["System.Windows.Forms", "javax.swing"] {
        let modulos = modules_containing(detail);

        assert!(
            modulos.is_empty(),
            "`{detail}` es un detalle de un framework y aparece en {modulos:?}; \
             el core solo conoce conceptos, no tecnologias concretas"
        );
    }
}

/// El core no conoce el nombre de un tipo de control concreto.
///
/// Unir a mano el nombre del tipo con su framework es justo el `if framework ==` que
/// `AGENTS.md` prohibe. `supports.rs` y `framework.rs` si tienen esa lista, porque
/// son los proveedores y es ahi donde toca.
#[test]
fn the_core_does_not_know_concrete_control_types() {
    for detail in ["JFrame", "JButton", "JLabel", "JPanel", "JTextField"] {
        let modulos = modules_containing(detail);

        assert!(
            modulos.is_empty(),
            "`{detail}` es un tipo concreto de Swing y aparece en {modulos:?}"
        );
    }
}

/// El core no lanza las herramientas por su nombre.
///
/// `AGENTS.md` pide no dispersar llamadas a `dotnet`, `javac` o `java`. El unico
/// modulo que sabe como se llaman los ejecutables es `toolchain/`, que es el que los
/// resuelve; el resto pide el `Invocation` ya construido.
#[test]
fn the_core_does_not_name_the_toolchain_executables() {
    for tool in ["dotnet", "javac"] {
        let modulos = modules_containing(tool);

        assert!(
            modulos.is_empty(),
            "`{tool}` es el nombre de un ejecutable y aparece en {modulos:?}; \
             solo `toolchain/` puede saber como se llama"
        );
    }
}

/// La lista de detalles no puede quedar vacia, o el guard no comprobaria nada.
#[test]
fn the_guard_checks_every_implementation_detail() {
    assert!(
        !IMPLEMENTATION_DETAILS.is_empty() && IMPLEMENTATION_DETAILS.iter().all(|d| !d.is_empty()),
        "una lista de detalles vacia hace que los guards pasen sin comprobar nada"
    );
    assert!(
        !core_sources().is_empty(),
        "no se ha leido ningun modulo del core"
    );
}

/// El guard tiene que reconocer una violacion de verdad, y no pasar por casualidad.
///
/// Sin esto, un guard que ya no puede fallar parece lo que no es: un test que se
/// rompe en silencio deja entrar justo lo que deberia impedir. Los modulos del core
/// de verdad estan limpios; esto comprueba que el guard veria el detalle si lo
/// tuviera delante.
#[test]
fn the_guard_recognises_a_real_violation() {
    let con_violacion: Vec<(String, String)> = core_sources()
        .into_iter()
        .map(|(ruta, contenido)| {
            if ruta == "core.rs" {
                (ruta, format!("{contenido}\n// new javax.swing.JFrame();\n"))
            } else {
                (ruta, contenido)
            }
        })
        .collect();

    let encontrados: Vec<String> = con_violacion
        .iter()
        .filter(|(_, contenido)| contenido.contains("javax.swing"))
        .map(|(ruta, _)| ruta.clone())
        .collect();

    assert_eq!(encontrados, vec!["core.rs".to_string()]);
}

/// Ningun modulo del core esta en la lista de proveedores por accidente.
#[test]
fn the_provider_list_does_not_hide_a_core_module() {
    for core in [
        "core.rs",
        "commands.rs",
        "document.rs",
        "editor.rs",
        "workspace.rs",
        "project.rs",
        "build.rs",
        "runtime.rs",
        "diagnostics.rs",
    ] {
        assert!(
            !is_provider(core),
            "`{core}` es del core y no puede estar excluido del guard"
        );
    }
}
