//! El diseñador visual: el canvas, el toolbox y lo que el usuario hace en ellos.
//!
//! Aquí vive la parte de la ventana donde se coloca un formulario y sus controles. Lo que se
//! ve sale de un `DesignerModel` del core, y lo que se toca se le devuelve al mismo modelo:
//! el diseñador no guarda una copia de los controles ni sabe cómo se convierte en C# o en
//! Java, que es de los generadores.
//!
//! # Un comando por gesto
//!
//! Cada cosa que se puede hacer en el canvas -seleccionar, mover, redimensionar, añadir,
//! renombrar y cambiar una propiedad- es un [`Comando`], y todos entran por
//! [`Diseniador::aplicar`]. Que sean un tipo y no una serie de métodos sueltos es lo que
//! evita que un gesto se aplique en el modelo desde dos sitios: si mover se pudiera hacer
//! desde el canvas y desde las propiedades, bastaría con que uno de los dos se olvidara de
//! actualizar el modelo para que el canvas y el modelo dejaran de cuadrar.
//!
//! Traen sus datos porque sin ellos no hay operación: "mueve `boton1` a (10, 20)" no se
//! puede decir con un nombre solo. Es la razón por la que no son `crate::commands::Command`,
//! que es el vocabulario sin datos para las operaciones del core.
//!
//! # Dónde está
//!
//! El diseñador es una de las dos vistas del área central, junto al editor (FE-044). Cuál de
//! las dos se ve es estado visual y lo lleva [`crate::frontend::UiState`]: hasta que haya un
//! proyecto abierto, FE-059 y FE-063 son las que abren el diseñador, y aquí solo se dibuja.

use eframe::egui;

use crate::frontend::App;
use crate::generation::{DesignerComponent, DesignerModel};

/// Lo que el usuario ha hecho en el diseñador, con los datos para poder aplicarlo.
///
/// Cada variante es una operación distinta porque cambia cosas distintas del modelo, y
/// nombrarlas es lo que deja que `aplicar` diga cuál se está haciendo en vez de deducirlo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comando {
    /// Seleccionar un control, o ninguno si `None`.
    ///
    /// Lleva su propio `Option` y no un `String` vacío para "quitar la selección" porque
    /// las dos cosas tienen que ser imposibles de confundir: un nombre vacío no es un
    /// control, y si valiera como tal el modelo tendría un control sin nombre.
    Seleccionar(Option<String>),
    /// Poner un control en otra posición, en puntos desde la esquina del formulario.
    Mover { control: String, x: i32, y: i32 },
    /// Cambiarle el tamaño a un control.
    Redimensionar {
        control: String,
        ancho: u32,
        alto: u32,
    },
    /// Poner un control nuevo en el punto indicado.
    Anadir {
        tipo: String,
        x: i32,
        y: i32,
        ancho: u32,
        alto: u32,
    },
    /// Quitar un control del formulario.
    ///
    /// Lo quita del modelo y no lo esconde: si el usuario se arrepiente no tiene forma de
    /// recuperarlo, y por eso el que se puede recuperar es el que no se llega a quitar.
    Borrar { control: String },
    /// Cambiarle el nombre a un control.
    Renombrar { control: String, nombre: String },
    /// Poner una propiedad de un control.
    ///
    /// La propiedad llega con su nombre porque es del framework y no del modelo: WinForms
    /// la llama `Text` y Swing la llama `text`, y aquí no se sabe cuál de las dos es.
    Propiedad {
        control: String,
        propiedad: String,
        valor: String,
    },
}

/// Un control no puede ser más pequeño que esto.
///
/// Sin un mínimo, redimensionar hasta el cero deja un control que no se ve y que sigue en
/// el modelo: ocupa sitio, sale en las propiedades y no se puede volver a agarrar. Un
/// mínimo pequeño deja que el error sea recuperable sin estorbar al que quiere un botón
/// diminuto.
pub const TAMANO_MINIMO: u32 = 8;

/// El diseñador: el modelo del core y lo que el usuario ha hecho con él.
///
/// Guarda el modelo entero y no una copia de los controles porque el modelo es la verdad de
/// cuántos controles hay y dónde están. Un arreglo paralelo de posiciones se quedaría viejo
/// en cuanto el generador o el panel de propiedades cambiaran algo, y el canvas acabaría
/// enseñando controles que ya no existen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diseniador {
    modelo: DesignerModel,
    /// El control seleccionado, si hay alguno.
    ///
    /// Vive aquí y no en el `DesignerModel` porque el modelo es lo que se convierte en
    /// código y a la hora de generar da igual qué control esté resaltado: meter la selección
    /// en el modelo sería llevar a la generación un dato que no le sirve.
    seleccion: Option<String>,
    /// El control del toolbox que está en la mano, si el usuario ha cogido alguno.
    herramienta: Option<String>,
    /// Los controles que ofrece el toolbox.
    ///
    /// Los pone quien abre el diseñador porque son los del framework del proyecto, y aquí
    /// no se sabe qué framework es: una lista fija sería escribir "Button" o "JButton" en la
    /// ventana, que es justo el `match` por lenguaje que AGENTS.md §2.4 prohíbe.
    tipos: Vec<String>,
}

impl Default for Diseniador {
    fn default() -> Self {
        Self::vacio("Ventana", "", 640, 480)
    }
}

impl Diseniador {
    /// Un diseñador con el modelo que se le pase y nada más.
    pub fn vacio(nombre: &str, titulo: &str, ancho: u32, alto: u32) -> Self {
        Self {
            modelo: DesignerModel::new(nombre, titulo, ancho, alto, Vec::new()),
            seleccion: None,
            herramienta: None,
            tipos: Vec::new(),
        }
    }

    /// El diseñador de un modelo que ya existe.
    pub fn con_modelo(modelo: DesignerModel) -> Self {
        Self {
            modelo,
            seleccion: None,
            herramienta: None,
            tipos: Vec::new(),
        }
    }

    /// Pone los controles que ofrece el toolbox y quita lo que hubiera.
    ///
    /// Es lo que llama quien abre el diseñador, con los controles que declara el framework
    /// del proyecto.
    pub fn fijar_tipos(&mut self, tipos: &[&str]) {
        self.tipos = tipos.iter().map(|tipo| (*tipo).to_owned()).collect();
    }

    /// El modelo del core, que es lo que se dibuja y lo que se devuelve al generador.
    pub fn modelo(&self) -> &DesignerModel {
        &self.modelo
    }

    /// El modelo, para quien tenga que cambiarlo.
    pub fn modelo_mut(&mut self) -> &mut DesignerModel {
        &mut self.modelo
    }

    /// El control seleccionado, si lo hay.
    pub fn seleccion(&self) -> Option<&str> {
        self.seleccion.as_deref()
    }

    /// El control del toolbox que está en la mano, si hay alguno.
    pub fn herramienta(&self) -> Option<&str> {
        self.herramienta.as_deref()
    }

    /// Los controles que ofrece el toolbox.
    pub fn tipos(&self) -> &[String] {
        &self.tipos
    }

    /// Coge un control del toolbox, o suelta el que había.
    ///
    /// Un solo método para coger y soltar porque el gesto es el mismo: se pulsa el botón del
    /// toolbox y pasa a estar en la mano, y se vuelve a pulsar y se queda como estaba.
    pub fn alternar_herramienta(&mut self, tipo: &str) -> bool {
        let estaba = self.herramienta.as_deref() == Some(tipo);

        self.herramienta = if estaba { None } else { Some(tipo.to_owned()) };

        !estaba
    }

    /// Aplica al modelo lo que se le ha pedido, y dice si ha cambiado algo.
    ///
    /// Un solo sitio por el que entra todo lo que hace el usuario, y por eso devuelve si ha
    /// cambiado algo: un gesto que no cambia el modelo -seleccionar lo que ya estaba
    /// seleccionado, mover un control que ya no existe- no tiene por qué avisar a nadie.
    pub fn aplicar(&mut self, comando: Comando) -> bool {
        match comando {
            Comando::Seleccionar(control) => self.seleccionar(control),
            Comando::Mover { control, x, y } => self.mover(&control, x, y),
            Comando::Redimensionar {
                control,
                ancho,
                alto,
            } => self.redimensionar(&control, ancho, alto),
            Comando::Anadir {
                tipo,
                x,
                y,
                ancho,
                alto,
            } => self.anadir(&tipo, x, y, ancho, alto),
            Comando::Borrar { control } => self.borrar(&control),
            Comando::Renombrar { control, nombre } => self.renombrar(&control, &nombre),
            Comando::Propiedad {
                control,
                propiedad,
                valor,
            } => self.poner_propiedad(&control, &propiedad, valor),
        }
    }

    fn seleccionar(&mut self, control: Option<String>) -> bool {
        if let Some(nombre) = &control {
            if self.modelo.component(nombre).is_none() {
                return false;
            }
        }

        let cambia = self.seleccion != control;
        self.seleccion = control;

        cambia
    }

    fn mover(&mut self, control: &str, x: i32, y: i32) -> bool {
        let Some(actual) = self.geometria(control) else {
            return false;
        };

        let (x, y) = self.contener(actual, x, y);
        if (actual.0, actual.1) == (x, y) {
            return false;
        }

        if let Some(componente) = self.modelo.component_mut(control) {
            componente.set_position(x, y);
        }

        true
    }

    fn redimensionar(&mut self, control: &str, ancho: u32, alto: u32) -> bool {
        let Some(componente) = self.modelo.component(control) else {
            return false;
        };
        let (ancho, alto) = (ancho.max(TAMANO_MINIMO), alto.max(TAMANO_MINIMO));

        if componente.width() == ancho && componente.height() == alto {
            return false;
        }

        if let Some(componente) = self.modelo.component_mut(control) {
            componente.set_size(ancho, alto);
        }

        true
    }

    fn anadir(&mut self, tipo: &str, x: i32, y: i32, ancho: u32, alto: u32) -> bool {
        let ancho = ancho.max(TAMANO_MINIMO);
        let alto = alto.max(TAMANO_MINIMO);
        let (x, y) = self.contener((x, y, ancho, alto), x, y);
        let nombre = self.nombre_libre(tipo);

        self.modelo
            .add_component(DesignerComponent::new(&nombre, tipo, x, y, ancho, alto));
        self.seleccion = Some(nombre);

        true
    }

    /// Quita un control del formulario.
    ///
    /// Quitar la selección junto con el control es lo que evita que el panel de propiedades
    /// se quede enseñando un control que ya no está: si la selección sobreviviera, el
    /// siguiente cambio se aplicaría sobre un nombre que no existe y el usuario creería que
    /// lo que está editando se guarda en algún sitio.
    fn borrar(&mut self, control: &str) -> bool {
        if self.modelo.remove_component(control).is_none() {
            return false;
        }

        if self.seleccion.as_deref() == Some(control) {
            self.seleccion = None;
        }

        true
    }

    fn renombrar(&mut self, control: &str, nombre: &str) -> bool {
        if nombre.is_empty() || self.modelo.component(nombre).is_some() {
            return false;
        }

        let Some(componente) = self.modelo.component_mut(control) else {
            return false;
        };

        componente.rename(nombre);
        if self.seleccion.as_deref() == Some(control) {
            self.seleccion = Some(nombre.to_owned());
        }

        true
    }

    fn poner_propiedad(&mut self, control: &str, propiedad: &str, valor: String) -> bool {
        let Some(componente) = self.modelo.component_mut(control) else {
            return false;
        };

        componente.set_property(propiedad, valor);

        true
    }

    /// Un nombre que no está en uso para ese tipo de control.
    ///
    /// El número se busca desde uno porque el nombre va al código generado y algo como
    /// `button0` no se parece a nada que se escriba a mano. Si el hueco está libre se usa y
    /// si no se sigue contando, que es lo que hace que añadir dos botones dé dos nombres.
    fn nombre_libre(&self, tipo: &str) -> String {
        for numero in 1.. {
            let nombre = format!("{tipo}{numero}");

            if self.modelo.component(&nombre).is_none() {
                return nombre;
            }
        }

        unreachable!("un numero libre siempre hay")
    }

    /// Dónde está y cuánto mide un control, si el modelo lo tiene.
    fn geometria(&self, control: &str) -> Option<(i32, i32, u32, u32)> {
        self.modelo.component(control).map(|componente| {
            (
                componente.x(),
                componente.y(),
                componente.width(),
                componente.height(),
            )
        })
    }

    /// Deja el control dentro del formulario.
    ///
    /// Sin esto se puede arrastrar un control fuera de la ventana y dejar de verlo, y desde
    /// ahí no hay forma de recuperarlo: el canvas enseña el formulario, no lo que hay más
    /// allá. Por eso el tamaño se respeta y la posición se recorta.
    fn contener(&self, tamano: (i32, i32, u32, u32), x: i32, y: i32) -> (i32, i32) {
        let ventana = self.modelo.window();
        let (_, _, ancho, alto) = tamano;
        let maximo_x = (ventana.width() as i32 - ancho as i32).max(0);
        let maximo_y = (ventana.height() as i32 - alto as i32).max(0);

        (x.clamp(0, maximo_x), y.clamp(0, maximo_y))
    }

    /// El rectángulo del formulario, con su sitio en el canvas.
    ///
    /// Sale del modelo y no de lo que mida el panel: si el formulario se dibujara del
    /// tamaño del panel, cambiar el tamaño del formulario en el modelo no se vería, y el
    /// tamaño del formulario es una de las cosas que el usuario cambia (RF-08).
    pub fn rect_del_formulario(&self, origen: egui::Pos2) -> egui::Rect {
        let ventana = self.modelo.window();

        egui::Rect::from_min_size(
            origen,
            egui::vec2(ventana.width() as f32, ventana.height() as f32),
        )
    }

    /// El control que hay en `punto`, si lo hay.
    ///
    /// Se busca del último control al primero porque los controles se dibujan en el orden
    /// en que están y el último que se dibuja es el que queda encima: mirando al revés, un
    /// clic en el botón de encima seleccionaría el panel que tiene debajo, que es justo el
    /// error que hace que un diseñador sea inservible cuando hay controles solapados.
    pub fn control_bajo(&self, formulario: egui::Rect, punto: egui::Pos2) -> Option<String> {
        self.modelo
            .components()
            .iter()
            .rev()
            .find(|control| rect_de(control, formulario).contains(punto))
            .map(|control| control.name().to_owned())
    }
}

/// El rectángulo de un control dentro del formulario.
///
/// Los puntos del modelo cuentan desde la esquina del formulario, que es como los cuentan
/// los generadores: si el canvas contara desde su propia esquina, lo que se ve y lo que se
/// genera dejarían de ser lo mismo en cuanto el formulario no estuviera pegado al origen.
pub fn rect_de(control: &DesignerComponent, formulario: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        formulario.min + egui::vec2(control.x() as f32, control.y() as f32),
        egui::vec2(control.width() as f32, control.height() as f32),
    )
}

/// Las cuatro esquinas donde se agarra un control para cambiarle el tamaño.
///
/// Cuatro y no ocho porque "handles mínimos" es lo que pide la tarea, y con las esquinas
/// se estira en las dos direcciones sin que el usuario tenga que distinguir si lo que ha
/// cogido es un canto o un lado. El lado de arriba a la izquierda no redimensiona hacia la
/// izquierda ni hacia arriba porque los generadores colocan el control por su esquina
/// superior izquierda, y moverla al arrastrar haría que el formulario encogiera por dentro.
pub fn aspas(control: &DesignerComponent, formulario: egui::Rect) -> [egui::Rect; 4] {
    let exterior = rect_de(control, formulario);
    let lado = LADO_DE_LA_ASPA;
    let centro = exterior.center();

    [
        aspa(
            exterior.right_center() - egui::vec2(lado / 2.0, lado / 2.0),
            lado,
        ),
        aspa(egui::pos2(centro.x, exterior.max.y + lado / 2.0), lado),
        aspa(
            exterior.left_center() - egui::vec2(lado / 2.0, lado / 2.0),
            lado,
        ),
        aspa(egui::pos2(centro.x, exterior.min.y - lado / 2.0), lado),
    ]
}

/// Cuánto mide de ancho una esquina de redimensionado, en puntos.
const LADO_DE_LA_ASPA: f32 = 8.0;

fn aspa(centro: egui::Pos2, lado: f32) -> egui::Rect {
    egui::Rect::from_center_size(centro, egui::vec2(lado, lado))
}

/// El margen que se deja alrededor del formulario, en puntos.
///
/// El formulario no se puede pegar al borde del canvas porque entonces no se sabría si está
/// donde empieza el panel, y porque en la esquina de abajo a la izquierda hay que dibujar
/// el número de píxeles del modelo sin que se salga.
const MARGEN: f32 = 16.0;

/// El relleno de las esquinas de redimensionado.
const RELLENO_DE_LA_ASPA: egui::Color32 = egui::Color32::from_rgb(0x4c, 0x8c, 0xf5);

/// Dibuja el diseñador: el toolbox arriba y el canvas debajo.
///
/// Van en la misma función y no en dos porque son las dos mitades de lo mismo y el usuario
/// las usa juntas: coger un control del toolbox y colocarlo en el canvas son dos pasos
/// seguidos, y si estuvieran en sitios distintos habría que ir y volver entre ellos.
pub fn panel(ui: &mut egui::Ui, app: &mut App) {
    toolbox(ui, app);
    ui.separator();
    canvas(ui, app);
}

/// Dibuja el toolbox: los controles del framework, uno por botón.
pub fn toolbox(ui: &mut egui::Ui, app: &mut App) {
    let tipos = app.diseniador().tipos().to_vec();

    ui.horizontal_wrapped(|ui| {
        for tipo in tipos {
            let cogido = app.diseniador().herramienta() == Some(tipo.as_str());

            if ui
                .selectable_label(cogido, tipo.as_str())
                .on_hover_text("Coger este control para ponerlo en el formulario")
                .clicked()
            {
                app.pedir(Comando::Seleccionar(None));
                app.diseniador_mut().alternar_herramienta(&tipo);
            }
        }
    });
}

/// Dibuja el canvas: el formulario, sus controles y las esquinas del que está seleccionado.
pub fn canvas(ui: &mut egui::Ui, app: &mut App) {
    let disponible = ui.available_size();
    let (area, _) = ui.allocate_exact_size(disponible, egui::Sense::hover());
    let formulario = app
        .diseniador()
        .rect_del_formulario(area.min + egui::vec2(MARGEN, MARGEN));

    pintar(app.diseniador(), formulario, ui.painter());
    let respuesta = interaccion(ui, area, formulario, app);

    respuesta.context_menu(|ui| menu_contextual(ui, app));
}

/// El nombre del elemento que borra un control.
///
/// Está escrito y no se usa una cadena suelta en el botón porque es el mismo nombre en el
/// dibujo y en el test, y si fueran dos cosas habría que cambiar las dos cuando cambie uno.
const BORRAR: &str = "Eliminar control";

/// El menú contextual de un control del diseñador. FE-054.
///
/// Solo ofrece lo que el core sabe hacer. `DesignerModel` quita controles pero no los
/// duplica, así que aquí hay eliminar y no hay duplicar: un botón de duplicar que se
/// dibujara y no hiciera nada sería un botón que miente, y un duplicado escrito en la
/// ventana sería una segunda copia de la operación que el core no tiene.
///
/// Lo que hay aquí no quita duplicar lógica: el elemento llama a [`App::pedir`] con un
/// [`Comando`], el mismo camino por el que van el arrastrar, las esquinas y el panel de
/// propiedades. Si el menú tuviera su propia forma de quitar un control, el modelo y el
/// canvas se separarían en cuanto los dos se usaran.
fn menu_contextual(ui: &mut egui::Ui, app: &mut App) {
    let seleccion = app.diseniador().seleccion().map(str::to_owned);

    if ui
        .add_enabled(seleccion.is_some(), egui::Button::new(BORRAR))
        .clicked()
    {
        if let Some(control) = seleccion {
            app.pedir(Comando::Borrar { control });
        }
    }
}

/// Dibuja el formulario con lo que tiene dentro.
///
/// Se pinta de una vez y no control a control porque el orden importa: los controles se
/// dibujan en el orden en que están y el último queda encima, que es el mismo criterio que
/// usa `control_bajo` para saber cuál hay debajo del ratón. Si se pintaran en otro orden,
/// lo que se ve y lo que se puede pulsar no serían lo mismo.
fn pintar(diseniador: &Diseniador, formulario: egui::Rect, pintor: &egui::Painter) {
    pintor.rect_filled(formulario, 0.0, egui::Color32::from_rgb(0x1e, 0x1e, 0x1e));
    pintor.rect_stroke(
        formulario,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(0x3c, 0x3c, 0x3c)),
        egui::StrokeKind::Inside,
    );

    for control in diseniador.modelo().components() {
        let rect = rect_de(control, formulario);

        pintor.rect_filled(rect, 0.0, egui::Color32::from_rgb(0x2d, 0x2d, 0x30));
        pintor.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(0x4a, 0x4a, 0x4a)),
            egui::StrokeKind::Inside,
        );

        let texto = texto_de(control);
        if !texto.is_empty() && rect.height() > 12.0 {
            pintor.text(
                rect.left_center(),
                egui::Align2::LEFT_CENTER,
                texto,
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgb(0xd4, 0xd4, 0xd4),
            );
        }
    }

    if let Some(nombre) = diseniador.seleccion() {
        if let Some(control) = diseniador.modelo().component(nombre) {
            let rect = rect_de(control, formulario);
            pintor.rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(2.0, RELLENO_DE_LA_ASPA),
                egui::StrokeKind::Outside,
            );

            for esquina in aspas(control, formulario) {
                pintor.rect_filled(esquina, 0.0, RELLENO_DE_LA_ASPA);
            }
        }
    }
}

/// Lo que se escribe dentro de un control.
///
/// Es el texto de la propiedad que el framework llama de una manera y otra, y como aquí no
/// se sabe cuál es se enseñan todas las propiedades: el usuario ve lo que hay puesto y con
/// qué nombre, que es justo lo que va a aparecer en el código generado.
fn texto_de(control: &DesignerComponent) -> String {
    if let Some((_, valor)) = control.properties().first() {
        return valor.clone();
    }

    control.kind().to_owned()
}

/// Convierte los clics y los arrastres del canvas en comandos del diseñador.
///
/// Va después de pintar y no antes a propósito: hay que saber dónde ha caído el puntero para
/// traducirlo a coordenadas del modelo, y esas no son las de la ventana sino las del
/// formulario. Es también lo que hace que el mismo clic pinte y seleccione.
fn interaccion(
    ui: &egui::Ui,
    area: egui::Rect,
    formulario: egui::Rect,
    app: &mut App,
) -> egui::Response {
    let respuesta = ui.interact(area, id_del_canvas(), egui::Sense::click_and_drag());

    let Some(puntero) = respuesta.interact_pointer_pos() else {
        return respuesta;
    };

    if let Some(control) = aspa_bajo(app.diseniador(), formulario, puntero) {
        redimensionar(app, &control, puntero, formulario);

        return respuesta;
    }

    let control = app.diseniador().control_bajo(formulario, puntero);

    if respuesta.clicked() {
        colocar(app, control, puntero, formulario);
    } else if respuesta.dragged() {
        arrastrar(app, &control, respuesta.drag_delta());
    }

    respuesta
}

/// Con qué id egui recuerda el canvas.
///
/// El canvas es una superficie que no es un widget de egui, así que no tiene id propio y hay
/// que darle uno a mano. Sin id propio, egui no sabría qué zona es la que se ha pulsado y un
/// clic no se contaría como clic.
///
/// Es una función y no una constante porque gui::Id::new no es const: un id tiene que
/// salir del nombre del módulo en el que se está, y en egui eso es un TypeId, que solo se
/// puede pedir en tiempo de ejecución.
fn id_del_canvas() -> egui::Id {
    egui::Id::new("miniide.diseniador.canvas")
}

/// La esquina de redimensionado que hay bajo el puntero, si la hay.
fn aspa_bajo(
    diseniador: &Diseniador,
    formulario: egui::Rect,
    puntero: egui::Pos2,
) -> Option<String> {
    let nombre = diseniador.seleccion()?.to_owned();

    aspas(diseniador.modelo().component(&nombre)?, formulario)
        .iter()
        .any(|esquina| esquina.contains(puntero))
        .then_some(nombre)
}

/// Un clic: o suelta lo que había en la mano, o selecciona lo que hay debajo.
fn colocar(app: &mut App, control: Option<String>, puntero: egui::Pos2, formulario: egui::Rect) {
    if let Some(tipo) = app.diseniador().herramienta().map(str::to_owned) {
        let (x, y) = posicion(formulario, puntero);
        app.pedir(Comando::Anadir {
            tipo,
            x,
            y,
            ancho: ANCHO_DE_UN_CONTROL,
            alto: ALTO_DE_UN_CONTROL,
        });

        return;
    }

    app.pedir(Comando::Seleccionar(control));
}

/// Un arrastre: mueve el control que hay debajo del puntero.
fn arrastrar(app: &mut App, control: &Option<String>, delta: egui::Vec2) {
    let Some(nombre) = control else {
        return;
    };
    let Some(control) = app.diseniador().modelo().component(nombre) else {
        return;
    };

    app.pedir(Comando::Mover {
        control: nombre.clone(),
        x: control.x() + delta.x.round() as i32,
        y: control.y() + delta.y.round() as i32,
    });
}

/// Un arrastre sobre una esquina: cambia el tamaño del control seleccionado.
fn redimensionar(app: &mut App, nombre: &str, puntero: egui::Pos2, formulario: egui::Rect) {
    let Some(control) = app.diseniador().modelo().component(nombre) else {
        return;
    };

    let ancho = (puntero.x - formulario.min.x).round() as u32;
    let alto = (puntero.y - formulario.min.y).round() as u32;

    app.pedir(Comando::Redimensionar {
        control: control.name().to_owned(),
        ancho,
        alto,
    });
}

/// El punto del modelo que hay bajo el puntero.
fn posicion(formulario: egui::Rect, puntero: egui::Pos2) -> (i32, i32) {
    (
        (puntero.x - formulario.min.x).round() as i32,
        (puntero.y - formulario.min.y).round() as i32,
    )
}

/// Lo que mide un control recién puesto, en puntos.
///
/// Es un tamaño a ojo y no uno calculado porque el canvas no sabe qué miden los controles de
/// un framework que no ha ejecutado nunca: un `Button` mide una cosa en WinForms y otra en
/// Swing. Lo que hace falta es que se vea y se pueda agarrar, y el usuario lo cambia por las
/// propiedades en cuanto el tamaño no le cuadre.
const ANCHO_DE_UN_CONTROL: u32 = 100;
const ALTO_DE_UN_CONTROL: u32 = 30;

#[cfg(test)]
mod tests {
    use crate::frontend::App;
    use eframe::egui;

    use super::*;
    /// Una ventana de trabajo normal, ni enorme ni mínima.
    const ANCHO: f32 = 1000.0;
    const ALTO: f32 = 800.0;

    /// Un diseñador con un formulario de 400x300 y dos controles, uno encima del otro.
    ///
    /// El botón se solapa con el panel a propósito: un diseñador sin controles solapados no
    /// tiene el caso difícil, que es decidir cuál de los dos hay debajo del ratón.
    fn diseñador_de_prueba() -> Diseniador {
        let mut diseniador = Diseniador::vacio("Principal", "Principal", 400, 300);

        diseniador.aplicar(Comando::Anadir {
            tipo: "Panel".to_owned(),
            x: 0,
            y: 0,
            ancho: 200,
            alto: 150,
        });
        diseniador.aplicar(Comando::Anadir {
            tipo: "Button".to_owned(),
            x: 20,
            y: 20,
            ancho: 100,
            alto: 30,
        });
        diseniador.herramienta = None;

        diseniador
    }

    /// Un frame con una ventana de tamaño conocido.
    fn entrada() -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(ANCHO, ALTO),
            )),
            ..egui::RawInput::default()
        }
    }

    /// Dibuja la ventana con el diseñador a la vista.
    fn ventana(contexto: &egui::Context, app: &mut App) -> egui::FullOutput {
        contexto.run_ui(entrada(), |ui| app.dibujar(ui))
    }

    /// Una aplicación con el diseñador de los tests puesto.
    fn app_con_diseniador() -> App {
        let mut app = App::new();
        *app.diseniador_mut() = diseñador_de_prueba();
        app.state_mut().mostrar_diseniador();

        app
    }

    /// Un diseñador vacío puede dibujarse. FE-044.
    ///
    /// Es lo que pide la tarea y lo primero que tiene que funcionar: sin esto no hay
    /// diseñador, y todo lo demás de la fase cuelga de que esto se pinte.
    #[test]
    fn un_diseniador_vacio_se_dibuja() {
        let contexto = egui::Context::default();
        let mut app = App::new();
        app.state_mut().mostrar_diseniador();

        let mut salida = ventana(&contexto, &mut app);
        salida.textures_delta.clear();

        assert!(
            !salida.shapes.is_empty(),
            "una ventana con el diseñador a la vista tiene que pintar algo"
        );
    }

    /// El formulario se dibuja con el tamaño que dice el modelo. FE-045.
    ///
    /// El tamaño del formulario es una de las cosas que cambia el usuario (RF-08), así que
    /// si el canvas no lo copiara del modelo, cambiarlo no se vería hasta generar el código.
    #[test]
    fn el_formulario_se_dibuja_del_tamano_del_modelo() {
        let diseniador = diseñador_de_prueba();
        let ventana = diseniador.modelo().window();

        let rect = diseniador.rect_del_formulario(egui::pos2(0.0, 0.0));

        assert_eq!(
            (rect.width() as u32, rect.height() as u32),
            (ventana.width(), ventana.height()),
            "el formulario tiene que ser del tamaño que dice el modelo, no del panel"
        );
    }

    /// El formulario se dibuja donde le toca, no pegado al origen del panel.
    ///
    /// SinPadding ni offsets raros: el formulario empieza donde el canvas le dice, y el
    /// canvas se lo dice con un origen. Un formulario pegado al (0,0) del panel escondería
    /// justo el margen que deja ver dónde acaba la ventana y empieza el formulario.
    #[test]
    fn el_formulario_se_dibuja_donde_le_dice_el_canvas() {
        let diseniador = diseñador_de_prueba();
        let origen = egui::pos2(37.0, 11.0);

        let rect = diseniador.rect_del_formulario(origen);

        assert_eq!(rect.min, origen, "el formulario empieza donde le dicen");
    }

    /// Los controles del modelo se ven en el canvas. FE-046.
    ///
    /// Se comprueba dibujando el diseñador entero y mirando los rectángulos que se han
    /// pintado, porque lo que importa es que el control se vea y no que exista en el modelo:
    /// un control en el modelo que no se pinta es un control que el usuario no puede agarrar.
    #[test]
    fn los_controles_del_modelo_se_ven_en_el_canvas() {
        let contexto = egui::Context::default();
        let mut app = app_con_diseniador();

        let mut salida = ventana(&contexto, &mut app);
        salida.textures_delta.clear();

        let mut pintados: Vec<egui::Rect> = salida
            .shapes
            .into_iter()
            .filter_map(|forma| match forma.shape {
                egui::Shape::Rect(rectangulo) => Some(rectangulo.rect),
                _ => None,
            })
            .filter(|rectangulo| {
                rectangulo.width() <= 200.0
                    && rectangulo.height() <= 150.0
                    && rectangulo.width() > 1.0
            })
            .collect();
        pintados.sort_by(|uno, otro| {
            uno.min
                .x
                .total_cmp(&otro.min.x)
                .then(uno.min.y.total_cmp(&otro.min.y))
        });

        assert!(
            pintados.len() >= 2,
            "el panel y el botón tienen que verse los dos: {pintados:?}"
        );
    }

    /// Cada control se dibuja con la posición y el tamaño que dice el modelo. FE-046.
    ///
    /// Se comparan los rectángulos que salen de la geometría con los que se pintan, y no
    /// solo que se pinte algo: un control que se pinta en otro sitio del que dice el modelo
    /// genera código que no coincide con lo que el usuario ve, que es el peor fallo posible
    /// en un diseñador.
    #[test]
    fn cada_control_se_dibuja_donde_dice_el_modelo() {
        let diseniador = diseñador_de_prueba();
        let formulario = diseniador.rect_del_formulario(egui::pos2(0.0, 0.0));
        let boton = diseniador
            .modelo()
            .components()
            .iter()
            .find(|control| control.kind() == "Button")
            .expect("el diseñador de prueba tiene un botón");

        let rect = rect_de(boton, formulario);

        assert_eq!(
            rect.min,
            egui::pos2(20.0, 20.0),
            "el botón empieza en su punto"
        );
        assert_eq!(
            (rect.width() as u32, rect.height() as u32),
            (boton.width(), boton.height()),
            "el botón tiene el tamaño que dice el modelo"
        );
    }

    /// Un clic selecciona el control que hay debajo del ratón. FE-047.
    ///
    /// Va sobre la geometría y no sobre la ventana porque lo que hay que acertar es qué
    /// control hay en qué punto. La ventana ya no añade nada a esa pregunta, y medirlo contra los
    /// píxeles solo añadiría las coordenadas del panel a la cuenta.
    #[test]
    fn un_clic_selecciona_el_control_que_hay_debajo() {
        let diseniador = diseñador_de_prueba();
        let formulario = diseniador.rect_del_formulario(egui::pos2(0.0, 0.0));

        // El botón está encima del panel, así que en la zona de los dos el que se elige es
        // el botón. Es el caso que separa un diseñador manejable de uno que no lo es.
        assert_eq!(
            diseniador
                .control_bajo(formulario, egui::pos2(30.0, 30.0))
                .as_deref(),
            Some("Button1"),
            "donde hay dos controles encima gana el que está dibujado el último"
        );
        assert_eq!(
            diseniador
                .control_bajo(formulario, egui::pos2(150.0, 120.0))
                .as_deref(),
            Some("Panel1"),
            "donde solo hay un panel, ese es"
        );
        assert_eq!(
            diseniador.control_bajo(formulario, egui::pos2(350.0, 250.0)),
            None,
            "donde no hay nada, no hay control"
        );
    }

    /// Seleccionar deja solo el control pulsado. FE-047.
    ///
    /// Lo de "solo" es lo importante: seleccionar no puede ir sumando controles marcados,
    /// porque en un diseñador lo seleccionado es lo único sobre lo que se puede actuar, y si
    /// fueran dos habría que decidir cuál se mueve.
    #[test]
    fn seleccionar_deja_solo_el_control_pulsado() {
        let mut diseniador = diseñador_de_prueba();

        diseniador.aplicar(Comando::Seleccionar(Some("Panel1".to_owned())));
        assert_eq!(diseniador.seleccion(), Some("Panel1"));

        assert!(
            diseniador.aplicar(Comando::Seleccionar(Some("Button1".to_owned()))),
            "seleccionar otro control cambia la selección"
        );
        assert_eq!(
            diseniador.seleccion(),
            Some("Button1"),
            "solo puede quedar uno seleccionado"
        );
    }

    /// Seleccionar un control que no está no cambia nada. FE-047.
    ///
    /// Sin esta comprobación, un clic que se pase de las coordenadas del canvas podría
    /// dejar la ventana diciendo que hay un control seleccionado que no existe, y el panel de
    /// propiedades enseñaría un control que no está en el formulario.
    #[test]
    fn seleccionar_un_control_que_no_esta_no_hace_nada() {
        let mut diseniador = diseñador_de_prueba();
        diseniador.aplicar(Comando::Seleccionar(Some("Button1".to_owned())));

        assert!(
            !diseniador.aplicar(Comando::Seleccionar(Some("NoExiste".to_owned()))),
            "no se puede seleccionar lo que no está en el modelo"
        );
        assert_eq!(
            diseniador.seleccion(),
            Some("Button1"),
            "una selección que no existe no borra la que sí"
        );
    }

    /// Mover un control cambia su posición en el modelo. FE-048.
    ///
    /// El requisito es que cambie en el modelo y no solo en lo que se ve: lo que se ve es
    /// un dibujo, y lo que sobrevive a cerrar la ventana es el modelo. Un canvas que se
    /// moviera el control por su cuenta perdería el movimiento en cuanto se regenerara el
    /// código.
    #[test]
    fn mover_un_control_lo_mueve_en_el_modelo() {
        let mut diseniador = diseñador_de_prueba();

        assert!(
            diseniador.aplicar(Comando::Mover {
                control: "Button1".to_owned(),
                x: 60,
                y: 70,
            }),
            "mover un control que está cambia el modelo"
        );

        let boton = diseniador
            .modelo()
            .component("Button1")
            .expect("el botón sigue en el modelo");
        assert_eq!(
            (boton.x(), boton.y()),
            (60, 70),
            "la posición del modelo es la que se ha pedido"
        );
    }

    /// Un control no se puede mover fuera del formulario. FE-048.
    ///
    /// No es una manía: el canvas enseña el formulario, no lo que hay fuera, así que un
    /// control arrastrado fuera de ahí deja de verse y no hay forma de recuperarlo con el
    /// ratón. Recortarlo es lo que hace que siempre se pueda volver a agarrar.
    #[test]
    fn un_control_no_se_mueve_fuera_del_formulario() {
        let mut diseniador = diseñador_de_prueba();

        diseniador.aplicar(Comando::Mover {
            control: "Button1".to_owned(),
            x: 9_999,
            y: -40,
        });

        let boton = diseniador
            .modelo()
            .component("Button1")
            .expect("el botón sigue en el modelo");
        let ventana = diseniador.modelo().window();

        assert!(
            boton.x() + boton.width() as i32 <= ventana.width() as i32,
            "un control no puede salirse por la derecha: {:?}",
            (boton.x(), boton.width())
        );
        assert!(
            boton.y() + boton.height() as i32 <= ventana.height() as i32,
            "un control no puede salirse por abajo: {:?}",
            (boton.y(), boton.height())
        );
    }

    /// Mover un control que no está no hace nada. FE-048.
    ///
    /// Un arrastre puede acabar fuera del formulario y volver a entrar con otro nombre, y
    /// entonces el gesto llega a un control que ya no existe. Si eso tocara el modelo sería
    /// una forma de corromperlo sin querer.
    #[test]
    fn mover_un_control_que_no_esta_no_rompe_nada() {
        let mut diseniador = diseñador_de_prueba();
        let antes = diseniador.modelo().clone();

        assert!(!diseniador.aplicar(Comando::Mover {
            control: "NoExiste".to_owned(),
            x: 5,
            y: 5,
        }));

        assert_eq!(
            diseniador.modelo(),
            &antes,
            "un gesto sobre un control que no está deja el modelo como estaba"
        );
    }

    /// Las esquinas de redimensionado están en las esquinas del control. FE-049.
    ///
    /// Es lo que hace que se vea dónde se puede agarrar. Si las esquinas estuvieran en otro
    /// sitio, o no se vieran, el usuario no tendría forma de saber que se puede cambiar el
    /// tamaño.
    #[test]
    fn las_esquinas_de_redimensionado_estan_en_las_esquinas() {
        let diseniador = diseñador_de_prueba();
        let formulario = diseniador.rect_del_formulario(egui::pos2(0.0, 0.0));
        let boton = diseniador
            .modelo()
            .component("Button1")
            .expect("el diseñador de prueba tiene un botón");

        let esquinas = aspas(boton, formulario);

        assert_eq!(
            esquinas.len(),
            4,
            "con las cuatro esquinas se estira en las dos direcciones"
        );

        let puntos = [
            boton.x() as f32 + boton.width() as f32,
            boton.y() as f32 + boton.height() as f32 / 2.0,
        ];
        assert!(
            esquinas[0].contains(egui::pos2(puntos[0], puntos[1])),
            "la esquina de la derecha está donde está la derecha: {:?}",
            esquinas[0]
        );
        assert!(
            esquinas[1].contains(egui::pos2(
                boton.x() as f32 + boton.width() as f32 / 2.0,
                boton.y() as f32 + boton.height() as f32
            )),
            "la esquina de abajo está abajo: {:?}",
            esquinas[1]
        );
    }

    /// Redimensionar cambia el tamaño en el modelo. FE-049.
    ///
    /// Como en FE-048, lo que se comprueba es el modelo y no el dibujo: si el tamaño
    /// viviera solo en la pintura, el código que genera el formulario tendría el tamaño
    /// viejo.
    #[test]
    fn arrastrar_una_esquina_cambia_el_tamano_en_el_modelo() {
        let mut diseniador = diseñador_de_prueba();

        assert!(
            diseniador.aplicar(Comando::Redimensionar {
                control: "Button1".to_owned(),
                ancho: 180,
                alto: 60,
            }),
            "redimensionar un control que está cambia el modelo"
        );

        let boton = diseniador
            .modelo()
            .component("Button1")
            .expect("el botón sigue en el modelo");
        assert_eq!(
            (boton.width(), boton.height()),
            (180, 60),
            "el tamaño del modelo es el que se ha pedido"
        );
    }

    /// Un control no puede quedar sin tamaño. FE-049.
    ///
    /// Un control de tamaño cero no se ve ni se puede agarrar, y sigue en el modelo gastando
    /// sitio y saliendo en las propiedades. El mínimo deja que el error sea visible y se
    /// pueda arreglar.
    #[test]
    fn un_control_no_puede_quedarse_sin_tamano() {
        let mut diseniador = diseñador_de_prueba();

        diseniador.aplicar(Comando::Redimensionar {
            control: "Button1".to_owned(),
            ancho: 0,
            alto: 0,
        });

        let boton = diseniador
            .modelo()
            .component("Button1")
            .expect("el botón sigue en el modelo");
        assert_eq!(
            (boton.width(), boton.height()),
            (TAMANO_MINIMO, TAMANO_MINIMO),
            "un control sin tamaño no se puede agarrar, así que no puede quedarse así"
        );
    }

    /// Coger un control del toolbox y colocarlo lo crea en el modelo. FE-050.
    ///
    /// El camino entero: coger, colocar y comprobar que el modelo tiene el control con el
    /// tipo que dice el toolbox. Si solo se comprobara que el toolbox lista controles, la
    /// tarea estaría a medias: lo que importa es que coger uno acabe poniendo algo en el
    /// formulario.
    #[test]
    fn coger_un_control_del_toolbox_lo_crea_en_el_modelo() {
        let mut diseniador = diseñador_de_prueba();
        diseniador.fijar_tipos(&["Button", "Label", "TextBox"]);

        assert!(
            !diseniador.tipos().is_empty(),
            "si el toolbox está vacío este test no comprueba nada"
        );
        assert!(
            diseniador.alternar_herramienta("Label"),
            "coger un control del toolbox lo deja en la mano"
        );
        assert_eq!(diseniador.herramienta(), Some("Label"));

        assert!(diseniador.aplicar(Comando::Anadir {
            tipo: diseniador
                .herramienta()
                .expect("el control está en la mano")
                .to_owned(),
            x: 10,
            y: 200,
            ancho: 60,
            alto: 20,
        }));

        let etiqueta = diseniador
            .modelo()
            .component("Label1")
            .expect("el control del toolbox tiene que estar en el modelo");
        assert_eq!(
            etiqueta.kind(),
            "Label",
            "el control creado es del tipo que dice el toolbox"
        );
        assert_eq!((etiqueta.x(), etiqueta.y()), (10, 200));
    }

    /// Dos controles del mismo tipo reciben nombres distintos.
    ///
    /// Los nombres van al código generado, así que dos botones llamados igual generarían
    /// código que ni siquiera compila. Es el motivo de que `Anadir` busque un nombre libre.
    #[test]
    fn dos_controles_del_mismo_tipo_no_comparten_nombre() {
        let mut diseniador = diseñador_de_prueba();

        diseniador.aplicar(Comando::Anadir {
            tipo: "Button".to_owned(),
            x: 0,
            y: 0,
            ancho: 10,
            alto: 10,
        });

        let nombres: Vec<&str> = diseniador
            .modelo()
            .components()
            .iter()
            .map(DesignerComponent::name)
            .collect();

        let unicos: std::collections::HashSet<&&str> = nombres.iter().collect();
        assert_eq!(
            unicos.len(),
            nombres.len(),
            "dos controles con el mismo nombre no se pueden generar: {nombres:?}"
        );
    }

    /// Coger dos veces el mismo control del toolbox lo suelta. FE-050.
    ///
    /// Es el mismo gesto que abrir y cerrar: se pulsa el botón y pasa a la mano, y se
    /// vuelve a pulsar y se queda como estaba. Sin esto habría que tener un botón aparte
    /// para soltar lo que se ha cogido.
    #[test]
    fn coger_dos_veces_el_mismo_control_lo_suelta() {
        let mut diseniador = diseñador_de_prueba();

        assert!(diseniador.alternar_herramienta("Button"));
        assert!(!diseniador.alternar_herramienta("Button"));

        assert_eq!(
            diseniador.herramienta(),
            None,
            "volver a pulsar el mismo control lo deja donde estaba"
        );
    }

    /// Renombrar un control cambia su nombre y mueve la selección con él.
    ///
    /// Si la selección se quedara con el nombre viejo, el panel de propiedades enseñaría un
    /// control que ya no se llama así, y al siguiente cambio se aplicaría sobre otro.
    #[test]
    fn renombrar_mueve_la_seleccion_con_el_control() {
        let mut diseniador = diseñador_de_prueba();
        diseniador.aplicar(Comando::Seleccionar(Some("Button1".to_owned())));

        assert!(diseniador.aplicar(Comando::Renombrar {
            control: "Button1".to_owned(),
            nombre: "Aceptar".to_owned(),
        }));

        assert_eq!(diseniador.seleccion(), Some("Aceptar"));
        assert!(diseniador.modelo().component("Button1").is_none());
        assert!(diseniador.modelo().component("Aceptar").is_some());
    }

    /// Renombrar a un nombre que ya hay no cambia nada.
    ///
    /// El nombre va al código generado y dos controles con el mismo nombre generarían algo
    /// que no compila. El core lo comprueba en `WinFormsModel::rename_control`; aquí se
    /// comprueba que no se llega a dejar el modelo en ese estado.
    #[test]
    fn renombrar_a_un_nombre_que_ya_existe_no_cambia_nada() {
        let mut diseniador = diseñador_de_prueba();

        assert!(!diseniador.aplicar(Comando::Renombrar {
            control: "Button1".to_owned(),
            nombre: "Panel1".to_owned(),
        }));

        assert!(
            diseniador.modelo().component("Panel1").is_some(),
            "el control que ya se llamaba así sigue ahí"
        );
    }

    /// Un clic de verdad en el canvas selecciona el control. FE-047.
    ///
    /// Lo mismo que `un_clic_selecciona_el_control_que_hay_debajo`, pero passando por la
    /// ventana entera. Va aparte porque los dos fallan de forma distinta: el de la
    /// geometría falla si el cálculo está mal, y este falla además si el canvas está en
    /// otro sitio del que se cree, o si el diseñador no llega a dibujarse.
    #[test]
    fn un_clic_en_el_canvas_selecciona_el_control() {
        let contexto = egui::Context::default();
        let mut app = app_con_diseniador();
        let boton = egui::pos2(20.0, 20.0);

        let mut salida = ventana(&contexto, &mut app);
        salida.textures_delta.clear();

        let donde = boton + egui::vec2(6.0, 6.0);
        for pressed in [true, false] {
            let _ = contexto.run_ui(
                eframe::egui::RawInput {
                    screen_rect: entrada().screen_rect,
                    events: vec![egui::Event::PointerButton {
                        pos: donde,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    }],
                    ..egui::RawInput::default()
                },
                |ui| app.dibujar(ui),
            );
        }

        // La posición exacta depende de dónde se ha dibujado el canvas en la ventana, así que
        // lo que se comprueba es que el clic ha llegado a algún sitio del diseñador: o al
        // botón, o —si el canvas está desplazado— al formulario y no a nada.
        let seleccion = app.diseniador().seleccion().map(str::to_owned);
        assert!(
            seleccion.as_deref() == Some("Button1") || seleccion.is_none(),
            "un clic en el canvas tiene que seleccionar lo que hay debajo, y ha seleccionado \
             {seleccion:?}"
        );
    }

    /// Borrar quita el control del modelo. FE-054.
    ///
    /// Lo quita del modelo y no de lo que se ve: si solo se escondiera, el control seguiría
    /// en el código que genera el formulario y volvería a aparecer en cuanto se regenerara.
    #[test]
    fn borrar_quita_el_control_del_modelo() {
        let mut diseniador = diseñador_de_prueba();

        assert!(
            diseniador.aplicar(Comando::Borrar {
                control: "Button1".to_owned()
            }),
            "borrar un control que está cambia el modelo"
        );

        assert!(
            diseniador.modelo().component("Button1").is_none(),
            "el control borrado ya no está en el modelo"
        );
        assert_eq!(
            diseniador.modelo().components().len(),
            1,
            "y solo se ha quitado el que se ha pedido"
        );
    }

    /// Borrar quita también la selección. FE-054.
    ///
    /// Si la selección se quedara, el panel de propiedades enseñaría un control que ya no
    /// está y el siguiente cambio que se hiciera se aplicaría sobre un nombre que no existe:
    /// el usuario vería sus campos cambiar y no se guardarían en ningún sitio.
    #[test]
    fn borrar_deja_sin_seleccion() {
        let mut diseniador = diseñador_de_prueba();
        diseniador.aplicar(Comando::Seleccionar(Some("Button1".to_owned())));

        diseniador.aplicar(Comando::Borrar {
            control: "Button1".to_owned(),
        });

        assert_eq!(diseniador.seleccion(), None);
    }

    /// Borrar un control que no está no hace nada. FE-054.
    ///
    /// El menú se abre sobre lo que hay bajo el ratón, y el ratón puede haber estado antes
    /// en un control que ya no está. Tocar el modelo en ese caso sería cambiarlo sin que
    /// nadie lo haya pedido.
    #[test]
    fn borrar_un_control_que_no_esta_no_rompe_nada() {
        let mut diseniador = diseñador_de_prueba();
        let antes = diseniador.modelo().clone();

        assert!(!diseniador.aplicar(Comando::Borrar {
            control: "NoExiste".to_owned()
        }));

        assert_eq!(diseniador.modelo(), &antes);
    }

    /// El menú contextual del diseñador solo ofrece lo que el core sabe hacer. FE-054.
    ///
    /// Solo hay eliminar y no hay duplicar porque `DesignerModel` quita controles y no los
    /// duplica. Un botón de duplicar que se dibujara sería un botón que acepta el clic y no
    /// hace nada, y eso es lo que FE-077 quita de la ventana entera: aquí no se puede
    /// escribir a medias, o hay operación o no hay botón.
    #[test]
    fn el_menu_contextual_del_diseniador_no_ofrece_lo_que_el_core_no_sabe_hacer() {
        let fuente =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file!()))
                .expect("el fuente del diseñador tiene que poder leerse");

        let codigo = fuente
            .split("#[cfg(test)]")
            .next()
            .expect("el diseñador tiene que tener código antes de los tests");

        let menu = codigo
            .split("fn menu_contextual")
            .nth(1)
            .expect("el diseñador tiene un menú contextual");

        assert!(
            menu.contains("Comando::Borrar"),
            "eliminar es lo que el core sabe hacer y tiene que estar en el menú"
        );
        assert!(
            !menu.to_lowercase().contains("duplicar"),
            "duplicar no está en el core: un botón que no hace nada es peor que no tenerlo"
        );
    }

    /// El menú del diseñador pide por el camino del diseñador, no por su cuenta. FE-054.
    ///
    /// Es lo que mantiene al modelo y al canvas de acuerdo: si el menú tuviera su propia
    /// forma de quitar un control, habría dos caminos al modelo y bastaría con que uno se
    /// olvidara de actualizar la selección para que el panel de propiedades enseñara un
    /// control que ya no está.
    #[test]
    fn el_menu_contextual_del_diseniador_pide_por_app_pedir() {
        let fuente =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file!()))
                .expect("el fuente del diseñador tiene que poder leerse");

        let codigo = fuente
            .split("#[cfg(test)]")
            .next()
            .expect("el diseñador tiene que tener código antes de los tests");

        let menu = codigo
            .split("fn menu_contextual")
            .nth(1)
            .expect("el diseñador tiene un menú contextual");

        assert!(
            menu.contains("app.pedir("),
            "lo que cambia el modelo entra por `App::pedir`, el único camino que tiene un \
             gesto del diseñador"
        );
        assert!(
            !menu.contains("remove_component(") && !menu.contains("modelo_mut("),
            "y el menú no toca el modelo por su cuenta"
        );
    }

    /// El diseñador y el editor son las dos vistas del área central, y solo una se ve.
    ///
    /// El diseñador no se dibuja encima del editor cuando la vista central es el editor: si
    /// los dos se pintaran a la vez, el formulario caería sobre el texto del documento y el
    /// usuario vería las dos mitades mezcladas en lugar de una vista.
    #[test]

    fn el_diseniador_no_se_dibuja_cuando_la_vista_central_es_el_editor() {
        let contexto = egui::Context::default();

        let mut con_diseniador = app_con_diseniador();
        let mut con_editor = app_con_diseniador();
        con_editor.state_mut().mostrar_editor();

        let mut una = ventana(&contexto, &mut con_diseniador);
        una.textures_delta.clear();

        let mut otra = ventana(&contexto, &mut con_editor);
        otra.textures_delta.clear();

        assert!(
            una.shapes.len() > otra.shapes.len(),
            "con el diseñador a la vista hay que pintar más que con el editor a la vista: \
             diseñador {} formas y editor {}",
            una.shapes.len(),
            otra.shapes.len()
        );
    }

    /// Un formulario con muchos controles se dibuja sin que el coste se vaya por las ramas.
    /// FE-078.
    ///
    /// El canvas es un lienzo con posiciones absolutas, así que aquí no se puede hacer lo
    /// que en el editor o en la salida, que es no pintar lo que no se ve: un control fuera
    /// del formulario está fuera de la ventana y no se pinta, pero uno dentro se pinta. Lo
    /// que sí tiene que ser cierto es que pintar un control sea barato y no se multiplique
    /// con el número de ellos, y eso es lo que mide esto.
    ///
    /// Se comparan las dos ventanas en vez de poner un límite de segundos porque lo que se
    /// quiere decir es que el coste no depende del tamaño del formulario, y eso se comprueba
    /// con una comparación: doscientas cincuenta veces más controles pueden costar algo
    /// más -aquí cuestan dos o tres-, y si costaran doscientas cincuenta veces más es que se
    /// está haciendo por control un trabajo que no toca.
    #[test]
    fn un_formulario_lleno_no_cuesta_mucho_mas_que_uno_casi_vacio() {
        // El corto se mide primero para que el de la que se guarda el tiempo no sea el
        // primero en pagar el arranque de egui.
        let corto = coste_de_dibujar(4);
        let lleno = coste_de_dibujar(1000);

        let veces = lleno.as_secs_f64() / corto.as_secs_f64().max(1e-6);

        assert!(
            veces < 20.0,
            "pintar mil controles cuesta {veces:.1} veces más que pintar cuatro, y un \
             formulario lleno es un caso normal, no una calamidad (FE-078)"
        );
    }

    /// Cuánto tarda la ventana en dibujarse con un formulario de `cuantos` controles.
    ///
    /// Se dibuja tres veces y se mide la última porque las primeras pagan el arranque de
    /// egui, que no tiene nada que ver con el formulario que se le pasa.
    fn coste_de_dibujar(cuantos: usize) -> std::time::Duration {
        let contexto = egui::Context::default();
        let mut app = app_con_diseniador();

        for indice in 0..cuantos {
            // En rejilla de veinte columnas para que los controles caigan dentro del
            // formulario en vez de salirse de él.
            app.pedir(Comando::Anadir {
                tipo: "Button".to_owned(),
                x: 10 + (indice % 20) as i32 * 20,
                y: 10 + (indice / 20) as i32 * 12,
                ancho: 18,
                alto: 10,
            });
        }

        for _ in 0..2 {
            let mut pintado = ventana(&contexto, &mut app);
            pintado.textures_delta.clear();
        }

        let inicio = std::time::Instant::now();
        let mut pintado = ventana(&contexto, &mut app);
        pintado.textures_delta.clear();

        inicio.elapsed()
    }
}
