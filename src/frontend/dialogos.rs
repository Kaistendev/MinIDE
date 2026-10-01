//! Los diálogos de la ventana: las decisiones que hay que tomar antes de seguir.
//!
//! Son las dos cosas que no se pueden decidir por el usuario: si al cerrar un documento con
//! cambios se guarda o se tiran (FE-055), y qué se hace cuando la herramienta que hace
//! falta no está instalada (FE-056). Las dos son una pregunta, y las dos se contestan con
//! botones.
//!
//! Un `Dialogo` es estado de la ventana y vive en [`crate::frontend::UiState`], porque
//! aparece y desaparece con ella. Lo que hay dentro es lo mínimo para dibujarlo y para
//! saber a qué se refiere lo que se conteste: una ruta o un mensaje. Ni el documento ni su
//! texto ni el error del core, que son cosas del core y no de un modal.
//!
//! Lo que se contesta no se ejecuta aquí: el diálogo pide `Save` o `CloseDocument` por el
//! mismo camino que el resto de la ventana, y quien los ejecuta es el core. Un diálogo que
//! guardara el documento por su cuenta tendría dos caminos para guardar y no habría forma de
//! saber cuál de los dos falló.

use eframe::egui;

use crate::commands::Command;
use crate::frontend::app::App;

/// La pregunta que la ventana le hace al usuario y que no puede contestar sola.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialogo {
    /// Cerrar un documento con cambios sin guardar. FE-055.
    ///
    /// Guarda la ruta y no el documento porque lo único que hay que hacer con ella es
    /// quitarle su pestaña: si aquí hubiera un documento, el modal tendría una copia de su
    /// texto y sería la segunda verdad del documento entero.
    CerrarDocumento { ruta: String },
    /// Falta la herramienta que construye o ejecuta el proyecto. FE-056.
    ///
    /// El mensaje lo dice el proveedor, que es quien sabe cómo se busca la herramienta y
    /// cómo se llama. Escribirlo aquí sería tener dos sitios donde se dice qué le falta al
    /// equipo del usuario, y el día que el JDK dijera otra cosa uno de los dos mentiría.
    FaltaHerramienta { mensaje: String },
}

/// Los tres botones de la pregunta al cerrar, en el orden en que se contestan.
///
/// Guardar es lo que se quiere casi siempre, descartar es lo que se quiere cuando el cambio
/// no vale, y cancelar es lo que se quiere cuando el clic ha sido un error. Que estén en
/// una lista y no sueltos es lo que permite que los tests y el dibujo miren lo mismo: si cada
/// botón estuviera escrito en su sitio habría dos listas que se pueden desincronizar.
///
/// El de guardar es el mismo nombre que el del menú y el de la barra, y viene de ahí en vez
/// de escribirse: guardar es guardar en los tres sitios, y si el nombre se escribiera dos
/// veces el día que uno cambiara el otro el diálogo ofrecería una cosa distinta a la que
/// ofrece el botón de al lado (FE-076).
pub const BOTONES_DEL_CIERRE: [&str; 3] = [
    crate::frontend::acciones::GUARDAR.nombre,
    "Descartar",
    "Cancelar",
];

/// El botón de guardar del diálogo, que es la acción de guardar y no otra cosa.
///
/// Va en su propia constante porque un `match` solo puede comparar con constantes y no con
/// el campo de otra constante, y porque comparar el botón con el nombre de la acción es
/// justo lo que dice FE-076: si el botón fuera otro, no guardaría.
const GUARDAR_Y_CERRAR: &str = crate::frontend::acciones::GUARDAR.nombre;

/// Lo que dice el recuadro del diálogo.
///
/// Va escrito y no se deja que crezca con el contenido porque los dos diálogos tienen los
/// mismos botones y el mismo sitio: uno que encogiera y otro creciera movería los botones
/// mientras el usuario va a leer lo que dice.
const ANCHO: f32 = 420.0;

/// Con qué id egui recuerda el diálogo.
///
/// El diálogo no es un widget de egui, es un modal que MiniIDE abre y cierra, así que el id
/// lo pone MiniIDE. Que sea siempre el mismo es lo que hace que egui lo pueda cerrar solo
/// cuando el usuario pulsa fuera, que es lo que espera cualquiera que vea un modal.
fn id() -> egui::Id {
    egui::Id::new("miniide.dialogo")
}

/// Dibuja el diálogo que esté abierto, si hay alguno.
///
/// Se dibuja en un [`egui::Modal`] y no en un `Area` porque tiene que tapar la ventana: un
/// aviso que se lee mientras el formulario sigue por detrás puede leerse de reojo, y lo que
/// hay que decidir aquí es si se pierde el trabajo de un rato.
pub fn panel(contexto: &egui::Context, app: &mut App) {
    let Some(dialogo) = app.state().dialogo().cloned() else {
        return;
    };

    egui::Modal::new(id())
        .frame(egui::Frame::window(&contexto.style_of(contexto.theme())))
        .show(contexto, |ui| contenido(ui, app, &dialogo));
}

/// Un diálogo: lo que dice y los botones con los que se contesta.
fn contenido(ui: &mut egui::Ui, app: &mut App, dialogo: &Dialogo) {
    match dialogo {
        Dialogo::CerrarDocumento { ruta } => cerrar_documento(ui, app, ruta),
        Dialogo::FaltaHerramienta { mensaje } => falta_herramienta(ui, app, mensaje),
    }
}

/// Lo que pregunta el diálogo de cerrar.
///
/// El nombre del documento va en la pregunta y no solo en el título del modal porque un
/// modal con dos documentos modificados tiene que decir cuál de los dos se está a punto de
/// perder; sin eso la pregunta no se puede contestar.
pub fn pregunta_por_el_cierre(ruta: &str) -> String {
    format!("{ruta} tiene cambios sin guardar. ¿Qué quieres hacer con ellos?")
}

/// La pregunta al cerrar un documento con cambios. FE-055.
///
/// Los botones salen de [`BOTONES_DEL_CIERRE`] y en ese orden, que es el que ve el usuario.
fn cerrar_documento(ui: &mut egui::Ui, app: &mut App, ruta: &str) {
    ui.set_width(ANCHO);
    ui.label(pregunta_por_el_cierre(ruta));

    ui.horizontal(|ui| {
        for nombre in BOTONES_DEL_CIERRE {
            if !ui.button(nombre).clicked() {
                continue;
            }

            match nombre {
                GUARDAR_Y_CERRAR => cerrar(app, ruta, true),
                "Descartar" => cerrar(app, ruta, false),
                _ => app.state_mut().cerrar_dialogo(),
            }
        }
    });
}

/// Cierra el documento de `ruta`, guardándolo antes si el usuario lo ha pedido.
///
/// Guardar y cerrar son dos comandos y van en ese orden. No pueden ser uno solo porque son
/// dos cosas distintas: `Save` deja el documento guardado en el core y `CloseDocument` lo
/// saca de la sesión, y si el guardado falla el documento sigue abierto y el error lo
/// enseña el core.
fn cerrar(app: &mut App, ruta: &str, guardar: bool) {
    if guardar {
        app.emitir(Command::Save);
    }

    app.state_mut().cerrar_pestana(ruta);
    app.emitir(Command::CloseDocument);
    app.state_mut().cerrar_dialogo();
}

/// El aviso de que falta la herramienta. FE-056.
///
/// Un botón solo, que es lo que hace falta: aquí no hay nada que decidir, solo algo que
/// saber.
fn falta_herramienta(ui: &mut egui::Ui, app: &mut App, mensaje: &str) {
    ui.set_width(ANCHO);
    ui.label(mensaje);

    ui.horizontal(|ui| {
        if ui.button("Entendido").clicked() {
            app.state_mut().cerrar_dialogo();
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::frontend::tabs::Pestana;
    use crate::frontend::App;
    use eframe::egui;

    use super::*;

    const ANCHO_DE_LA_VENTANA: f32 = 1000.0;
    const ALTO_DE_LA_VENTANA: f32 = 800.0;

    /// El hueco en el que se dibuja la tira de pestañas.
    ///
    /// Es el mismo que usa `tabs`: lo que se mira aquí son las cruces de cerrar, y solo se
    /// distinguen si la tira está sola en su hueco.
    const ANCHO_DE_LA_TIRA: f32 = 600.0;

    /// Una ventana con un documento modificado y otro sin tocar.
    fn ventana_con_cambios() -> App {
        let mut app = App::new();
        app.state_mut()
            .abrir_pestana(Pestana::new("src/Form1.cs", true));
        app.state_mut()
            .abrir_pestana(Pestana::new("src/Program.cs", false));

        app
    }

    fn entrada() -> egui::RawInput {
        entrada_con(&[])
    }

    fn entrada_con(eventos: &[egui::Event]) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(ANCHO_DE_LA_VENTANA, ALTO_DE_LA_VENTANA),
            )),
            events: eventos.to_vec(),
            ..egui::RawInput::default()
        }
    }

    /// Los rectángulos estrechos que se han pintado en la ventana, de izquierda a derecha.
    ///
    /// Los anchos quedan fuera porque son el recuadro del modal y las zonas del layout: lo
    /// que se busca aquí son botones, y un botón es estrecho. De izquierda a derecha porque
    /// así los escribe el `horizontal` que los dibuja y así los ve el usuario.
    fn botones_pintados(contexto: &egui::Context, app: &mut App) -> Vec<egui::Rect> {
        // Dos frames y no uno: egui crea el area del modal en el primer frame y lo pinta
        // en el siguiente, y con un solo frame el modal no habria llegado a verse.
        let mut ignorado = contexto.run_ui(entrada(), |ui| app.dibujar(ui));
        ignorado.textures_delta.clear();
        let mut salida = contexto.run_ui(entrada(), |ui| app.dibujar(ui));
        salida.textures_delta.clear();

        let mut rectangulos: Vec<egui::Rect> = salida
            .shapes
            .into_iter()
            .filter_map(|forma| match forma.shape {
                egui::Shape::Rect(rectangulo) => Some(rectangulo.rect),
                _ => None,
            })
            .filter(|rectangulo| rectangulo.width() < ANCHO_DE_LA_VENTANA / 2.0)
            .collect();
        rectangulos.sort_by(|una, otra| una.min.x.total_cmp(&otra.min.x));
        rectangulos.dedup();

        rectangulos
    }

    /// Los botones del diálogo, que son los que no estaban antes de que se abriera.
    ///
    /// El modal se dibuja por encima de todo lo demás y sus botones no están en ninguna zona
    /// de la ventana, así que se distinguen por ser nuevos.
    fn botones_del_dialogo(
        contexto: &egui::Context,
        app: &mut App,
        antes: &[egui::Rect],
    ) -> Vec<egui::Rect> {
        let mut nuevos: Vec<egui::Rect> = botones_pintados(contexto, app)
            .into_iter()
            .filter(|rectangulo| !antes.contains(rectangulo))
            .collect();
        nuevos.sort_by(|una, otra| una.min.x.total_cmp(&otra.min.x));

        nuevos
    }

    /// Pulsa y suelta en `pos`.
    fn pulsar(contexto: &egui::Context, app: &mut App, pos: egui::Pos2) {
        for pressed in [true, false] {
            let mut salida = contexto.run_ui(
                entrada_con(&[egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::default(),
                }]),
                |ui| app.dibujar(ui),
            );
            salida.textures_delta.clear();
        }
    }

    /// Cierra la pestaña `indice` con su cruz, como haría el usuario.
    ///
    /// La tira se dibuja en su propio hueco porque los rectángulos se buscan por el orden en
    /// que se pintan y dentro de la ventana hay más cosas. Cada pestaña pinta su nombre y su
    /// cruz uno detrás de otro, así que la cruz de la pestaña `indice` es el rectángulo en la
    /// posición impar que le toca.
    fn cerrar_pestana(contexto: &egui::Context, app: &mut App, indice: usize) {
        let tira = |contexto: &egui::Context, app: &mut App, eventos: &[egui::Event]| {
            let mut salida = contexto.run_ui(entrada_con(eventos), |ui| {
                ui.allocate_ui(egui::vec2(ANCHO_DE_LA_TIRA, ALTO_DE_LA_VENTANA), |ui| {
                    crate::frontend::tabs::panel(ui, app);
                });
            });
            salida.textures_delta.clear();

            salida
        };

        let mut rectangulos: Vec<egui::Rect> = tira(contexto, app, &[])
            .shapes
            .into_iter()
            .filter_map(|forma| match forma.shape {
                egui::Shape::Rect(rectangulo) => Some(rectangulo.rect),
                _ => None,
            })
            .collect();
        rectangulos.sort_by(|una, otra| una.min.x.total_cmp(&otra.min.x));
        rectangulos.dedup();

        let cruz = rectangulos
            .get(indice * 2 + 1)
            .copied()
            .unwrap_or_else(|| panic!("la tira no tiene esa cruz: {rectangulos:?}"));

        for pressed in [true, false] {
            tira(
                contexto,
                app,
                &[egui::Event::PointerButton {
                    pos: cruz.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::default(),
                }],
            );
        }
    }

    /// Una ventana nueva no tiene ninguna pregunta abierta.
    ///
    /// Es lo primero que tiene que cumplirse: un diálogo que apareciera solo ya sería un
    /// fallo, y por eso la ventana arranca sin ninguno.
    #[test]
    fn una_ventana_nueva_no_tiene_ningun_dialogo() {
        let app = App::new();

        assert_eq!(app.state().dialogo(), None);
    }

    /// Un documento modificado no se cierra sin preguntar. FE-055.
    ///
    /// Es el requisito entero de la tarea: cerrar un documento con cambios sin preguntar es
    /// perder el trabajo de un rato sin aviso, y por eso la pestaña sigue ahí y la pregunta
    /// aparece en su lugar. Se comprueba también que no se pide nada al core, porque preguntar
    /// y cerrar a la vez sería cerrar y preguntar.
    #[test]
    fn un_documento_modificado_no_se_cierra_sin_preguntar() {
        let contexto = egui::Context::default();
        let mut app = ventana_con_cambios();

        cerrar_pestana(&contexto, &mut app, 0);

        assert!(
            app.state()
                .pestanas()
                .iter()
                .any(|pestana| pestana.ruta() == "src/Form1.cs"),
            "un documento modificado no puede desaparecer sin una decisión del usuario"
        );
        assert_eq!(
            app.peticiones(),
            Vec::<Command>::new(),
            "preguntar no es cerrar: no se pide nada al core hasta que el usuario conteste"
        );
        assert_eq!(
            app.state().dialogo(),
            Some(&Dialogo::CerrarDocumento {
                ruta: "src/Form1.cs".to_owned()
            }),
            "y lo que aparece es la pregunta por ese documento"
        );
    }

    /// La pregunta dice qué documento es y qué hay que decidir. FE-055.
    ///
    /// Sin el nombre del documento la pregunta no sirve: con dos pestañas modificadas no hay
    /// forma de saber cuál se está a punto de perder.
    #[test]
    fn la_pregunta_dice_que_documento_es() {
        let pregunta = pregunta_por_el_cierre("src/Form1.cs");

        assert!(
            pregunta.contains("src/Form1.cs"),
            "la pregunta tiene que decir qué documento es: {pregunta:?}"
        );
        assert!(
            pregunta.contains("cambios sin guardar"),
            "y tiene que decir por qué pregunta: {pregunta:?}"
        );
    }

    /// Guardar guarda y después cierra. FE-055.
    ///
    /// El orden importa y por eso van los dos comandos en el mismo test: guardar sin cerrar
    /// dejaría el documento abierto cuando el usuario ya ha dicho que lo cierre, y cerrar sin
    /// guardar tiraría su trabajo después de que haya pedido que se guarde.
    #[test]
    fn guardar_pide_guardar_y_luego_cerrar() {
        let contexto = egui::Context::default();
        let mut app = ventana_con_cambios();
        let antes = botones_pintados(&contexto, &mut app);

        cerrar_pestana(&contexto, &mut app, 0);
        let elementos = botones_del_dialogo(&contexto, &mut app, &antes);
        pulsar(&contexto, &mut app, elementos[0].center());

        assert_eq!(
            app.peticiones(),
            vec![Command::Save, Command::CloseDocument],
            "guardar y después cerrar, en ese orden"
        );
        assert_eq!(
            app.state().dialogo(),
            None,
            "la pregunta ya está contestada"
        );
        assert!(
            !app.state()
                .pestanas()
                .iter()
                .any(|pestana| pestana.ruta() == "src/Form1.cs"),
            "el documento se ha cerrado"
        );
    }

    /// Descartar cierra sin pedir guardar. FE-055.
    ///
    /// Descartar es la opción de tirar el trabajo, así que no puede pasar por guardar: si
    /// descartara guardara primero, el botón que promete tirar los cambios acabaría
    /// escribiéndolos en el disco.
    #[test]
    fn descartar_cierra_sin_guardar() {
        let contexto = egui::Context::default();
        let mut app = ventana_con_cambios();
        let antes = botones_pintados(&contexto, &mut app);

        cerrar_pestana(&contexto, &mut app, 0);
        let elementos = botones_del_dialogo(&contexto, &mut app, &antes);
        pulsar(&contexto, &mut app, elementos[1].center());

        assert_eq!(
            app.peticiones(),
            vec![Command::CloseDocument],
            "descartar cierra y no guarda: si guardara, no descartaría nada"
        );
        assert_eq!(app.state().dialogo(), None);
        assert_eq!(
            app.state().pestanas().len(),
            1,
            "solo se ha cerrado el documento descartado"
        );
    }

    /// Cancelar deja el documento como estaba. FE-055.
    ///
    /// Cancelar es no hacer nada, y se mide que no haga nada: ni cerrar la pestaña, ni pedir
    /// nada al core, ni dejar la pregunta a medias.
    #[test]
    fn cancelar_no_cierra_ni_pregunta() {
        let contexto = egui::Context::default();
        let mut app = ventana_con_cambios();
        let antes = botones_pintados(&contexto, &mut app);

        cerrar_pestana(&contexto, &mut app, 0);
        let elementos = botones_del_dialogo(&contexto, &mut app, &antes);
        pulsar(&contexto, &mut app, elementos[2].center());

        assert_eq!(app.peticiones(), Vec::<Command>::new());
        assert_eq!(app.state().dialogo(), None, "cancelar cierra la pregunta");
        assert_eq!(
            app.state().pestanas().len(),
            2,
            "y deja los dos documentos donde estaban"
        );
    }

    /// Un documento sin cambios se cierra sin preguntar. FE-055.
    ///
    /// Es lo que evita que la pregunta salga siempre: preguntar por un documento que no tiene
    /// nada que perder obliga a contestar una pregunta cuya respuesta es siempre la misma.
    #[test]
    fn un_documento_sin_cambios_se_cierra_sin_preguntar() {
        let contexto = egui::Context::default();
        let mut app = ventana_con_cambios();
        let antes = botones_pintados(&contexto, &mut app);

        cerrar_pestana(&contexto, &mut app, 1);

        assert_eq!(
            app.peticiones(),
            vec![Command::CloseDocument],
            "sin cambios no hay nada que decidir"
        );
        assert_eq!(app.state().dialogo(), None);
        assert_eq!(app.state().pestanas().len(), 1);
        assert!(
            botones_del_dialogo(&contexto, &mut app, &antes).is_empty(),
            "y no aparece ninguna pregunta"
        );
    }

    /// La pregunta al cerrar tiene un botón por cada decisión posible. FE-055.
    ///
    /// Se mira la lista y no lo pintado porque lo que importa es que estén las tres
    /// respuestas: si mañana el modal perdiera "Descartar", un usuario que no quiere guardar
    /// su trabajo se vería obligado a cancelar y a perderlo igualmente.
    #[test]
    fn la_pregunta_al_cerrar_tiene_las_tres_decisiones() {
        assert_eq!(
            BOTONES_DEL_CIERRE,
            [GUARDAR_Y_CERRAR, "Descartar", "Cancelar"],
            "cerrar un documento modificado se contesta de tres maneras y si no son tres no \
             hay decisión que tomar"
        );
        for nombre in BOTONES_DEL_CIERRE {
            assert!(
                !nombre.is_empty(),
                "un botón sin texto no se ve: {nombre:?}"
            );
        }
    }

    /// El botón de guardar del diálogo es el mismo que el del menú. FE-076.
    ///
    /// Guardar es guardar: si el botón del diálogo se llamara de otra manera, el usuario
    /// tendría dos nombres para la misma operación y no sabría si el modal guarda lo mismo
    /// que el botón de arriba.
    #[test]
    fn el_boton_de_guardar_del_dialogo_es_la_accion_de_guardar() {
        assert_eq!(GUARDAR_Y_CERRAR, crate::frontend::acciones::GUARDAR.nombre);
        assert_eq!(
            crate::frontend::acciones::GUARDAR.comando,
            Command::Save,
            "y lo que hay detrás es el mismo comando que pide el botón de la barra"
        );
    }

    /// El aviso de que falta la herramienta se puede cerrar. FE-056.
    ///
    /// Es lo único que hay que poder hacer con un aviso: leerlo y quitarlo de en medio. Si no
    /// se pudiera cerrar, taparía la ventana para siempre.
    #[test]
    fn el_aviso_de_que_falta_la_herramienta_se_puede_cerrar() {
        let contexto = egui::Context::default();
        let mut app = App::new();
        let antes = botones_pintados(&contexto, &mut app);
        app.state_mut().abrir_dialogo(Dialogo::FaltaHerramienta {
            mensaje: "no se ha encontrado el JDK en este equipo".to_owned(),
        });

        let elementos = botones_del_dialogo(&contexto, &mut app, &antes);
        assert_eq!(
            elementos.len(),
            1,
            "un aviso no es una pregunta y solo tiene el botón de cerrar"
        );

        pulsar(&contexto, &mut app, elementos[0].center());

        assert_eq!(app.state().dialogo(), None, "el aviso se cierra");
    }
}
