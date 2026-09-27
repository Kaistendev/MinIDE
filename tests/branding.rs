//! La marca de la aplicación: el logo del repositorio y el icono del ejecutable.
//!
//! Son dos cosas distintas que a veces se confunden. El PNG de `assets/` es el logo y
//! lo usa la ventana; el `.ico` es lo que Windows pinta en el Explorador, en la barra
//! de tareas y en el acceso directo, y se compila dentro del ejecutable. Los dos
//! salen del mismo PNG, pero el segundo hay que regenerarlo cuando el logo cambie.

use std::fs;

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> Vec<u8> {
    let path = repo_root().join(relative);

    fs::read(&path).unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()))
}

/// Lados que Windows le pide a un icono.
///
/// Windows no reescala: elige el tamaño que necesita de los que hay. Con un solo
/// tamaño, la barra de tareas y el Explorador enseñan un logo borroso.
const LADOS: [u16; 4] = [16, 32, 48, 256];

/// El icono del ejecutable trae los tamaños que Windows usa.
///
/// Comprueba el `.ico` de verdad: su cabecera y su tabla de entradas. Es lo que
/// distingue un icono bien formado de uno que Windows no sabe leer, y es la forma de
/// que un icono regenerado a medias no llegue al ejecutable sin que nadie se entere.
#[test]
fn the_executable_icon_has_the_sizes_windows_asks_for() {
    let ico = read("assets/LogoMiniIDE.ico");

    assert!(
        ico.len() > 6,
        "un .ico son una cabecera y al menos una entrada"
    );
    assert_eq!(
        u16::from_le_bytes([ico[0], ico[1]]),
        0,
        "un .ico empieza con un reservado a cero"
    );
    assert_eq!(
        u16::from_le_bytes([ico[2], ico[3]]),
        1,
        "un .ico declara el tipo 1, que es icono"
    );

    let declarados = u16::from_le_bytes([ico[4], ico[5]]);
    assert_eq!(
        declarados,
        LADOS.len() as u16,
        "el icono tiene que traer {} tamaños",
        LADOS.len()
    );

    let mut lados = Vec::new();

    for indice in 0..declarados as usize {
        let entrada = 6 + indice * 16;
        let ancho = ico[entrada];
        let alto = ico[entrada + 1];
        let bytes = u32::from_le_bytes([
            ico[entrada + 8],
            ico[entrada + 9],
            ico[entrada + 10],
            ico[entrada + 11],
        ]) as usize;
        let offset = u32::from_le_bytes([
            ico[entrada + 12],
            ico[entrada + 13],
            ico[entrada + 14],
            ico[entrada + 15],
        ]) as usize;

        assert_eq!(
            ancho, alto,
            "un icono es cuadrado: la entrada {indice} dice {ancho}x{alto}"
        );
        assert!(
            offset + bytes <= ico.len(),
            "la entrada {indice} se sale del archivo"
        );

        // El ancho del directorio es de un byte, y 256 no cabe: el 0 significa 256.
        lados.push(if ancho == 0 { 256 } else { u16::from(ancho) });
    }

    assert_eq!(
        lados, LADOS,
        "el icono tiene que traer los lados que Windows usa"
    );
}

/// El icono se mete en el ejecutable al compilar.
///
/// Un `.ico` en `assets/` no aparece en el Explorador por mucho que este: hace falta
/// que la compilacion lo incruste en el ejecutable. Sin esto, el unico sitio donde se
/// veria el logo es dentro de la ventana.
#[test]
fn the_build_embeds_the_executable_icon() {
    let build_rs = std::fs::read_to_string(repo_root().join("build.rs"))
        .expect("src/build.rs tiene que existir");

    assert!(
        build_rs.contains("LogoMiniIDE.ico"),
        "build.rs tiene que incrustar el icono del ejecutable"
    );
    assert!(
        build_rs.contains("winresource"),
        "el icono se incruste con winresource, que es Rust puro y no necesita el SDK"
    );
    assert!(
        build_rs.contains("rerun-if-changed"),
        "build.rs tiene que declarar que depende del icono, o cambiarla no recompila"
    );
}

/// El logo del que sale todo esta en `assets/` y con el nombre que espera la
/// compilacion.
///
/// Los tres archivos salen del mismo PNG. El logo es el original y no lo usa el
/// codigo: es la fuente de la que se generan los otros dos, asi que tiene que estar a
/// mano para poder regenerarlos. El icono de la ventana lo incrusta la biblioteca y el
/// del ejecutable la compilacion, y los dos se quedan viejos en el mismo sitio si el
/// logo cambia.
#[test]
fn the_logo_and_its_executable_icon_live_in_assets() {
    let assets = repo_root().join("assets");

    assert!(
        assets.join("LogoMiniIDE.png").is_file(),
        "assets/LogoMiniIDE.png es el logo original"
    );
    assert!(
        assets.join("LogoMiniIDE-icon.png").is_file(),
        "assets/LogoMiniIDE-icon.png es el icono de la ventana, generado del logo"
    );
    assert!(
        assets.join("LogoMiniIDE.ico").is_file(),
        "assets/LogoMiniIDE.ico es el icono del ejecutable, generado del logo"
    );
}
