//! T-085: tests de regresion del documento y del editor.
//!
//! Los unitarios de `src/document.rs` y `src/editor.rs` comprueban metodo por
//! metodo. Estos comprueban los estados que se rompen al encadenar operaciones:
//! un fallo a mitad de camino que deja un paso en el historial, una posicion que
//! se recalcula despues de una edicion, una busqueda que se come sus propias
//! coincidencias. Los bugs que estos tests cubren estan listados en
//! `docs/tasks.md`, junto a T-085.

use miniide::document::{TextPosition, TextRange};
use miniide::editor::{Document, DocumentPath, OpenTabs, Tab};

fn at(line: u32, column: u32) -> TextPosition {
    TextPosition::new(line, column)
}

/// Un documento con una sola palabra, para los casos que no necesitan mas.
fn documento() -> Document {
    Document::new("hola")
}

/// El texto que hay en `range`, o el fallo del core si el rango no existe.
fn texto_de(document: &Document, range: TextRange) -> String {
    document
        .buffer()
        .text_in(range)
        .expect("el rango existe")
        .to_string()
}

/// La busqueda tiene que avanzar por el texto y no quedarse en la misma
/// coincidencia.
///
/// Un `find_all` quebuscara siempre desde el principio de lo ya encontrado
/// devolveria la misma coincidencia para siempre; uno que avanza solo un caracter
/// contaria dos veces las que se solapan.
#[test]
fn a_search_reports_each_match_once_even_when_they_overlap() {
    let document = Document::new("aaaa");

    let found = document.buffer().find_all("aa");

    let ranges: Vec<(u32, u32)> = found
        .iter()
        .map(|range| (range.start().column(), range.end().column()))
        .collect();

    assert_eq!(ranges, vec![(0, 2), (2, 4)], "{found:?}");
}

/// Un texto vacio no se puede editar, pero inserting en el no es un error.
///
/// La posicion (0,0) existe en un documento sin texto: es lo unico que se puede
/// insertar. Si `insert` la rechazara, no habria forma de escribir en un archivo
/// nuevo.
#[test]
fn an_empty_document_can_be_written_into_at_its_only_position() {
    let mut document = Document::new("");

    document.insert(at(0, 0), "nuevo").unwrap();

    assert_eq!(document.buffer().text(), "nuevo");
    assert!(document.is_modified());
}

/// Una posicion que no existe se rechaza y el documento se queda como estaba.
///
/// `insert` apila el estado anterior en el historial antes de tocar el buffer. Si
/// la escritura fallara, el undo devolveria un documento que el usuario nunca vio
/// y `can_undo` diria que hay algo que deshacer.
#[test]
fn a_refused_edit_changes_nothing_and_leaves_nothing_to_undo() {
    let mut document = documento();

    let result = document.insert(at(5, 0), "lejos");

    assert!(result.is_err(), "una linea que no existe se rechaza");
    assert_eq!(document.buffer().text(), "hola");
    assert!(!document.is_modified());
    assert!(!document.can_undo(), "un fallo no deja un paso de deshacer");
    assert!(!document.can_redo());
}

/// Lo mismo con una columna que se sale de su linea.
///
/// `hola` tiene cuatro caracteres en la linea 0: la columna 4 es el final de la
/// linea y la columna 5 ya esta en otra parte del documento que este no tiene.
#[test]
fn a_column_past_the_end_of_its_line_is_refused() {
    let mut document = Document::new("hola\nadios");

    assert!(document.insert(at(0, 5), "x").is_err());
    assert_eq!(document.buffer().text(), "hola\nadios");
    assert!(!document.can_undo());
}

/// El cursor sube y baja sin quedarse en una columna que no existe.
///
/// De una linea larga a una corta, la columna se recorta al final de la corta; de
/// una corta a una larga, se queda donde estaba en vez de pegarse al final.
#[test]
fn moving_vertically_keeps_the_column_inside_the_line_it_lands_on() {
    let mut document = Document::new("abcd\nab\nefghij");

    document.move_to(at(0, 4));
    document.move_down();
    assert_eq!(document.selection().at(), at(1, 2), "columna recortada");

    document.move_down();
    assert_eq!(
        document.selection().at(),
        at(2, 2),
        "la columna no se pega al final de la linea larga"
    );

    document.move_up();
    assert_eq!(document.selection().at(), at(1, 2));
    document.move_up();
    assert_eq!(document.selection().at(), at(0, 2));
}

/// En los extremos del documento el cursor no se sale.
///
/// Mover a la izquierda estando en (0,0) o a la derecha estando al final no puede
/// inventar una posicion anterior a la primera ni posterior a la ultima.
#[test]
fn the_cursor_stays_inside_the_document_at_its_edges() {
    let mut document = documento();

    document.move_left();
    assert_eq!(document.selection().at(), at(0, 0));

    for _ in 0..10 {
        document.move_right();
    }
    assert_eq!(document.selection().at(), at(0, 4), "no pasa del final");

    document.move_up();
    assert_eq!(
        document.selection().at(),
        at(0, 4),
        "no sube de la primera linea"
    );
    document.move_down();
    assert_eq!(
        document.selection().at(),
        at(0, 4),
        "no baja de la ultima linea"
    );
}

/// Reemplazar no se come las coincidencias que crea el propio reemplazo.
///
/// Cambiar `a` por `aa` termina en un texto finito. Un reemplazo que volviera a
/// buscar sobre el texto ya cambiado entraria en un bucle infinito, que es como
/// se colgaba el reemplazar todo.
#[test]
fn replacing_everything_terminates_when_the_replacement_contains_the_search() {
    let mut document = Document::new("a");

    let changed = document.replace_all("a", "aa").unwrap();

    assert_eq!(changed, 1);
    assert_eq!(document.buffer().text(), "aa");
}

/// Lo mismo buscando una vez: el rango se procesa, no se vuelve a buscar.
///
/// El reemplazo contiene lo que se buscaba. Si `replace_match` volviera a buscar
/// por su cuenta, el texto crecia sin fin; aqui solo se toca el rango dado.
#[test]
fn replacing_a_match_that_contains_the_search_replaces_it_only_once() {
    let mut document = Document::new("hola");
    let found = document.buffer().find_all("hola");

    document.replace_match(found[0], "hola mundo").unwrap();

    assert_eq!(document.buffer().text(), "hola mundo");

    document.undo();
    assert_eq!(document.buffer().text(), "hola");
}

/// Un rango al reves da el mismo texto que al derecho.
///
/// La seleccion se normaliza al moverse, asi que da igual en que direccion se
/// haya arrastrado el raton: lo seleccionado es lo mismo.
#[test]
fn a_selection_made_backwards_reads_the_same_text() {
    let mut document = Document::new("abcdef");

    document.select(at(0, 4), at(0, 1));
    assert_eq!(
        texto_de(&document, document.selection().range().unwrap()),
        "bcd"
    );

    document.select(at(0, 1), at(0, 4));
    assert_eq!(
        texto_de(&document, document.selection().range().unwrap()),
        "bcd"
    );
}

/// Borrar un rango que cruza el fin de linea une las lineas que quedan.
///
/// El texto se queda pegado: no se cuela un `\n` ni se pierde el caracter de la
/// izquierda al hacer la cuenta del rango.
#[test]
fn removing_a_range_across_lines_joins_what_is_left() {
    let mut document = Document::new("uno\ndos\ntres");

    document.remove(TextRange::new(at(0, 2), at(1, 2))).unwrap();

    assert_eq!(document.buffer().text(), "uns\ntres");
    assert!(document.undo());
    assert_eq!(document.buffer().text(), "uno\ndos\ntres");
}

/// Escribir en el documento cambia su estado, deshacer lo devuelve y volver a
/// escribir tira lo que se habia deshecho.
///
/// El historial tiene que seguir siendo el de la rama actual: deshacer, escribir
/// otra vez y deshacer otra vez quita lo ultimo, no lo que quedo antes del primer
/// deshacer.
#[test]
fn writing_again_after_an_undo_only_undoes_the_new_edit() {
    let mut document = Document::new("");

    document.insert(at(0, 0), "uno").unwrap();
    document.insert(at(0, 3), " dos").unwrap();
    assert_eq!(document.buffer().text(), "uno dos");

    document.undo();
    assert_eq!(document.buffer().text(), "uno");
    assert!(document.can_redo());

    document.insert(at(0, 3), " tres").unwrap();
    assert_eq!(document.buffer().text(), "uno tres");
    assert!(!document.can_redo(), "la rama deshecha se pierde");

    document.undo();
    assert_eq!(document.buffer().text(), "uno");
}

/// Deshacer y rehacer devuelven el texto exacto, con el `\r\n` de Windows.
///
/// El historial guarda el texto entero, no una lista de diferencias: si el
/// borrador guardara solo lo escrito, volver a deshacer un cambio con finales de
/// linea de Windows perderia el `\r`.
#[test]
fn undo_and_redo_give_back_windows_line_endings() {
    let mut document = Document::new("uno\r\ndos");

    document.insert(at(1, 3), "\r\ntres").unwrap();
    assert_eq!(document.buffer().text(), "uno\r\ndos\r\ntres");

    document.undo();
    assert_eq!(document.buffer().text(), "uno\r\ndos");

    document.redo();
    assert_eq!(document.buffer().text(), "uno\r\ndos\r\ntres");
}

/// Guardar no deshace el historial ni el estado modificado.
///
/// Guardar es un cambio en el disco, no en el documento: despues de guardar se
/// sigue pudiendo deshacer, y el documento vuelve a estar modificado al escribir
/// otra vez.
#[test]
fn saving_keeps_the_history_and_a_later_edit_marks_the_document_again() {
    let ruta = temporal("miniide-t085-guardar");
    let mut tab = Tab::new(DocumentPath::new(&ruta).unwrap(), Document::new("hola"));

    tab.document_mut().insert(at(0, 4), " mundo").unwrap();
    tab.save().unwrap();

    assert!(!tab.is_modified(), "guardar limpia lo modificado");
    assert!(tab.document().can_undo(), "guardar no vacia el historial");

    tab.document_mut().insert(at(0, 10), "!").unwrap();
    assert!(tab.is_modified(), "escribir despues vuelve a marcar");

    let _ = std::fs::remove_file(&ruta);
}

/// Cerrar una pestana y volver a abrir el mismo archivo empieza sin historial.
///
/// Reabrir no hereda el documento anterior: si lo heredara, un deshacer podria
/// escribir sobre un archivo que el usuario ya habia cerrado y vuelto a abrir
/// con otro contenido.
#[test]
fn reopening_a_closed_document_starts_without_its_history() {
    let ruta = temporal("miniide-t085-reabrir");
    std::fs::write(&ruta, "hola").unwrap();

    let mut tabs = OpenTabs::new();
    tabs.open(DocumentPath::new(&ruta).unwrap(), "hola");
    tabs.get_mut(0)
        .unwrap()
        .document_mut()
        .insert(at(0, 4), " mundo")
        .unwrap();
    assert!(tabs.close(0).is_some());

    tabs.open(DocumentPath::new(&ruta).unwrap(), "hola");
    let tab = tabs.get(0).unwrap();

    assert!(
        !tab.document().can_undo(),
        "el documento nuevo no trae historial"
    );
    assert!(!tab.is_modified());
    assert_eq!(tab.document().buffer().text(), "hola");

    let _ = std::fs::remove_file(&ruta);
}

/// Cerrar la pestana activa deja activa la que quede.
///
/// Un indice de mas al cerrar hacia que la pestana activa fuese la siguiente en
/// vez de la que tocaba, y el editor abria un documento que el usuario no habia
/// pedido.
#[test]
fn closing_a_tab_leaves_the_previous_one_active() {
    let mut tabs = OpenTabs::new();

    tabs.open(path("uno.cs"), "uno");
    tabs.open(path("dos.cs"), "dos");
    tabs.open(path("tres.cs"), "tres");
    assert_eq!(tabs.active(), Some(2));

    tabs.close(2);

    assert_eq!(tabs.active(), Some(1));
    assert_eq!(tabs.get(1).unwrap().file_name(), "dos.cs");
}

/// Abrir dos veces el mismo archivo no crea dos pestanas.
///
/// Las pestanas se identifican por su ruta: el mismo archivo abierto otra vez se
/// elige, no se duplica, para que no haya dos copias del mismo documento
/// desincronizadas.
#[test]
fn opening_the_same_document_twice_reuses_its_tab() {
    let mut tabs = OpenTabs::new();

    let first = tabs.open(path("Main.cs"), "hola");
    tabs.get_mut(first)
        .unwrap()
        .document_mut()
        .insert(at(0, 4), " mundo")
        .unwrap();
    let second = tabs.open(path("Main.cs"), "hola");

    assert_eq!(first, second);
    assert_eq!(tabs.len(), 1);
    assert_eq!(
        tabs.get(first).unwrap().document().buffer().text(),
        "hola mundo",
        "la segunda apertura no borra lo escrito en la primera"
    );
}

/// Una ruta de documento no puede estar vacia.
///
/// Una ruta sin nombre de archivo no es un documento que se pueda abrir, asi que
/// se rechaza al construirla y no cuando ya hay una pestana con algo que no es un
/// archivo. Que una ruta sea un directorio de verdad lo dice el disco, y eso lo
/// comprueba `Tab::save`, no el constructor.
#[test]
fn a_document_path_needs_a_real_path() {
    assert!(DocumentPath::new("").is_err());
    assert_eq!(
        DocumentPath::new("src/Main.cs").unwrap().file_name(),
        "Main.cs"
    );
}

/// El archivo que se guarda es el que dice la pestana.
///
/// Guardar escribe donde dice la ruta y no en el directorio de trabajo: si se
/// escribiera donde este el proceso, el archivo del proyecto se perderia.
#[test]
fn saving_writes_to_the_path_of_the_tab_and_nowhere_else() {
    let ruta = temporal("miniide-t085-donde");
    let mut tab = Tab::new(DocumentPath::new(&ruta).unwrap(), Document::new("hola"));

    tab.document_mut().insert(at(0, 4), " mundo").unwrap();
    tab.save().unwrap();

    assert_eq!(std::fs::read_to_string(&ruta).unwrap(), "hola mundo");

    let _ = std::fs::remove_file(&ruta);
}

/// Una ruta temporal y un nombre de archivo, para las pruebas que tocan disco.
fn temporal(nombre: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(nombre)
}

fn path(valor: &str) -> DocumentPath {
    DocumentPath::new(valor).expect("una ruta de documento")
}
