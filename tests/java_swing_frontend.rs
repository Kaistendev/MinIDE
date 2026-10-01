//! FE-061 a FE-064: un proyecto de Java con Swing de principio a fin en la ventana.
//!
//! Es el archivo hermano de `winforms_frontend.rs` y hace lo mismo con el otro framework que
//! soporta el MVP: que al abrir un proyecto aparezca su árbol, que sus fuentes se abran en el
//! editor y se puedan escribir, y que la ventana se pueda ver en su vista de diseño. Que las
//! dos integraciones compartan el camino es la idea de FE-057: la ventana no tiene un sitio
//! para C# y otro para Java, tiene uno que pregunta al framework del proyecto.
//!
//! La ventana se dibuja de verdad con egui y se le pasan clics y teclas de verdad, porque es
//! la única forma de comprobar que lo que el usuario hace llega a donde tiene que llegar.
//!
//! FE-064 —los errores de `javac` en el panel de abajo— no se prueba aquí porque necesita un
//! JDK instalado para que la compilación que falle sea de verdad. Se prueba en
//! `src/frontend/app.rs`, con la salida real del compilador pasada por el parser del JDK.

use std::path::PathBuf;
use std::sync::Arc;

use miniide::commands::Command;
use miniide::core::ProjectType;
use miniide::frontend::App;
use miniide::templates::create_project;
use miniide::toolchain::JdkToolchain;

/// Una ventana de trabajo normal, ni enorme ni mínima.
const ANCHO: f32 = 1000.0;
const ALTO: f32 = 800.0;

/// La ventana del proyecto de Swing, que es el archivo donde vive su diseño.
const VENTANA: &str = "src/main/java/MainWindow.java";

/// La marca de una carpeta cerrada, tal y como la pinta el explorador.
const CERRADA: &str = "▸";

/// MiniIDE con un proyecto de Java con Swing abierto, con sus archivos en el disco.
///
/// El directorio lleva el nombre del test porque los tests corren a la vez y dos proyectos con
/// el mismo nombre se pisarían.
fn miniide_con_un_proyecto_de_swing(nombre: &str) -> (App, PathBuf) {
    let raiz = std::env::temp_dir().join(nombre);
    let _ = std::fs::remove_dir_all(&raiz);

    let proyecto =
        create_project(ProjectType::JavaSwing, &raiz).expect("la plantilla de Swing se crea");

    let mut app = App::new();
    app.abrir(proyecto, Arc::new(JdkToolchain));

    (app, raiz)
}

/// Un frame con una ventana de tamaño conocido y estos eventos de por medio.
fn entrada(eventos: &[eframe::egui::Event]) -> eframe::egui::RawInput {
    eframe::egui::RawInput {
        screen_rect: Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            eframe::egui::vec2(ANCHO, ALTO),
        )),
        events: eventos.to_vec(),
        ..eframe::egui::RawInput::default()
    }
}

/// Dibuja la ventana y deja quietas las texturas del frame.
fn dibujar(
    context: &eframe::egui::Context,
    app: &mut App,
    eventos: &[eframe::egui::Event],
) -> eframe::egui::FullOutput {
    let mut salida = context.run_ui(entrada(eventos), |ui| app.dibujar(ui));
    salida.textures_delta.clear();

    salida
}

/// Los textos que se han pintado, con el punto donde empieza cada uno.
fn pintado(salida: &eframe::egui::FullOutput) -> Vec<(String, eframe::egui::Pos2)> {
    salida
        .shapes
        .iter()
        .filter_map(|forma| match &forma.shape {
            eframe::egui::Shape::Text(texto) => {
                Some((texto.galley.job.text.to_string(), texto.pos))
            }
            _ => None,
        })
        .collect()
}

/// Si alguno de los textos pintados es exactamente `texto`.
fn se_ve(salida: &eframe::egui::FullOutput, texto: &str) -> bool {
    pintado(salida).iter().any(|(escrito, _)| escrito == texto)
}

/// Un clic de verdad sobre el texto `texto`.
///
/// Se busca el texto entero y no un trozo a propósito: "Main.java" está dentro de
/// "MainWindow.java", y un clic con un trozo de nombre caería en el archivo que no es. La
/// pulsación y la soltura van en frames distintos porque egui solo cuenta un clic si las dos
/// caen dentro del mismo widget.
fn pulsar(context: &eframe::egui::Context, app: &mut App, texto: &str) {
    let posicion = pintado(&dibujar(context, app, &[]))
        .into_iter()
        .find(|(escrito, _)| escrito == texto)
        .map(|(_, posicion)| posicion + eframe::egui::vec2(4.0, 4.0))
        .unwrap_or_else(|| panic!("no se ha pintado {texto:?}"));

    for pressed in [true, false] {
        let _ = dibujar(
            context,
            app,
            &[eframe::egui::Event::PointerButton {
                pos: posicion,
                button: eframe::egui::PointerButton::Primary,
                pressed,
                modifiers: eframe::egui::Modifiers::default(),
            }],
        );
    }
}

/// Un clic y una soltura sobre el texto `texto`, y luego la ventana ejecuta lo que se ha
/// pedido, que es lo que pasa entre frame y frame.
fn pulsar_y_avanzar(context: &eframe::egui::Context, app: &mut App, texto: &str) {
    pulsar(context, app, texto);
    app.avanzar(context);
}

/// Escribe `texto` en el documento abierto, como lo haría el usuario.
///
/// Primero un clic dentro del editor y después el texto, y en frames distintos: el editor toma
/// el foco con el clic —no solo con que el ratón pase por encima— y el foco se aplica al
/// terminar el frame en que se ha pedido.
fn escribir(context: &eframe::egui::Context, app: &mut App, texto: &str) {
    let dentro = eframe::egui::pos2(450.0, 300.0);

    for eventos in [
        vec![eframe::egui::Event::PointerMoved(dentro)],
        vec![
            eframe::egui::Event::PointerButton {
                pos: dentro,
                button: eframe::egui::PointerButton::Primary,
                pressed: true,
                modifiers: eframe::egui::Modifiers::default(),
            },
            eframe::egui::Event::PointerButton {
                pos: dentro,
                button: eframe::egui::PointerButton::Primary,
                pressed: false,
                modifiers: eframe::egui::Modifiers::default(),
            },
            eframe::egui::Event::PointerMoved(dentro),
        ],
        vec![
            eframe::egui::Event::PointerMoved(dentro),
            eframe::egui::Event::Text(texto.to_owned()),
        ],
    ] {
        let _ = dibujar(context, app, &eventos);
    }
}

/// Despliega las carpetas del árbol y abre un fuente de `src/main/java`.
///
/// Es el camino que tiene que hacer el usuario hasta el archivo, y está en un sitio solo
/// porque lo necesitan tres tests. Con una carpeta ya desplegada el texto que se busca es el
/// de la siguiente, y el explorador dibuja la marca junto al nombre, así que el texto a pulsar
/// es la marca y el nombre.
fn abrir_un_fuente(context: &eframe::egui::Context, app: &mut App, fuente: &str) {
    for carpeta in ["src", "main", "java"] {
        pulsar_y_avanzar(context, app, &format!("{CERRADA} {carpeta}"));
    }

    pulsar_y_avanzar(context, app, fuente);
}

/// Al abrir un proyecto de Swing se ve su árbol y se puede diseñar su ventana. FE-061.
///
/// Lo primero que se ve de un proyecto es su árbol, y en Java es un árbol de verdad: los
/// fuentes están dentro de `src/main/java`. Con el árbol delante se comprueba también la otra
/// mitad de FE-061: las vistas que declara Swing y los controles de su toolbox, que no son los
/// del otro framework.
#[test]
fn al_abrir_un_proyecto_de_swing_se_ve_su_arbol_y_puede_disenarse() {
    let context = eframe::egui::Context::default();
    let (mut app, raiz) = miniide_con_un_proyecto_de_swing("miniide-fe061-integra");

    let salida = dibujar(&context, &mut app, &[]);
    assert!(
        se_ve(&salida, "pom.xml"),
        "el archivo de proyecto se ve en el árbol: {:?}",
        pintado(&salida)
    );
    assert!(
        se_ve(&salida, &format!("{CERRADA} src")),
        "y los fuentes están debajo de una carpeta: {:?}",
        pintado(&salida)
    );

    assert!(
        app.vistas().diseniable(),
        "Swing habilita su vista de diseño: {:?}",
        app.vistas()
    );
    assert!(
        app.vistas().controles().iter().any(|c| c == "JButton"),
        "con los controles de Swing: {:?}",
        app.vistas().controles()
    );

    let _ = std::fs::remove_dir_all(&raiz);
}

/// Se llega a la ventana de Swing por el árbol, se abre y se puede escribir en ella. FE-062.
///
/// Se llega desplegando las carpetas una a una porque es lo que tiene que hacer el usuario: los
/// fuentes de Java están tres niveles más abajo y si el árbol no se despliega no hay forma de
/// llegar a ellos. De ahí que el test pulse las tres carpetas y luego el archivo.
#[test]
fn se_llega_a_la_ventana_de_swing_por_el_arbol_y_se_puede_escribir() {
    let context = eframe::egui::Context::default();
    let (mut app, raiz) = miniide_con_un_proyecto_de_swing("miniide-fe062-integra");

    abrir_un_fuente(&context, &mut app, "MainWindow.java");

    assert_eq!(
        app.state().pestanas().len(),
        1,
        "pulsar un fuente abre una pestaña: {:?}",
        app.state().pestanas()
    );
    assert_eq!(
        app.state().pestanas()[0].ruta(),
        VENTANA,
        "con la ruta del archivo dentro del proyecto"
    );

    // El texto del archivo se está viendo.
    let salida = dibujar(&context, &mut app, &[]);
    assert!(
        se_ve(&salida, "MainWindow"),
        "el texto del documento tiene que verse en el editor: {:?}",
        pintado(&salida)
    );

    // Y se puede escribir y guardar.
    escribir(&context, &mut app, "// escrito desde la ventana\n");
    assert!(
        app.state().pestanas()[0].esta_modificada(),
        "escribir tiene que marcar la pestaña: {:?}",
        app.state().pestanas()
    );

    app.emitir(Command::Save);
    app.avanzar(&context);

    let en_disco = std::fs::read_to_string(raiz.join(VENTANA)).expect("el archivo se puede leer");
    assert!(
        en_disco.starts_with("// escrito desde la ventana"),
        "lo escrito tiene que estar en el archivo: {en_disco}"
    );
    assert!(
        en_disco.contains("public class MainWindow"),
        "y el resto del archivo sigue ahí: {en_disco}"
    );

    let _ = std::fs::remove_dir_all(&raiz);
}

/// La ventana de Swing se puede ver en su vista de diseño y volver al código. FE-063.
///
/// A diferencia del otro framework, aquí el diseño vive en el mismo archivo que el código: no
/// hay un archivo generado al lado. El toolbox tiene que ofrecer los controles de Swing, que
/// es lo que declara el framework, y no los del otro.
#[test]
fn la_ventana_de_swing_se_puede_ver_en_codigo_y_en_disenio() {
    let context = eframe::egui::Context::default();
    let (mut app, raiz) = miniide_con_un_proyecto_de_swing("miniide-fe063-integra");

    abrir_un_fuente(&context, &mut app, "MainWindow.java");

    let con_codigo = dibujar(&context, &mut app, &[]);
    assert!(
        se_ve(&con_codigo, "Designer"),
        "con la ventana abierta tiene que haber forma de pasar al diseño: {:?}",
        pintado(&con_codigo)
    );

    pulsar_y_avanzar(&context, &mut app, "Designer");

    let salida = dibujar(&context, &mut app, &[]);
    assert!(
        se_ve(&salida, "JButton"),
        "el toolbox ofrece los controles de Swing: {:?}",
        pintado(&salida)
    );
    assert!(
        !se_ve(&salida, "Button"),
        "y no los del otro framework: {:?}",
        pintado(&salida)
    );
    assert!(
        !se_ve(&salida, "MainWindow"),
        "y el código de la ventana ya no se está viendo: {:?}",
        pintado(&salida)
    );

    pulsar_y_avanzar(&context, &mut app, "Code");
    assert!(
        se_ve(&dibujar(&context, &mut app, &[]), "MainWindow"),
        "y se vuelve al código de la misma ventana"
    );

    let _ = std::fs::remove_dir_all(&raiz);
}

/// Una clase de Java que no es la ventana no ofrece un diseñador.
///
/// El otro lado de FE-063: el conmutor aparece porque el diseño del proyecto es el de esta
/// clase. Con la otra clase del proyecto —la que lanza la aplicación— abierta no hay diseño
/// detrás, y un diseñador de `Main` sería un formulario que no existe en el código.
#[test]
fn una_clase_de_java_que_no_es_la_ventana_no_ofrece_el_diseniador() {
    let context = eframe::egui::Context::default();
    let (mut app, raiz) = miniide_con_un_proyecto_de_swing("miniide-fe063-main-integra");

    abrir_un_fuente(&context, &mut app, "Main.java");

    assert_eq!(
        app.state().pestanas().len(),
        1,
        "el fuente se abre igual: lo que no tiene es un diseño"
    );
    let salida = dibujar(&context, &mut app, &[]);
    assert!(
        !se_ve(&salida, "Designer"),
        "sin diseño detrás no hay conmutor: {:?}",
        pintado(&salida)
    );

    let _ = std::fs::remove_dir_all(&raiz);
}
