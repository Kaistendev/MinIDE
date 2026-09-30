//! El editor: el documento que se está viendo y su texto.
//!
//! Aquí se pinta el documento y se le pasa lo que el usuario escribe, y no hay nada más.
//! El texto, el cursor, la selección y lo modificado son del core: el editor los pide
//! prestados, los enseña y lo que se escribe entra por el documento, que es lo único que
//! puede marcarlo como modificado y lo único que sabe deshacerlo. Un editor con su propio
//! texto tendría dos copias del archivo, y la que no se editara sería la que se guardara.
//!
//! No se usa el `TextEdit` de egui porque trae su propio texto: es un widget con su buffer,
//! y este editor tiene que escribir en el del core. Se pinta a mano por eso, y por lo
//! mismo el ancho de un carácter y el alto de una línea son los que hay que usar para
//! saber dónde está el cursor, que es lo que no se puede pedir a egui sin su texto.

use crate::document::{TextPosition, TextRange};
use crate::editor::Document;
use crate::language::LanguageProvider;

use super::acciones::{self, Accion, BUSCAR, COPIAR, CORTAR, DESHACER, PEGAR, REEMPLAZAR, REHACER};
use super::app::App;
use super::layout::Zona;
use eframe::egui;

/// La fuente con la que se ve el código.
///
/// Monoespaciada porque en un editor el sitio de un carácter se calcula con su número de
/// columna, y eso solo es cierto si todos los caracteres ocupan lo mismo. Con la fuente de
/// proporción normal, la columna diez de una línea no está debajo de la columna diez de la
/// de abajo, y el cursor y la selección no se pueden poner en su sitio.
const ESTILO: egui::TextStyle = egui::TextStyle::Monospace;

/// El aire que hay alrededor del texto, en puntos.
const MARGEN: f32 = 4.0;

/// El hueco de la izquierda donde van los números de línea, en puntos.
///
/// Entra en el ancho de la columna de números y por eso el texto no empieza en el margen
/// de la ventana: si empezara ahí, la línea wouldn't estar debajo de su número.
const ANCHO_DE_LOS_NUMEROS: f32 = 34.0;

/// Lo que mide de ancho el cursor, en puntos.
///
/// Fino porque es el cursor: un cursor ancho tapa el carácter que tiene debajo, que es
/// justo el sitio donde se está escribiendo.
const GROSOR_DEL_CURSOR: f32 = 2.0;

/// La comilla que abre y cierra una cadena.
///
/// Va aquí y no en el lenguaje porque en los dos lenguajes del MVP es la misma, y porque
/// un editor que no sabe qué es una cadena no puede pintar las cadenas de otro color. Si
/// algún día hubiera un lenguaje donde no fuera, la cotización pasa al proveedor del
/// lenguaje, que es donde vive lo que es de cada lenguaje.
const COMILLA: char = '"';

/// Un documento con el lenguaje en que está escrito: lo que el editor tiene que enseñar.
///
/// El documento es del core y entra prestado. El lenguaje es el proveedor que le
/// corresponde a ese archivo, y el editor no sabe de qué lenguaje se trata: le pregunta
/// por los comentarios y por las palabras clave, y sin lenguaje pinta el texto entero
/// igual, que es mejor que suponer uno.
pub struct PestanaVisual<'a> {
    documento: Option<&'a mut Document>,
    lenguaje: Option<&'a dyn LanguageProvider>,
}

impl<'a> PestanaVisual<'a> {
    /// El editor del documento de `documento`, escrito en `lenguaje` si se sabe cuál es.
    pub fn nuevo(
        documento: Option<&'a mut Document>,
        lenguaje: Option<&'a dyn LanguageProvider>,
    ) -> Self {
        Self {
            documento,
            lenguaje,
        }
    }

    /// El editor sin documento, que es lo que hay en la ventana mientras no haya ninguno
    /// abierto. FE-058 es la que abre el primero.
    pub fn vacio() -> Self {
        Self::nuevo(None, None)
    }
}

/// Dibuja el editor en `ui`.
///
/// Sin documento no dibuja nada: el editor vacío no es un hueco con un borde, es que no
/// hay nada que mirar. Con documento lo pinta entero, y lo que se ve de él es lo que hay
/// en él: el texto, el cursor donde el core lo tiene, la selección donde el core la tiene
/// y los números de línea a la izquierda.
pub fn panel(ui: &mut egui::Ui, vista: PestanaVisual<'_>, app: &mut App) {
    let Some(documento) = vista.documento else {
        return;
    };

    ui.set_min_size(ui.available_size());

    // El editor escribe solo cuando es lo que tiene el foco, y lo tiene cuando el ratón
    // está dentro: si escribiera con cualquier tecla, el usuario escribiría en el
    // documento mientras está escribiendo en otro sitio, que es un diálogo o una
    // búsqueda.
    // El editor toma su id de la zona en la que vive y no de donde esté dibujado, que es
    // lo que hacen los paneles del layout: si el id dependiera del sitio, al abrirse una
    // pestaña el editor bajaría y perdería el foco sin que el usuario hubiera hecho nada.
    let id = Zona::Central.id().with("editor");
    if ui.rect_contains_pointer(ui.max_rect()) {
        ui.memory_mut(|memoria| memoria.request_focus(id));
    }

    // Las teclas van antes que el dibujo. Lo que se escribe entra en el documento y lo
    // que se ve es lo que hay después de escribir, y no al revés: si se dibujara primero,
    // cada tecla que se escribiera se vería un frame más tarde, que es un retraso que se
    // nota al escribir deprisa.
    teclado(ui, documento, &id);

    let medidas = Medidas::de(ui);
    let texto = documento.buffer().text();
    let lineas: Vec<&str> = texto.split('\n').collect();
    let seleccion = documento.selection().range();
    let cursor = documento.selection().at();
    let ancho_del_texto = lineas
        .iter()
        .map(|linea| linea.chars().count() as f32)
        .fold(0.0_f32, f32::max)
        * medidas.caracter;
    let alto = ui.available_height();
    let ancho = ui.available_width();

    // El desplazamiento lo mueve la rueda y se queda en el estado visual de la ventana,
    // y no en el documento: es dónde se está mirando, no qué se está mirando.
    let pedido = rueda(ui);
    let vertical = desplazamiento_calculado(
        app.state().desplazamiento_vertical(),
        pedido.y,
        alto,
        MARGEN * 2.0 + medidas.linea * lineas.len() as f32,
    );
    let horizontal = desplazamiento_calculado(
        app.state().desplazamiento_horizontal(),
        pedido.x,
        ancho,
        ANCHO_DE_LOS_NUMEROS + MARGEN * 2.0 + ancho_del_texto,
    );
    app.state_mut().set_desplazamiento_vertical(vertical);
    app.state_mut().set_desplazamiento_horizontal(horizontal);

    // El desplazamiento no se aplica a las medidas sino a lo que se pinta: las medidas son
    // las del texto y el texto no se mueve, lo que se mueve es por dónde se enseña.
    let desplazamiento = -egui::vec2(horizontal, vertical);

    if let Some(mancha) = rectangulo_de_seleccion(seleccion, ancho - MARGEN, &medidas) {
        ui.painter().rect_filled(
            mancha.translate(desplazamiento),
            0.0,
            color_de_la_seleccion(),
        );
    }

    for numero in lineas_visibles(vertical, alto, &medidas, lineas.len()) {
        pintar_linea(
            ui,
            numero,
            lineas[numero],
            vista.lenguaje,
            &medidas,
            desplazamiento,
        );
    }

    let cursor = rectangulo_del_cursor(cursor, &medidas).translate(desplazamiento);
    ui.painter().rect_filled(cursor, 0.0, COLOR_DEL_CURSOR);
}

/// Pinta una línea: su número a la izquierda y su texto con su color.
///
/// La línea se pinta desplazada, y su número con ella: un número que se quedara quieto
/// mientras el texto baja sería el número de otra línea, y un gutter que no cuadra con el
/// texto no sirve para señalar una línea.
fn pintar_linea(
    ui: &mut egui::Ui,
    numero: usize,
    linea: &str,
    lenguaje: Option<&dyn LanguageProvider>,
    medidas: &Medidas,
    desplazamiento: egui::Vec2,
) {
    let fuente = ESTILO.resolve(ui.style());
    let arriba = egui::pos2(
        ANCHO_DE_LOS_NUMEROS - MARGEN,
        medidas.punto(numero as u32, 0).y,
    ) + desplazamiento;

    ui.painter().text(
        arriba,
        egui::Align2::LEFT_TOP,
        format!("{}", numero + 1),
        fuente.clone(),
        COLOR_DE_LOS_NUMEROS,
    );

    let mut columna = 0_u32;
    for token in trocear(linea, lenguaje) {
        ui.painter().text(
            medidas.punto(numero as u32, columna) + desplazamiento,
            egui::Align2::LEFT_TOP,
            token.texto,
            fuente.clone(),
            token.clase.color(),
        );
        columna += token.texto.chars().count() as u32;
    }
}

/// Lo que se ha pedido con la rueda en este frame.
///
/// Se lee y se pone a cero, y no solo se lee, porque la rueda es de quien está debajo del
/// ratón: si el editor la leyera y la dejara, el que viniera después se desplazaría otra
/// vez por la misma vuelta. Es lo que hace el scroll area de egui.
fn rueda(ui: &mut egui::Ui) -> egui::Vec2 {
    ui.input_mut(|entrada| {
        let delta = entrada.smooth_scroll_delta;
        entrada.smooth_scroll_delta = egui::Vec2::ZERO;

        delta
    })
}

/// Las acciones del menú contextual del editor. FE-053.
///
/// Son las mismas que el menú Editar, y son las mismas [`Accion`] y no una lista parecida:
/// un menú contextual que escribiera sus propios comandos acabaría siendo un segundo sitio
/// donde corregir "Deshacer", y el día que uno cambiara el otro se quedaría atrás.
///
/// No está aquí la lista del menú Editar porque esa vive en `menu::MENUS` y es un `&'static`
/// de otro módulo; aquí lo que se repite son las acciones, que es lo que tiene que ser la
/// misma cosa para que un clic en un sitio y en el otro pidan lo mismo.
const ACCIONES_DEL_CONTEXTO: &[Accion] =
    &[DESHACER, REHACER, COPIAR, CORTAR, PEGAR, BUSCAR, REEMPLAZAR];

/// Con qué id egui recuerda la zona del editor a la que se le abre el menú contextual.
///
/// Es un id aparte del que usa el editor para el foco porque son dos cosas distintas: el
/// foco es de dónde viene el teclado y el id del menú es de qué zona se ha pulsado con el
/// botón derecho. Si compartieran id, egui no podría saber cuál de los dos es el que se ha
/// pulsado, y el menú se abriría al hacer clic en cualquier parte.
pub fn id_del_contexto() -> egui::Id {
    Zona::Central.id().with("editor.contexto")
}

/// Abre el menú contextual del editor sobre `respuesta`.
///
/// Va con la respuesta del área del editor y no sobre un `Ui` suelto porque egui necesita
/// saber qué zona es la que se ha pulsado con el botón derecho: sin la respuesta, el menú se
/// abriría en el punto del ratón sin importar dónde está el editor, y en una ventana con
/// un panel al lado el menú saldría dentro del panel de al lado.
///
/// Los botones se dibujan con [`acciones::boton`], que es el único sitio donde un clic se
/// convierte en un comando. Por eso lo que hay en el menú contextual del editor y lo que
/// hay en el menú principal piden exactamente lo mismo: las dos listas están escritas con
/// las mismas [`Accion`].
pub fn menu_contextual(respuesta: egui::Response, app: &mut App) {
    respuesta.context_menu(|ui| {
        for accion in ACCIONES_DEL_CONTEXTO {
            acciones::boton(ui, app, *accion);
        }
    });
}

/// Por dónde se está viendo, después de haber aplicado la rueda.
///
/// Se queda siempre dentro del contenido: no se puede desplazar antes del principio ni
/// más allá del final, porque más allá del final no hay nada que mirar y por debajo del
/// principio se vería hueco. El hueco tampoco sale shifting: un desplazamiento mayor que
/// el alto del texto dejaría las líneas en negativo, que no es un sitio donde pintar.
pub fn desplazamiento_calculado(actual: f32, pedido: f32, alto: f32, contenido: f32) -> f32 {
    let maximo = (contenido - alto).max(0.0);

    (actual + pedido).clamp(0.0, maximo)
}

/// Las líneas del documento que se ven con el desplazamiento `vertical`.
///
/// Se cuentan por el alto de la línea y no por el desplazamiento porque el alto de la
/// línea es lo que mide una fila del texto, que es lo que hay que saltarse. Se pintan solo
/// estas: un archivo de cinco mil líneas pintado entero cada frame sería un documentito
/// dibujándose sesenta veces por segundo, y con el desplazamiento a la vista se sabe
/// cuáles son sin mirar la memoria de egui por un id que calcula egui.
fn lineas_visibles(
    vertical: f32,
    alto: f32,
    medidas: &Medidas,
    lineas: usize,
) -> std::ops::Range<usize> {
    let primera = (((MARGEN - vertical) / medidas.linea).floor().max(0.0) as usize).min(lineas);
    let fin = ((((MARGEN + alto - vertical) / medidas.linea).ceil() as usize) + 1).min(lineas);

    primera..fin.max(primera)
}

/// Lo que se hace con las teclas cuando el editor es lo que tiene el foco.
///
/// Lo que se escribe entra por el documento, que es lo que lo marca y lo que lo deshace;
/// y las flechas se lo dejan al documento también, que es quien sabe recorrer el texto.
/// La ventana no sabe qué es una línea ni dónde acaba una, y si lo supiera lo tendría que
/// mantener igual que el core.
///
/// Copiar, cortar y pegar llegan como eventos y no como teclas porque el portapapeles es
/// del sistema: quien lo lee y quien lo escribe es la integración, y aquí solo se le pide
/// lo que hay seleccionado y lo que se quiere pegar.
fn teclado(ui: &mut egui::Ui, documento: &mut Document, id: &egui::Id) {
    if !ui.memory(|memoria| memoria.has_focus(*id)) {
        return;
    }

    let teclas = teclas_pulsadas(ui);
    let del_portapapeles = ui.input(|entrada| {
        entrada
            .events
            .iter()
            .filter_map(|evento| match evento {
                egui::Event::Copy => Some(Portapapeles::Copiar),
                egui::Event::Cut => Some(Portapapeles::Cortar),
                egui::Event::Paste(texto) => Some(Portapapeles::Pegar(texto.clone())),
                _ => None,
            })
            .collect::<Vec<_>>()
    });
    let escrito: Vec<String> = ui.input(|entrada| {
        entrada
            .events
            .iter()
            .filter_map(|evento| match evento {
                egui::Event::Text(texto) => Some(texto.clone()),
                _ => None,
            })
            .collect()
    });

    for (tecla, alargar) in teclas {
        if let Some(direccion) = direccion_de(tecla) {
            mover(documento, direccion, alargar);
        } else {
            borrar(documento, tecla == egui::Key::Delete);
        }
    }

    for operacion in del_portapapeles {
        portapapeles(ui, documento, operacion);
    }

    for texto in escrito {
        // `replace` sustituye la selección si la hay y, si no, inserta en el cursor, que
        // es justo lo que hace escribir encima de lo seleccionado.
        let _ = documento.replace(texto);
    }
}

/// Hacia dónde lleva la tecla de mover el cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direccion {
    Izquierda,
    Derecha,
    Arriba,
    Abajo,
}

/// Lo que se hace con el portapapeles.
enum Portapapeles {
    Copiar,
    Cortar,
    Pegar(String),
}

/// Si `tecla` es una de las cuatro flechas, hacia dónde lleva.
fn direccion_de(tecla: egui::Key) -> Option<Direccion> {
    match tecla {
        egui::Key::ArrowLeft => Some(Direccion::Izquierda),
        egui::Key::ArrowRight => Some(Direccion::Derecha),
        egui::Key::ArrowUp => Some(Direccion::Arriba),
        egui::Key::ArrowDown => Some(Direccion::Abajo),
        _ => None,
    }
}

/// Las teclas de editar que se han pulsado en este frame, con si|Mayús iba pulsado.
///
/// Se mira lo que ha pasado y no lo que queda pulsado, porque una tecla se lee una vez y
/// no mientras se mantenga: con la flecha mantenida el cursor correría a cada frame, que
/// es lo único que se quiere. Con Mayús la misma flecha no mueve el cursor sino que
/// alarga la selección, y son dos cosas distintas con la misma tecla.
fn teclas_pulsadas(ui: &egui::Ui) -> Vec<(egui::Key, bool)> {
    ui.input(|entrada| {
        entrada
            .events
            .iter()
            .filter_map(|evento| match evento {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } if direccion_de(*key).is_some()
                    || *key == egui::Key::Backspace
                    || *key == egui::Key::Delete =>
                {
                    Some((*key, modifiers.shift))
                }
                _ => None,
            })
            .collect()
    })
}

/// Mueve el cursor, o alarga la selección si la tecla iba con Mayús.
fn mover(documento: &mut Document, direccion: Direccion, alargar: bool) {
    if !alargar {
        match direccion {
            Direccion::Izquierda => documento.move_left(),
            Direccion::Derecha => documento.move_right(),
            Direccion::Arriba => documento.move_up(),
            Direccion::Abajo => documento.move_down(),
        }

        return;
    }

    let inicio = documento.selection().at();
    let ancla = documento.selection().anchor().unwrap_or(inicio);
    let mut cursor = crate::editor::Cursor::new(documento.buffer(), inicio);

    match direccion {
        Direccion::Izquierda => cursor.move_left(documento.buffer()),
        Direccion::Derecha => cursor.move_right(documento.buffer()),
        Direccion::Arriba => cursor.move_up(documento.buffer()),
        Direccion::Abajo => cursor.move_down(documento.buffer()),
    }

    documento.select(ancla, cursor.at());
}

/// Copia, corta o pega.
///
/// Copiar y cortar no hacen nada sin selección: copiar lo vacío no es copiar, y cortar lo
/// vacío sería dejar al usuario sin poder deshacer. Pegar sustituye la selección si la hay,
/// que es lo mismo que hace escribir encima de lo seleccionado.
fn portapapeles(ui: &egui::Ui, documento: &mut Document, operacion: Portapapeles) {
    match operacion {
        Portapapeles::Copiar => copiar(documento, ui.ctx(), false),
        Portapapeles::Cortar => copiar(documento, ui.ctx(), true),
        Portapapeles::Pegar(texto) => {
            let _ = documento.replace(texto);
        }
    }
}

/// Pone en el portapapeles lo seleccionado, y lo quita si además es un corte.
///
/// El texto seleccionado lo da el core, que es quien sabe qué hay seleccionado en un
/// documento; aquí solo se copia lo que diga y, si es un corte, se sustituye por nada,
/// que es lo que quita la selección y deja el documento modificado.
fn copiar(documento: &mut Document, contexto: &egui::Context, quitar: bool) {
    let Ok(Some(texto)) = documento.selection().selected_text(documento.buffer()) else {
        return;
    };
    let texto = texto.to_owned();

    contexto.copy_text(texto);

    if quitar {
        let _ = documento.replace("");
    }
}
/// Borra el carácter de delante del cursor o el de detrás.
///
/// Con una selección borra lo seleccionado, y no un carácter: seleccionar cinco y pulsar
/// Retroceso quitaría uno y dejaría la selección puesta, que no es lo que significa
/// borrar. La selección la quita el core con `replace`, que es lo mismo que hace al
/// escribir encima de lo seleccionado.
///
/// El rango de un solo carácter lo monta un `Cursor` del core en vez de calcularlo aquí
/// con las líneas: qué hay antes de un cursor es lógica del texto, y el core ya la tiene y
/// la tiene probada. Si el cursor está en el principio del documento no hay nada delante y
/// el rango queda vacío, que es lo que `remove` hace con un rango vacío: no cambia nada.
fn borrar(documento: &mut Document, hacia_delante: bool) {
    if !documento.selection().is_empty() {
        let _ = documento.replace("");
        return;
    }

    let inicio = documento.selection().at();
    let mut cursor = crate::editor::Cursor::new(documento.buffer(), inicio);

    if hacia_delante {
        cursor.move_right(documento.buffer());
    } else {
        cursor.move_left(documento.buffer());
    }

    let fin = cursor.at();

    if fin != inicio {
        let _ = documento.remove(TextRange::new(inicio, fin));
    }
}

/// Las medidas del texto: lo que mide un carácter y lo que mide una línea.
///
/// Se miden una vez por frame y no en cada trozo porque son de la fuente y la fuente no
/// cambia dentro de un frame; y se guardan para poder dibujar la selección y el cursor
/// sin volver a preguntar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Medidas {
    /// Lo que mide de ancho un carácter, en puntos.
    pub caracter: f32,
    /// Lo que mide de alto una línea, en puntos.
    pub linea: f32,
}

impl Medidas {
    /// Las medidas de la fuente con la que se ve el código.
    pub fn de(ui: &egui::Ui) -> Self {
        let fuente = ESTILO.resolve(ui.style());

        Self {
            caracter: ui.fonts_mut(|fuentes| fuentes.glyph_width(&fuente, ' ')),
            linea: ui.text_style_height(&ESTILO),
        }
    }

    /// El punto donde empieza la columna `columna` de la línea `linea`.
    ///
    /// La columna se cuenta en caracteres, que es como la cuenta el core, y con la fuente
    /// monoespaciada un carácter ocupa lo mismo que otro, así que la columna son las veces
    /// que mide un carácter. Se le suma el hueco de los números de línea para que el texto
    /// empiece a su derecha y no debajo del gutter.
    pub fn punto(&self, linea: u32, columna: u32) -> egui::Pos2 {
        egui::pos2(
            ANCHO_DE_LOS_NUMEROS + MARGEN + self.caracter * columna as f32,
            MARGEN + self.linea * linea as f32,
        )
    }
}

/// El rectángulo del cursor: una raya fina donde el core lo tiene puesto.
///
/// Se dibuja siempre, tenga el editor el foco o no: el cursor es el sitio donde se está
/// escribiendo, y si desapareciera al perder el foco, con el menú abierto no se sabría
/// dónde se va a seguir escribiendo.
pub fn rectangulo_del_cursor(posicion: TextPosition, medidas: &Medidas) -> egui::Rect {
    let inicio = medidas.punto(posicion.line(), posicion.column());

    egui::Rect::from_min_size(inicio, egui::vec2(GROSOR_DEL_CURSOR, medidas.linea))
}

/// El rectángulo que cubre la selección, si la hay.
///
/// Cubre desde donde empieza lo seleccionado hasta donde acaba, y si lo seleccionado
/// salta de línea, las líneas del medio enteras: una selección de tres líneas que solo
/// marcara la primera y la última dejaría el texto del medio sin marcar, que es
/// justamente donde se lee lo que se ha seleccionado.
pub fn rectangulo_de_seleccion(
    rango: Option<TextRange>,
    ancho: f32,
    medidas: &Medidas,
) -> Option<egui::Rect> {
    let rango = rango?;
    let inicio = rango.start();
    let fin = rango.end();
    let arriba = medidas.punto(inicio.line(), inicio.column());
    let abajo = medidas.punto(fin.line(), fin.column());
    let derecha = if fin.line() > inicio.line() {
        ancho
    } else {
        abajo.x
    };

    Some(egui::Rect::from_min_max(
        egui::pos2(arriba.x, medidas.punto(inicio.line(), 0).y),
        egui::pos2(
            derecha.max(arriba.x),
            medidas.punto(fin.line(), 0).y + medidas.linea,
        ),
    ))
}

/// Un trozo de línea y de qué clase es, que es de qué color se pinta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
    /// El texto del trozo, tal cual está en la línea.
    pub texto: &'a str,
    /// De qué clase es.
    pub clase: Clase,
}

impl<'a> Token<'a> {
    /// Un trozo de `texto` de la clase `clase`.
    pub fn nuevo(texto: &'a str, clase: Clase) -> Self {
        Self { texto, clase }
    }
}

/// Qué es un trozo de texto a efectos de cómo se ve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clase {
    /// Código de siempre.
    Normal,
    /// Palabra clave del lenguaje.
    Palabra,
    /// Comentario.
    Comentario,
    /// Cadena.
    Cadena,
}

impl Clase {
    /// El color con el que se pinta un trozo de esta clase.
    ///
    /// El comentario se apaga y la palabra clave se marca porque es lo que hay que
    /// encontrar de un vistazo: la estructura del archivo. Los colores son los de egui
    /// para que el resaltado no sea una cosa propia que hay que mantener en dos temas.
    pub fn color(self) -> egui::Color32 {
        match self {
            Clase::Normal => COLOR_DEL_TEXTO,
            Clase::Palabra => egui::Color32::from_rgb(0x56, 0x9c, 0xd6),
            Clase::Comentario => egui::Color32::from_rgb(0x6a, 0x99, 0x55),
            Clase::Cadena => egui::Color32::from_rgb(0xce, 0x91, 0x78),
        }
    }
}

const COLOR_DEL_TEXTO: egui::Color32 = egui::Color32::from_rgb(0xd4, 0xd4, 0xd4);
const COLOR_DE_LOS_NUMEROS: egui::Color32 = egui::Color32::from_rgb(0x85, 0x85, 0x85);
const COLOR_DEL_CURSOR: egui::Color32 = egui::Color32::from_rgb(0xf0, 0xf0, 0xf0);

/// El color con el que se marca lo seleccionado.
///
/// A medio transparencia para que se vea el texto que hay debajo.
fn color_de_la_seleccion() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(70, 110, 170, 90)
}

/// Parte una línea en trozos de los que se sabe el color.
///
/// Va por la línea entera y no por el archivo porque el resaltado tiene que decidir dónde
/// acaba un comentario y eso solo se sabe leyendo, y va con las reglas más simples que
/// hay: un comentario de una línea se come el resto de la línea, una cadena va desde su
/// comilla hasta la siguiente y una palabra es una racha de letras y cifras.
///
/// Lo que no mira, y por eso es básico: los comentarios de varias líneas, las cadenas de
/// varias líneas y los caracteres de escape. Un archivo con esas cosas se verá con la
/// cadena sin cerrar hasta el final de la línea, que es feo pero no es peor que no
/// pintar nada.
pub fn trocear<'a>(linea: &'a str, lenguaje: Option<&dyn LanguageProvider>) -> Vec<Token<'a>> {
    let Some(lenguaje) = lenguaje else {
        return (!linea.is_empty())
            .then(|| Token::nuevo(linea, Clase::Normal))
            .into_iter()
            .collect();
    };

    let edicion = lenguaje.editing();
    let palabras = lenguaje.keywords();

    if let Some(marca) = edicion.line_comment() {
        if linea.trim_start().starts_with(marca) {
            return vec![Token::nuevo(linea, Clase::Comentario)];
        }
    }

    let mut tokens = Vec::new();
    let resto = linea;
    let mut desde = 0;
    let mut indice = 0;

    let caracteres: Vec<char> = resto.chars().collect();
    while indice < caracteres.len() {
        let caracter = caracteres[indice];
        let aqui = indice_to_byte(resto, indice);

        if caracter == COMILLA {
            let (fin, clase) = if let Some(cerrada) = cerrar_cadena(&caracteres, indice) {
                (cerrada, Clase::Cadena)
            } else {
                (caracteres.len(), Clase::Cadena)
            };

            if desde < aqui {
                tokens.push(Token::nuevo(&resto[desde..aqui], Clase::Normal));
            }
            tokens.push(Token::nuevo(
                &resto[aqui..indice_to_byte(resto, fin)],
                clase,
            ));
            indice = fin;
            desde = indice_to_byte(resto, indice);
            continue;
        }

        if edicion
            .line_comment()
            .is_some_and(|marca| linea[aqui..].starts_with(marca))
        {
            if desde < aqui {
                tokens.push(Token::nuevo(&resto[desde..aqui], Clase::Normal));
            }
            tokens.push(Token::nuevo(&resto[aqui..], Clase::Comentario));
            return tokens;
        }

        if es_de_palabra(caracter) {
            let mut fin = indice;
            while fin < caracteres.len() && es_de_palabra(caracteres[fin]) {
                fin += 1;
            }

            let palabra = &resto[aqui..indice_to_byte(resto, fin)];
            if desde < aqui {
                tokens.push(Token::nuevo(&resto[desde..aqui], Clase::Normal));
            }
            let clase = if palabras.contains(&palabra) {
                Clase::Palabra
            } else {
                Clase::Normal
            };
            tokens.push(Token::nuevo(palabra, clase));
            indice = fin;
            desde = indice_to_byte(resto, fin);
            continue;
        }

        indice += 1;
    }

    if desde < resto.len() {
        tokens.push(Token::nuevo(&resto[desde..], Clase::Normal));
    }

    tokens
}

/// El índice del carácter siguiente a la comilla que cierra la cadena que empieza en
/// `desde`, o `None` si no la cierra en la línea.
fn cerrar_cadena(caracteres: &[char], desde: usize) -> Option<usize> {
    caracteres
        .iter()
        .enumerate()
        .skip(desde + 1)
        .find(|(_, caracter)| **caracter == COMILLA)
        .map(|(indice, _)| indice + 1)
}

/// Si un carácter puede ser parte de una palabra.
fn es_de_palabra(caracter: char) -> bool {
    caracter.is_alphanumeric() || caracter == '_'
}

/// El byte del texto que va después del carácter `indice`.
fn indice_to_byte(texto: &str, indice: usize) -> usize {
    texto
        .char_indices()
        .nth(indice)
        .map_or(texto.len(), |(byte, _)| byte)
}

#[cfg(test)]
mod tests {
    use crate::commands::Command;
    use crate::core::LanguageId;
    use crate::document::TextPosition;
    use crate::editor::Document;
    use crate::frontend::acciones;
    use crate::frontend::App;
    use crate::language::{EditingConfiguration, LanguageProvider};
    use eframe::egui;

    use super::{Clase, Medidas, PestanaVisual, Token};

    /// El menú contextual del editor ofrece lo mismo que el menú Editar. FE-053.
    ///
    /// Se comparan las dos listas enteras y no los comandos que piden: si fueran dos listas
    /// con las mismas acciones escritas dos veces, el día que una cambiara la otra se quedaría
    /// atrás sin que nada lo notara, y lo que se quedaría atrás es la mitad del editor, que
    /// es donde más se usan.
    #[test]
    fn el_menu_contextual_del_editor_es_el_menu_editar() {
        let del_editor: Vec<acciones::Accion> = super::ACCIONES_DEL_CONTEXTO.to_vec();
        let del_menu: Vec<acciones::Accion> = crate::frontend::menu::MENUS
            .iter()
            .find(|menu| menu.titulo == "Editar")
            .map(|menu| menu.acciones.to_vec())
            .unwrap_or_else(|| panic!("la barra de menús tiene un menú Editar"));

        assert!(
            !del_editor.is_empty(),
            "si la lista está vacía este test no comprueba nada"
        );
        assert_eq!(
            del_editor, del_menu,
            "el menú contextual del editor y el menú Editar ofrecen las mismas acciones, en el \
             mismo orden"
        );
    }

    /// Con el botón derecho sobre el editor se abre su menú contextual. FE-053.
    ///
    /// El botón derecho y no el izquierdo porque es el que lo abre, y porque con el otro hay
    /// que comprobar que el menú no aparece: si saliera, cada vez que se pulsara el ratón en
    /// un documento saltaría un menú encima.
    #[test]
    fn el_menu_contextual_del_editor_aparece_con_el_boton_derecho() {
        let contexto = egui::Context::default();
        let mut app = App::new();

        assert!(
            !contexto.any_popup_open(),
            "sin pulsar nada no hay ningún menú abierto"
        );

        let antes = rectangulos(&contexto, &mut app);
        for pressed in [true, false] {
            ventana(
                &contexto,
                &mut app,
                &raton_sobre(
                    egui::pos2(450.0, 200.0),
                    egui::PointerButton::Secondary,
                    pressed,
                ),
            );
        }

        let elementos = nuevos(&contexto, &mut app, &antes);
        assert_eq!(
            elementos.len(),
            super::ACCIONES_DEL_CONTEXTO.len(),
            "el botón derecho sobre el editor tiene que abrir un menú con un botón por acción: \
             {elementos:?}"
        );
    }

    /// Abrir el menú contextual del editor no pide nada. FE-053.
    ///
    /// El clic derecho se usa muchas más veces de las que se pulsa un elemento, así que pedir
    /// en cuanto se abre sería pedir casi siempre.
    #[test]
    fn abrir_el_menu_contextual_del_editor_no_pide_nada() {
        let contexto = egui::Context::default();
        let mut app = App::new();

        for pressed in [true, false] {
            ventana(
                &contexto,
                &mut app,
                &raton_sobre(
                    egui::pos2(450.0, 200.0),
                    egui::PointerButton::Secondary,
                    pressed,
                ),
            );
        }

        assert_eq!(app.peticiones(), Vec::<Command>::new());
    }

    /// El menú contextual pide sus comandos por el mismo sitio que el menú principal.
    /// FE-053.
    ///
    /// Se comprueba el código y no los clics porque lo que tiene que ser la misma cosa es
    /// el camino por el que un clic se convierte en un comando, y eso no se ve desde fuera:
    /// un menú que escribiera sus propios comandos también funcionaría, y el día que uno de
    /// los dos caminos cambiara el otro se quedaría atrás sin que nada lo notara.
    #[test]
    fn el_menu_contextual_del_editor_pide_por_el_camino_del_menu() {
        let fuente =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file!()))
                .expect("el fuente del editor tiene que poder leerse");

        let codigo = fuente
            .split("#[cfg(test)]")
            .next()
            .expect("el editor tiene que tener código antes de los tests");

        let menu = codigo
            .split("fn menu_contextual")
            .nth(1)
            .expect("el editor tiene un menú contextual");

        assert!(
            menu.contains("acciones::boton("),
            "un clic del menú contextual tiene que convertirse en comando por `acciones::boton`, \
             que es el único sitio donde eso pasa"
        );
        assert!(
            !menu.contains("app.emitir("),
            "y el menú no puede pedir por su cuenta: si lo hiciera, el menú y el atajo \
             dejarían de ser la misma operación"
        );
    }

    /// Dibuja la ventana con unos eventos de por medio.
    ///
    /// Se dibuja la ventana entera y no solo el editor porque lo que se comprueba es que el
    /// menú se abre en la ventana y no en un widget suelto, que es donde lo abriría un
    /// editor que se dibujara solo.
    fn ventana(contexto: &egui::Context, app: &mut App, eventos: &[egui::Event]) {
        // Dos frames porque el menu es un area de egui: se crea en el primero y se pinta en
        // el siguiente, y con uno solo el menu todavia no se habria visto.
        let mut antes = contexto.run_ui(entrada_con(&[]), |ui| app.dibujar(ui));
        antes.textures_delta.clear();

        let mut salida = contexto.run_ui(entrada_con(eventos), |ui| app.dibujar(ui));
        salida.textures_delta.clear();
    }

    /// Los rectángulos que se han pintado en la ventana, de izquierda a derecha y sin repetir.
    fn rectangulos(contexto: &egui::Context, app: &mut App) -> Vec<egui::Rect> {
        let mut salida = contexto.run_ui(entrada_con(&[]), |ui| app.dibujar(ui));
        salida.textures_delta.clear();

        let mut rectangulos: Vec<egui::Rect> = salida
            .shapes
            .into_iter()
            .filter_map(|forma| match forma.shape {
                egui::Shape::Rect(rectangulo) => Some(rectangulo.rect),
                _ => None,
            })
            .collect();
        rectangulos.sort_by(|una, otra| una.min.x.total_cmp(&otra.min.x));
        rectangulos.dedup();

        rectangulos
    }

    /// Los rectángulos que no estaban antes de abrir el menú, de arriba abajo.
    ///
    /// Los botones del menú no están en ninguna zona de la ventana, así que se distinguen por
    /// ser nuevos, y van de arriba abajo porque así los escribe el horizontal que los
    /// dibuja y así los ve el usuario.
    fn nuevos(contexto: &egui::Context, app: &mut App, antes: &[egui::Rect]) -> Vec<egui::Rect> {
        let mut nuevos: Vec<egui::Rect> = rectangulos(contexto, app)
            .into_iter()
            .filter(|rectangulo| !antes.contains(rectangulo))
            .collect();
        nuevos.sort_by(|una, otra| una.min.y.total_cmp(&otra.min.y));

        nuevos
    }

    /// Un frame con el raton en pos y luego pulsando con oton.
    ///
    /// El movimiento va aparte del clic porque egui no sabe donde esta el raton hasta que se
    /// le dice: un clic sin movimiento previo no se cuenta como clic sobre ningun widget, que
    /// es lo mismo que pasa con un raton real que se mueve y luego pulsa.
    fn raton_sobre(pos: egui::Pos2, boton: egui::PointerButton, pressed: bool) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: boton,
                pressed,
                modifiers: egui::Modifiers::default(),
            },
        ]
    }

    /// Una ventana de trabajo normal, ni enorme ni mínima.
    const ANCHO: f32 = 1000.0;
    const ALTO: f32 = 800.0;

    /// El hueco donde se dibuja el editor. Alto a propósito para que quepan unas quince
    /// líneas y se vea que un documento largo no cabe entero.
    const ALTO_DEL_EDITOR: f32 = 200.0;

    /// El ancho del hueco del editor, que es ancho de sobra para una línea normal.
    const ANCHO_DEL_EDITOR: f32 = 600.0;

    /// Un frame con una ventana de tamaño conocido.
    fn entrada_con(eventos: &[egui::Event]) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(ANCHO, ALTO),
            )),
            events: eventos.to_vec(),
            ..egui::RawInput::default()
        }
    }

    /// Un lenguaje de mentira con las palabras y el comentario que le digan.
    ///
    /// Va en los tests y no se usan los de verdad a propósito: si el editor se sabe las
    /// palabras clave de algún lenguaje, un test con un lenguaje inventado se seguiría
    /// passando, y lo que se quiere comprobar es que las palabras se las da quien llama.
    struct Falso {
        palabras: &'static [&'static str],
        comentario: Option<&'static str>,
    }

    impl LanguageProvider for Falso {
        fn id(&self) -> LanguageId {
            LanguageId::CSharp
        }

        fn extensions(&self) -> &'static [&'static str] {
            &["falso"]
        }

        fn editing(&self) -> EditingConfiguration {
            EditingConfiguration::new(self.comentario, None, "  ")
        }

        fn keywords(&self) -> &'static [&'static str] {
            self.palabras
        }
    }

    fn csharp_falso() -> Falso {
        Falso {
            palabras: &["CLASE"],
            comentario: Some("//"),
        }
    }

    /// Un texto pintado: qué se ha escrito, dónde y de qué color.
    ///
    /// egui no enseña el texto de los widgets en la salida de un frame, pero sí lo que ha
    /// pintado: cada texto va en su propia forma con su punto y su color, y de eso se
    /// sacan las tres cosas que se pueden comprobar de lo que se ve -qué se ha escrito,
    /// dónde y de qué color- sin inventarse una forma de mirar dentro del widget.
    #[derive(Debug, Clone, PartialEq)]
    struct Pintado {
        texto: String,
        x: f32,
        y: f32,
        /// Lo que ocupa el texto de lado, que es lo que dice si una línea cabe.
        ancho: f32,
        color: egui::Color32,
    }

    /// Dibuja el editor con unos eventos de por medio y devuelve lo que ha pintado.
    fn pintar(
        context: &egui::Context,
        documento: Option<&mut Document>,
        lenguaje: Option<&dyn LanguageProvider>,
        eventos: &[egui::Event],
    ) -> Vec<Pintado> {
        pintar_con(
            context,
            &mut App::new(),
            documento,
            lenguaje,
            eventos,
            ANCHO_DEL_EDITOR,
        )
    }

    /// Lo mismo con una aplicación que se queda entre frames, que es lo que necesita el
    /// editor para recordar por dónde se está viendo, y en un hueco del ancho que se le
    /// diga, para poder tener un editor estrecho.
    fn pintar_con(
        context: &egui::Context,
        app: &mut App,
        documento: Option<&mut Document>,
        lenguaje: Option<&dyn LanguageProvider>,
        eventos: &[egui::Event],
        ancho: f32,
    ) -> Vec<Pintado> {
        pintados_de(&dibujar(context, app, documento, lenguaje, eventos, ancho))
    }

    /// Dibuja el editor y devuelve la salida del frame.
    ///
    /// Se limpia el delta de texturas porque al pintar texto de verdad egui deja fuentes
    /// nuevas en la salida, y egui no deja tirar un delta sin avisar: en la ventana los
    /// aplica, y aquí no hay nadie que los aplique.
    ///
    /// El hueco del editor es de tamaño fijo y no crece con lo que se pinta, porque un
    /// hueco que crece con el contenido es un hueco sin scroll: en la ventana el editor
    /// está dentro de un panel de tamaño fijo, y aquí hay que dárselo.
    fn dibujar(
        context: &egui::Context,
        app: &mut App,
        documento: Option<&mut Document>,
        lenguaje: Option<&dyn LanguageProvider>,
        eventos: &[egui::Event],
        ancho: f32,
    ) -> egui::FullOutput {
        let mut documento = documento;

        let mut salida = context.run_ui(entrada_con(eventos), |ui| {
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(ancho, ALTO_DEL_EDITOR),
                )),
                |ui| {
                    super::panel(
                        ui,
                        PestanaVisual::nuevo(documento.as_deref_mut(), lenguaje),
                        app,
                    );
                },
            );
        });
        salida.textures_delta.clear();

        salida
    }

    /// Los textos que se han pintado, con el punto donde empieza cada uno y su color.
    fn pintados_de(salida: &egui::FullOutput) -> Vec<Pintado> {
        let mut pintados: Vec<Pintado> = salida
            .shapes
            .iter()
            .filter_map(|forma| match &forma.shape {
                egui::Shape::Text(texto) => {
                    let seccion = texto.galley.job.sections.first();

                    Some(Pintado {
                        texto: texto.galley.job.text.to_string(),
                        x: texto.pos.x,
                        y: texto.pos.y,
                        ancho: texto.galley.rect.width(),
                        color: seccion.map_or(egui::Color32::WHITE, |s| s.format.color),
                    })
                }
                _ => None,
            })
            .collect();
        pintados.sort_by(|uno, otro| uno.y.total_cmp(&otro.y).then(uno.x.total_cmp(&otro.x)));

        pintados
    }

    /// Los textos pintados, en orden, pegados.
    fn todo_pintado(context: &egui::Context, documento: &mut Document) -> String {
        pintar(context, Some(documento), None, &[])
            .iter()
            .map(|pintado| pintado.texto.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// El punto del editor donde se hace clic para darle el foco.
    /// Un punto dentro del hueco del editor, que en estos tests está en lo alto a la
    /// izquierda.
    ///
    /// No es el centro de la ventana porque el editor se dibuja en un hueco de doscientas
    /// de alto y el centro de la ventana cae por debajo, fuera de él.
    fn dentro(context: &egui::Context) -> egui::Pos2 {
        let _ = context;

        egui::pos2(200.0, 100.0)
    }

    /// Un clic de verdad, con su pulsación y su soltura, y el puntero donde se ha pulsado.
    ///
    /// El `PointerMoved` va con el clic porque cada frame empieza con la entrada que se le
    /// pasa y no con la de antes: sin el, el clic se registra en un sitio que en el frame
    /// siguiente no existe.
    fn clic(posicion: egui::Pos2) -> Vec<egui::Event> {
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::PointerButton {
                pos: posicion,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            })
            .chain(std::iter::once(egui::Event::PointerMoved(posicion)))
            .collect()
    }

    /// Una tecla, con su pulsación y su soltura, y el puntero donde está.
    ///
    /// El puntero también va aquí por lo mismo que en el clic, y porque el editor solo
    /// escribe cuando el ratón está dentro: es el foco de la ventana y no una casilla
    /// aparte.
    fn tecla(tecla: egui::Key, posicion: egui::Pos2) -> Vec<egui::Event> {
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key: tecla,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            })
            .chain(std::iter::once(egui::Event::PointerMoved(posicion)))
            .collect()
    }

    fn documento(texto: &str) -> Document {
        Document::new(texto)
    }

    fn en(linea: u32, columna: u32) -> TextPosition {
        TextPosition::new(linea, columna)
    }

    // ---------------------------------------------------------------- el troceado

    /// Un comentario se ve desde su marca hasta el final de la línea.
    ///
    /// Es lo más simple que se puede hacer bien: la marca de comentario de una línea se
    /// come el resto de la línea, y lo que hay detrás de la marca es comentario aunque
    /// tenga comillas o palabras clave dentro.
    #[test]
    fn un_comentario_se_ve_hasta_el_final_de_la_linea() {
        let troceado = super::trocear("CLASE // hola CLASE", Some(&csharp_falso()));

        assert_eq!(
            troceado,
            vec![
                Token::nuevo("CLASE", Clase::Palabra),
                Token::nuevo(" ", Clase::Normal),
                Token::nuevo("// hola CLASE", Clase::Comentario),
            ]
        );
    }

    /// Una cadena no se ve como comentario aunque lleve dentro la marca de comentario.
    ///
    /// Va en su propio test porque es donde falla un troceado que va looking for la
    /// marca sin mirar si está dentro de una cadena: `String s = "//";` se pintaría
    /// entero como comentario y el código se perdería de vista.
    #[test]
    fn una_cadena_no_es_un_comentario() {
        let troceado = super::trocear("CLASE = \"// no soy comentario\";", Some(&csharp_falso()));

        assert_eq!(
            troceado,
            vec![
                Token::nuevo("CLASE", Clase::Palabra),
                Token::nuevo(" = ", Clase::Normal),
                Token::nuevo("\"// no soy comentario\"", Clase::Cadena),
                Token::nuevo(";", Clase::Normal),
            ]
        );
    }

    /// Las palabras clave son las que dice el lenguaje, no las que sabe el editor.
    ///
    /// Se comprueba con un lenguaje inventado cuyas palabras no son las de ningún
    /// lenguaje de verdad: `CLASE` se resalta porque el lenguaje la dice, y `namespace` no
    /// se resalta aunque sea palabra clave de C#, porque este lenguaje no la tiene. Si el
    /// editor tuviera las palabras escritas, las dos se resaltarían.
    #[test]
    fn las_palabras_clave_las_dice_el_lenguaje() {
        let troceado = super::trocear("CLASE namespace", Some(&csharp_falso()));

        assert_eq!(
            troceado,
            vec![
                Token::nuevo("CLASE", Clase::Palabra),
                Token::nuevo(" ", Clase::Normal),
                Token::nuevo("namespace", Clase::Normal),
            ]
        );
    }

    /// Sin lenguaje no se distingue nada: el texto se ve entero igual.
    ///
    /// Es lo que hay en un archivo cuyo lenguaje no se sabe, y es mejor a medias que con
    /// un resaltado que puede ser del lenguaje equivocado: si el editor no sabe qué
    /// lenguaje es, no puede saber qué palabras son clave.
    #[test]
    fn sin_lenguaje_no_se_distingue_nada() {
        let troceado = super::trocear("CLASE // hola \"cadena\"", None);

        assert_eq!(
            troceado,
            vec![Token::nuevo("CLASE // hola \"cadena\"", Clase::Normal)]
        );
    }

    /// Una línea vacía se queda sin trozos.
    ///
    /// Sin trozo no hay nada que pintar, y un trozo vacío pinta un rectángulo de ancho
    /// cero, que es ruido en la salida de un frame.
    #[test]
    fn una_linea_vacia_no_tiene_trozos() {
        assert!(super::trocear("", Some(&csharp_falso())).is_empty());
    }

    // ------------------------------------------------------- donde se ve cada cosa

    /// El cursor se ve en la posición que dice el core, no en otra.
    ///
    /// La posición sale del core y aquí solo se convierte en píxeles, que es la parte que
    /// es de la ventana. Se comprueba que la columna y la línea del core son las que
    /// desplazan el cursor, y que un carácter es lo que mide de ancho.
    #[test]
    fn el_cursor_se_ve_donde_dice_el_core() {
        let medidas = Medidas {
            caracter: 10.0,
            linea: 20.0,
        };

        assert_eq!(
            medidas.punto(0, 0),
            medidas.punto(0, 1) - egui::vec2(10.0, 0.0)
        );
        assert_eq!(
            medidas.punto(0, 0),
            medidas.punto(1, 0) - egui::vec2(0.0, 20.0)
        );
        assert_eq!(
            medidas.punto(2, 3),
            medidas.punto(0, 0) + egui::vec2(30.0, 40.0)
        );
    }

    /// El cursor es una raya fina en la columna del core, del alto de una línea.
    ///
    /// Fina y del alto de una línea porque es el cursor: si fuera ancha taparía el
    /// carácter que tiene debajo, y si fuera más baja que la línea se vería que está
    /// entre dos.
    #[test]
    fn el_cursor_es_una_raya_fina_del_alto_de_una_linea() {
        let medidas = Medidas {
            caracter: 10.0,
            linea: 20.0,
        };
        let cursor = super::rectangulo_del_cursor(en(1, 2), &medidas);

        assert_eq!(cursor.width(), super::GROSOR_DEL_CURSOR);
        assert_eq!(cursor.height(), 20.0);
        assert_eq!(cursor.min.y, medidas.punto(1, 2).y);
    }

    /// Sin selección no se pinta nada encima del texto.
    ///
    /// Sin selección, `None`: una mancha donde no hay nada seleccionado sería un documento
    /// con partes de otro color que nadie ha seleccionado.
    #[test]
    fn sin_seleccion_no_hay_mancha() {
        let medidas = Medidas {
            caracter: 10.0,
            linea: 20.0,
        };

        assert!(super::rectangulo_de_seleccion(None, 500.0, &medidas).is_none());
    }

    /// Una selección en una línea va de su columna a la otra.
    ///
    /// Es el caso de un clic con Mayúsculas, y lo que se mira es que la mancha empieza en
    /// la columna donde empieza lo seleccionado y no antes: si empezara en el margen
    /// izquierdo, la marca sería más ancha que lo seleccionado.
    #[test]
    fn una_seleccion_en_una_linea_va_de_una_columna_a_otra() {
        let medidas = Medidas {
            caracter: 10.0,
            linea: 20.0,
        };
        let seleccion = super::rectangulo_de_seleccion(
            Some(crate::document::TextRange::new(en(0, 2), en(0, 5))),
            500.0,
            &medidas,
        )
        .unwrap();

        assert_eq!(seleccion.min.x, medidas.punto(0, 2).x);
        assert_eq!(seleccion.max.x, medidas.punto(0, 5).x);
        assert_eq!(seleccion.height(), 20.0);
    }

    /// Una selección de varias líneas cubre de principio a fin.
    ///
    /// Cubre también las líneas del medio enteras: una selección de tres líneas que solo
    /// se pintara en la primera y en la última dejaría el texto del medio sin marcar, que
    /// es donde se lee lo seleccionado.
    #[test]
    fn una_seleccion_de_varias_lineas_cubre_de_principio_a_fin() {
        let medidas = Medidas {
            caracter: 10.0,
            linea: 20.0,
        };
        let seleccion = super::rectangulo_de_seleccion(
            Some(crate::document::TextRange::new(en(1, 2), en(3, 1))),
            500.0,
            &medidas,
        )
        .unwrap();

        assert_eq!(seleccion.min.x, medidas.punto(1, 2).x);
        assert_eq!(seleccion.max.x, 500.0);
        assert_eq!(seleccion.min.y, medidas.punto(1, 0).y);
        assert_eq!(seleccion.max.y, medidas.punto(3, 0).y + 20.0);
    }

    // ------------------------------------------------------------ lo que se ve de verdad

    /// El editor existe: con un documento enseña su texto.
    ///
    /// Es FE-019 en su forma más pequeña: no hay un rectángulo de "editor" que comprobar,
    /// porque el editor es lo que pinta. Si pinta el texto del documento, existe.
    #[test]
    fn el_editor_enseña_el_texto_del_documento() {
        let context = egui::Context::default();
        let mut document = documento("uno\ndos\ntres");

        let pintado = todo_pintado(&context, &mut document);

        assert!(pintado.contains("uno"), "{pintado}");
        assert!(pintado.contains("dos"), "{pintado}");
        assert!(pintado.contains("tres"), "{pintado}");
    }

    /// El texto que se ve es el del documento, entero y en orden.
    ///
    /// Cada línea tiene que verse tal cual, con su texto y con su sitio: un editor que
    /// enseñara el texto pero cortara la última línea, o las ordenara al revés, estaría
    /// enseñando otro documento.
    #[test]
    fn el_texto_se_ve_entero_y_en_orden() {
        let context = egui::Context::default();
        let mut document = documento("primera\nsegunda\ntercera");

        let pintado = todo_pintado(&context, &mut document);

        let primera = pintado.find("primera").expect("la primera línea se ve");
        let segunda = pintado.find("segunda").expect("la segunda línea se ve");
        let tercera = pintado.find("tercera").expect("la tercera línea se ve");
        assert!(primera < segunda && segunda < tercera, "{pintado}");
    }

    /// Sin documento el editor no enseña nada.
    ///
    /// Es lo que hay en la ventana mientras no haya ningún documento abierto, y es
    /// preferible a un hueco con un borde: el editor sin documento no es un hueco, es que
    /// no hay nada que mirar.
    #[test]
    fn sin_documento_no_se_enseña_nada() {
        let context = egui::Context::default();

        let pintado = pintar(&context, None, None, &[]);

        assert!(
            pintado.is_empty(),
            "sin documento no hay texto que enseñar: {pintado:?}"
        );
    }

    /// El cursor se ve dibujado, y donde el core dice que está.
    ///
    /// Se comprueba en la salida del frame y no solo en el cálculo: el cálculo dice
    /// dónde debería estar el cursor, y lo que importa es que haya una raya en ese sitio.
    #[test]
    fn el_cursor_aparece_donde_el_core_lo_situa() {
        let context = egui::Context::default();
        let mut document = documento("uno\ndos");
        document.move_to(en(1, 2));

        let salida = dibujar(
            &context,
            &mut App::new(),
            Some(&mut document),
            None,
            &[],
            400.0,
        );
        let cursores: Vec<egui::Rect> = salida
            .shapes
            .iter()
            .filter_map(|forma| match &forma.shape {
                egui::Shape::Rect(rectangulo)
                    if rectangulo.rect.width() == super::GROSOR_DEL_CURSOR =>
                {
                    Some(rectangulo.rect)
                }
                _ => None,
            })
            .collect();
        let primera_linea = pintar(&context, Some(&mut document), None, &[])
            .iter()
            .find(|pintado| pintado.texto == "uno")
            .map(|pintado| pintado.y)
            .expect("se ve la primera línea");

        assert_eq!(cursores.len(), 1, "un cursor y solo uno: {cursores:?}");
        assert!(
            cursores[0].min.y > primera_linea,
            "el cursor está en la segunda línea, que es donde el core lo puso: {:?} y {primera_linea}",
            cursores[0]
        );
    }

    /// Escribir pone el texto en el documento del core y no en otro sitio.
    ///
    /// Lo que se comprueba es que el documento del core es el que cambia, y que cambia
    /// de verdad: con contenido, marcado como modificado y con un paso para deshacer. Un
    /// editor con su propio texto no valdría: sus cambios se irían con la ventana.
    #[test]
    fn escribir_pone_el_texto_en_el_documento_del_core() {
        let context = egui::Context::default();
        let mut document = documento("uno");
        pintar(&context, Some(&mut document), None, &clic(dentro(&context)));

        // El texto va en un frame aparte del clic porque el foco se pide al terminar el
        // del clic: si escribieran en el mismo frame, el editor todavía no tendría el foco
        // y no escribiría, que es lo que hace con el ratón fuera.
        let eventos = [
            vec![egui::Event::PointerMoved(dentro(&context))],
            vec![egui::Event::Text("X".to_owned())],
        ]
        .concat();
        pintar(&context, Some(&mut document), None, &eventos);

        assert_eq!(document.buffer().text(), "Xuno");
        assert!(document.is_modified());
        assert!(document.can_undo());
    }

    /// Borrar quita el carácter de al lado del cursor, en el documento del core.
    ///
    /// Se comprueba con la tecla de borrar hacia atrás porque es la que más se usa, y
    /// con la de borrar hacia delante en su propio test, que es un caso distinto: borra lo
    /// que hay delante y no lo de detrás. El cursor se pone antes con el core porque en
    /// esta fase no hay todavía poder situarlo con el ratón: si estuviera al principio del
    /// documento, borrar hacia atrás no tendría nada detrás que borrar, que es lo que
    /// tiene que pasar y no lo que se quiere comprobar aquí.
    #[test]
    fn la_tecla_de_borrar_quita_el_caracter_de_detras() {
        let context = egui::Context::default();
        let mut document = documento("uno");
        document.move_to(en(0, 2));
        pintar(&context, Some(&mut document), None, &clic(dentro(&context)));

        pintar(
            &context,
            Some(&mut document),
            None,
            &tecla(egui::Key::Backspace, dentro(&context)),
        );

        assert_eq!(document.buffer().text(), "uo");
    }

    /// La tecla de borrar hacia delante quita el de delante, no el de detrás.
    ///
    /// Su propio test porque es el otro sentido: si los dos borrados hicieran lo mismo,
    /// uno de los dos tests seguiría pasando.
    #[test]
    fn la_tecla_de_borrar_hacia_delante_quita_el_de_delante() {
        let context = egui::Context::default();
        let mut document = documento("uno");
        pintar(&context, Some(&mut document), None, &clic(dentro(&context)));

        pintar(
            &context,
            Some(&mut document),
            None,
            &tecla(egui::Key::Delete, dentro(&context)),
        );

        assert_eq!(document.buffer().text(), "no");
    }

    /// Las cuatro flechas mueven el cursor del core.
    ///
    /// Se comprueba el cursor del documento, no lo que se ve: si la ventana moviera su
    /// propio cursor y el del core se quedara, se escribiría en un sitio y se vería en
    /// otro.
    #[test]
    fn las_flechas_mueven_el_cursor_del_documento() {
        let context = egui::Context::default();
        let mut document = documento("uno\ndos");
        let inicio = document.selection().at();
        pintar(&context, Some(&mut document), None, &clic(dentro(&context)));

        pintar(
            &context,
            Some(&mut document),
            None,
            &tecla(egui::Key::ArrowRight, dentro(&context)),
        );
        assert_eq!(document.selection().at(), en(0, 1));

        let eventos = [
            tecla(egui::Key::ArrowDown, dentro(&context)),
            tecla(egui::Key::ArrowLeft, dentro(&context)),
        ]
        .concat();
        pintar(&context, Some(&mut document), None, &eventos);
        assert_eq!(document.selection().at(), en(1, 0));

        pintar(
            &context,
            Some(&mut document),
            None,
            &tecla(egui::Key::ArrowUp, dentro(&context)),
        );
        assert_eq!(document.selection().at(), inicio);
    }

    /// Un documento de varias pantallas se puede recorrer.
    ///
    /// Con cuarenta líneas en un hueco de doscientas no caben todas, y solo caben si hay
    /// desplazamiento: se pinta lo que se ve y la rueda lo mueve. Se comprueba que después
    /// de girar la rueda ya no se ve la primera línea, que es la prueba de que se ha movido
    /// y no de que se ha dibujado una barra.
    #[test]
    fn un_documento_de_varias_pantallas_se_puede_recorrer() {
        let context = egui::Context::default();
        let mut app = App::new();
        let lineas: Vec<String> = (1..=40).map(|linea| format!("linea{linea:02}")).collect();
        let mut document = documento(&lineas.join("\n"));

        let depurado = pintar_con(
            &context,
            &mut app,
            Some(&mut document),
            None,
            &[],
            ANCHO_DEL_EDITOR,
        );
        let antes = visibles(&depurado);
        assert!(
            antes.contains("linea01"),
            "al principio se ve la primera: {antes}"
        );
        assert!(
            !antes.contains("linea40"),
            "y no se ve la última, que no cabe: {antes}"
        );

        // Una vuelta por frame, que es como se gira la rueda de verdad.
        for _ in 0..3 {
            pintar_con(
                &context,
                &mut app,
                Some(&mut document),
                None,
                &rueda(egui::vec2(0.0, 20.0), dentro(&context)),
                ANCHO_DEL_EDITOR,
            );
        }
        let despues = visibles(&pintar_con(
            &context,
            &mut app,
            Some(&mut document),
            None,
            &[],
            ANCHO_DEL_EDITOR,
        ));

        assert!(
            !despues.contains("linea 1"),
            "después de girar la rueda ya no se ve la primera línea: {despues}"
        );
        assert!(
            despues.contains("linea05") || despues.contains("linea06"),
            "y se ve una de las que estaban debajo: {despues}"
        );
    }

    /// Una línea más ancha que la pantalla se puede recorrer de lado.
    ///
    /// Con una línea de doscientos caracteres en un hueco de doscientos no cabe, y sin
    /// desplazamiento de lado no habría forma de ver el final. Se comprueba que el texto se
    /// ha desplazado y que el final de la línea ha entrado en la pantalla, que es lo mismo
    /// que decir que se puede llegar a ver.
    #[test]
    fn una_linea_mas_ancha_que_la_pantalla_se_puede_recorrer() {
        let context = egui::Context::default();
        let estrecho = 200.0;
        let mut app = App::new();
        let mut document = documento(&format!("{}FINAL", "x".repeat(200)));

        let antes = pintar_con(&context, &mut app, Some(&mut document), None, &[], estrecho);
        let linea = antes
            .iter()
            .find(|pintado| pintado.texto.ends_with("FINAL"))
            .expect("la línea se pinta");

        for _ in 0..30 {
            pintar_con(
                &context,
                &mut app,
                Some(&mut document),
                None,
                &rueda(egui::vec2(100.0, 0.0), dentro(&context)),
                estrecho,
            );
        }
        let despues = pintar_con(&context, &mut app, Some(&mut document), None, &[], estrecho);
        let despues = despues
            .iter()
            .find(|pintado| pintado.texto.ends_with("FINAL"))
            .expect("la línea se sigue pintando");

        assert!(
            linea.ancho + linea.x > estrecho,
            "una línea de doscientos caracteres no cabe en doscientos puntos: {:?}",
            linea
        );
        assert!(
            despues.x < 0.0,
            "el principio de la línea se ha ido de la pantalla: {}",
            despues.x
        );
        assert!(
            despues.x + despues.ancho < estrecho,
            "y el final de la línea ha entrado: {}",
            despues.x + despues.ancho
        );
    }

    /// Mayús con una flecha alarga la selección en vez de mover el cursor.
    ///
    /// Sin esto no habría manera de seleccionar nada, y sin selección no se podría copiar,
    /// cortar ni pegar: la selección es lo que hace que las tres cosas tengan sobre qué
    /// trabajar.
    #[test]
    fn mayus_con_una_flecha_alarga_la_seleccion() {
        let context = egui::Context::default();
        let mut document = documento("uno\ndos");
        pintar(&context, Some(&mut document), None, &clic(dentro(&context)));

        pintar(
            &context,
            Some(&mut document),
            None,
            &tecla_con(egui::Key::ArrowRight, dentro(&context), true),
        );
        pintar(
            &context,
            Some(&mut document),
            None,
            &tecla_con(egui::Key::ArrowRight, dentro(&context), true),
        );

        assert_eq!(
            document
                .selection()
                .selected_text(document.buffer())
                .unwrap(),
            Some("un"),
            "dos pulsaciones de Mayús con la flecha seleccionan dos caracteres"
        );
        assert_eq!(
            document.buffer().text(),
            "uno\ndos",
            "seleccionar no toca el texto"
        );
    }

    /// Copiar pone en el portapapeles lo que está seleccionado, y nada si no hay selección.
    #[test]
    fn copiar_pone_la_seleccion_en_el_portapapeles() {
        let context = egui::Context::default();
        let mut app = App::new();
        let mut document = documento("uno\ndos");
        document.select(en(0, 0), en(0, 2));
        pintar(&context, Some(&mut document), None, &clic(dentro(&context)));

        let salida = dibujar(
            &context,
            &mut app,
            Some(&mut document),
            None,
            &[egui::Event::Copy],
            ANCHO_DEL_EDITOR,
        );

        assert_eq!(copiado(&salida).as_deref(), Some("un"));
        assert_eq!(document.buffer().text(), "uno\ndos", "copiar no quita nada");
    }

    /// Pegar sustituye lo seleccionado por lo que hay en el portapapeles.
    ///
    /// Con una selección que cubre texto a propósito: pegar encima de lo seleccionado es lo
    /// que hace un editor, y pegar sin selección sería insertar.
    #[test]
    fn pegar_sustituye_lo_seleccionado() {
        let context = egui::Context::default();
        let mut app = App::new();
        let mut document = documento("uno");

        document.select(en(0, 0), en(0, 3));
        pintar(&context, Some(&mut document), None, &clic(dentro(&context)));

        let _ = dibujar(
            &context,
            &mut app,
            Some(&mut document),
            None,
            &[egui::Event::Paste("dos".to_owned())],
            ANCHO_DEL_EDITOR,
        );

        assert_eq!(document.buffer().text(), "dos");
        assert!(
            document.is_modified(),
            "pegar es editar, y editar marca el documento"
        );
    }

    /// Cortar copia y quita lo seleccionado.
    ///
    /// Su propio test porque es las dos cosas: si solo copiara, se perdería lo cortado, y si
    /// solo quitara, el texto cortado se iría sin dejar copia.
    #[test]
    fn cortar_copia_y_quita_lo_seleccionado() {
        let context = egui::Context::default();
        let mut app = App::new();
        let mut document = documento("uno");
        document.select(en(0, 0), en(0, 3));
        pintar(&context, Some(&mut document), None, &clic(dentro(&context)));

        let salida = dibujar(
            &context,
            &mut app,
            Some(&mut document),
            None,
            &[egui::Event::Cut],
            ANCHO_DEL_EDITOR,
        );

        assert_eq!(document.buffer().text(), "");
        assert_eq!(copiado(&salida).as_deref(), Some("uno"));
    }

    /// Borrar con una selección quita lo seleccionado entero.
    ///
    /// Con una selección de tres caracteres y no de uno: borrar lo seleccionado quita lo
    /// seleccionado, y si quitara solo un carácter, seleccionar cinco y pulsar Retroceso
    /// quitaría uno y dejaría la selección puesta.
    #[test]
    fn borrar_con_seleccion_quita_lo_seleccionado() {
        let context = egui::Context::default();
        let mut document = documento("uno dos");
        document.select(en(0, 0), en(0, 3));
        pintar(&context, Some(&mut document), None, &clic(dentro(&context)));

        pintar(
            &context,
            Some(&mut document),
            None,
            &tecla(egui::Key::Backspace, dentro(&context)),
        );

        assert_eq!(document.buffer().text(), " dos");
    }

    /// Escribir con una selección sustituye lo seleccionado.
    ///
    /// No es una prueba de la fase del teclado sino de que la selección es real: si al
    /// escribir se insertara en el cursor en vez de sustituir, la selección no serviría
    /// para nada y copiar y cortar serían la única manera de quitar texto.
    #[test]
    fn escribir_con_seleccion_sustituye_lo_seleccionado() {
        let context = egui::Context::default();
        let mut document = documento("uno");
        document.select(en(0, 0), en(0, 3));
        pintar(&context, Some(&mut document), None, &clic(dentro(&context)));

        pintar(
            &context,
            Some(&mut document),
            None,
            &[
                egui::Event::PointerMoved(dentro(&context)),
                egui::Event::Text("X".to_owned()),
            ],
        );

        assert_eq!(document.buffer().text(), "X");
    }

    /// El texto que el editor ha mandado al portapapeles del sistema en este frame.
    ///
    /// egui no deja leer el portapapeles, solo escribirlo: lo pone en un comando de salida
    /// que en la ventana se aplica y en un test se mira. Se mira el comando y no otra
    /// cosa porque es lo que el editor ha pedido, y no lo que el sistema ha hecho con ello.
    fn copiado(salida: &egui::FullOutput) -> Option<String> {
        salida
            .platform_output
            .commands
            .iter()
            .find_map(|comando| match comando {
                egui::OutputCommand::CopyText(texto) => Some(texto.clone()),
                _ => None,
            })
    }

    /// El desplazamiento no se pasa ni del principio ni del final.
    ///
    /// Con un hueco más alto que el texto no se puede desplazar nada, y con un texto muy
    /// largo tampoco se puede salir por arriba ni llegar más allá de la última línea: si se
    /// pudiera, aparecería un hueco vacío donde no hay nada, que es un hueco que parece un
    /// error.
    #[test]
    fn el_desplazamiento_no_se_pasa_de_los_extremos() {
        assert_eq!(
            super::desplazamiento_calculado(0.0, -50.0, 200.0, 600.0),
            0.0
        );
        assert_eq!(
            super::desplazamiento_calculado(380.0, 50.0, 200.0, 600.0),
            400.0
        );
        assert_eq!(
            super::desplazamiento_calculado(0.0, 50.0, 200.0, 100.0),
            0.0
        );
        assert_eq!(
            super::desplazamiento_calculado(50.0, 10.0, 200.0, 600.0),
            60.0
        );
    }

    /// Cada línea que se ve lleva su número a la izquierda, y en su sitio.
    ///
    /// El número va en la izquierda porque es donde se busca, y empieza en uno porque
    /// para quien lee el archivo la primera línea es la uno: si empezara en cero, el
    /// número y la cuenta no coincidirían. Y va a la altura de su línea porque un número
    /// que no esté a la altura de su línea no señala nada, y un número en el mismo sitio
    /// para todas sería un contador y no un gutter.
    #[test]
    fn cada_linea_muestra_su_numero_a_la_izquierda() {
        let context = egui::Context::default();
        let mut document = documento("uno\ndos\ntres");

        let pintado = pintar(&context, Some(&mut document), None, &[]);
        let numeros: Vec<(&str, f32, f32)> = pintado
            .iter()
            .filter(|pintado| pintado.texto.parse::<u32>().is_ok())
            .map(|pintado| (pintado.texto.as_str(), pintado.x, pintado.y))
            .collect();
        let texto_x = pintado
            .iter()
            .find(|pintado| pintado.texto == "uno")
            .expect("se ve el texto")
            .x;

        assert_eq!(numeros.len(), 3, "una línea, un número: {numeros:?}");
        assert_eq!(
            numeros.iter().map(|(n, _, _)| *n).collect::<Vec<_>>(),
            vec!["1", "2", "3"],
            "los números empiezan en uno y van en orden"
        );
        assert!(
            numeros.iter().all(|(_, x, _)| *x < texto_x),
            "los números quedan a la izquierda del texto: {numeros:?}"
        );
        assert!(
            numeros[0].2 < numeros[1].2 && numeros[1].2 < numeros[2].2,
            "y cada número está a la altura de su línea: {numeros:?}"
        );
    }

    /// Un archivo de C# y uno de Java se distinguen.
    ///
    /// Se resaltan con los lenguajes de verdad, cada uno con las palabras suyas: `namespace`
    /// es palabra clave de C# y de Java no lo es, y `package` al revés. Eso es lo que se
    /// comprueba, y no que los dos se vean de colores distintos: los dos son un tipo de
    /// palabra y se pintan igual, y lo que los distingue es cuáles lo son. Si el editor no
    /// mirara al lenguaje que le dan, los dos se verían igual.
    #[test]
    fn un_archivo_de_csharp_y_uno_de_java_se_distinguen() {
        let context = egui::Context::default();
        let mut document = documento("namespace Proyecto {}");
        let csharp = pintar(
            &context,
            Some(&mut document),
            Some(&crate::supports::CSharp),
            &[],
        );
        let palabra_csharp = csharp
            .iter()
            .find(|pintado| pintado.texto == "namespace")
            .map(|pintado| pintado.color)
            .expect("namespace se pinta");
        let codigo_csharp = csharp
            .iter()
            .find(|pintado| pintado.texto == "Proyecto")
            .map(|pintado| pintado.color)
            .expect("el nombre se pinta");

        let mut document = documento("package proyecto;");
        let java = pintar(
            &context,
            Some(&mut document),
            Some(&crate::supports::Java),
            &[],
        );
        let palabra_java = java
            .iter()
            .find(|pintado| pintado.texto == "package")
            .map(|pintado| pintado.color)
            .expect("package se pinta");
        let codigo_java = java
            .iter()
            .find(|pintado| pintado.texto == "proyecto")
            .map(|pintado| pintado.color)
            .expect("el nombre se pinta");

        assert_ne!(
            palabra_csharp, codigo_csharp,
            "en C# namespace es palabra clave y el nombre del proyecto no"
        );
        assert_ne!(
            palabra_java, codigo_java,
            "en Java package es palabra clave y el nombre del proyecto no"
        );

        // Y al revés: lo que es palabra clave de uno no lo es del otro.
        let package_en_csharp = pintar(
            &context,
            Some(&mut documento("package proyecto;")),
            Some(&crate::supports::CSharp),
            &[],
        )
        .iter()
        .find(|pintado| pintado.texto == "package")
        .map(|pintado| pintado.color)
        .expect("package se pinta en C#");

        assert_ne!(
            palabra_java, package_en_csharp,
            "package es palabra clave en Java y en C# es un nombre normal"
        );
    }

    /// Un comentario y una cadena se ven distintos del texto normal.
    ///
    /// Con el lenguaje de verdad, que es el que trae los comentarios: un comentario que
    /// se ve como el código no se lee como un comentario.
    #[test]
    fn un_comentario_y_una_cadena_se_ven_distintos() {
        let context = egui::Context::default();
        let mut document = documento("int a = 1; // uno");
        let pintado = pintar(
            &context,
            Some(&mut document),
            Some(&crate::supports::CSharp),
            &[],
        );
        let comentario = pintado
            .iter()
            .find(|pintado| pintado.texto.contains("// uno"))
            .expect("el comentario se pinta");
        let codigo = pintado
            .iter()
            .find(|pintado| pintado.texto.contains("int"))
            .expect("el código se pinta");

        assert_ne!(comentario.color, codigo.color);
    }

    /// Lo que se ve de verdad, que es lo que cae dentro del hueco del editor.
    ///
    /// El editor pinta todas las líneas y deja que egui recorte las que no caben, así que
    /// en la salida del frame están todas. Para mirar lo que ve el usuario hay que
    /// quedarse con las que caen dentro del hueco, y eso es lo que se hace aquí.
    fn visibles(pintados: &[Pintado]) -> String {
        let dentro: Vec<&str> = pintados
            .iter()
            .filter(|pintado| pintado.y >= 0.0 && pintado.y < ALTO_DEL_EDITOR)
            .map(|pintado| pintado.texto.as_str())
            .collect();

        dentro.join("\n")
    }

    /// Una tecla con Mayús, con su pulsación y su soltura, y el puntero donde está.
    fn tecla_con(tecla: egui::Key, posicion: egui::Pos2, mayus: bool) -> Vec<egui::Event> {
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key: tecla,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers {
                    shift: mayus,
                    ..egui::Modifiers::default()
                },
            })
            .chain(std::iter::once(egui::Event::PointerMoved(posicion)))
            .collect()
    }

    /// Una vuelta de rueda, con el puntero donde está y sus modificadores.
    ///
    /// El puntero va en la vuelta porque egui decide a qué área pertenece cada vuelta de
    /// rueda según dónde esté el puntero, y si no está en ninguna no hay a quién aplicarla.
    fn rueda(delta: egui::Vec2, posicion: egui::Pos2) -> Vec<egui::Event> {
        vec![
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta,
                phase: egui::TouchPhase::Move,
                modifiers: egui::Modifiers::default(),
            },
            egui::Event::PointerMoved(posicion),
        ]
    }
}
