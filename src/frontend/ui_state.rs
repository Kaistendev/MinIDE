//! El estado visual de la interfaz.
//!
//! Aqui vive lo que se ve y lo que se ha visto, y nada mas: qué panel está abierto,
//! qué pestaña se está viendo, qué elemento está resaltado, qué dice la barra de
//! estado. Es estado de la ventana, y desaparece con ella.
//!
//! Lo que no puede vivir aqui es el estado del dominio. El texto de un documento, el
//! cursor, la selección, el historial, el proyecto y su configuración son del core y
//! se piden, no se guardan: si la interfaz guardara una copia, la copia se pondría al
//! día hasta que dejara de estarlo, y el IDE empezaría a guardar y mostrar cosas que
//! el core ya no sabe.
//!
//! Por eso este archivo no importa nada del core. No es una costumbre: es lo que hace
//! que la separación sea de verdad, y hay un test que lo comprueba.

use std::collections::BTreeSet;

use super::dialogos::Dialogo;
use crate::frontend::tabs::Pestana;

/// El estado visual de la ventana.
///
/// Solo lo que se ve: qué dice la barra de estado, qué panel está abierto, qué
/// elemento está resaltado. Ahora mismo solo la barra de estado y las carpetas abiertas del
/// explorador; el resto de las zonas llega con la tarea de cada una.
///
/// Deliberadamente no guarda el titulo de la ventana. El titulo lo decide la
/// aplicación (`titulo`) y, cuando haya documentos abiertos, se compondrá con el
/// documento activo, que es del core. Guardarlo aqui seria tener la misma verdad en
/// dos sitios: la del titulo, que es fija, y la del documento, que no.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct UiState {
    status: Option<String>,
    /// La fila que está seleccionada en el explorador, si hay alguna.
    ///
    /// Es la fila, no el archivo: aquí no hay ningún dato del proyecto, solo con qué fila
    /// hay que pintar la marca de seleccionado. Que sea su ruta y no un puntero al modelo
    /// del core es lo que permite que el estado visual no dependa de él.
    seleccion: Option<String>,
    /// Las carpetas que están abiertas en el árbol del explorador.
    ///
    /// Se guardan sus rutas relativas y no las carpetas: aquí no hay ningún dato del
    /// proyecto, solo con qué fila volver a abrir lo que el usuario cerró. Que sean rutas y
    /// no punteros al modelo del core es lo que permite que el estado visual no dependa de
    /// él.
    expandidas: BTreeSet<String>,
    /// Las pestañas que se están viendo, en el orden en que se abrieron.
    ///
    /// Son las pestañas, no los documentos: cada una es la ruta del documento que hay
    /// detrás y si tiene cambios sin guardar, que es lo que se ve. El documento entero
    /// no está aquí y no se guarda, porque en cuanto la ventana guardara una copia del
    /// texto habría dos y la que no se editara sería la que se guarda.
    ///
    /// Viven aquí y no en el core porque el core no las necesita: su lista de documentos
    /// abiertos y cuál está activo son lo suyo (`OpenTabs`), y esto es lo que se ve de
    /// eso. Que las dos esten al día es de quien las actualiza al recibir la noticia, que
    /// es la que ejecuta los comandos.
    pestanas: Vec<Pestana>,
    /// La pestaña que se está viendo, por su ruta.
    ///
    /// Por su ruta y no por su sitio en la lista porque el sitio cambia al cerrar otra
    /// pestaña, y la que se está viendo no cambia por eso. Con ninguna ventana con
    /// documentos no hay ninguna activa, y con documentos siempre hay una: la que se
    /// abrió la última.
    pestana_activa: Option<String>,
    /// Cuánto se ha desplazado el editor en vertical, en puntos.
    ///
    /// Es de la ventana y no del documento porque no cambia lo que hay escrito: es dónde
    /// se está mirando, que es lo mismo que el tamaño de un panel. El editor lo mueve con
    /// la rueda y lo deja donde se pueda, y el límite lo pone él porque el alto del
    /// contenido solo lo sabe quien lo pinta.
    desplazamiento_vertical: f32,
    /// Cuánto se ha desplazado el editor de lado, en puntos.
    ///
    /// Véase [`Self::desplazamiento_vertical`]. Es aparte porque una línea larga se
    /// recorre de lado sin que el documento baje, que es como se lee una línea que no
    /// cabe.
    desplazamiento_horizontal: f32,
    /// La búsqueda que se está haciendo, si hay alguna. FE-032 y FE-033.
    ///
    /// Vive en el estado visual y no en el core porque es un diálogo de la ventana: qué
    /// se ha escrito en el campo de búsqueda es de quien lo escribió, y lo que el core
    /// responda -las coincidencias- es otra cosa, que todavía no le llega (FE-070 es la que
    /// la trae). Estar abierta o cerrada también es de la ventana.
    busqueda: Option<Busqueda>,
    /// Qué se está viendo en el área central. FE-044.
    ///
    /// El editor y el diseñador son la misma zona y solo puede verse uno, así que cuál está
    /// es estado de la ventana. Es un dato y no un modelo entero porque no hay nada del
    /// diseñador aquí: el modelo es del core y lo tiene `Diseniador`, y esto solo dice si se
    /// está mirando.
    vista_central: VistaCentral,
    /// La pregunta que la ventana le está haciendo al usuario, si hay alguna. FE-055 y FE-056.
    ///
    /// Vive aquí y no en el core porque es una pregunta de la ventana: quién la decide y
    /// cuándo la hace son de la ventana. Lo que hay dentro no es del dominio: una ruta o un
    /// mensaje, no el documento ni el error.
    dialogo: Option<Dialogo>,
    /// A qué archivo y a qué línea hay que ir, si hay algo pendiente. FE-072.
    ///
    /// Es un destino y no un salto ya hecho porque el comando que lo dispara —abrir un
    /// documento— no lleva datos: el archivo al que hay que ir se dice marcando su fila, como
    /// con cualquier otro. Lo que la ventana guarda aquí es la línea, que no cabe en una fila
    /// marcada, y el archivo, para saber a cuál de los documentos abiertos pertenece el
    /// destino cuando el documento se abra.
    ///
    /// Vive en el estado visual y no en el core porque es una intención de la ventana, como
    /// la fila seleccionada: lo que el core tiene es el documento con su cursor, y a dónde se
    /// quiere saltar es de quien lo ha pedido.
    destino: Option<Destino>,
}

/// A qué archivo y a qué línea se va cuando se abra. FE-072.
///
/// Es una ruta y una línea, y no el documento, porque el documento al que se va puede estar
/// cerrado: el destino se pide antes de abrirlo, que es lo que hace la lista de diagnósticos
/// al pulsar un error de un archivo que nadie tiene abierto.
///
/// La línea es la del core, contada desde cero, y no la que se ve en pantalla, que se cuenta
/// desde uno: si aquí se sumara el uno, un error en la primera línea llevaría el cursor a la
/// segunda, y el usuario buscaría el error una línea más abajo de donde está.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destino {
    /// La clave del archivo, la misma que usan las pestañas y el explorador.
    ruta: String,
    /// La línea, contada desde cero como las del core.
    linea: u32,
}

impl Destino {
    /// El destino al archivo de `ruta`, en la `linea` de cero.
    pub fn new(ruta: impl Into<String>, linea: u32) -> Self {
        Self {
            ruta: ruta.into(),
            linea,
        }
    }

    /// La clave del archivo al que hay que ir.
    pub fn ruta(&self) -> &str {
        &self.ruta
    }

    /// La línea a la que hay que ir, contada desde cero.
    pub fn linea(&self) -> u32 {
        self.linea
    }
}

/// El desplazamiento del editor en los dos ejes, en puntos.
///
/// Va en su propio tipo y no sueltos porque el editor lo necesita *entregado y devuelto* en
/// el mismo frame: pinta el documento que le presta el core y, a la vez, guarda por dónde
/// se ha desplazado, que es estado de la ventana. Preguntar por el estado de la ventana con
/// el documento en la mano y escribir en él son dos préstamos que no pueden ser a la vez, así
/// que quien lo pinta lo recibe copiado y lo devuelve escrito (FE-058).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Desplazamiento {
    vertical: f32,
    horizontal: f32,
}

impl Desplazamiento {
    /// Un desplazamiento de `vertical` y `horizontal` puntos.
    pub fn new(vertical: f32, horizontal: f32) -> Self {
        Self {
            vertical,
            horizontal,
        }
    }

    /// Cuánto se ha desplazado en vertical, en puntos.
    pub fn vertical(self) -> f32 {
        self.vertical
    }

    /// Cuánto se ha desplazado de lado, en puntos.
    pub fn horizontal(self) -> f32 {
        self.horizontal
    }
}

/// Lo que se está viendo en el área central.
///
/// Son dos y no más porque son las dos que el plan de la ventana describe. Añadir una tercera
/// sería un docking, y `docs/frontend-plan.md` §13 dice que en el MVP no.
///
/// No hay un `nombre()` aquí a propósito: lo que pone el conmutor de vistas son "Code" y
/// "Designer", y están en `layout`, que es donde están los botones. Un nombre de vista en dos
/// sitios es un nombre que se queda atrás en uno de los dos, y el día que el conmutador
/// dijera una cosa y el estado otra no se sabría cuál se ve (FE-076).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum VistaCentral {
    /// El editor de código. Es lo que se ve al abrir MiniIDE.
    #[default]
    Editor,
    /// El diseñador visual. FE-044.
    Diseniador,
}

/// La búsqueda que se está haciendo: qué se busca y con qué se reemplaza.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Busqueda {
    consulta: String,
    /// El texto de reemplazo, y si la búsqueda lo trae.
    ///
    /// Es `None` cuando la búsqueda se abrió sin reemplazo -Ctrl+F- y `Some` cuando se
    /// abrió con reemplazo -Ctrl+H-, aunque esté vacío: un reemplazo vacío es un
    /// reemplazo, y por eso el "tiene o no tiene" se mira en el `Option` y no en si el
    /// texto está vacío.
    reemplazo: Option<String>,
}

impl Busqueda {
    /// Lo que se está buscando.
    pub fn consulta(&self) -> &str {
        &self.consulta
    }

    /// Con qué se reemplaza, si esta búsqueda trae reemplazo.
    pub fn reemplazo(&self) -> Option<&str> {
        self.reemplazo.as_deref()
    }

    /// El texto de lo que se busca, para escribir en él.
    pub fn consulta_mut(&mut self) -> &mut String {
        &mut self.consulta
    }

    /// El texto de reemplazo, si esta búsqueda trae reemplazo.
    pub fn reemplazo_mut(&mut self) -> Option<&mut String> {
        self.reemplazo.as_mut()
    }
}

impl UiState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Qué se está viendo en el área central.
    pub fn vista_central(&self) -> VistaCentral {
        self.vista_central
    }

    /// Enseñar el diseñador en el área central.
    ///
    /// No abre ningún formulario: esto solo cambia lo que se ve. El modelo lo pone quien
    /// sabe cuál es (FE-059 y FE-063), y un diseñador sin modelo es un formulario vacío, que
    /// es exactamente lo que tiene que poder dibujarse (FE-044).
    pub fn mostrar_diseniador(&mut self) {
        self.vista_central = VistaCentral::Diseniador;
    }

    /// Volver al editor en el área central.
    pub fn mostrar_editor(&mut self) {
        self.vista_central = VistaCentral::Editor;
    }

    /// La pregunta que la ventana le está haciendo al usuario, si hay alguna.
    pub fn dialogo(&self) -> Option<&Dialogo> {
        self.dialogo.as_ref()
    }

    /// Abre una pregunta.
    ///
    /// Abrir una pregunta que ya está abierta la cambia por la nueva, y no la deja como
    /// estaba: dos preguntas a la vez no tienen respuesta, porque la ventana solo sabe
    /// hacer una. Lo que sí se conserva es la que estaba si es la misma, para que un aviso
    /// que llega dos veces -dos fallos seguidos de la misma herramienta- no pierda lo que el
    /// usuario estaba leyendo.
    pub fn abrir_dialogo(&mut self, dialogo: Dialogo) {
        self.dialogo = Some(dialogo);
    }

    /// Cierra la pregunta y se olvida de ella.
    ///
    /// Se olvida porque una pregunta cerrada y luego abierta otra vez por lo mismo no puede
    /// enseñarla como si fuera nueva: la respuesta ya está dada.
    pub fn cerrar_dialogo(&mut self) {
        self.dialogo = None;
    }

    /// Si hay alguna pregunta abierta.
    pub fn hay_dialogo_abierto(&self) -> bool {
        self.dialogo.is_some()
    }

    /// Lo que dice la barra de estado, si dice algo.
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// Pone un mensaje en la barra de estado, sustituyendo el que hubiera.
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status = Some(message.into());
    }

    /// Quita el mensaje de la barra de estado.
    pub fn clear_status(&mut self) {
        self.status = None;
    }

    /// La fila que está seleccionada en el explorador, si hay alguna.
    ///
    /// Una ventana nueva no tiene ninguna seleccionada, y por eso el explorador se abre
    /// sin nada marcado: no hay nada que haya seleccionado el usuario todavía.
    pub fn seleccion(&self) -> Option<&str> {
        self.seleccion.as_deref()
    }

    /// Marca la fila de `ruta` como seleccionada.
    ///
    /// Se llama `seleccionar` y no `set_seleccion` porque seleccionar sustituye: solo hay
    /// una fila seleccionada, y en un explorador puedenuscarse dos a la vez. Cambiar de
    /// archivo no deja dos marcados, porque el que estaba antes ya no está abierto.
    pub fn seleccionar(&mut self, ruta: &str) {
        self.seleccion = Some(ruta.to_owned());
    }

    /// A dónde hay que ir cuando se abra un documento, si hay algo pendiente. FE-072.
    ///
    /// Lo normal es que no haya nada: un destino se pone al pulsar un diagnóstico y se
    /// consume en cuanto el documento correspondiente está abierto, así que una ventana en la
    /// que no se ha pulsado ningún error no tiene a dónde ir.
    pub fn destino(&self) -> Option<&Destino> {
        self.destino.as_ref()
    }

    /// Anota que hay que ir a `ruta`, en la `linea` de cero. FE-072.
    ///
    /// Sustituye el destino que hubiera porque solo se puede ir a un sitio: si el usuario
    /// pulsa un error y luego otro sin haber EXPECTADO al primero, al segundo se va.
    pub fn ir_a(&mut self, ruta: impl Into<String>, linea: u32) {
        self.destino = Some(Destino::new(ruta, linea));
    }

    /// El destino pendiente si es el del archivo de `ruta`, y lo quita.
    ///
    /// Se consume aquí y no en quien lo puso porque el destino tiene que desaparecer en cuanto
    /// se cumple: si se quedara, volver a abrir el mismo archivo por el explorador llevaría al
    /// cursor a un error que ya se está arreglando, y saltar solo tiene sentido la primera vez.
    ///
    /// Devuelve el destino en vez de aplicarlo porque el cursor es del documento y el documento
    /// es del core: quien lo mueve es quien lo tiene en la mano, que es la aplicación.
    pub fn tomar_el_destino_de(&mut self, ruta: &str) -> Option<Destino> {
        if self.destino.as_ref()?.ruta() != ruta {
            return None;
        }

        self.destino.take()
    }

    /// Si la carpeta de `ruta` está abierta en el árbol del explorador.
    ///
    /// Una carpeta que no está en la lista está cerrada, y una ventana nueva no tiene
    /// ninguna abierta: un árbol que saliera con todo desplegado escondería el proyecto
    /// entero detrás de carpetas que el usuario no ha pedido abrir.
    pub fn esta_expandida(&self, ruta: &str) -> bool {
        self.expandidas.contains(ruta)
    }

    /// Abre la carpeta de `ruta` si estaba cerrada y la cierra si estaba abierta, y
    /// devuelve si queda abierta.
    ///
    /// Una función que alterna y no dos, una de abrir y otra de cerrar, porque quien llama
    /// es una fila que solo sabe que se ha pulsado: decidir si tocaba abrir o cerrar es lo
    /// único que sabe, y aquí se decide.
    pub fn alternar_expansion(&mut self, ruta: &str) -> bool {
        if self.expandidas.contains(ruta) {
            self.expandidas.remove(ruta);
            false
        } else {
            self.expandidas.insert(ruta.to_owned());
            true
        }
    }

    /// Las pestañas que se están viendo, en el orden en que se abrieron.
    pub fn pestanas(&self) -> &[Pestana] {
        &self.pestanas
    }

    /// La pestaña que se está viendo, si hay alguna.
    pub fn pestana_activa(&self) -> Option<&str> {
        self.pestana_activa.as_deref()
    }

    /// El nombre del documento que se está viendo, si hay alguno.
    ///
    /// Es lo que enseña la barra de estado y no la ruta, porque en una barra de sitio no cabe
    /// una ruta y lo que se lee de un vistazo es el nombre del archivo. Sale de la lista de
    /// pestañas y no de la ruta guardada para no mirar en dos sitios: si la pestaña no está,
    /// no hay documento, que es lo que dice la barra (FE-068).
    pub fn documento_activo(&self) -> Option<&str> {
        let ruta = self.pestana_activa.as_deref()?;

        self.pestanas
            .iter()
            .find(|pestana| pestana.ruta() == ruta)
            .map(Pestana::nombre)
    }

    /// Enseña la pestaña de `pestana` y la deja a la vista.
    ///
    /// Abrir es pedir verlo, así que la pestaña que se abre queda a la vista, y es lo que
    /// hace el core con la suya: `OpenTabs::open` deja activa la que abre. No son dos
    /// reglas que coincidan, es la misma: la ventana enseña lo que le ha pedido el
    /// usuario, y el core guarda cuál es.
    ///
    /// Un documento que ya está abierto no añade una segunda pestaña: la trae a la vista
    /// y ya está, con los cambios que tuviera. Con dos pestañas del mismo archivo el
    /// usuario cerraría una y seguiría viendo el mismo documento sin saber cuál se ha
    /// cerrado, y el core no haría otra cosa: `OpenTabs::open` devuelve la que ya había.
    pub fn abrir_pestana(&mut self, pestana: Pestana) {
        let ruta = pestana.ruta().to_owned();

        match self
            .pestanas
            .iter_mut()
            .find(|abierta| abierta.ruta() == ruta)
        {
            Some(abierta) => *abierta = pestana,
            None => self.pestanas.push(pestana),
        }

        self.pestana_activa = Some(ruta);
    }

    /// Deja a la vista la pestaña de `ruta`, y devuelve si estaba abierta.
    ///
    /// Devolver si lo estaba porque la ruta la elige quien llama, que puede estar
    /// equivocada, y dejar marcada una pestaña que no existe no es un error que se pueda
    /// ignorar: se vería un documento en una pestaña y el resaltado en ninguna.
    pub fn activar_pestana(&mut self, ruta: &str) -> bool {
        if !self.pestanas.iter().any(|pestana| pestana.ruta() == ruta) {
            return false;
        }

        self.pestana_activa = Some(ruta.to_owned());

        true
    }

    /// Quita la pestaña de `ruta`, y devuelve si estaba abierta.    ///
    /// Al quitar la que se estaba viendo pasa a estarlo la que ocupa su sitio, que es la
    /// siguiente, y si era la última, la nueva última. Es la misma regla que la del core
    /// en `OpenTabs::close` y se llama igual porque es la misma: si cerraran distinto, al
    /// usuario se le cerraría la pestaña que está viendo y se quedaría viendo un
    /// documento que él cree haber cerrado.
    ///
    /// Quitar la pestaña no es cerrar el documento: eso lo pide el core con su comando,
    /// que es quien sabe si el documento tenía cambios sin guardar. Aquí solo desaparece
    /// lo que se ve de él.
    pub fn cerrar_pestana(&mut self, ruta: &str) -> bool {
        let indice = match self
            .pestanas
            .iter()
            .position(|pestana| pestana.ruta() == ruta)
        {
            Some(indice) => indice,
            None => return false,
        };

        self.pestanas.remove(indice);

        if self.pestana_activa.as_deref() == Some(ruta) {
            self.pestana_activa = (!self.pestanas.is_empty()).then(|| {
                self.pestanas[indice.min(self.pestanas.len() - 1)]
                    .ruta()
                    .to_owned()
            });
        }

        true
    }

    /// Cuánto se ha desplazado el editor en vertical, en puntos.
    pub fn desplazamiento_vertical(&self) -> f32 {
        self.desplazamiento_vertical
    }

    /// Deja el editor desplazado `puntos` en vertical.
    pub fn set_desplazamiento_vertical(&mut self, puntos: f32) {
        self.desplazamiento_vertical = puntos;
    }

    /// Cuánto se ha desplazado el editor de lado, en puntos.
    pub fn desplazamiento_horizontal(&self) -> f32 {
        self.desplazamiento_horizontal
    }

    /// Deja el editor desplazado `puntos` de lado.
    pub fn set_desplazamiento_horizontal(&mut self, puntos: f32) {
        self.desplazamiento_horizontal = puntos;
    }

    /// Los dos desplazamientos de golpe, que es como los necesita el editor.
    pub fn desplazamiento(&self) -> Desplazamiento {
        Desplazamiento {
            vertical: self.desplazamiento_vertical,
            horizontal: self.desplazamiento_horizontal,
        }
    }

    /// Deja el editor desplazado como dice `desplazamiento`, en los dos ejes.
    pub fn set_desplazamiento(&mut self, desplazamiento: Desplazamiento) {
        self.desplazamiento_vertical = desplazamiento.vertical;
        self.desplazamiento_horizontal = desplazamiento.horizontal;
    }

    /// Abre la búsqueda, con reemplazo si `con_reemplazo` es cierto.
    ///
    /// Volver a abrirla con las teclas de búsqueda no la cierra: si ya estaba abierta, la
    /// deja como estaba con su consulta, que es lo que quiere quien abre la búsqueda
    /// otra vez con Ctrl+F para cambiar el término. Y abrirla con reemplazo cuando ya
    /// estaba con reemplazo tampoco pierde la consulta.
    pub fn abrir_busqueda(&mut self, con_reemplazo: bool) {
        match &mut self.busqueda {
            Some(busqueda) if con_reemplazo && busqueda.reemplazo.is_none() => {
                busqueda.reemplazo = Some(String::new());
            }
            Some(_) => {}
            None => {
                self.busqueda = Some(Busqueda {
                    consulta: String::new(),
                    reemplazo: con_reemplazo.then(String::new),
                });
            }
        }
    }

    /// Cierra la búsqueda y se olvida de lo que se estaba buscando.
    ///
    /// Se olvida porque una búsqueda cerrada y luego abierta con la misma consulta
    /// buscaría lo mismo de la última vez, y quien la abre quiere empezar de cero.
    pub fn cerrar_busqueda(&mut self) {
        self.busqueda = None;
    }

    /// Si hay alguna búsqueda abierta.
    pub fn esta_abierta_la_busqueda(&self) -> bool {
        self.busqueda.is_some()
    }

    /// Si la búsqueda abierta trae reemplazo, que es lo que decide si se ve el campo.
    pub fn la_busqueda_tiene_reemplazo(&self) -> bool {
        self.busqueda
            .as_ref()
            .is_some_and(|busqueda| busqueda.reemplazo.is_some())
    }

    /// La búsqueda que se está haciendo, si hay alguna.
    pub fn busqueda(&self) -> Option<&Busqueda> {
        self.busqueda.as_ref()
    }

    /// La búsqueda que se está haciendo, para escribir en sus campos.
    pub fn busqueda_mut(&mut self) -> Option<&mut Busqueda> {
        self.busqueda.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_visual_state_has_nothing_to_show_yet() {
        let state = UiState::new();

        assert_eq!(state.status(), None);
    }

    #[test]
    fn status_can_be_set_and_cleared() {
        let mut state = UiState::default();

        state.set_status("Compilando...");
        assert_eq!(state.status(), Some("Compilando..."));

        state.clear_status();
        assert_eq!(state.status(), None);
    }

    #[test]
    fn setting_a_status_replaces_the_previous_one() {
        let mut state = UiState::new();

        state.set_status("Compilando...");
        state.set_status("1 error");

        assert_eq!(state.status(), Some("1 error"));
    }

    /// Una ventana nueva no tiene ninguna fila seleccionada.
    ///
    /// El explorador se abre sin nada marcado, porque no hay nada que haya seleccionado el
    /// usuario todavía, y un explorador con una fila marcada al abrir está mintiendo sobre
    /// qué está abierto.
    #[test]
    fn a_new_visual_state_has_nothing_selected() {
        let state = UiState::new();

        assert_eq!(state.seleccion(), None);
    }

    /// Seleccionar una fila sustituye a la que estaba.
    ///
    /// Solo hay una fila seleccionada a la vez: si seleccionar dejara las dos marcadas, en
    /// un explorador con varios archivos abiertos no se sabría cuál está abierto, y el
    /// archivo que se acaba de abrir no se distinguiría de los otros.
    #[test]
    fn selecting_a_row_replaces_the_previous_one() {
        let mut state = UiState::new();

        state.seleccionar("src/Form1.cs");
        assert_eq!(state.seleccion(), Some("src/Form1.cs"));

        state.seleccionar("src/Program.cs");
        assert_eq!(state.seleccion(), Some("src/Program.cs"));
    }

    /// Una ventana nueva no tiene ninguna carpeta abierta.
    ///
    /// Un árbol que saliera con todo desplegado escondería el proyecto entero detrás de
    /// carpetas que el usuario no ha pedido abrir, y lo que no está abierto es lo que hace
    /// falta ver de un vistazo.
    #[test]
    fn a_new_visual_state_has_nothing_expanded() {
        let state = UiState::new();

        assert!(!state.esta_expandida("src"));
    }

    /// Una carpeta se abre y se cierra con el mismo gesto.
    ///
    /// El gesto es el mismo porque es el mismo botón: el que se pulsa es una fila, y una
    /// fila no sabe si tocaba abrir o cerrar.
    #[test]
    fn a_folder_opens_and_closes_with_the_same_gesture() {
        let mut state = UiState::new();

        assert!(state.alternar_expansion("src"), "el primer gesto abre");
        assert!(state.esta_expandida("src"));

        assert!(!state.alternar_expansion("src"), "el segundo gesto cierra");
        assert!(!state.esta_expandida("src"));
    }

    /// Las carpetas van unas por unas.
    ///
    /// Abrir una carpeta no puede abrir ni cerrar las demás: en un proyecto con "src" y
    /// "docs", desplegar uno tiene que dejar el otro como estaba, o el usuario no podría
    /// trabajar con los dos a la vez.
    #[test]
    fn folders_expand_one_by_one() {
        let mut state = UiState::new();

        state.alternar_expansion("src");

        assert!(state.esta_expandida("src"));
        assert!(!state.esta_expandida("docs"), "abrir una no abre las otras");

        state.alternar_expansion("docs");

        assert!(state.esta_expandida("src") && state.esta_expandida("docs"));

        state.alternar_expansion("src");

        assert!(!state.esta_expandida("src"));
        assert!(
            state.esta_expandida("docs"),
            "cerrar una no cierra las otras"
        );
    }

    /// El desplazamiento se lee y se escribe en los dos ejes a la vez.
    ///
    /// Es lo que necesita el editor desde FE-058: pinta el documento que le presta el core
    /// y devuelve el desplazamiento por el que lo ha pintado, así que los dos ejes tienen que
    /// viajar juntos. Si se escribieran por separado, quien pintara podría devolver un eje y
    /// dejar el otro como estaba, y un documento que baja pero no se desplaza de lado parece
    /// un editor roto.
    #[test]
    fn el_desplazamiento_se_va_y_vuelve_en_los_dos_ejes() {
        let mut state = UiState::new();

        assert_eq!(state.desplazamiento(), Desplazamiento::default());

        state.set_desplazamiento(Desplazamiento::new(40.0, 12.0));

        assert_eq!(state.desplazamiento_vertical(), 40.0);
        assert_eq!(state.desplazamiento_horizontal(), 12.0);
        assert_eq!(
            state.desplazamiento(),
            Desplazamiento::new(40.0, 12.0),
            "los dos ejes se leen y se escriben en la misma operación"
        );
    }

    /// El nombre del documento que se está viendo sale de la pestaña activa. FE-068.
    ///
    /// Con pestañas: sale de la que está activa y no de la última abierta, porque son cosas
    /// distintas en cuanto el usuario cambia de documento. Sin pestañas no hay documento, y
    /// eso es lo que dice la barra de estado.
    #[test]
    fn el_nombre_del_documento_que_se_esta_viendo_sale_de_su_pestana() {
        use crate::frontend::tabs::Pestana;

        let mut state = UiState::new();

        assert_eq!(
            state.documento_activo(),
            None,
            "sin pestañas no hay documento que enseñar"
        );

        state.abrir_pestana(Pestana::new("src/Form1.cs", false));
        state.abrir_pestana(Pestana::new("src/Program.cs", true));
        assert_eq!(
            state.documento_activo(),
            Some("Program.cs"),
            "se ve el nombre del archivo, no su ruta"
        );

        state.activar_pestana("src/Form1.cs");
        assert_eq!(
            state.documento_activo(),
            Some("Form1.cs"),
            "al cambiar de pestaña cambia el documento que se ve"
        );

        state.cerrar_pestana("src/Form1.cs");
        assert_eq!(
            state.documento_activo(),
            Some("Program.cs"),
            "al cerrar la que se veía pasa a verse la de al lado, que es la misma regla que \
             sigue el core: la barra no puede mirar un documento que no está"
        );

        state.cerrar_pestana("src/Program.cs");
        assert_eq!(
            state.documento_activo(),
            None,
            "y sin pestañas no hay documento"
        );
    }

    /// Modulos del core. El estado visual no puede importar ninguno.
    const MODULOS_DEL_CORE: &[&str] = &[
        "build",
        "commands",
        "core",
        "diagnostics",
        "document",
        "editor",
        "framework",
        "generation",
        "language",
        "project",
        "runtime",
        "supports",
        "templates",
        "toolchain",
        "workspace",
    ];

    /// El estado visual no depende del core.
    ///
    /// Es la forma de comprobar que la interfaz no guarda el dominio: si este archivo
    /// importara el documento, el proyecto o el editor, el estado visual empezaria a
    /// depender de tipos que son del core, y con ellos a entrar en ellos. Ahi esta el
    /// peligro de la copia: un documento en la interfaz obliga a decidir quien tiene
    /// la verdad, y siempre se acaban teniendo las dos.
    ///
    /// Se comprueban los `use` y no el texto entero a proposito: la regla es que el
    /// estado visual no *importa* el core, y porque el test se escribe aqui dentro, un
    /// `contains` sobre todo el archivo se encontraria a si mismo con el nombre de
    /// cada modulo.
    #[test]
    fn the_visual_state_does_not_depend_on_the_core() {
        let source =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file!()))
                .expect("readable source of the visual state");

        let uses: Vec<&str> = source
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("use crate::"))
            .filter(|line| {
                MODULOS_DEL_CORE
                    .iter()
                    .any(|module| line.contains(&format!("::{module}")))
            })
            .collect();

        assert!(
            uses.is_empty(),
            "el estado visual es de la interfaz y no puede importar el core: {uses:?}"
        );
    }
}
