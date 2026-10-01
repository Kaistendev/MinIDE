//! Lo que la ventana puede y no puede tener. FE-076 y FE-079.
//!
//! Estos dos tests no miran la ventana dibujada sino su código, y son los únicos del
//! proyecto que se leen los fuentes del frontend. Las dos reglas no se pueden ver en un
//! píxel, y por eso se comprueban donde se rompen:
//!
//! * FE-076 pide que una acción equivalente se llame igual en el menú, en la barra y en
//!   los mensajes. El menú y la barra ya comparten las mismas constantes, así que lo
//!   único que puede desunirlas es un módulo que escriba el nombre a su mano: el botón
//!   del diálogo de cerrar que pone "Guardar" y el campo de búsqueda que se llama
//!   "Buscar" son dos sitios donde el nombre puede quedarse atrás si alguien renombra la
//!   acción.
//! * FE-079 pide que ningún widget decida cómo se compila, cómo se ejecuta una
//!   herramienta o qué código se genera. Eso no se ve en un píxel porque no debería
//!   existir: la ventana pide, y quien compila y quien genera son el core y sus
//!   proveedores.
//!
//! Los dos leen el mismo código por la misma razón: un `contains` sobre el archivo
//! entero se encontraría a sí mismo con los nombres y las palabras que busca, así que
//! de cada fuente solo se lee lo que hay antes de `#[cfg(test)]` y lo que no es un
//! comentario.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Los dos únicos sitios del frontend que pueden saber de herramientas.
///
/// `app.rs` es el que recibe el proveedor para pasárselo a la ventana, y `operaciones.rs`
/// es el que compila y ejecuta. Cualquier otro módulo que los mencione está decidiendo
/// algo que no le toca (FE-079).
const CON_HERRAMIENTAS: &[&str] = &["app.rs", "operaciones.rs"];

/// El archivo donde se escriben los nombres de las acciones, una sola vez.
const ACCIONES: &str = "acciones.rs";

#[test]
fn un_nombre_de_accion_se_escribe_solo_donde_esta_la_accion() {
    let nombres = nombres_de_las_acciones();

    assert!(
        !nombres.is_empty(),
        "las acciones tienen que tener nombres, o este test no mira nada"
    );

    for fuente in fuentes() {
        if fuente.file_name().is_some_and(|nombre| nombre == ACCIONES) {
            continue;
        }

        if let Some(nombre) = nombres
            .intersection(&literales(&codigo_sin_titulos(&fuente)))
            .next()
        {
            panic!(
                "{:?} escribe el nombre de una acción, {:?}, y ese nombre se cambia en un \
                 sitio solo: se pide a `acciones` la acción y se usa su nombre",
                nombre_archivo(&fuente),
                nombre
            );
        }
    }
}

/// El código de un fuente sin los títulos de los menús.
///
/// Los títulos son nombres de sección —"Archivo", "Editar", "Compilar"— y no nombres de
/// acción, así que no cuentan: "Compilar" es a la vez el nombre de un menú y el de una
/// acción, y que se escriban igual en los dos sitios es lo que hace que el menú y el botón
/// se llamen como la operación que tienen. Lo que no puede pasar es que uno de los dos se
/// quede atrás.
fn codigo_sin_titulos(fuente: &Path) -> String {
    codigo(fuente)
        .lines()
        .filter(|linea| !linea.trim_start().starts_with("titulo:"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn la_ventana_no_lanza_procesos_ni_dice_el_nombre_de_una_herramienta() {
    for fuente in fuentes() {
        let codigo = codigo(&fuente);

        for prohibido in ["std::process", "std::process::Command"] {
            assert!(
                !codigo.contains(prohibido),
                "{:?} lanza procesos y lanzar procesos es del runtime del core: {prohibido}",
                nombre_archivo(&fuente)
            );
        }

        for herramienta in [
            "\"dotnet\"",
            "\"javac\"",
            "\"java\"",
            "\"javadoc\"",
            "\"javap\"",
        ] {
            assert!(
                !codigo.contains(herramienta),
                "{:?} nombra una herramienta externa, y el nombre de las herramientas lo \
                 tienen los proveedores del toolchain: {herramienta}",
                nombre_archivo(&fuente)
            );
        }
    }
}

#[test]
fn solo_los_dos_sitios_de_las_herramientas_conocen_el_build_y_el_toolchain() {
    for fuente in fuentes() {
        let nombre = nombre_archivo(&fuente);

        if CON_HERRAMIENTAS.contains(&nombre.as_str()) {
            continue;
        }

        let codigo = codigo(&fuente);

        for modulo in ["crate::build", "crate::toolchain"] {
            assert!(
                !codigo.contains(modulo),
                "{:?} usa {modulo}, y un widget que compila o que pregunta por las \
                 herramientas se ha pasado de ventana: eso es de {CON_HERRAMIENTAS:?}",
                nombre
            );
        }
    }
}

#[test]
fn solo_el_punto_de_ejecucion_arranca_una_compilacion_y_mira_si_ha_ido_bien() {
    for fuente in fuentes() {
        let nombre = nombre_archivo(&fuente);

        if nombre == "operaciones.rs" {
            continue;
        }

        let codigo = codigo(&fuente);

        for llamado in ["start_build(", "succeeded("] {
            assert!(
                !codigo.contains(llamado),
                "{:?} decide cómo va una compilación con {llamado}, y esa decisión es de \
                 `operaciones.rs`: los demás solo enseñan el estado que le llega",
                nombre
            );
        }
    }
}

#[test]
fn la_ventana_no_escribe_archivos_ella_misma() {
    for fuente in fuentes() {
        let codigo = codigo(&fuente);

        for escritura in [
            "fs::write",
            "fs::create_dir",
            "fs::remove_",
            "fs::rename",
            "File::create",
        ] {
            assert!(
                !codigo.contains(escritura),
                "{:?} escribe en el disco con {escritura}, y guardar un documento es \
                 trabajo del core: la ventana lo pide con un comando",
                nombre_archivo(&fuente)
            );
        }
    }
}

/// Los fuentes del frontend, ordenados para que un fallo salga siempre igual.
fn fuentes() -> Vec<PathBuf> {
    let directorio = directorio_del_frontend();

    let mut fuentes: Vec<PathBuf> = fs::read_dir(&directorio)
        .unwrap_or_else(|error| panic!("{} tiene que poder leerse: {error}", directorio.display()))
        .map(|entrada| {
            entrada
                .unwrap_or_else(|error| panic!("una entrada de {}: {error}", directorio.display()))
                .path()
        })
        .filter(|ruta| ruta.extension().is_some_and(|extension| extension == "rs"))
        .collect();
    fuentes.sort();

    assert!(
        !fuentes.is_empty(),
        "el frontend tiene que tener fuentes que mirar"
    );

    fuentes
}

/// La carpeta donde vive el frontend.
fn directorio_del_frontend() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/frontend")
}

/// El nombre del archivo, que es como se habla de él en el mensaje del fallo.
fn nombre_archivo(fuente: &Path) -> String {
    fuente
        .file_name()
        .and_then(|nombre| nombre.to_str())
        .unwrap_or("<sin nombre>")
        .to_owned()
}

/// El código de un fuente: lo que hay antes de los tests y sin comentarios.
///
/// Lo de partir por `#[cfg(test)]` es lo que deja que los tests usen archivos y
/// herramientas de verdad sin que este test los prohíba, y lo de quitar las líneas que
/// empiezan por `//` es para que un nombre del que se habla en un comentario no cuente
/// como un sitio donde se escribe.
fn codigo(fuente: &Path) -> String {
    let completo = fs::read_to_string(fuente)
        .unwrap_or_else(|error| panic!("{:?} tiene que poder leerse: {error}", fuente.display()));

    completo
        .split("#[cfg(test)]")
        .next()
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|linea| !linea.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Los textos entre comillas del código, que es lo único que el usuario ve escrito.
///
/// Se deja fuera lo que lleva barras invertidas porque ahí las comillas de dentro hacen
/// que un texto sea dos y no uno, y porque en el frontend no hay ninguno.
fn literales(codigo: &str) -> BTreeSet<String> {
    codigo
        .split('"')
        .skip(1)
        .step_by(2)
        .filter(|texto| !texto.contains('\\') && !texto.contains('\n'))
        .map(str::to_owned)
        .collect()
}

/// Los nombres de todas las acciones, leídos de donde se escriben.
///
/// No se repiten aquí a mano porque un nombre repetido en el test es un segundo sitio
/// donde cambiarlo, y este test justamente va de que no los haya.
fn nombres_de_las_acciones() -> BTreeSet<String> {
    let fuente = fs::read_to_string(directorio_del_frontend().join(ACCIONES))
        .expect("el archivo de las acciones tiene que poder leerse");

    let constantes = fuente
        .split("pub fn boton")
        .next()
        .expect("las acciones tienen que estar antes de dibujarlas");

    let mut nombres = BTreeSet::new();
    for linea in constantes.lines() {
        let Some(nombre) = linea.trim().strip_prefix("nombre: ") else {
            continue;
        };

        let nombre = nombre.trim_end_matches(',').trim();
        let entrecomillado = nombre
            .strip_prefix('"')
            .and_then(|resto| resto.strip_suffix('"'))
            .unwrap_or_else(|| panic!("una acción sin nombre entrecomillado: {linea}"));

        nombres.insert(entrecomillado.to_owned());
    }

    nombres
}
