use std::fs;
use std::path::{Path, PathBuf};

use crate::core::{CoreError, CoreResult};
use crate::document::{TextBuffer, TextPosition, TextRange};

/// Posicion mas cercana a `at` que exista de verdad en el documento.
///
/// Una linea que no existe se recorta al final de la ultima linea y una
/// columna que se pasa del final de su linea se recorta a ese final.
fn clamp(buffer: &TextBuffer, at: TextPosition) -> TextPosition {
    let last_line = buffer.line_count() - 1;

    if at.line() > last_line {
        return TextPosition::new(last_line, buffer.line_length(last_line));
    }

    let length = buffer.line_length(at.line());

    if at.column() > length {
        return TextPosition::new(at.line(), length);
    }

    at
}

/// Posicion del cursor dentro de un documento.
///
/// Todos los movimientos recortan la posicion a un lugar real del documento,
/// de modo que el cursor nunca queda apuntando a una posicion invalida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    at: TextPosition,
}

impl Cursor {
    /// Crea el cursor en `at`, recortandolo al documento si hace falta.
    pub fn new(buffer: &TextBuffer, at: TextPosition) -> Self {
        let mut cursor = Self { at };

        cursor.normalize(buffer);

        cursor
    }

    pub fn at(&self) -> TextPosition {
        self.at
    }

    /// Coloca el cursor en `at`, recortandolo al documento si hace falta.
    pub fn move_to(&mut self, buffer: &TextBuffer, at: TextPosition) {
        self.at = at;

        self.normalize(buffer);
    }

    /// Recorta la posicion a la mas cercana que exista en el documento.
    ///
    /// Hace falta despues de una edicion que pueda haber dejado al cursor
    /// en una linea inexistente o en una columna mas alla del final de la
    /// linea.
    pub fn normalize(&mut self, buffer: &TextBuffer) {
        self.at = clamp(buffer, self.at);
    }

    pub fn move_left(&mut self, buffer: &TextBuffer) {
        self.normalize(buffer);

        let line = self.at.line();

        if self.at.column() > 0 {
            self.at = TextPosition::new(line, self.at.column() - 1);
        } else if line > 0 {
            self.at = TextPosition::new(line - 1, buffer.line_length(line - 1));
        }
    }

    pub fn move_right(&mut self, buffer: &TextBuffer) {
        self.normalize(buffer);

        let line = self.at.line();
        let length = buffer.line_length(line);

        if self.at.column() < length {
            self.at = TextPosition::new(line, self.at.column() + 1);
        } else if line + 1 < buffer.line_count() {
            self.at = TextPosition::new(line + 1, 0);
        }
    }

    pub fn move_up(&mut self, buffer: &TextBuffer) {
        self.normalize(buffer);

        let line = self.at.line();

        if line > 0 {
            let column = self.at.column().min(buffer.line_length(line - 1));
            self.at = TextPosition::new(line - 1, column);
        }
    }

    pub fn move_down(&mut self, buffer: &TextBuffer) {
        self.normalize(buffer);

        let line = self.at.line();

        if line + 1 < buffer.line_count() {
            let column = self.at.column().min(buffer.line_length(line + 1));
            self.at = TextPosition::new(line + 1, column);
        }
    }
}

/// Posicion que queda justo despues de escribir `inserted` en `at`.
fn position_after(at: TextPosition, inserted: &str) -> TextPosition {
    let newlines = inserted.matches('\n').count() as u32;

    if newlines == 0 {
        return TextPosition::new(at.line(), at.column() + inserted.chars().count() as u32);
    }

    let last_line = inserted.rsplit('\n').next().unwrap_or_default();

    TextPosition::new(at.line() + newlines, last_line.chars().count() as u32)
}

/// Seleccion del documento: un cursor y, si la hay, la posicion por la que
/// empezo a seleccionarse.
///
/// Sin ancla no hay nada seleccionado. El rango se normaliza, asi que da igual
/// en que direccion se haya arrastrado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    anchor: Option<TextPosition>,
    cursor: Cursor,
}

impl Selection {
    /// Empieza sin seleccion, con el cursor dado.
    pub fn new(cursor: Cursor) -> Self {
        Self {
            anchor: None,
            cursor,
        }
    }

    /// Posicion del cursor.
    pub fn at(&self) -> TextPosition {
        self.cursor.at()
    }

    /// Posicion por la que empezo la seleccion, o `None` si no hay seleccion.
    pub fn anchor(&self) -> Option<TextPosition> {
        self.anchor
    }

    /// Mueve el cursor y descarta la seleccion.
    pub fn move_to(&mut self, buffer: &TextBuffer, at: TextPosition) {
        self.anchor = None;
        self.cursor.move_to(buffer, at);
    }

    /// Mueve el cursor a la izquierda y descarta la seleccion.
    ///
    /// Y la descarta porque mover no es alargar: alargar la seleccion es lo que hace
    /// `extend_to`, que es a donde va la seleccion mientras se pulsa Mayusculas. Si
    /// moverse dejara la seleccion, cada flecha moveria el final de lo seleccionado y no
    /// habria forma de escribir donde quiere uno.
    pub fn move_left(&mut self, buffer: &TextBuffer) {
        self.anchor = None;
        self.cursor.move_left(buffer);
    }

    /// Véase [`Self::move_left`].
    pub fn move_right(&mut self, buffer: &TextBuffer) {
        self.anchor = None;
        self.cursor.move_right(buffer);
    }

    /// Véase [`Self::move_left`].
    pub fn move_up(&mut self, buffer: &TextBuffer) {
        self.anchor = None;
        self.cursor.move_up(buffer);
    }

    /// Véase [`Self::move_left`].
    pub fn move_down(&mut self, buffer: &TextBuffer) {
        self.anchor = None;
        self.cursor.move_down(buffer);
    }

    /// Mueve el cursor creando la seleccion si no habia, o alargandola si la
    /// habia. La ancla se queda donde estaba.
    pub fn extend_to(&mut self, buffer: &TextBuffer, at: TextPosition) {
        if self.anchor.is_none() {
            self.anchor = Some(self.cursor.at());
        }

        self.cursor.move_to(buffer, at);
    }

    /// Recorta cursor y ancla despues de una edicion hecha fuera de la
    /// seleccion.
    pub fn normalize(&mut self, buffer: &TextBuffer) {
        if let Some(anchor) = self.anchor {
            self.anchor = Some(clamp(buffer, anchor));
        }

        self.cursor.normalize(buffer);
    }

    /// No hay nada seleccionado: no hay ancla o el rango no cubre texto.
    pub fn is_empty(&self) -> bool {
        match self.anchor {
            None => true,
            Some(anchor) => anchor == self.cursor.at(),
        }
    }

    /// Rango seleccionado, o `None` si no hay seleccion.
    pub fn range(&self) -> Option<TextRange> {
        self.anchor
            .map(|anchor| TextRange::new(anchor, self.cursor.at()))
    }

    /// Texto seleccionado. `None` si no hay seleccion; `Some("")` si la
    /// seleccion esta colapsada en un punto.
    pub fn selected_text<'a>(&self, buffer: &'a TextBuffer) -> CoreResult<Option<&'a str>> {
        match self.range() {
            None => Ok(None),
            Some(range) => buffer.text_in(range).map(Some),
        }
    }

    /// Sustituye el texto seleccionado por `text` y deja el cursor al final de
    /// lo escrito.
    ///
    /// Sin seleccion inserta `text` en la posicion del cursor, que es lo que
    /// hace una pulsacion de tecla normal. Escribir texto vacio borra la
    /// seleccion. La seleccion queda descartada y el cursor nunca queda
    /// invalido.
    pub fn replace(&mut self, buffer: &mut TextBuffer, text: impl Into<String>) -> CoreResult<()> {
        let text = text.into();

        let at = match self.range() {
            Some(range) => range.start(),
            None => self.cursor.at(),
        };

        if let Some(range) = self.range() {
            buffer.remove(range)?;
        }

        buffer.insert(at, text.as_str())?;

        self.anchor = None;
        self.cursor.move_to(buffer, position_after(at, &text));

        Ok(())
    }
}

/// Estado del documento antes de una edicion, tal y como se necesita para
/// deshacerla.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Snapshot {
    text: String,
    anchor: Option<TextPosition>,
    cursor: TextPosition,
}

/// Documento abierto: su contenido, su seleccion y si ha cambiado desde el
/// ultimo guardado.
///
/// Es la forma de editar de verdad: toda mutacion del contenido pasa por aqui,
/// asi que no se puede editar sin marcar el documento ni sin dejar un paso de
/// deshacer. La seleccion se maneja
/// con `move_to`, `select` y `normalize_selection` en vez de exponerla
/// mutablemente, para que la unica forma de tocar el estado del documento sea
/// un metodo que dice si lo marca o no.
///
/// El historial son copias completas del contenido, una por edicion. Es simple
/// y exacto, a cambio de consumir memoria proporcional al numero de ediciones.
/// Editar descarta lo que habia para rehacer: en cuanto el documento se aparta
/// de su historia, esa rama ya no existe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    buffer: TextBuffer,
    selection: Selection,
    modified: bool,
    saved: String,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
}

impl Document {
    /// Documento nuevo, sin modificar, con el cursor al principio.
    ///
    /// El contenido con el que se crea hace de referencia: deshacer hasta
    /// volver a el deja el documento sin modificar, como si estuviera guardado.
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let buffer = TextBuffer::new(text.clone());
        let selection = Selection::new(Cursor::new(&buffer, TextPosition::new(0, 0)));

        Self {
            buffer,
            selection,
            modified: false,
            saved: text,
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    pub fn buffer(&self) -> &TextBuffer {
        &self.buffer
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// Si el contenido ha cambiado desde el ultimo guardado.
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Da el contenido por guardado. No escribe en disco: solo deja de avisar
    /// de que hay cambios sin escribir.
    pub fn mark_saved(&mut self) {
        self.saved = self.buffer.text().to_owned();
        self.modified = false;
    }

    /// Si hay alguna edicion que deshacer.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Si hay alguna edicion deshecha que rehacer.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Deshace la ultima edicion y devuelve si habia alguna que deshacer.
    ///
    /// Vuelve a poner el contenido, el cursor y la seleccion tal y como estaban
    /// antes de editarlos, y vuelve a calcular si el documento esta modificado:
    /// deshacer hasta el contenido guardado deja de marcarlo. Lo que se deja
    /// atras queda preparado para rehacerse.
    pub fn undo(&mut self) -> bool {
        let Some(snapshot) = self.undo.pop() else {
            return false;
        };

        let left_behind = self.snapshot();

        self.redo.push(left_behind);
        self.restore(snapshot);

        true
    }

    /// Rehace la ultima edicion deshecha y devuelve si habia alguna.
    ///
    /// Es el simetrico de `undo`: vuelve a poner el contenido, el cursor y la
    /// seleccion de despues de esa edicion, y deja el estado anterior listo
    /// para volver a deshacerse.
    pub fn redo(&mut self) -> bool {
        let Some(snapshot) = self.redo.pop() else {
            return false;
        };

        let left_behind = self.snapshot();

        self.undo.push(left_behind);
        self.restore(snapshot);

        true
    }

    /// Vuelve a poner el contenido, el cursor y la seleccion de una copia
    /// anterior y recalcula si el documento esta modificado.
    fn restore(&mut self, snapshot: Snapshot) {
        self.buffer = TextBuffer::new(snapshot.text);

        match snapshot.anchor {
            None => self.move_to(snapshot.cursor),
            Some(anchor) => self.select(anchor, snapshot.cursor),
        }

        self.modified = self.buffer.text() != self.saved;
    }

    /// Copia del estado actual para poder deshacerlo o rehacerlo.
    fn snapshot(&self) -> Snapshot {
        Snapshot {
            text: self.buffer.text().to_owned(),
            anchor: self.selection.anchor(),
            cursor: self.selection.at(),
        }
    }

    /// Mueve el cursor. No modifica el contenido.
    pub fn move_to(&mut self, at: TextPosition) {
        self.selection.move_to(&self.buffer, at);
    }

    /// Mueve el cursor una posicion a la izquierda. No modifica el contenido.
    ///
    /// Las cuatro direcciones estan en el documento y no en la ventana porque el
    /// recorrido del cursor es logica del texto y no de pantalla: el final de una linea,
    /// el principio de la siguiente y el recorte de la columna al cambiar de linea son
    /// reglas del documento, y la ventana que las duplicara las tendria que mantener
    /// igual que el core.
    ///
    /// Moverse nunca marca el documento: es lo que distingue mover de editar, y si no,
    /// un paseo con las flechas dejaria el archivo con cambios sin guardar.
    pub fn move_left(&mut self) {
        self.selection.move_left(&self.buffer);
    }

    /// Mueve el cursor una posicion a la derecha. No modifica el contenido.
    pub fn move_right(&mut self) {
        self.selection.move_right(&self.buffer);
    }

    /// Mueve el cursor una linea arriba. No modifica el contenido.
    pub fn move_up(&mut self) {
        self.selection.move_up(&self.buffer);
    }

    /// Mueve el cursor una linea abajo. No modifica el contenido.
    pub fn move_down(&mut self) {
        self.selection.move_down(&self.buffer);
    }

    /// Selecciona de `first` a `second`, en el orden que sea. No modifica el
    /// contenido.
    pub fn select(&mut self, first: TextPosition, second: TextPosition) {
        self.selection.move_to(&self.buffer, first);
        self.selection.extend_to(&self.buffer, second);
    }

    /// Recorta cursor y ancla despues de una edicion hecha por otra via.
    pub fn normalize_selection(&mut self) {
        self.selection.normalize(&self.buffer);
    }

    /// Inserta `text` en `at` y marca el documento si cambio el contenido.
    pub fn insert(&mut self, at: TextPosition, text: impl Into<String>) -> CoreResult<()> {
        let text = text.into();

        if text.is_empty() {
            return self.buffer.insert(at, text.as_str());
        }

        let previous = self.snapshot();

        self.buffer.insert(at, text.as_str())?;

        self.undo.push(previous);
        self.redo.clear();
        self.modified = true;

        Ok(())
    }

    /// Borra `range` y marca el documento si quito contenido.
    pub fn remove(&mut self, range: TextRange) -> CoreResult<()> {
        if range.is_empty() {
            return self.buffer.remove(range);
        }

        let previous = self.snapshot();

        self.buffer.remove(range)?;

        self.undo.push(previous);
        self.redo.clear();
        self.modified = true;

        Ok(())
    }

    /// Sustituye la seleccion por `text`, o inserta en el cursor si no hay
    /// seleccion, y marca el documento si cambio el contenido.
    pub fn replace(&mut self, text: impl Into<String>) -> CoreResult<()> {
        let text = text.into();

        if text.is_empty() && self.selection.is_empty() {
            return self.selection.replace(&mut self.buffer, text);
        }

        let previous = self.snapshot();

        self.selection.replace(&mut self.buffer, text)?;

        self.undo.push(previous);
        self.redo.clear();
        self.modified = true;

        Ok(())
    }

    /// Sustituye el rango `range`, normalmente una coincidencia de `find_all`,
    /// por `replacement` y deja el cursor al final de lo escrito, sin
    /// seleccion.
    ///
    /// Un rango que no existe en el documento devuelve
    /// `CoreError::InvalidPosition` y no cambia nada, igual que `remove` y
    /// `insert`. Ocupa un solo paso del historial.
    pub fn replace_match(
        &mut self,
        range: TextRange,
        replacement: impl Into<String>,
    ) -> CoreResult<()> {
        let replacement = replacement.into();

        if range.is_empty() && replacement.is_empty() {
            return Ok(());
        }

        let previous = self.snapshot();

        self.buffer.remove(range)?;
        self.buffer.insert(range.start(), replacement.as_str())?;

        self.undo.push(previous);
        self.redo.clear();
        self.modified = true;
        self.move_to(position_after(range.start(), &replacement));

        Ok(())
    }

    /// Sustituye todas las apariciones de `needle` por `replacement` y devuelve
    /// cuantas ha sustituido.
    ///
    /// Todo el reemplazo cuenta como un solo paso del historial, se haga el
    /// numero de sustituciones que se haga, y descarta lo que hubiera para
    /// rehacer. Sin coincidencias no cambia nada ni deja paso. La seleccion se
    /// recorta al final para que siga siendo valida.
    pub fn replace_all(&mut self, needle: &str, replacement: &str) -> CoreResult<usize> {
        let ranges = self.buffer.find_all(needle);

        if ranges.is_empty() {
            return Ok(0);
        }

        let previous = self.snapshot();

        if let Err(error) = self.replace_ranges(&ranges, replacement) {
            self.restore(previous.clone());

            return Err(error);
        }

        self.undo.push(previous);
        self.redo.clear();
        self.modified = true;
        self.normalize_selection();

        Ok(ranges.len())
    }

    /// Sustituye los rangos dados, de derecha a izquierda.
    ///
    /// Ir de derecha a izquierda deja validas las posiciones de los rangos que
    /// todavia no se han tocado. El historial lo lleva quien llama, para que un
    /// lote de sustituciones sea un solo paso.
    fn replace_ranges(&mut self, ranges: &[TextRange], replacement: &str) -> CoreResult<()> {
        for range in ranges.iter().rev() {
            self.buffer.remove(*range)?;
            self.buffer.insert(range.start(), replacement)?;
        }

        Ok(())
    }
}

/// Ruta del archivo que hay abierto en un documento.
///
/// Se valida al construirla y siempre tiene nombre de archivo, asi que una
/// pestaña puede mostrar su nombre sin tener que tratar el caso de que no lo
/// tenga.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentPath {
    value: PathBuf,
}

impl DocumentPath {
    /// Devuelve `CoreError::InvalidPath` si la ruta esta vacia o no tiene
    /// nombre de archivo.
    pub fn new(value: impl Into<PathBuf>) -> CoreResult<Self> {
        let value = value.into();

        if value.as_os_str().is_empty() || value.file_name().is_none() {
            return Err(CoreError::InvalidPath(value.display().to_string()));
        }

        Ok(Self { value })
    }

    pub fn as_path(&self) -> &Path {
        &self.value
    }

    /// Nombre del archivo, sin los directorios.
    pub fn file_name(&self) -> &str {
        self.value
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
    }
}

/// Un documento abierto en una pestaña.
///
/// La pestaña es la dueña del documento: como lo tiene por valor, no hay dos
/// pestañas apuntando al mismo documento, y editarlo pasa por `Document`, que
/// ya se encarga de marcarlo y de guardar el historial.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    path: DocumentPath,
    document: Document,
}

impl Tab {
    pub fn new(path: DocumentPath, document: Document) -> Self {
        Self { path, document }
    }

    pub fn path(&self) -> &DocumentPath {
        &self.path
    }

    /// Nombre del archivo, que es lo que muestra la pestaña.
    pub fn file_name(&self) -> &str {
        self.path.file_name()
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Editar el documento desde la pestaña. Es la via normal para escribir en
    /// un documento abierto.
    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.document
    }

    /// Si el documento tiene cambios sin guardar, que es lo que la pestaña
    /// señala con un asterisco.
    pub fn is_modified(&self) -> bool {
        self.document.is_modified()
    }

    /// Escribe el contenido del documento en la ruta de la pestaña y lo da por
    /// guardado.
    ///
    /// El contenido se escribe tal cual, sin traducir los saltos de linea: lo
    /// que se guarda es exactamente lo que hay en el documento y lo que se
    /// vuelve a leer. Si no habia archivo, se crea; si habia, se sobrescribe.
    ///
    /// Solo si la escritura termina bien se limpia la marca de modificado. Si
    /// falla, el documento sigue marcado y con su contenido intacto.
    ///
    /// No es una escritura atomica: si la escritura se corta a medias, el
    /// archivo puede quedar a medias.
    pub fn save(&mut self) -> CoreResult<()> {
        let path = self.path.as_path();

        fs::write(path, self.document.buffer().text())
            .map_err(|error| CoreError::from_io(path, &error))?;

        self.document.mark_saved();

        Ok(())
    }
}

/// Los documentos abiertos, en el orden en que se abrieron, y cuál se está viendo.
///
/// Una ruta solo puede estar abierta una vez: al abrir un documento que ya esta
/// abierto se devuelve su pestaña en vez de crear otra, y se conservan sus
/// cambios y su historial.
///
/// Ademas de la lista sabe cuál de las pestañas es la activa, que es el documento
/// que se está viendo. No lo sapia antes, y T-027 lo dejo fuera a proposito por ser
/// política y no estructura: que al abrir un documento se vea ese documento, y qué
/// pasa con la activa cuando se cierra, son decisiones que hay que tomar a
/// propósito, no un detalle que salga solo.
///
/// La activa se guarda como indice y no como ruta porque es lo que se devuelve al
/// que la cambia, y quien la cambia -la ventana, al pulsar una pestaña- la cambia
/// por sitio, no por nombre. El indice se mantiene al cerrar: ver
/// [`Self::close`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenTabs {
    tabs: Vec<Tab>,
    active: Option<usize>,
}

impl OpenTabs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Abre el documento de `path` con `text` y devuelve el indice de su
    /// pestaña. Si ya estaba abierto, devuelve ese mismo indice y no toca el
    /// documento.
    ///
    /// La pestaña que se abre queda activa, y la que ya estaba abierta tambien: abrir
    /// un documento es pedir verlo, asi que lo que se acaba de pedir no puede
    /// quedarse a la espalda de lo que ya se estaba viendo. Volver a abrir uno que ya
    /// lo esta no crea una segunda pestaña: lo trae a delante.
    pub fn open(&mut self, path: DocumentPath, text: impl Into<String>) -> usize {
        let index = if let Some(index) = self.index_of(&path) {
            index
        } else {
            self.tabs.push(Tab::new(path, Document::new(text)));

            self.tabs.len() - 1
        };

        self.active = Some(index);

        index
    }

    /// El indice de la pestaña que se está viendo, o `None` si no hay ninguna.
    ///
    /// No hay ninguna con pestañas cerradas todas, y no hay ninguna en una lista
    /// vacía: un documento abierto siempre es el que se está viendo, asi que con
    /// pestañas siempre hay una activa.
    pub fn active(&self) -> Option<usize> {
        self.active
    }

    /// Deja activa la pestaña del indice. Devuelve `false` si no hay esa pestaña.
    ///
    /// Es como [`Self::close`], que tambien devuelve `None` en vez de fallar: el
    /// indice lo elige quien llama y puede estar equivocado, y una lista de
    /// documentos no es el sitio donde se Avisa de un error de quien la usa.
    pub fn activate(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }

        self.active = Some(index);

        true
    }

    pub fn get(&self, index: usize) -> Option<&Tab> {
        self.tabs.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Tab> {
        self.tabs.get_mut(index)
    }

    /// Cierra la pestaña y la devuelve, para que quien la cierra pueda mirar si
    /// el documento tenia cambios sin guardar. Devuelve `None` si no habia
    /// ninguna pestaña en ese indice.
    ///
    /// Cerrar la activa no deja al usuario sin nada que ver, y aqui esta esa
    /// politica: pasa a estar activa la pestaña que ocupa el sitio que ha quedado
    /// libre, que es la siguiente, y si la que se cierra era la ultima, la nueva
    /// ultima. Cerrar otra no cambia el documento que se está viendo, aunque la que
    /// se cierre este antes: el indice se mueve con la lista y la activa se mueve
    /// con el.
    pub fn close(&mut self, index: usize) -> Option<Tab> {
        if index >= self.tabs.len() {
            return None;
        }

        let closed = self.tabs.remove(index);

        match self.active {
            Some(active) if active == index => {
                self.active = (!self.tabs.is_empty()).then(|| index.min(self.tabs.len() - 1));
            }
            Some(active) if active > index => self.active = Some(active - 1),
            _ => {}
        }

        Some(closed)
    }

    /// Indice de la pestaña que tiene abierto `path`.
    pub fn index_of(&self, path: &DocumentPath) -> Option<usize> {
        self.tabs.iter().position(|tab| tab.path() == path)
    }

    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tab> {
        self.tabs.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::CoreError;
    use crate::document::TextRange;

    fn buffer(text: &str) -> TextBuffer {
        TextBuffer::new(text)
    }

    fn at(line: u32, column: u32) -> TextPosition {
        TextPosition::new(line, column)
    }

    fn selection(buffer: &TextBuffer, first: TextPosition, second: TextPosition) -> Selection {
        let mut selection = Selection::new(Cursor::new(buffer, first));

        selection.extend_to(buffer, second);

        selection
    }

    #[test]
    fn a_new_cursor_sits_at_its_position() {
        let buffer = buffer("uno\ndos");

        let cursor = Cursor::new(&buffer, at(1, 2));

        assert_eq!(cursor.at(), at(1, 2));
    }

    #[test]
    fn moving_right_advances_one_character() {
        let buffer = buffer("uno\ndos");
        let mut cursor = Cursor::new(&buffer, at(0, 0));

        cursor.move_right(&buffer);

        assert_eq!(cursor.at(), at(0, 1));
    }

    #[test]
    fn moving_right_at_the_end_of_a_line_goes_to_the_next_one() {
        let buffer = buffer("uno\ndos");
        let mut cursor = Cursor::new(&buffer, at(0, 3));

        cursor.move_right(&buffer);

        assert_eq!(cursor.at(), at(1, 0));
    }

    #[test]
    fn moving_right_at_the_end_of_the_document_stays_in_place() {
        let buffer = buffer("uno\ndos");
        let mut cursor = Cursor::new(&buffer, at(1, 3));

        cursor.move_right(&buffer);

        assert_eq!(cursor.at(), at(1, 3));
    }

    #[test]
    fn moving_left_goes_back_one_character() {
        let buffer = buffer("uno\ndos");
        let mut cursor = Cursor::new(&buffer, at(1, 1));

        cursor.move_left(&buffer);

        assert_eq!(cursor.at(), at(1, 0));
    }

    #[test]
    fn moving_left_at_the_start_of_a_line_goes_to_the_previous_one() {
        let buffer = buffer("uno\ndos");
        let mut cursor = Cursor::new(&buffer, at(1, 0));

        cursor.move_left(&buffer);

        assert_eq!(cursor.at(), at(0, 3));
    }

    #[test]
    fn moving_left_at_the_start_of_the_document_stays_in_place() {
        let buffer = buffer("uno\ndos");
        let mut cursor = Cursor::new(&buffer, at(0, 0));

        cursor.move_left(&buffer);

        assert_eq!(cursor.at(), at(0, 0));
    }

    #[test]
    fn moving_down_keeps_the_column() {
        let buffer = buffer("abcd\nef");
        let mut cursor = Cursor::new(&buffer, at(0, 2));

        cursor.move_down(&buffer);

        assert_eq!(cursor.at(), at(1, 2));
    }

    #[test]
    fn moving_down_clamps_the_column_to_a_shorter_line() {
        let buffer = buffer("abcd\ne");
        let mut cursor = Cursor::new(&buffer, at(0, 3));

        cursor.move_down(&buffer);

        assert_eq!(cursor.at(), at(1, 1));
    }

    #[test]
    fn moving_down_at_the_last_line_stays_in_place() {
        let buffer = buffer("uno\ndos");
        let mut cursor = Cursor::new(&buffer, at(1, 0));

        cursor.move_down(&buffer);

        assert_eq!(cursor.at(), at(1, 0));
    }

    #[test]
    fn moving_up_keeps_the_column() {
        let buffer = buffer("abcd\nef");
        let mut cursor = Cursor::new(&buffer, at(1, 2));

        cursor.move_up(&buffer);

        assert_eq!(cursor.at(), at(0, 2));
    }

    #[test]
    fn moving_up_clamps_the_column_to_a_shorter_line() {
        let buffer = buffer("ab\ncdef");
        let mut cursor = Cursor::new(&buffer, at(1, 3));

        cursor.move_up(&buffer);

        assert_eq!(cursor.at(), at(0, 2));
    }

    #[test]
    fn moving_up_at_the_first_line_stays_in_place() {
        let buffer = buffer("uno\ndos");
        let mut cursor = Cursor::new(&buffer, at(0, 1));

        cursor.move_up(&buffer);

        assert_eq!(cursor.at(), at(0, 1));
    }

    #[test]
    fn moving_to_a_column_past_the_line_clamps_it() {
        let buffer = buffer("uno\ndos");
        let mut cursor = Cursor::new(&buffer, at(0, 0));

        cursor.move_to(&buffer, at(0, 99));

        assert_eq!(cursor.at(), at(0, 3));
    }

    #[test]
    fn moving_to_a_line_that_does_not_exist_lands_at_the_end_of_the_document() {
        let buffer = buffer("uno\ndos");
        let mut cursor = Cursor::new(&buffer, at(0, 0));

        cursor.move_to(&buffer, at(9, 0));

        assert_eq!(cursor.at(), at(1, 3));
    }

    #[test]
    fn moving_counts_columns_in_characters_and_not_in_bytes() {
        let buffer = buffer("áéí");
        let mut cursor = Cursor::new(&buffer, at(0, 0));

        cursor.move_right(&buffer);

        assert_eq!(cursor.at(), at(0, 1));
    }

    #[test]
    fn a_cursor_left_on_a_removed_line_is_normalized() {
        let mut buffer = buffer("uno\ndos");
        let mut cursor = Cursor::new(&buffer, at(1, 3));

        buffer.remove(TextRange::new(at(0, 3), at(1, 0))).unwrap();
        cursor.normalize(&buffer);

        assert_eq!(cursor.at(), at(0, 6));
    }

    #[test]
    fn a_cursor_left_past_the_end_of_a_shorter_line_is_normalized() {
        let mut buffer = buffer("abcdef");
        let mut cursor = Cursor::new(&buffer, at(0, 5));

        buffer.remove(TextRange::new(at(0, 4), at(0, 6))).unwrap();
        cursor.normalize(&buffer);

        assert_eq!(cursor.at(), at(0, 4));
    }

    #[test]
    fn a_new_selection_has_no_anchor() {
        let buffer = buffer("uno\ndos");
        let selection = Selection::new(Cursor::new(&buffer, at(0, 1)));

        assert_eq!(selection.at(), at(0, 1));
        assert!(selection.is_empty());
        assert_eq!(selection.range(), None);
    }

    #[test]
    fn extending_from_the_cursor_creates_a_range() {
        let buffer = buffer("uno\ndos");
        let selection = selection(&buffer, at(0, 1), at(1, 1));

        assert!(!selection.is_empty());
        assert_eq!(selection.range(), Some(TextRange::new(at(0, 1), at(1, 1))));
    }

    #[test]
    fn extending_twice_keeps_the_original_anchor() {
        let buffer = buffer("uno\ndos");
        let mut selection = selection(&buffer, at(0, 0), at(0, 2));

        selection.extend_to(&buffer, at(0, 3));

        assert_eq!(selection.range(), Some(TextRange::new(at(0, 0), at(0, 3))));
    }

    #[test]
    fn a_selection_can_be_read_from_the_buffer() {
        let buffer = buffer("uno\ndos");
        let selection = selection(&buffer, at(0, 1), at(1, 1));

        assert_eq!(selection.selected_text(&buffer).unwrap(), Some("no\nd"));
    }

    #[test]
    fn a_selection_built_backwards_covers_the_same_text() {
        let buffer = buffer("uno\ndos");
        let selection = selection(&buffer, at(1, 1), at(0, 1));

        assert_eq!(selection.range(), Some(TextRange::new(at(0, 1), at(1, 1))));
        assert_eq!(selection.selected_text(&buffer).unwrap(), Some("no\nd"));
    }

    #[test]
    fn a_collapsed_selection_is_empty() {
        let buffer = buffer("uno\ndos");
        let selection = selection(&buffer, at(0, 1), at(0, 1));

        assert!(selection.is_empty());
        assert_eq!(selection.selected_text(&buffer).unwrap(), Some(""));
    }

    #[test]
    fn moving_without_extending_clears_the_selection() {
        let buffer = buffer("uno\ndos");
        let mut selection = selection(&buffer, at(0, 1), at(0, 3));

        selection.move_to(&buffer, at(0, 0));

        assert!(selection.is_empty());
        assert_eq!(selection.range(), None);
    }

    #[test]
    fn replacing_a_selection_swaps_its_text() {
        let mut buffer = buffer("uno\ndos");
        let mut selection = selection(&buffer, at(0, 1), at(1, 2));

        selection.replace(&mut buffer, "X").unwrap();

        assert_eq!(buffer.text(), "uXs");
        assert_eq!(selection.at(), at(0, 2));
        assert!(selection.is_empty());
    }

    #[test]
    fn replacing_a_selection_with_nothing_removes_it() {
        let mut buffer = buffer("uno\ndos");
        let mut selection = selection(&buffer, at(0, 1), at(1, 2));

        selection.replace(&mut buffer, "").unwrap();

        assert_eq!(buffer.text(), "us");
        assert_eq!(selection.at(), at(0, 1));
    }

    #[test]
    fn replacing_with_more_lines_leaves_the_cursor_at_the_end() {
        let mut buffer = buffer("ab");
        let mut selection = selection(&buffer, at(0, 0), at(0, 2));

        selection.replace(&mut buffer, "1\n2").unwrap();

        assert_eq!(buffer.text(), "1\n2");
        assert_eq!(selection.at(), at(1, 1));
    }

    #[test]
    fn typing_without_a_selection_inserts_at_the_cursor() {
        let mut buffer = buffer("abc");
        let mut selection = Selection::new(Cursor::new(&buffer, at(0, 1)));

        selection.replace(&mut buffer, "!").unwrap();

        assert_eq!(buffer.text(), "a!bc");
        assert_eq!(selection.at(), at(0, 2));
    }

    #[test]
    fn a_rejected_replace_leaves_the_text_and_the_selection_untouched() {
        let mut buffer = buffer("uno\ndos");
        let mut selection = selection(&buffer, at(1, 0), at(1, 3));

        buffer.remove(TextRange::new(at(0, 3), at(1, 0))).unwrap();
        let result = selection.replace(&mut buffer, "X");

        assert!(matches!(result, Err(CoreError::InvalidPosition(_))));
        assert_eq!(buffer.text(), "unodos");
        assert_eq!(selection.range(), Some(TextRange::new(at(1, 0), at(1, 3))));
    }

    #[test]
    fn a_selection_past_the_end_of_a_shorter_line_is_normalized() {
        let mut buffer = buffer("abcdef");
        let mut selection = selection(&buffer, at(0, 1), at(0, 5));

        buffer.remove(TextRange::new(at(0, 4), at(0, 6))).unwrap();
        selection.normalize(&buffer);

        assert_eq!(selection.range(), Some(TextRange::new(at(0, 1), at(0, 4))));
    }

    #[test]
    fn a_new_document_is_not_modified() {
        let document = Document::new("uno");

        assert_eq!(document.buffer().text(), "uno");
        assert_eq!(document.selection().at(), at(0, 0));
        assert!(!document.is_modified());
    }

    #[test]
    fn inserting_marks_the_document() {
        let mut document = Document::new("uno");

        document.insert(at(0, 3), "!").unwrap();

        assert_eq!(document.buffer().text(), "uno!");
        assert!(document.is_modified());
    }

    #[test]
    fn removing_marks_the_document() {
        let mut document = Document::new("uno");

        document.remove(TextRange::new(at(0, 0), at(0, 1))).unwrap();

        assert_eq!(document.buffer().text(), "no");
        assert!(document.is_modified());
    }

    #[test]
    fn replacing_a_selection_marks_the_document() {
        let mut document = Document::new("uno");
        document.select(at(0, 1), at(0, 3));

        document.replace("X").unwrap();

        assert_eq!(document.buffer().text(), "uX");
        assert!(document.is_modified());
    }

    #[test]
    fn typing_marks_the_document() {
        let mut document = Document::new("ab");

        document.move_to(at(0, 1));
        document.replace("!").unwrap();

        assert_eq!(document.buffer().text(), "a!b");
        assert!(document.is_modified());
    }

    #[test]
    fn inserting_nothing_does_not_mark_the_document() {
        let mut document = Document::new("uno");

        document.insert(at(0, 1), "").unwrap();

        assert_eq!(document.buffer().text(), "uno");
        assert!(!document.is_modified());
    }

    #[test]
    fn removing_nothing_does_not_mark_the_document() {
        let mut document = Document::new("uno");

        document.remove(TextRange::collapsed(at(0, 1))).unwrap();

        assert_eq!(document.buffer().text(), "uno");
        assert!(!document.is_modified());
    }

    #[test]
    fn moving_the_cursor_does_not_mark_the_document() {
        let mut document = Document::new("uno");

        document.move_to(at(0, 2));

        assert_eq!(document.selection().at(), at(0, 2));
        assert!(!document.is_modified());
    }

    #[test]
    fn a_rejected_edit_does_not_mark_the_document() {
        let mut document = Document::new("uno");

        let result = document.insert(at(9, 9), "X");

        assert!(matches!(result, Err(CoreError::InvalidPosition(_))));
        assert_eq!(document.buffer().text(), "uno");
        assert!(!document.is_modified());
    }

    #[test]
    fn marking_saved_clears_the_modified_state() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();

        document.mark_saved();

        assert!(!document.is_modified());
    }

    #[test]
    fn editing_after_a_save_marks_the_document_again() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.mark_saved();

        document.insert(at(0, 4), "?").unwrap();

        assert!(document.is_modified());
    }

    #[test]
    fn writing_the_same_text_still_marks_the_document() {
        let mut document = Document::new("uno");
        document.select(at(0, 0), at(0, 3));

        document.replace("uno").unwrap();

        assert_eq!(document.buffer().text(), "uno");
        assert!(document.is_modified());
    }

    #[test]
    fn a_selection_can_be_normalized_after_an_edit_from_another_way() {
        let mut document = Document::new("abcdef");
        document.select(at(0, 1), at(0, 5));

        document.remove(TextRange::new(at(0, 4), at(0, 6))).unwrap();
        document.normalize_selection();

        assert_eq!(
            document.selection().range(),
            Some(TextRange::new(at(0, 1), at(0, 4)))
        );
    }

    #[test]
    fn a_new_document_cannot_be_undone() {
        let mut document = Document::new("uno");

        assert!(!document.can_undo());
        assert!(!document.undo());
        assert_eq!(document.buffer().text(), "uno");
    }

    #[test]
    fn undo_restores_the_previous_text() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();

        assert!(document.undo());

        assert_eq!(document.buffer().text(), "uno");
    }

    #[test]
    fn undo_restores_the_previous_cursor() {
        let mut document = Document::new("uno");
        document.move_to(at(0, 2));
        document.insert(at(0, 2), "!").unwrap();

        assert!(document.undo());

        assert_eq!(document.selection().at(), at(0, 2));
    }

    #[test]
    fn undo_restores_the_previous_selection() {
        let mut document = Document::new("uno");
        document.select(at(0, 1), at(0, 3));
        document.replace("X").unwrap();

        assert!(document.undo());

        assert_eq!(document.buffer().text(), "uno");
        assert_eq!(
            document.selection().range(),
            Some(TextRange::new(at(0, 1), at(0, 3)))
        );
    }

    #[test]
    fn undo_restores_a_backwards_selection() {
        let mut document = Document::new("uno");
        document.select(at(0, 3), at(0, 1));
        document.replace("X").unwrap();

        assert!(document.undo());

        assert_eq!(document.selection().anchor(), Some(at(0, 3)));
        assert_eq!(document.selection().at(), at(0, 1));
    }

    #[test]
    fn undo_is_rejected_after_inserting_nothing() {
        let mut document = Document::new("uno");
        document.insert(at(0, 1), "").unwrap();

        assert!(!document.can_undo());
        assert!(!document.undo());
    }

    #[test]
    fn undo_is_rejected_after_removing_nothing() {
        let mut document = Document::new("uno");
        document.remove(TextRange::collapsed(at(0, 1))).unwrap();

        assert!(!document.can_undo());
        assert!(!document.undo());
    }

    #[test]
    fn each_edit_is_undone_one_at_a_time() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.insert(at(0, 4), "?").unwrap();

        assert!(document.undo());
        assert_eq!(document.buffer().text(), "uno!");

        assert!(document.undo());
        assert_eq!(document.buffer().text(), "uno");

        assert!(!document.undo());
    }

    #[test]
    fn undo_restores_text_after_a_remove() {
        let mut document = Document::new("uno\ndos");
        document.remove(TextRange::new(at(0, 0), at(1, 2))).unwrap();
        assert_eq!(document.buffer().text(), "s");

        assert!(document.undo());

        assert_eq!(document.buffer().text(), "uno\ndos");
    }

    #[test]
    fn a_rejected_edit_does_not_add_an_undo_step() {
        let mut document = Document::new("uno");

        let result = document.insert(at(9, 9), "X");

        assert!(matches!(result, Err(CoreError::InvalidPosition(_))));
        assert!(!document.can_undo());
        assert!(!document.undo());
    }

    #[test]
    fn undo_back_to_the_saved_state_clears_the_modified_state() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.mark_saved();
        document.insert(at(0, 4), "?").unwrap();

        assert!(document.undo());

        assert_eq!(document.buffer().text(), "uno!");
        assert!(!document.is_modified());
    }

    #[test]
    fn undo_while_other_edits_remain_keeps_the_document_modified() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.mark_saved();
        document.insert(at(0, 4), "?").unwrap();
        document.insert(at(0, 5), "#").unwrap();

        assert!(document.undo());

        assert_eq!(document.buffer().text(), "uno!?");
        assert!(document.is_modified());
    }

    #[test]
    fn a_new_document_cannot_be_redone() {
        let mut document = Document::new("uno");

        assert!(!document.can_redo());
        assert!(!document.redo());
        assert_eq!(document.buffer().text(), "uno");
    }

    #[test]
    fn redo_reapplies_an_undone_edit() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.undo();

        assert!(document.redo());

        assert_eq!(document.buffer().text(), "uno!");
    }

    #[test]
    fn redo_restores_the_cursor_of_the_undone_edit() {
        let mut document = Document::new("uno");
        document.move_to(at(0, 2));
        document.replace("!").unwrap();
        document.undo();
        assert_eq!(document.selection().at(), at(0, 2));

        assert!(document.redo());

        assert_eq!(document.buffer().text(), "un!o");
        assert_eq!(document.selection().at(), at(0, 3));
    }

    #[test]
    fn redo_restores_the_selection_of_the_undone_edit() {
        let mut document = Document::new("uno");
        document.select(at(0, 1), at(0, 3));
        document.replace("X").unwrap();
        document.undo();

        assert!(document.redo());

        assert_eq!(document.buffer().text(), "uX");
        assert_eq!(document.selection().at(), at(0, 2));
        assert_eq!(document.selection().range(), None);
    }

    #[test]
    fn edits_can_be_redone_one_at_a_time() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.insert(at(0, 4), "?").unwrap();
        document.undo();
        document.undo();

        assert!(document.redo());
        assert_eq!(document.buffer().text(), "uno!");

        assert!(document.redo());
        assert_eq!(document.buffer().text(), "uno!?");

        assert!(!document.redo());
    }

    #[test]
    fn undo_after_redo_goes_back_again() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.undo();
        document.redo();

        assert!(document.undo());

        assert_eq!(document.buffer().text(), "uno");
    }

    #[test]
    fn redo_restores_text_after_a_remove() {
        let mut document = Document::new("uno\ndos");
        document.remove(TextRange::new(at(0, 0), at(1, 2))).unwrap();
        document.undo();

        assert!(document.redo());

        assert_eq!(document.buffer().text(), "s");
    }

    #[test]
    fn a_new_edit_discards_the_redo_history() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.undo();

        document.insert(at(0, 3), "?").unwrap();

        assert!(!document.can_redo());
        assert!(!document.redo());
        assert_eq!(document.buffer().text(), "uno?");
    }

    #[test]
    fn editing_by_removing_also_discards_the_redo_history() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.undo();

        document.remove(TextRange::new(at(0, 0), at(0, 1))).unwrap();

        assert!(!document.can_redo());
        assert!(!document.redo());
    }

    #[test]
    fn editing_by_replacing_also_discards_the_redo_history() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.undo();
        document.select(at(0, 0), at(0, 1));

        document.replace("Z").unwrap();

        assert!(!document.can_redo());
        assert!(!document.redo());
    }

    #[test]
    fn moving_the_cursor_keeps_the_redo_history() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.undo();

        document.move_to(at(0, 1));

        assert!(document.can_redo());
        assert!(document.redo());
        assert_eq!(document.buffer().text(), "uno!");
    }

    #[test]
    fn marking_saved_keeps_the_redo_history() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.undo();

        document.mark_saved();

        assert!(document.can_redo());
        assert!(document.redo());
        assert_eq!(document.buffer().text(), "uno!");
    }

    #[test]
    fn a_rejected_edit_keeps_the_redo_history() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.undo();

        let result = document.insert(at(9, 9), "X");

        assert!(matches!(result, Err(CoreError::InvalidPosition(_))));
        assert!(document.can_redo());
        assert!(document.redo());
        assert_eq!(document.buffer().text(), "uno!");
    }

    #[test]
    fn redo_restores_the_modified_state() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.mark_saved();
        document.insert(at(0, 4), "?").unwrap();
        document.undo();
        assert!(!document.is_modified());

        assert!(document.redo());

        assert_eq!(document.buffer().text(), "uno!?");
        assert!(document.is_modified());
    }

    #[test]
    fn a_match_can_be_replaced() {
        let mut document = Document::new("uno dos");

        document
            .replace_match(TextRange::new(at(0, 4), at(0, 7)), "tres")
            .unwrap();

        assert_eq!(document.buffer().text(), "uno tres");
        assert_eq!(document.selection().at(), at(0, 8));
        assert_eq!(document.selection().range(), None);
        assert!(document.is_modified());
    }

    #[test]
    fn a_replaced_match_leaves_one_undo_step() {
        let mut document = Document::new("uno dos");

        document
            .replace_match(TextRange::new(at(0, 4), at(0, 7)), "tres")
            .unwrap();

        assert!(document.undo());

        assert_eq!(document.buffer().text(), "uno dos");
        assert!(!document.can_undo());
    }

    #[test]
    fn a_rejected_replacement_leaves_the_document_untouched() {
        let mut document = Document::new("uno dos");

        let result = document.replace_match(TextRange::new(at(0, 4), at(9, 9)), "tres");

        assert!(matches!(result, Err(CoreError::InvalidPosition(_))));
        assert_eq!(document.buffer().text(), "uno dos");
        assert!(!document.is_modified());
        assert!(!document.can_undo());
    }

    #[test]
    fn every_match_can_be_replaced_at_once() {
        let mut document = Document::new("uno dos uno");

        let replaced = document.replace_all("uno", "tres").unwrap();

        assert_eq!(replaced, 2);
        assert_eq!(document.buffer().text(), "tres dos tres");
    }

    #[test]
    fn replacing_every_match_is_a_single_undo_step() {
        let mut document = Document::new("uno dos uno tres uno");

        document.replace_all("uno", "x").unwrap();

        assert!(document.undo());

        assert_eq!(document.buffer().text(), "uno dos uno tres uno");
        assert!(!document.can_undo());
    }

    #[test]
    fn replacing_all_with_shorter_text_keeps_the_positions_right() {
        let mut document = Document::new("aaaa");

        let replaced = document.replace_all("aa", "b").unwrap();

        assert_eq!(replaced, 2);
        assert_eq!(document.buffer().text(), "bb");
    }

    #[test]
    fn replacing_everything_with_more_lines_works() {
        let mut document = Document::new("uno");

        let replaced = document.replace_all("uno", "1\n2").unwrap();

        assert_eq!(replaced, 1);
        assert_eq!(document.buffer().text(), "1\n2");
    }

    #[test]
    fn replacing_everything_with_nothing_removes_it() {
        let mut document = Document::new("uno");

        let replaced = document.replace_all("uno", "").unwrap();

        assert_eq!(replaced, 1);
        assert_eq!(document.buffer().text(), "");
    }

    #[test]
    fn replacing_everything_discards_the_redo_history() {
        let mut document = Document::new("uno");
        document.insert(at(0, 3), "!").unwrap();
        document.undo();

        document.replace_all("uno", "x").unwrap();

        assert!(!document.can_redo());
        assert!(!document.redo());
    }

    #[test]
    fn replacing_without_matches_changes_nothing() {
        let mut document = Document::new("uno dos");

        let replaced = document.replace_all("xyz", "a").unwrap();

        assert_eq!(replaced, 0);
        assert_eq!(document.buffer().text(), "uno dos");
        assert!(!document.is_modified());
        assert!(!document.can_undo());
    }

    #[test]
    fn replacing_with_an_empty_needle_changes_nothing() {
        let mut document = Document::new("uno dos");

        let replaced = document.replace_all("", "a").unwrap();

        assert_eq!(replaced, 0);
        assert_eq!(document.buffer().text(), "uno dos");
        assert!(!document.can_undo());
    }

    #[test]
    fn replacing_everything_leaves_the_selection_valid() {
        let mut document = Document::new("uno\ndos");
        document.move_to(at(1, 3));

        document.replace_all("uno\ndos", "a").unwrap();

        assert_eq!(document.buffer().text(), "a");
        assert_eq!(document.selection().at(), at(0, 1));
    }

    fn path(value: &str) -> DocumentPath {
        DocumentPath::new(value).unwrap()
    }

    #[test]
    fn a_document_path_keeps_its_value() {
        let document_path = path("src/main.rs");

        assert_eq!(document_path.as_path(), Path::new("src/main.rs"));
    }

    #[test]
    fn a_document_path_gives_its_file_name() {
        let document_path = path("src/main.rs");

        assert_eq!(document_path.file_name(), "main.rs");
    }

    #[test]
    fn an_empty_document_path_is_rejected() {
        let result = DocumentPath::new("");

        assert!(matches!(result, Err(CoreError::InvalidPath(_))));
    }

    #[test]
    fn a_document_path_without_a_file_name_is_rejected() {
        let result = DocumentPath::new("..");

        assert!(matches!(result, Err(CoreError::InvalidPath(_))));
    }

    #[test]
    fn a_tab_keeps_its_path_and_its_document() {
        let tab = Tab::new(path("src/main.rs"), Document::new("uno"));

        assert_eq!(tab.file_name(), "main.rs");
        assert_eq!(tab.document().buffer().text(), "uno");
    }

    #[test]
    fn a_new_tab_is_not_modified() {
        let tab = Tab::new(path("src/main.rs"), Document::new("uno"));

        assert!(!tab.is_modified());
    }

    #[test]
    fn a_tab_reports_its_document_as_modified() {
        let mut tab = Tab::new(path("src/main.rs"), Document::new("uno"));

        tab.document_mut().insert(at(0, 3), "!").unwrap();

        assert!(tab.is_modified());
        assert_eq!(tab.document().buffer().text(), "uno!");
    }

    #[test]
    fn a_new_set_of_tabs_is_empty() {
        let tabs = OpenTabs::new();

        assert!(tabs.is_empty());
        assert_eq!(tabs.len(), 0);
    }

    #[test]
    fn a_document_can_be_opened() {
        let mut tabs = OpenTabs::new();

        let index = tabs.open(path("src/main.rs"), "uno");

        assert_eq!(index, 0);
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs.get(0).unwrap().document().buffer().text(), "uno");
    }

    #[test]
    fn several_documents_can_be_open_at_once() {
        let mut tabs = OpenTabs::new();

        let first = tabs.open(path("src/main.rs"), "uno");
        let second = tabs.open(path("src/other.rs"), "dos");

        assert_eq!(first, 0);
        assert_eq!(second, 1);
        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs.get(1).unwrap().file_name(), "other.rs");
    }

    #[test]
    fn the_same_document_cannot_be_open_twice() {
        let mut tabs = OpenTabs::new();

        let first = tabs.open(path("src/main.rs"), "uno");
        let again = tabs.open(path("src/main.rs"), "otro texto");

        assert_eq!(first, again);
        assert_eq!(tabs.len(), 1);
    }

    #[test]
    fn opening_an_open_document_keeps_its_edits() {
        let mut tabs = OpenTabs::new();
        let index = tabs.open(path("src/main.rs"), "uno");
        tabs.get_mut(index)
            .unwrap()
            .document_mut()
            .insert(at(0, 3), "!")
            .unwrap();

        let again = tabs.open(path("src/main.rs"), "otro texto");

        assert_eq!(again, index);
        assert_eq!(tabs.get(index).unwrap().document().buffer().text(), "uno!");
    }

    #[test]
    fn reopening_an_open_document_keeps_its_history() {
        let mut tabs = OpenTabs::new();
        let index = tabs.open(path("src/main.rs"), "uno");
        tabs.get_mut(index)
            .unwrap()
            .document_mut()
            .insert(at(0, 3), "!")
            .unwrap();

        tabs.open(path("src/main.rs"), "uno");

        assert!(tabs.get_mut(index).unwrap().document_mut().undo());
        assert_eq!(tabs.get(index).unwrap().document().buffer().text(), "uno");
    }

    #[test]
    fn an_open_document_is_found_by_its_path() {
        let mut tabs = OpenTabs::new();
        tabs.open(path("src/main.rs"), "uno");
        tabs.open(path("src/other.rs"), "dos");

        assert_eq!(tabs.index_of(&path("src/other.rs")), Some(1));
    }

    #[test]
    fn a_path_that_is_not_open_is_not_found() {
        let tabs = OpenTabs::new();

        assert_eq!(tabs.index_of(&path("src/main.rs")), None);
    }

    #[test]
    fn editing_a_tab_leaves_the_others_alone() {
        let mut tabs = OpenTabs::new();
        tabs.open(path("src/main.rs"), "uno");
        let other = tabs.open(path("src/other.rs"), "dos");

        tabs.get_mut(0)
            .unwrap()
            .document_mut()
            .insert(at(0, 3), "!")
            .unwrap();

        assert_eq!(tabs.get(other).unwrap().document().buffer().text(), "dos");
        assert!(!tabs.get(other).unwrap().is_modified());
    }

    #[test]
    fn a_closed_tab_is_given_back_and_is_gone() {
        let mut tabs = OpenTabs::new();
        tabs.open(path("src/main.rs"), "uno");

        let closed = tabs.close(0);

        assert_eq!(closed.unwrap().file_name(), "main.rs");
        assert_eq!(tabs.len(), 0);
        assert!(tabs.is_empty());
    }

    #[test]
    fn closing_a_tab_that_is_not_open_gives_nothing() {
        let mut tabs = OpenTabs::new();

        assert!(tabs.close(9).is_none());
    }

    #[test]
    fn the_open_tabs_can_be_walked() {
        let mut tabs = OpenTabs::new();
        tabs.open(path("src/main.rs"), "uno");
        tabs.open(path("src/other.rs"), "dos");

        let names: Vec<&str> = tabs.iter().map(Tab::file_name).collect();

        assert_eq!(names, vec!["main.rs", "other.rs"]);
    }

    /// El nombre del archivo de la pestaña activa, o `None` si no hay ninguna.
    ///
    /// Se compara por nombre y no por indice a proposito: el indice depende de cuantas
    /// pestañas haya abiertas, y lo que se quiere saber es que se esta viendo el
    /// documento correcto.
    fn activa(tabs: &OpenTabs) -> Option<String> {
        tabs.active()
            .and_then(|indice| tabs.get(indice))
            .map(|tab| tab.file_name().to_owned())
    }

    #[test]
    fn a_new_set_of_tabs_has_no_active_one() {
        let tabs = OpenTabs::new();

        assert_eq!(tabs.active(), None);
    }

    /// Abrir un documento lo deja activo.
    ///
    /// Abrir es pedir que se vea: si se abriera un documento y no se viera, habria que
    /// pedirlo otra vez para que apareciera, y el usuario tendria que adivinar por que no
    /// pasa nada.
    #[test]
    fn opening_a_document_leaves_it_active() {
        let mut tabs = OpenTabs::new();

        tabs.open(path("src/main.rs"), "uno");

        assert_eq!(activa(&tabs).as_deref(), Some("main.rs"));
    }

    /// La ultima pestaña abierta es la activa, no la primera.
    ///
    /// Va en su propio test porque es la otra mitad de lo mismo: si se comprobara solo que
    /// "se abre algo activo", pasaria con la primera pestaña y no con la ultima, que es la
    /// que se ha pedido.
    #[test]
    fn the_last_document_opened_is_the_active_one() {
        let mut tabs = OpenTabs::new();
        tabs.open(path("src/main.rs"), "uno");

        tabs.open(path("src/other.rs"), "dos");

        assert_eq!(activa(&tabs).as_deref(), Some("other.rs"));
    }

    /// Abrir un documento que ya esta abierto lo trae a delante.
    ///
    /// Es lo que el usuario quiere al pulsarlo otra vez en el explorador: no crear una
    /// segunda pestaña del mismo archivo, sino volver a la que ya tenia con sus cambios.
    #[test]
    fn reopening_an_open_document_brings_its_tab_forward() {
        let mut tabs = OpenTabs::new();
        tabs.open(path("src/main.rs"), "uno");
        tabs.open(path("src/other.rs"), "dos");

        let index = tabs.open(path("src/main.rs"), "otro texto");

        assert_eq!(index, 0);
        assert_eq!(tabs.len(), 2);
        assert_eq!(activa(&tabs).as_deref(), Some("main.rs"));
    }

    #[test]
    fn the_active_tab_can_be_changed() {
        let mut tabs = OpenTabs::new();
        tabs.open(path("src/main.rs"), "uno");
        tabs.open(path("src/other.rs"), "dos");

        let changed = tabs.activate(0);

        assert!(changed);
        assert_eq!(activa(&tabs).as_deref(), Some("main.rs"));
    }

    /// Activar una pestaña que no existe no cambia nada.
    ///
    /// El indice lo elige quien llama, que puede estar equivocado: activar no puede
    /// inventarse una pestaña ni dejar la activa en un sitio que ya no es el suyo.
    #[test]
    fn activating_a_tab_that_is_not_open_changes_nothing() {
        let mut tabs = OpenTabs::new();
        tabs.open(path("src/main.rs"), "uno");

        let changed = tabs.activate(9);

        assert!(!changed);
        assert_eq!(activa(&tabs).as_deref(), Some("main.rs"));
    }

    /// Al cerrar la activa pasa a estarlo la que ocupa su sitio.
    ///
    /// La politica se decide aqui y no donde se guarda la lista porque es de este tipo de
    /// dato y en ningun otro sitio: es la que hace que cerrar la pestaña de en medio no
    /// deje al usuario sin nada que ver, que es la sensacion de haber perdido el
    /// documento.
    #[test]
    fn al_cerrar_la_activa_pasa_a_ser_activa_la_que_ocupa_su_sitio() {
        let mut tabs = OpenTabs::new();
        tabs.open(path("uno.rs"), "uno");
        tabs.open(path("dos.rs"), "dos");
        tabs.open(path("tres.rs"), "tres");
        tabs.activate(1);

        tabs.close(1);

        assert_eq!(activa(&tabs).as_deref(), Some("tres.rs"));
    }

    /// Si la activa era la ultima, pasa a estarlo la nueva ultima.
    ///
    /// La otra mitad de la misma politica: donde no hay sitio al que irse hacia delante,
    /// se mira hacia atras, y no se deja al usuario sin nada abierto.
    #[test]
    fn al_cerrar_la_ultima_activa_pasa_a_ser_activa_la_nueva_ultima() {
        let mut tabs = OpenTabs::new();
        tabs.open(path("uno.rs"), "uno");
        tabs.open(path("dos.rs"), "dos");
        tabs.activate(1);

        tabs.close(1);

        assert_eq!(activa(&tabs).as_deref(), Some("uno.rs"));
    }

    #[test]
    fn al_cerrar_la_unica_pestana_no_queda_ninguna_activa() {
        let mut tabs = OpenTabs::new();
        tabs.open(path("uno.rs"), "uno");

        tabs.close(0);

        assert_eq!(tabs.active(), None);
        assert_eq!(activa(&tabs), None);
    }

    /// Cerrar otra pestaña deja abierta la que se estaba viendo.
    ///
    /// Aunque la que se cierre este antes: el indice se mueve, y lo que tiene que seguir
    /// activo es el mismo documento, no el mismo numero.
    /// Cerrar otra pestaña deja abierta la que se estaba viendo.
    ///
    /// Aunque la que se cierre este antes: el indice se mueve, y lo que tiene que seguir
    /// activo es el mismo documento, no el mismo numero.
    #[test]
    fn al_cerrar_otra_pestana_no_cambia_el_documento_que_se_ve() {
        let mut tabs = OpenTabs::new();
        tabs.open(path("uno.rs"), "uno");
        tabs.open(path("dos.rs"), "dos");
        tabs.open(path("tres.rs"), "tres");
        tabs.activate(2);

        tabs.close(0);

        assert_eq!(activa(&tabs).as_deref(), Some("tres.rs"));
    }

    /// Un archivo con un cursor que se puede mover con el teclado.
    ///
    /// Tres líneas de distinto largo para que se vea que moverse no es contar caracteres:
    /// en la línea corta no se puede pasar de su final y en la larga sí, y arriba y abajo
    /// la columna se recorta a lo que quepa en la línea de destino.
    fn documento_para_mover() -> Document {
        Document::new("uno\ndos\ntres")
    }

    #[test]
    fn el_cursor_se_mueve_a_la_izquierda() {
        let mut document = documento_para_mover();
        document.move_to(at(0, 2));

        document.move_left();

        assert_eq!(document.selection().at(), at(0, 1));
    }

    #[test]
    fn el_cursor_se_mueve_a_la_derecha() {
        let mut document = documento_para_mover();
        document.move_to(at(1, 1));

        document.move_right();

        assert_eq!(document.selection().at(), at(1, 2));
    }

    #[test]
    fn el_cursor_sube_y_baja_entre_lineas() {
        let mut document = documento_para_mover();
        document.move_to(at(2, 2));

        document.move_up();
        assert_eq!(document.selection().at(), at(1, 2));

        document.move_down();
        assert_eq!(document.selection().at(), at(2, 2));
    }

    /// Moverse a la izquierda desde el principio de una línea salta al final de la de
    /// arriba.
    ///
    /// Es el comportamiento de cualquier editor y el que hace que mover a la izquierda
    /// repetidamente llegue al principio del documento: si se parara en el principio de la
    /// línea habría que pulsarlo tantas veces como líneas tiene el archivo.
    #[test]
    fn el_principio_de_una_linea_va_al_final_de_la_anterior() {
        let mut document = documento_para_mover();
        document.move_to(at(1, 0));

        document.move_left();

        assert_eq!(document.selection().at(), at(0, 3));
    }

    /// El final de una línea salta al principio de la de abajo, y no más allá de la última.
    ///
    /// Las dos partes en un test porque son la misma regla vista desde los dos lados: el
    /// cursor recorre el documento entero como si fuera una sola línea de texto.
    #[test]
    fn el_final_de_una_linea_va_al_principio_de_la_siguiente() {
        let mut document = documento_para_mover();
        document.move_to(at(0, 3));

        document.move_right();
        assert_eq!(document.selection().at(), at(1, 0));

        document.move_to(at(2, 4));
        document.move_right();
        assert_eq!(document.selection().at(), at(2, 4));
    }

    /// Moverse arriba o abajo deja la columna donde cabe en la línea de destino.
    ///
    /// Sin recortar, bajar desde la última columna de una línea larga llevaría a una
    /// posición que no existe, y el cursor quedaría apuntando a un sitio donde no hay
    /// nada que escribir.
    #[test]
    fn al_subir_o_bajar_la_columna_se_recorta_a_la_linea_de_destino() {
        let mut document = documento_para_mover();
        document.move_to(at(1, 3));

        document.move_up();

        assert_eq!(document.selection().at(), at(0, 3));
    }

    /// Moverse no toca el contenido ni marca el documento.
    ///
    /// Es lo que distingue mover de editar: si moverse marcara el documento, cualquier
    /// paseo con las flechas dejaría el archivo con cambios sin guardar.
    #[test]
    fn moverse_no_modifica_el_documento() {
        let mut document = documento_para_mover();
        document.move_to(at(0, 0));

        document.move_right();
        document.move_down();
        document.move_left();
        document.move_up();

        assert_eq!(document.buffer().text(), "uno\ndos\ntres");
        assert!(!document.is_modified());
        assert!(!document.can_undo());
    }

    /// Moverse con una selección la deja atrás.
    ///
    /// Sin quitarla, cada flecha alargaría la selección, que es lo que se hace pulsando
    /// Mayúsculas. Un editor en el que moverse selecciona lo que va dejando atrás no
    /// deja escribir donde quiere uno.
    #[test]
    fn moverse_deja_la_seleccion_atras() {
        let mut document = documento_para_mover();
        document.select(at(0, 3), at(0, 0));

        document.move_right();

        assert!(document.selection().is_empty());
        assert_eq!(document.selection().at(), at(0, 1));
    }

    /// Las cuatro direcciones del cursor se mueven en el mismo sitio.
    ///
    /// Se comprueba con las cuatro en un test porque el fallo interesante es que una se
    /// quede sin hacer nada, y con cuatro tests sueltos eso se vería como un test que
    /// falla y no como un movimiento que no existe.
    #[test]
    fn las_cuatro_direcciones_mueven_el_cursor() {
        let mut document = documento_para_mover();
        let inicio = document.selection().at();

        document.move_right();
        let derecha = document.selection().at();
        document.move_down();
        let abajo = document.selection().at();
        document.move_left();
        let izquierda = document.selection().at();
        document.move_up();

        assert_eq!(derecha, at(0, 1));
        assert_eq!(abajo, at(1, 1));
        assert_eq!(izquierda, at(1, 0));
        assert_eq!(document.selection().at(), inicio);
    }
}
