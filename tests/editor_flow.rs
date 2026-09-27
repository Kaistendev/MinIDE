//! Flujos del editor y del documento tal como los ve quien usa la API
//! publica: escribir, mover el cursor, seleccionar, deshacer, rehacer, buscar,
//! reemplazar y llevar el estado modificado a traves de las pestanas.
//!
//! Los unitarios de `src/` cubren metodo por metodo. Estos comprueban que las
//! piezas encadenadas se comportan, que es lo que pide T-028.

use miniide::document::{TextBuffer, TextPosition, TextRange};
use miniide::editor::{Cursor, Document, DocumentPath, OpenTabs};

fn at(line: u32, column: u32) -> TextPosition {
    TextPosition::new(line, column)
}

fn path(value: &str) -> DocumentPath {
    DocumentPath::new(value).unwrap()
}

// --- Documento vacio: la primera pulsacion de un archivo nuevo -----------------

#[test]
fn text_can_be_typed_into_an_empty_document() {
    let mut document = Document::new("");

    document.replace("hola").unwrap();

    assert_eq!(document.buffer().text(), "hola");
    assert_eq!(document.selection().at(), at(0, 4));
    assert!(document.is_modified());
}

#[test]
fn an_empty_document_can_be_undone_back_to_nothing() {
    let mut document = Document::new("");
    document.replace("hola").unwrap();

    assert!(document.undo());

    assert_eq!(document.buffer().text(), "");
    assert_eq!(document.selection().at(), at(0, 0));
    assert!(!document.is_modified());
}

#[test]
fn text_can_be_added_to_a_document_that_ends_with_a_new_line() {
    let mut document = Document::new("uno\n");

    document.insert(at(1, 0), "dos").unwrap();

    assert_eq!(document.buffer().text(), "uno\ndos");
    assert!(document.is_modified());
}

// --- Saltos de linea de Windows, que son los del sistema objetivo -------------

#[test]
fn the_cursor_moves_through_windows_line_endings() {
    let buffer = TextBuffer::new("uno\r\ndos");
    let mut cursor = Cursor::new(&buffer, at(0, 3));

    cursor.move_right(&buffer);
    assert_eq!(cursor.at(), at(0, 4));

    cursor.move_right(&buffer);
    assert_eq!(cursor.at(), at(1, 0));

    cursor.move_left(&buffer);
    assert_eq!(cursor.at(), at(0, 4));
}

#[test]
fn a_selection_over_windows_line_endings_keeps_the_carriage_return() {
    let mut document = Document::new("uno\r\ndos");

    document.select(at(0, 1), at(1, 2));

    assert_eq!(
        document
            .selection()
            .selected_text(document.buffer())
            .unwrap(),
        Some("no\r\ndo")
    );
}

#[test]
fn windows_line_endings_survive_an_edit_and_its_undo() {
    let mut document = Document::new("uno\r\ndos");

    document.insert(at(1, 3), "!").unwrap();
    assert_eq!(document.buffer().text(), "uno\r\ndos!");

    assert!(document.undo());

    assert_eq!(document.buffer().text(), "uno\r\ndos");
    assert!(!document.is_modified());
}

#[test]
fn a_search_finds_matches_in_windows_line_endings() {
    let buffer = TextBuffer::new("uno\r\ndos\r\nuno");

    let found = buffer.find_all("uno");

    assert_eq!(
        found,
        vec![
            TextRange::new(at(0, 0), at(0, 3)),
            TextRange::new(at(2, 0), at(2, 3)),
        ]
    );
}

// --- Caracteres que ocupan varios bytes ---------------------------------------

#[test]
fn a_selection_over_a_multibyte_character_gives_its_text() {
    let mut document = Document::new("áéí");

    document.select(at(0, 1), at(0, 2));

    assert_eq!(
        document
            .selection()
            .selected_text(document.buffer())
            .unwrap(),
        Some("é")
    );
}

#[test]
fn text_can_be_typed_after_a_multibyte_character() {
    let mut document = Document::new("áé");
    document.move_to(at(0, 2));

    document.replace("!").unwrap();

    assert_eq!(document.buffer().text(), "áé!");
    assert_eq!(document.selection().at(), at(0, 3));
}

#[test]
fn removing_a_multibyte_character_and_undoing_it_restores_the_text() {
    let mut document = Document::new("ñandú");

    document.remove(TextRange::new(at(0, 1), at(0, 2))).unwrap();
    assert_eq!(document.buffer().text(), "ñndú");

    assert!(document.undo());

    assert_eq!(document.buffer().text(), "ñandú");
}

// --- Flujos completos --------------------------------------------------------

#[test]
fn a_typed_selection_is_replaced_and_the_replacement_is_undone_on_its_own() {
    let mut document = Document::new("uno");

    document.move_to(at(0, 3));
    document.replace("dos").unwrap();
    document.select(at(0, 3), at(0, 6));
    document.replace("tres").unwrap();
    assert_eq!(document.buffer().text(), "unotres");

    assert!(document.undo());

    assert_eq!(document.buffer().text(), "unodos");
    assert_eq!(
        document.selection().range(),
        Some(TextRange::new(at(0, 3), at(0, 6)))
    );

    assert!(document.undo());

    assert_eq!(document.buffer().text(), "uno");
}

#[test]
fn a_replaced_selection_can_be_redone() {
    let mut document = Document::new("uno dos");
    document.select(at(0, 4), at(0, 7));

    document.replace("tres").unwrap();
    assert!(document.undo());
    assert_eq!(document.buffer().text(), "uno dos");

    assert!(document.redo());

    assert_eq!(document.buffer().text(), "uno tres");
    assert!(document.is_modified());
}

#[test]
fn everything_a_search_finds_can_be_replaced_and_undone_in_one_step() {
    let mut document = Document::new("uno dos uno tres uno");

    let found = document.buffer().find_all("uno");
    let replaced = document.replace_all("uno", "x").unwrap();

    assert_eq!(found.len(), 3);
    assert_eq!(replaced, 3);
    assert_eq!(document.buffer().text(), "x dos x tres x");

    assert!(document.undo());

    assert_eq!(document.buffer().text(), "uno dos uno tres uno");
    assert!(!document.can_undo());
    assert!(!document.is_modified());
}

#[test]
fn saving_clears_the_modified_state_of_only_that_document() {
    let mut tabs = OpenTabs::new();
    let main = tabs.open(path("src/main.cs"), "uno");
    let other = tabs.open(path("src/other.cs"), "dos");

    tabs.get_mut(main)
        .unwrap()
        .document_mut()
        .insert(at(0, 3), "!")
        .unwrap();
    tabs.get_mut(other)
        .unwrap()
        .document_mut()
        .insert(at(0, 3), "?")
        .unwrap();

    tabs.get_mut(main).unwrap().document_mut().mark_saved();

    assert!(!tabs.get(main).unwrap().is_modified());
    assert!(tabs.get(other).unwrap().is_modified());
}

#[test]
fn each_document_undoes_its_own_edits() {
    let mut tabs = OpenTabs::new();
    let main = tabs.open(path("src/main.cs"), "uno");
    let other = tabs.open(path("src/other.cs"), "dos");

    tabs.get_mut(main)
        .unwrap()
        .document_mut()
        .insert(at(0, 3), "!")
        .unwrap();
    tabs.get_mut(other)
        .unwrap()
        .document_mut()
        .insert(at(0, 3), "?")
        .unwrap();

    assert!(tabs.get_mut(main).unwrap().document_mut().undo());

    assert_eq!(tabs.get(main).unwrap().document().buffer().text(), "uno");
    assert_eq!(tabs.get(other).unwrap().document().buffer().text(), "dos?");
}

#[test]
fn closing_a_modified_document_gives_the_tab_back_to_ask_about_the_changes() {
    let mut tabs = OpenTabs::new();
    let index = tabs.open(path("src/main.cs"), "uno");
    tabs.get_mut(index)
        .unwrap()
        .document_mut()
        .insert(at(0, 3), "!")
        .unwrap();

    let closed = tabs.close(index).unwrap();

    assert!(closed.is_modified());
    assert_eq!(closed.file_name(), "main.cs");
    assert!(tabs.is_empty());
}
