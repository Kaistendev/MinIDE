//! Las pestañas: los documentos que están abiertos.
//!
//! Lo que hay aquí es la lista de pestañas y lo que se ve en ellas, y no otra cosa. El
//! documento que hay detrás de cada pestaña es del core: aquí está su ruta y si tiene
//! cambios sin guardar, que es lo que la pestaña dibuja, y el texto vive en el sitio
//! donde vive para que no haya dos copias que se puedan quedar distintas.
//!
//! Lo que se ve de una pestaña es su nombre, un asterisco si el documento está modificado
//! y el botón de cerrar. Con el botón principal se cambia de documento y con el de cerrar
//! se pide cerrarlo, y las dos cosas salen por `App::emitir` como las de cualquier otra
//! superficie (FE-009).

use std::path::Path;

use eframe::egui;

use crate::commands::Command;

use super::app::App;
use super::dialogos::Dialogo;

/// Una pestaña: qué documento se ve detrás y cómo se ve la pestaña.
///
/// Es de la ventana y no del documento. Aquí está la ruta, que es lo que identifica al
/// documento y lo que se le pasa al core para saber cuál es, y si tiene cambios sin
/// guardar, que es lo que la pestaña dibuja. El texto del documento no está y no se
/// guarda: vive en el core, y en cuanto la ventana guardara una copia el editor tendría
/// dos y la que no se editara sería la que se guarda.
///
/// La ruta y no un índice del core porque un índice depende de cuántas pestañas hay
/// abiertas: con dos documentos abiertos, el segundo es el número uno en un momento y en
/// otro es el cero. La ruta no depende de nada, que es justo lo que hace falta para
/// saber cuál es la que se está viendo y cuál se puede cerrar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pestana {
    ruta: String,
    modificado: bool,
}

impl Pestana {
    /// La pestaña del documento de `ruta`, modificada o no.
    pub fn new(ruta: impl Into<String>, modificado: bool) -> Self {
        Self {
            ruta: ruta.into(),
            modificado,
        }
    }

    /// La ruta del documento que hay detrás de la pestaña.
    pub fn ruta(&self) -> &str {
        &self.ruta
    }

    /// El nombre del archivo, que es lo que la pestaña enseña.
    ///
    /// Se saca de la ruta con las reglas de las rutas y no partiéndola a mano, que es
    /// como se lo saca el core: el nombre de un archivo es el último trozo, diga la ruta
    /// lo que diga, y una ruta que no tiene nombre se enseña entera porque es lo único
    /// que hay.
    pub fn nombre(&self) -> &str {
        Path::new(&self.ruta)
            .file_name()
            .and_then(|nombre| nombre.to_str())
            .unwrap_or(&self.ruta)
    }

    /// Si el documento tiene cambios sin guardar.
    pub fn esta_modificada(&self) -> bool {
        self.modificado
    }

    /// Lo que se ve escrito en la pestaña: el nombre, y el asterisco si está modificada.
    ///
    /// El asterisco va en el texto y no en un color porque el texto es lo que se lee de
    /// un vistazo y porque el color depende de cómo se ven los colores en la máquina del
    /// usuario. Un documento con cambios sin guardar que no se distingue de uno guardado
    /// es un documento que se pierde.
    pub fn texto(&self) -> String {
        if self.modificado {
            format!("{} *", self.nombre())
        } else {
            self.nombre().to_owned()
        }
    }
}

/// Dibuja la tira de pestañas, con la que se está viendo a la vista.
///
/// Va entera o no va: con documentos abiertos se dibujan todas, y sin ellas no se dibuja
/// nada. Una tira con un hueco de más no es una tira vacía, es un hueco, y un hueco en lo
/// alto del área central empuja el editor hacia abajo sin que haya nada que mirar.
///
/// La lista se copia antes de dibujar y no se dibuja la de verdad porque cerrar una
/// pestaña la quita de la lista mientras se la está dibujando, y entonces el bucle se
/// quedaría sin la que le toca: se acabaría en la última y saltaría el error. Lo que se
/// copia es lo que se ve de cada pestaña, no los documentos.
pub fn panel(ui: &mut egui::Ui, app: &mut App) {
    let pestanas: Vec<Pestana> = app.state().pestanas().to_vec();
    let activa = app.state().pestana_activa().map(str::to_owned);

    ui.horizontal(|ui| {
        for pestana in &pestanas {
            fila(ui, pestana, activa.as_deref() == Some(pestana.ruta()), app);
        }
    });
}

/// Dibuja una pestaña: su nombre, resaltada si es la que se ve, y su botón de cerrar.
///
/// Los dos botones hacen cosas distintas y no se pisan: con el principal se cambia de
/// documento y con el de cerrar se pide cerrarlo. Un botón que hiciera las dos cosas
/// cerraría el documento que el usuario solo quería ver, y uno que no hiciera ninguna no
/// serviría para nada.
fn fila(ui: &mut egui::Ui, pestana: &Pestana, activa: bool, app: &mut App) {
    let nombre = ui.selectable_label(activa, pestana.texto());
    let cerrar = ui.small_button("×");

    if nombre.clicked() {
        app.state_mut().activar_pestana(pestana.ruta());
        app.emitir(crate::commands::Command::ActivateDocument);
    }

    if cerrar.clicked() {
        pedir_cerrar(app, pestana.ruta());
    }
}

/// Cierra la pestaña de `ruta`, o pregunta antes si el documento tiene cambios. FE-055.
///
/// La pregunta no se pide desde el diálogo sino desde aquí, que es donde está el asterisco
/// que dice que el documento está modificado. Preguntarla más tarde dejaría un hueco entre
/// el clic y la pregunta en el que la ventana todavía no ha hecho nada, y en ese hueco el
/// usuario ya ha creído que el documento se ha cerrado.
fn pedir_cerrar(app: &mut App, ruta: &str) {
    let modificado = app
        .state()
        .pestanas()
        .iter()
        .any(|pestana| pestana.ruta() == ruta && pestana.esta_modificada());

    if modificado {
        app.state_mut().abrir_dialogo(Dialogo::CerrarDocumento {
            ruta: ruta.to_owned(),
        });

        return;
    }

    app.state_mut().cerrar_pestana(ruta);
    app.emitir(Command::CloseDocument);
}

#[cfg(test)]
mod tests {
    use crate::commands::Command;
    use crate::frontend::App;
    use eframe::egui;

    use super::Pestana;

    /// Una ventana de trabajo normal, ni enorme ni mínima.
    const ANCHO: f32 = 1000.0;
    const ALTO: f32 = 800.0;

    /// El hueco que se le da a la tira de pestañas al dibujarla. No importa que sea este
    /// y no otro: lo que se mira son las pestañas, no el ancho que les tocó.
    const ANCHO_DE_LA_TIRA: f32 = 600.0;

    /// Un frame con una ventana de tamaño conocido.
    fn entrada() -> egui::RawInput {
        entrada_con(&[])
    }

    /// Un frame con una ventana de tamaño conocido y estos eventos de por medio.
    ///
    /// Los eventos van en un frame aparte cada uno porque egui solo cuenta un clic si la
    /// pulsación y la soltura caen las dos en el widget.
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

    /// Una ventana con tres documentos abiertos, viendo el primero.
    ///
    /// Tres porque con uno no se distingue una lista de un elemento suelto, y hay uno
    /// modificado para que se vea la marca. Se activa el primero a mano para que quien
    /// lee el test sepa cuál está viendo sin tener que acordarse de que abrir deja
    /// activa la última.
    fn ventana_con_tres_documentos() -> App {
        let mut app = App::new();
        app.state_mut()
            .abrir_pestana(Pestana::new("src/Form1.cs", false));
        app.state_mut()
            .abrir_pestana(Pestana::new("src/Program.cs", true));
        app.state_mut()
            .abrir_pestana(Pestana::new("src/vistas/Main.java", false));
        app.state_mut().activar_pestana("src/Form1.cs");

        app
    }

    /// Dibuja la tira y devuelve los rectángulos que ha pintado, con su color.
    ///
    /// El color va incluido porque hay pestañas que se distinguen unas de otras solo por
    /// cómo se pintan: la que se está viendo va resaltada, y es el color lo que dice
    /// cuál es. Con los rectángulos solos no se podría saber.
    ///
    /// Se dibuja en un hueco de ancho conocido y no en la ventana entera porque lo que se
    /// mide es la tira, y en la ventana entera se cuela el menú, la barra y los paneles
    /// de alrededor.
    ///
    /// Van de izquierda a derecha y no de arriba abajo porque la tira es una fila y el
    /// botón de cerrar es más bajo que el nombre de la pestaña, de modo que por altura
    /// quedarían todos los nombres juntos y todos los botones detrás, y entonces no se
    /// podría saber a qué pestaña pertenece cada botón.
    fn pintados(context: &egui::Context, app: &mut App) -> Vec<(egui::Rect, egui::Color32)> {
        let mut salida = context.run_ui(entrada(), |ui| {
            ui.allocate_ui(egui::vec2(ANCHO_DE_LA_TIRA, ALTO), |ui| {
                super::panel(ui, app);
            });
        });
        salida.textures_delta.clear();

        let mut pintados: Vec<(egui::Rect, egui::Color32)> = salida
            .shapes
            .into_iter()
            .filter_map(|forma| match forma.shape {
                egui::Shape::Rect(rectangulo) => Some((rectangulo.rect, rectangulo.fill)),
                _ => None,
            })
            .collect();
        pintados.sort_by(|(una, _), (otra, _)| una.min.x.total_cmp(&otra.min.x));
        pintados.dedup();

        pintados
    }

    /// Los nombres de las pestañas y los botones de cerrar que lleva cada una.
    ///
    /// Cada pestaña pinta su nombre y, a su lado, su botón de cerrar, así que los
    /// rectángulos salen de dos en dos y en ese orden: el primero es el nombre de la
    /// primera pestaña y el segundo su botón de cerrar, el tercero el nombre de la segunda
    /// y el cuarto su botón, y así. Se separan porque lo que se mira y lo que se pulsa
    /// son cosas distintas: el nombre para ver cuál se está viendo, el botón para
    /// cerrarla.
    fn pestanas_y_cierres(
        context: &egui::Context,
        app: &mut App,
    ) -> (Vec<egui::Rect>, Vec<egui::Rect>) {
        let mut pestanas = Vec::new();
        let mut cierres = Vec::new();

        for (indice, (rectangulo, _)) in pintados(context, app).into_iter().enumerate() {
            if indice % 2 == 0 {
                pestanas.push(rectangulo);
            } else {
                cierres.push(rectangulo);
            }
        }

        (pestanas, cierres)
    }

    /// Dibuja la tira con unos eventos de por medio y tira lo que pinta.
    fn dibujar(context: &egui::Context, app: &mut App, eventos: &[egui::Event]) {
        let mut salida = context.run_ui(entrada_con(eventos), |ui| {
            ui.allocate_ui(egui::vec2(ANCHO_DE_LA_TIRA, ALTO), |ui| {
                super::panel(ui, app);
            });
        });
        salida.textures_delta.clear();
    }

    /// Pulsa y suelta donde egui ha pintado `rectangulo`.
    fn pulsar(context: &egui::Context, app: &mut App, rectangulo: egui::Rect) {
        for pressed in [true, false] {
            let boton = egui::Event::PointerButton {
                pos: rectangulo.center(),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            };
            dibujar(context, app, &[boton]);
        }
    }

    /// Una pestaña muestra el nombre del archivo, no la ruta entera.
    ///
    /// El nombre porque es lo que distingue un archivo de otro de un vistazo, y la ruta
    /// entera porque en una tira de pestañas no cabe y porque el directorio ya se ve en
    /// el explorador. Se comprueba con rutas de distintas profundidades porque el
    /// nombre es el último trozo y no el primero.
    #[test]
    fn una_pestana_muestra_el_nombre_del_archivo() {
        assert_eq!(Pestana::new("src/Form1.cs", false).nombre(), "Form1.cs");
        assert_eq!(Pestana::new("Form1.cs", false).nombre(), "Form1.cs");
        assert_eq!(
            Pestana::new("src/vistas/Main.java", false).nombre(),
            "Main.java"
        );
    }

    /// Una pestaña modificada se marca con un asterisco, y si no lo está no.
    ///
    /// El asterisco es lo que dice que hay cambios sin guardar, y por eso va en el texto
    /// de la pestaña y no en un color: el texto es lo que se lee de un vistazo y lo que
    /// no depende de cómo se ven los colores de la máquina del usuario.
    #[test]
    fn una_pestana_modificada_se_marca_con_un_asterisco() {
        assert_eq!(Pestana::new("src/Form1.cs", true).texto(), "Form1.cs *");
        assert_eq!(Pestana::new("src/Form1.cs", false).texto(), "Form1.cs");
    }

    /// Una ventana sin documentos no tiene pestañas ni tiene ninguna activa.
    ///
    /// Con el explorador vacío y sin proyecto abierto es lo que se ve, y está bien: una
    /// tira de pestañas con algo puesto que no sea un documento enseñaría algo que
    /// MiniIDE no tiene abierto.
    #[test]
    fn sin_documentos_no_hay_ninguna_pestana() {
        let app = App::new();

        assert!(app.state().pestanas().is_empty());
        assert_eq!(app.state().pestana_activa(), None);
    }

    /// Abrir un documento enseña su pestaña y la deja a la vista.
    ///
    /// Y la deja a la vista porque abrir es pedir verlo, que es lo mismo que dice el
    /// core: `OpenTabs::open` deja activa la pestaña que abre. Las dos reglas son la
    /// misma y por eso van en la misma frase, no porque una copie a la otra.
    #[test]
    fn abrir_un_documento_enseña_su_pestana_y_la_pone_visible() {
        let mut app = App::new();

        app.state_mut()
            .abrir_pestana(Pestana::new("src/Form1.cs", false));

        assert_eq!(app.state().pestanas().len(), 1);
        assert_eq!(app.state().pestanas()[0].ruta(), "src/Form1.cs");
        assert_eq!(app.state().pestana_activa(), Some("src/Form1.cs"));
    }

    /// Abrir un documento que ya está abierto no crea una segunda pestaña.
    ///
    /// Con dos pestañas del mismo archivo el usuario cerraría una y seguiría viendo el
    /// mismo documento, y no sabría cuál de las dos se ha cerrado. Es lo que hace el core
    /// en `OpenTabs::open`: devolver la que ya había y traerla a delante.
    #[test]
    fn abrir_un_documento_ya_abierto_no_crea_una_segunda_pestana() {
        let mut app = App::new();
        app.state_mut()
            .abrir_pestana(Pestana::new("src/Form1.cs", false));
        app.state_mut()
            .abrir_pestana(Pestana::new("src/Program.cs", false));

        app.state_mut()
            .abrir_pestana(Pestana::new("src/Form1.cs", true));

        assert_eq!(app.state().pestanas().len(), 2);
        assert_eq!(app.state().pestana_activa(), Some("src/Form1.cs"));
    }

    /// Al cerrar la activa pasa a estarlo la que ocupa su sitio.
    ///
    /// Es la misma regla que la del core en `OpenTabs::close` y por eso se llama igual:
    /// si se cerrara una y en la ventana no, el usuario cerraría la pestaña que está
    /// viendo y se quedaría viendo un documento que él cree haber cerrado.
    #[test]
    fn al_cerrar_la_activa_pasa_a_ser_activa_la_que_ocupa_su_sitio() {
        let mut app = ventana_con_tres_documentos();

        app.state_mut().cerrar_pestana("src/Form1.cs");

        assert_eq!(app.state().pestanas().len(), 2);
        assert_eq!(app.state().pestana_activa(), Some("src/Program.cs"));
    }

    /// Si la activa era la última, pasa a estarlo la nueva última.
    ///
    /// La otra mitad de la misma regla, en su propio test porque si el numerito del medio
    /// saliera mal, esta seguiría acertando y la otra no.
    #[test]
    fn al_cerrar_la_ultima_activa_pasa_a_ser_activa_la_nueva_ultima() {
        let mut app = ventana_con_tres_documentos();
        app.state_mut().activar_pestana("src/vistas/Main.java");

        app.state_mut().cerrar_pestana("src/vistas/Main.java");

        assert_eq!(app.state().pestana_activa(), Some("src/Program.cs"));
    }

    /// No se puede activar una pestaña que no está abierta.
    ///
    /// La ruta la elige quien llama y puede estar equivocada, y activar no puede
    /// dejar marcada una pestaña que no existe: se vería una ventana con el documento
    /// en una pestaña y el resaltado en ninguna.
    #[test]
    fn no_se_puede_activar_una_pestana_que_no_esta_abierta() {
        let mut app = ventana_con_tres_documentos();

        let activada = app.state_mut().activar_pestana("src/NoExiste.cs");

        assert!(!activada);
        assert_eq!(app.state().pestana_activa(), Some("src/Form1.cs"));
    }

    #[test]
    fn cerrar_una_pestana_que_no_esta_abierta_no_hace_nada() {
        let mut app = ventana_con_tres_documentos();

        let cerrada = app.state_mut().cerrar_pestana("src/NoExiste.cs");

        assert!(!cerrada);
        assert_eq!(app.state().pestanas().len(), 3);
        assert_eq!(app.state().pestana_activa(), Some("src/Form1.cs"));
    }

    /// Cada documento abierto tiene su pestaña, en el orden en que se abrieron.
    ///
    /// Tres documentos, tres pestañas: es lo que hace que la lista sea una lista y no una
    /// pestaña con la última de memoria. Y van en el orden en que se abrieron porque es
    /// el orden en que las fue pidiendo el usuario y en el que las espera encontrar.
    #[test]
    fn cada_documento_abierto_tiene_su_pestana() {
        let context = egui::Context::default();
        let mut app = ventana_con_tres_documentos();

        let (pestanas, cierres) = pestanas_y_cierres(&context, &mut app);

        assert_eq!(pestanas.len(), 3, "una pestaña por documento abierto");
        assert_eq!(
            cierres.len(),
            3,
            "una pestaña que se puede cerrar se puede cerrar"
        );
        assert!(
            pestanas[0].min.x < pestanas[1].min.x && pestanas[1].min.x < pestanas[2].min.x,
            "las pestañas van de izquierda a derecha en el orden en que se abrieron: {pestanas:?}"
        );
    }

    /// Sin documentos no se dibuja ninguna pestaña.
    ///
    /// Una tira con un rectángulo de más no es una tira vacía: es un hueco, y un hueco en
    /// la parte de arriba del área central empuja el editor hacia abajo sin que haya nada
    /// que mirar.
    #[test]
    fn sin_pestanas_no_se_dibuja_ninguna() {
        let context = egui::Context::default();
        let mut app = App::new();

        let (pestanas, cierres) = pestanas_y_cierres(&context, &mut app);

        assert!(pestanas.is_empty());
        assert!(cierres.is_empty());
    }

    /// La pestaña que se está viendo se distingue de las otras.
    ///
    /// Se distingue por el color con el que se pinta, y se mira ese color porque es lo
    /// único que dice cuál es la activa: sin resaltarla, el usuario no sabe qué documento
    /// tiene delante cuando hay tres abiertos con el mismo nombre de archivo en
    /// directorios distintos.
    #[test]
    fn la_pestana_que_se_esta_viendo_se_distingue_de_las_otras() {
        let context = egui::Context::default();
        let mut app = ventana_con_tres_documentos();

        let (pestanas, _) = pestanas_y_cierres(&context, &mut app);
        let (_, color_primera) = pintados(&context, &mut app)[0];
        let (_, color_segunda) = pintados(&context, &mut app)[2];

        assert_eq!(pestanas.len(), 3);
        assert_ne!(
            color_primera, color_segunda,
            "la pestaña que se está viendo tiene que verse distinta de las demás"
        );
    }

    /// La pestaña de un documento modificado se distingue de la que no lo está.
    ///
    /// La marca es el asterisco del texto, así que lo que cambia es lo ancha que es la
    /// pestaña: la segunda lleva un asterisco y es más ancha que la primera, y las dos
    /// son del mismo archivo para que lo único que las separe sea la marca. Se mide el
    /// ancho y no el texto porque el texto no se ve en la salida de un frame, y porque lo
    /// que importa es que se distingan de verdad, no que el asterisco esté escrito.
    #[test]
    fn la_pestana_modificada_se_distingue_de_la_que_no_lo_esta() {
        let context = egui::Context::default();
        let mut app = App::new();
        app.state_mut()
            .abrir_pestana(Pestana::new("src/Form1.cs", false));
        let (pestanas, _) = pestanas_y_cierres(&context, &mut app);

        let mut app = App::new();
        app.state_mut()
            .abrir_pestana(Pestana::new("src/Form1.cs", true));
        let (modificadas, _) = pestanas_y_cierres(&context, &mut app);

        assert!(
            modificadas[0].width() > pestanas[0].width(),
            "la pestaña modificada tiene que verse distinta: {:?} y {:?}",
            pestanas[0],
            modificadas[0]
        );
    }

    /// Pulsar otra pestaña cambia el documento que se ve, y lo pide una sola vez.
    ///
    /// Lo de "una sola vez" es lo que la distingue de un clic que pide dos veces, que es
    /// lo que pasa cuando el mismo clic se convierte en comando en dos sitios.
    #[test]
    fn pulsar_otra_pestana_cambia_el_documento_que_se_ve() {
        let context = egui::Context::default();
        let mut app = ventana_con_tres_documentos();
        let (pestanas, _) = pestanas_y_cierres(&context, &mut app);

        pulsar(&context, &mut app, pestanas[2]);

        assert_eq!(
            app.state().pestana_activa(),
            Some("src/vistas/Main.java"),
            "el documento que se ve es el de la pestaña que se ha pulsado"
        );
        assert_eq!(
            app.peticiones(),
            vec![Command::ActivateDocument],
            "cambiar de documento se pide una vez y solo esa"
        );
    }

    /// Cerrar una pestaña la hace desaparecer y pide cerrar el documento.
    ///
    /// Lo de disappear y lo de pedir son las dos mitades de FE-018 y van en un test
    /// porque si solo se comprobara una, cerrarla sería o un botón que no hace nada o una
    /// lista que se acorta sola.
    ///
    /// La pestaña desaparece de la tira y el documento se lo pide al core, que es quien
    /// sabe si se puede cerrar: `OpenTabs::close` devuelve la pestaña para que quien la
    /// cierra pueda mirar si tenía cambios sin guardar, y esa decisión no es de la
    /// ventana. Lo que la ventana no hace es decidirla por su cuenta.
    #[test]
    fn cerrar_una_pestana_la_hace_desaparecer_y_pide_cerrar_el_documento() {
        let context = egui::Context::default();
        let mut app = ventana_con_tres_documentos();
        let (_, cierres) = pestanas_y_cierres(&context, &mut app);

        pulsar(&context, &mut app, cierres[0]);

        let (pestanas, _) = pestanas_y_cierres(&context, &mut app);
        assert_eq!(
            pestanas.len(),
            2,
            "la pestaña cerrada desaparece de la tira"
        );
        assert!(
            !app.state()
                .pestanas()
                .iter()
                .any(|pestana| pestana.ruta() == "src/Form1.cs"),
            "la pestaña que se ha cerrado ya no está"
        );
        assert_eq!(
            app.peticiones(),
            vec![Command::CloseDocument],
            "cerrar se pide una vez y solo esa"
        );
    }

    /// Cerrar otra pestaña no cambia el documento que se está viendo.
    ///
    /// Y no lo pide cambiar, porque no lo ha cambiado: si se pidiera, el core se creería
    /// que el usuario está viendo otro documento del que cree.
    #[test]
    fn cerrar_otra_pestana_no_cambia_el_documento_que_se_ve() {
        let context = egui::Context::default();
        let mut app = ventana_con_tres_documentos();
        let (_, cierres) = pestanas_y_cierres(&context, &mut app);

        pulsar(&context, &mut app, cierres[2]);

        assert_eq!(app.state().pestanas().len(), 2);
        assert_eq!(
            app.state().pestana_activa(),
            Some("src/Form1.cs"),
            "cerrar otra pestaña deja viendo el mismo documento"
        );
        assert_eq!(
            app.peticiones(),
            vec![Command::CloseDocument],
            "cerrar otra pestaña no pide cambiar de documento"
        );
    }

    /// Una pestaña visual no guarda el documento.
    ///
    /// FE-015 lo pide así: las pestañas son de la ventana y los documentos son del core.
    /// En cuanto la pestaña guardara el texto, el editor tendría dos copias -la del core y
    /// la de la ventana- y la que no se editara sería la que se guarda.
    ///
    /// Se comprueba leyendo el archivo y no los tests porque lo que hay que ver es lo que
    /// el tipo puede llegar a guardar, y eso no se ve desde fuera: una pestaña con el
    /// texto dentro se parece muchísimo a una que solo guarda la ruta.
    #[test]
    fn una_pestana_visual_no_guarda_el_documento() {
        let archivo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("frontend")
            .join("tabs.rs");
        let fuente = std::fs::read_to_string(&archivo)
            .unwrap_or_else(|error| panic!("tabs.rs tiene que poder leerse: {error}"));
        let tipo = fuente
            .split("pub struct Pestana")
            .nth(1)
            .and_then(|resto| resto.split_once('{'))
            .and_then(|(_, resto)| resto.split_once('}'))
            .map(|(campos, _)| campos)
            .unwrap_or_default();

        assert!(
            !tipo.contains("Document") && !tipo.contains("TextBuffer"),
            "la pestaña visual guarda la ruta y si está modificada, no el documento: {tipo}"
        );
    }
}
