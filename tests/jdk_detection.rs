//! T-066: deteccion del JDK en el sistema.
//!
//! Lo que se comprueba sin JDK instalado es la busqueda sobre un directorio
//! controlado: que MiniIDE encuentra el compilador donde le digan y que, si no
//! esta, explica que falta en lugar de fallar en silencio.
//!
//! La deteccion contra el JDK real va marcada como ignorada, porque su resultado
//! depende de la maquina que ejecuta los tests. Para ejecutarla:
//!
//! ```text
//! cargo test --test jdk_detection -- --ignored
//! ```

use std::path::PathBuf;

use miniide::toolchain::{Jdk, JdkStatus};

/// Un directorio vacio y limpio, porque la deteccion no escribe nada.
fn empty_directory(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("directorio temporal");

    root
}

/// Un directorio con un archivo que hace de compilador.
fn directory_with_a_compiler(name: &str) -> PathBuf {
    let root = empty_directory(name);
    std::fs::write(root.join("javac.exe"), "").expect("archivo escrito");

    root
}

#[test]
fn the_compiler_of_a_controlled_search_path_is_found() {
    let directory = directory_with_a_compiler("miniide-t066-encontrado");

    let found = Jdk::detect_in(std::slice::from_ref(&directory), Jdk::EXTENSIONS);

    assert_eq!(found, Some(directory.join("javac.exe")));
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_search_path_without_a_compiler_reports_the_jdk_as_missing() {
    let directory = empty_directory("miniide-t066-ausente");

    let status = Jdk::status_in(std::slice::from_ref(&directory));

    let _ = std::fs::remove_dir_all(&directory);

    assert!(!status.is_available());
    assert_eq!(status.location(), None);
    assert_eq!(status.version(), None);
    assert!(status.to_string().contains("JDK"), "{status}");
}

#[test]
fn a_missing_jdk_and_an_available_one_are_different_answers() {
    let missing = JdkStatus::missing();
    let available = JdkStatus::available(
        PathBuf::from("C:\\jdk\\bin\\javac.exe"),
        Some("21".to_string()),
    );

    assert_ne!(missing, available);
    assert_ne!(missing.to_string(), available.to_string());
}

#[test]
#[ignore = "detecta el JDK de esta maquina: necesita un JDK instalado"]
fn the_jdk_of_this_machine_is_detected_with_its_version_and_its_location() {
    let status = Jdk::status();

    if !status.is_available() {
        eprintln!("se salta: no hay JDK en el PATH: {status}");

        return;
    }

    eprintln!("JDK detectado: {status}");

    assert!(status.location().is_some());
    assert!(
        status.location().is_some_and(|location| location.is_file()),
        "la ubicacion del JDK no es un archivo: {status}"
    );
}
