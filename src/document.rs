use crate::core::{CoreError, CoreResult};

/// Posicion dentro de un documento.
///
/// `line` y `column` empiezan en cero y `column` se cuenta en caracteres, no
/// en bytes: en `"áé"` la columna 1 cae entre `á` y `é`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct TextPosition {
    line: u32,
    column: u32,
}

impl TextPosition {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn column(&self) -> u32 {
        self.column
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRange {
    start: TextPosition,
    end: TextPosition,
}

impl TextRange {
    pub fn new(first: TextPosition, second: TextPosition) -> Self {
        if first <= second {
            Self {
                start: first,
                end: second,
            }
        } else {
            Self {
                start: second,
                end: first,
            }
        }
    }

    pub fn collapsed(at: TextPosition) -> Self {
        Self::new(at, at)
    }

    pub fn start(&self) -> TextPosition {
        self.start
    }

    pub fn end(&self) -> TextPosition {
        self.end
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn contains(&self, position: TextPosition) -> bool {
        if self.is_empty() {
            position == self.start
        } else {
            self.start <= position && position < self.end
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextBuffer {
    text: String,
}

impl TextBuffer {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Texto que cubre `range`, sin incluir el caracter de la posicion final.
    pub fn text_in(&self, range: TextRange) -> CoreResult<&str> {
        let start = self.offset_at(range.start())?;
        let end = self.offset_at(range.end())?;

        Ok(&self.text[start..end])
    }

    /// Rangos de todas las apariciones de `needle`, de izquierda a derecha y
    /// sin solaparse.
    ///
    /// La busqueda es literal y distingue mayusculas, y puede cruzar lineas.
    /// Una `needle` vacia no da ninguna coincidencia. Devuelve `TextRange` y no
    /// texto para que el llamador pueda marcar la coincidencia o borrarla.
    pub fn find_all(&self, needle: &str) -> Vec<TextRange> {
        if needle.is_empty() {
            return Vec::new();
        }

        let mut found = Vec::new();
        let mut from = 0;

        while let Some(at) = self.text[from..].find(needle) {
            let start = from + at;
            let end = start + needle.len();

            found.push(TextRange::new(
                self.position_at(start),
                self.position_at(end),
            ));

            from = end;
        }

        found
    }

    /// Posicion del caracter que empieza en el byte `offset`. Es el inverso de
    /// `offset_at`: cuenta caracteres, no bytes.
    fn position_at(&self, offset: usize) -> TextPosition {
        let mut line = 0;
        let mut column = 0;

        for character in self.text[..offset].chars() {
            if character == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }

        TextPosition::new(line, column)
    }

    /// Numero de lineas del documento. Un documento vacio tiene una linea.
    pub(crate) fn line_count(&self) -> u32 {
        self.text.bytes().filter(|byte| *byte == b'\n').count() as u32 + 1
    }

    /// Longitud en caracteres de la linea `line`, contando desde cero.
    ///
    /// Devuelve 0 si la linea no existe.
    pub(crate) fn line_length(&self, line: u32) -> u32 {
        let Ok(start) = self.line_start(line) else {
            return 0;
        };

        let rest = &self.text[start..];
        let end = rest.find('\n').unwrap_or(rest.len());

        rest[..end].chars().count() as u32
    }

    /// Inserta `text` en la posicion `at`.
    ///
    /// Devuelve `CoreError::InvalidPosition` si la linea o la columna no
    /// existen en el documento, y en ese caso el texto no cambia.
    pub fn insert(&mut self, at: TextPosition, text: impl Into<String>) -> CoreResult<()> {
        let offset = self.offset_at(at)?;

        self.text.insert_str(offset, &text.into());

        Ok(())
    }

    /// Borra el texto del rango `range`.
    ///
    /// Un rango vacio no cambia el texto. Si algun extremo no existe en el
    /// documento devuelve `CoreError::InvalidPosition` y no borra nada.
    /// Borrar un rango une las lineas que quedan a su alrededor.
    pub fn remove(&mut self, range: TextRange) -> CoreResult<()> {
        let start = self.offset_at(range.start())?;
        let end = self.offset_at(range.end())?;

        self.text.replace_range(start..end, "");

        Ok(())
    }

    fn offset_at(&self, at: TextPosition) -> CoreResult<usize> {
        let start = self.line_start(at.line())?;
        let rest = &self.text[start..];
        let mut column = 0;

        for (index, character) in rest.char_indices() {
            if column == at.column() {
                return Ok(start + index);
            }

            if character == '\n' {
                break;
            }

            column += 1;
        }

        if column == at.column() {
            return Ok(start + rest.len());
        }

        Err(CoreError::InvalidPosition(format!(
            "columna {} fuera de la linea {}",
            at.column(),
            at.line()
        )))
    }

    fn line_start(&self, line: u32) -> CoreResult<usize> {
        if line == 0 {
            return Ok(0);
        }

        let mut current = 0;

        for (index, byte) in self.text.bytes().enumerate() {
            if byte == b'\n' {
                current += 1;

                if current == line {
                    return Ok(index + 1);
                }
            }
        }

        Err(CoreError::InvalidPosition(format!(
            "linea {line} fuera del documento"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::CoreError;

    fn position(line: u32, column: u32) -> TextPosition {
        TextPosition::new(line, column)
    }

    #[test]
    fn a_buffer_stores_the_text_it_was_created_with() {
        let buffer = TextBuffer::new("class Main {}");

        assert_eq!(buffer.text(), "class Main {}");
    }

    #[test]
    fn a_default_buffer_is_empty() {
        let buffer = TextBuffer::default();

        assert_eq!(buffer.text(), "");
    }

    #[test]
    fn a_buffer_keeps_line_endings_untouched() {
        let buffer = TextBuffer::new("using System;\r\n\r\nclass Main {}\r\n");

        assert_eq!(buffer.text(), "using System;\r\n\r\nclass Main {}\r\n");
    }

    #[test]
    fn inserting_at_the_beginning_of_the_document() {
        let mut buffer = TextBuffer::new("uno\ndos");

        buffer.insert(position(0, 0), "X").unwrap();

        assert_eq!(buffer.text(), "Xuno\ndos");
    }

    #[test]
    fn inserting_at_the_end_of_the_document() {
        let mut buffer = TextBuffer::new("uno\ndos");

        buffer.insert(position(1, 3), "!").unwrap();

        assert_eq!(buffer.text(), "uno\ndos!");
    }

    #[test]
    fn inserting_in_the_middle_of_a_line() {
        let mut buffer = TextBuffer::new("uno\ndos");

        buffer.insert(position(1, 1), "-").unwrap();

        assert_eq!(buffer.text(), "uno\nd-os");
    }

    #[test]
    fn inserting_at_the_start_of_a_later_line() {
        let mut buffer = TextBuffer::new("uno\ndos");

        buffer.insert(position(1, 0), "X").unwrap();

        assert_eq!(buffer.text(), "uno\nXdos");
    }

    #[test]
    fn inserting_at_the_end_of_a_line_keeps_the_line_break() {
        let mut buffer = TextBuffer::new("uno\ndos");

        buffer.insert(position(0, 3), "!").unwrap();

        assert_eq!(buffer.text(), "uno!\ndos");
    }

    #[test]
    fn a_carriage_return_counts_as_part_of_its_line() {
        let mut buffer = TextBuffer::new("uno\r\ndos");

        buffer.insert(position(0, 4), "!").unwrap();

        assert_eq!(buffer.text(), "uno\r!\ndos");
    }

    #[test]
    fn inserting_into_an_empty_buffer_stores_the_text() {
        let mut buffer = TextBuffer::default();

        buffer.insert(position(0, 0), "uno").unwrap();

        assert_eq!(buffer.text(), "uno");
    }

    #[test]
    fn insert_counts_columns_in_characters_and_not_in_bytes() {
        let mut buffer = TextBuffer::new("áé");

        buffer.insert(position(0, 1), "X").unwrap();

        assert_eq!(buffer.text(), "áXé");
    }

    #[test]
    fn inserting_an_empty_text_leaves_the_buffer_untouched() {
        let mut buffer = TextBuffer::new("uno");

        buffer.insert(position(0, 1), "").unwrap();

        assert_eq!(buffer.text(), "uno");
    }

    #[test]
    fn inserting_at_a_line_that_does_not_exist_is_rejected() {
        let mut buffer = TextBuffer::new("uno\ndos");

        let result = buffer.insert(position(2, 0), "X");

        assert!(matches!(result, Err(CoreError::InvalidPosition(_))));
    }

    #[test]
    fn inserting_at_a_column_that_does_not_exist_is_rejected() {
        let mut buffer = TextBuffer::new("uno\ndos");

        let result = buffer.insert(position(0, 4), "X");

        assert!(matches!(result, Err(CoreError::InvalidPosition(_))));
    }

    #[test]
    fn a_rejected_insert_leaves_the_buffer_untouched() {
        let mut buffer = TextBuffer::new("uno\ndos");

        let result = buffer.insert(position(9, 9), "X");

        assert!(matches!(result, Err(CoreError::InvalidPosition(_))));
        assert_eq!(buffer.text(), "uno\ndos");
    }

    #[test]
    fn removing_a_range_inside_a_line() {
        let mut buffer = TextBuffer::new("uno\ndos");

        buffer
            .remove(TextRange::new(position(1, 1), position(1, 3)))
            .unwrap();

        assert_eq!(buffer.text(), "uno\nd");
    }

    #[test]
    fn removing_a_line_break_joins_both_lines() {
        let mut buffer = TextBuffer::new("uno\ndos");

        buffer
            .remove(TextRange::new(position(0, 1), position(1, 2)))
            .unwrap();

        assert_eq!(buffer.text(), "us");
    }

    #[test]
    fn removing_a_range_that_ends_at_the_end_of_the_document() {
        let mut buffer = TextBuffer::new("uno\ndos");

        buffer
            .remove(TextRange::new(position(0, 3), position(1, 3)))
            .unwrap();

        assert_eq!(buffer.text(), "uno");
    }

    #[test]
    fn removing_the_whole_document() {
        let mut buffer = TextBuffer::new("uno\ndos");

        buffer
            .remove(TextRange::new(position(0, 0), position(1, 3)))
            .unwrap();

        assert_eq!(buffer.text(), "");
    }

    #[test]
    fn removing_an_empty_range_leaves_the_text_untouched() {
        let mut buffer = TextBuffer::new("uno\ndos");

        buffer.remove(TextRange::collapsed(position(1, 1))).unwrap();

        assert_eq!(buffer.text(), "uno\ndos");
    }

    #[test]
    fn removing_an_empty_range_from_an_empty_document() {
        let mut buffer = TextBuffer::default();

        buffer.remove(TextRange::collapsed(position(0, 0))).unwrap();

        assert_eq!(buffer.text(), "");
    }

    #[test]
    fn removing_a_carriage_return_and_its_line_feed_together() {
        let mut buffer = TextBuffer::new("uno\r\ndos");

        buffer
            .remove(TextRange::new(position(0, 3), position(1, 0)))
            .unwrap();

        assert_eq!(buffer.text(), "unodos");
    }

    #[test]
    fn a_range_built_backwards_removes_the_same_text() {
        let mut buffer = TextBuffer::new("uno\ndos");

        buffer
            .remove(TextRange::new(position(1, 3), position(0, 1)))
            .unwrap();

        assert_eq!(buffer.text(), "u");
    }

    #[test]
    fn remove_counts_columns_in_characters_and_not_in_bytes() {
        let mut buffer = TextBuffer::new("áéí");

        buffer
            .remove(TextRange::new(position(0, 1), position(0, 2)))
            .unwrap();

        assert_eq!(buffer.text(), "áí");
    }

    #[test]
    fn removing_a_range_that_starts_after_the_document_is_rejected() {
        let mut buffer = TextBuffer::new("uno\ndos");

        let result = buffer.remove(TextRange::new(position(5, 0), position(5, 1)));

        assert!(matches!(result, Err(CoreError::InvalidPosition(_))));
    }

    #[test]
    fn removing_a_range_that_ends_after_the_document_is_rejected() {
        let mut buffer = TextBuffer::new("uno\ndos");

        let result = buffer.remove(TextRange::new(position(0, 0), position(1, 99)));

        assert!(matches!(result, Err(CoreError::InvalidPosition(_))));
    }

    #[test]
    fn a_rejected_removal_leaves_the_text_untouched() {
        let mut buffer = TextBuffer::new("uno\ndos");

        let result = buffer.remove(TextRange::new(position(0, 0), position(9, 9)));

        assert!(matches!(result, Err(CoreError::InvalidPosition(_))));
        assert_eq!(buffer.text(), "uno\ndos");
    }

    #[test]
    fn position_exposes_line_and_column() {
        let position = position(3, 7);

        assert_eq!(position.line(), 3);
        assert_eq!(position.column(), 7);
    }

    #[test]
    fn positions_are_ordered_by_line_and_then_column() {
        assert!(position(1, 5) < position(2, 0));
        assert!(position(1, 5) < position(1, 6));
        assert!(position(1, 6) > position(1, 5));
        assert!(position(2, 0) > position(1, 9));
    }

    #[test]
    fn the_default_position_is_the_first_character() {
        assert_eq!(TextPosition::default(), position(0, 0));
    }

    #[test]
    fn range_normalizes_reversed_bounds() {
        let range = TextRange::new(position(2, 0), position(1, 4));

        assert_eq!(range.start(), position(1, 4));
        assert_eq!(range.end(), position(2, 0));
    }

    #[test]
    fn collapsed_range_is_empty() {
        let range = TextRange::collapsed(position(1, 1));

        assert!(range.is_empty());
        assert!(range.contains(position(1, 1)));
        assert!(!range.contains(position(1, 2)));
    }

    #[test]
    fn contains_excludes_the_end_position() {
        let range = TextRange::new(position(0, 0), position(1, 0));

        assert!(range.contains(position(0, 0)));
        assert!(range.contains(position(0, 9)));
        assert!(!range.contains(position(1, 0)));
        assert!(!range.contains(position(2, 0)));
    }

    #[test]
    fn a_range_can_be_read_from_the_buffer() {
        let buffer = TextBuffer::new("uno\ndos");
        let range = TextRange::new(position(0, 1), position(1, 1));

        assert_eq!(buffer.text_in(range).unwrap(), "no\nd");
    }

    #[test]
    fn an_empty_range_reads_as_no_text() {
        let buffer = TextBuffer::new("uno");
        let range = TextRange::new(position(0, 2), position(0, 2));

        assert_eq!(buffer.text_in(range).unwrap(), "");
    }

    #[test]
    fn a_range_outside_the_buffer_cannot_be_read() {
        let buffer = TextBuffer::new("uno\ndos");
        let range = TextRange::new(position(0, 0), position(9, 9));

        assert!(matches!(
            buffer.text_in(range),
            Err(CoreError::InvalidPosition(_))
        ));
    }

    #[test]
    fn a_needle_found_in_one_line_gives_its_range() {
        let buffer = TextBuffer::new("uno dos");

        let found = buffer.find_all("dos");

        assert_eq!(found, vec![TextRange::new(position(0, 4), position(0, 7))]);
    }

    #[test]
    fn a_needle_found_several_times_gives_every_range() {
        let buffer = TextBuffer::new("uno dos uno");

        let found = buffer.find_all("uno");

        assert_eq!(
            found,
            vec![
                TextRange::new(position(0, 0), position(0, 3)),
                TextRange::new(position(0, 8), position(0, 11)),
            ]
        );
    }

    #[test]
    fn matches_do_not_overlap() {
        let buffer = TextBuffer::new("aaa");

        let found = buffer.find_all("aa");

        assert_eq!(found, vec![TextRange::new(position(0, 0), position(0, 2))]);
    }

    #[test]
    fn a_needle_that_is_not_there_gives_no_ranges() {
        let buffer = TextBuffer::new("uno dos");

        assert_eq!(buffer.find_all("xyz"), vec![]);
    }

    #[test]
    fn an_empty_needle_gives_no_ranges() {
        let buffer = TextBuffer::new("uno dos");

        assert_eq!(buffer.find_all(""), vec![]);
    }

    #[test]
    fn a_needle_found_on_another_line_gives_that_line() {
        let buffer = TextBuffer::new("uno\ndos");

        let found = buffer.find_all("dos");

        assert_eq!(found, vec![TextRange::new(position(1, 0), position(1, 3))]);
    }

    #[test]
    fn a_needle_spanning_lines_gives_a_range_over_them() {
        let buffer = TextBuffer::new("uno\ndos");

        let found = buffer.find_all("o\nd");

        assert_eq!(found, vec![TextRange::new(position(0, 2), position(1, 1))]);
    }

    #[test]
    fn a_needle_found_at_the_end_of_the_document_is_given() {
        let buffer = TextBuffer::new("uno");

        let found = buffer.find_all("uno");

        assert_eq!(found, vec![TextRange::new(position(0, 0), position(0, 3))]);
    }

    #[test]
    fn found_positions_count_characters_and_not_bytes() {
        let buffer = TextBuffer::new("áéí");

        let found = buffer.find_all("éí");

        assert_eq!(found, vec![TextRange::new(position(0, 1), position(0, 3))]);
    }

    #[test]
    fn a_needle_found_after_a_newline_keeps_counting_characters() {
        let buffer = TextBuffer::new("á\nñb");

        let found = buffer.find_all("ñb");

        assert_eq!(found, vec![TextRange::new(position(1, 0), position(1, 2))]);
    }
}
