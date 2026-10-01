//! FE-057 a FE-059: un proyecto de C# con Windows Forms de principio a fin en la ventana.
//!
//! Los tests de dentro del módulo miran cada pieza por separado —las vistas del proyecto, la
//! apertura de un documento, el diseñador—. Este archivo comprueba que encajan: que al abrir
//! un proyecto de verdad aparece su árbol, que al pulsar su archivo se abre en el editor y se
//! puede escribir en él, y que ese documento se puede cambiar por su vista de diseño.
//!
//! La ventana se dibuja de verdad con egui y se le pasan clics y teclas de verdad, porque es
//! la única forma de comprobar que lo que el usuario hace llega a donde tiene que llegar. No
//! hay ninguna ventana abierta: `eframe` solo aporta los widgets, y egui sabe ejecutar un
//! frame con un contexto propio.
//!
//! FE-060 —los errores de compilación en el panel de abajo— no se prueba aquí porque necesita
//! un SDK de .NET instalado para que la compilación que falle sea de verdad. Se prueba en
//! `src/frontend/layout.rs` y en `src/frontend/operaciones.rs`, con el resultado que contesta
//! el core, que es lo mismo que recibe la ventana cuando compila de verdad.

use std::path::PathBuf;
use std::sync::Arc;

use miniide::commands::Command;
use miniide::core::ProjectType;
use miniide::frontend::App;
use miniide::templates::create_project;
use miniide::toolchain::DotNetToolchain;

/// Una ventana de trabajo normal, ni enorme ni mínima.
const ANCHO: f32 = 1000.0;
const ALTO: f32 = 800.0;

/// Un proyecto de C# con Windows Forms con sus archivos en el disco, y MiniIDE abierto en él.
///
/// El directorio lleva el nombre del test porque los tests corren a la vez y dos proyectos con
/// el mismo nombre se pisarían. Lo que se crea no se borra al final: son cuatro archivos
/// pequeños, y borrarlos es lo que hace que un fallo deje sin poder mirar qué pasó.
fn miniide_con_un_proyecto_de_winforms(nombre: &str) -> (App, PathBuf) {
    let raiz = std::env::temp_dir().join(nombre);
    let _ = std::fs::remove_dir_all(&raiz);

    let proyecto = create_project(ProjectType::CSharpWinForms, &raiz)
        .expect("la plantilla de WinForms se crea");

    let mut app = App::new();
    app.abrir(proyecto, Arc::new(DotNetToolchain));

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
///
/// egui no deja mirar los widgets que se han dibujado, pero sí lo que se ha escrito y dónde,
/// que es justo lo que el usuario ve.
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
/// Se busca el texto entero y no un trozo a propósito: "Designer" es el conmutor de vistas y
/// también está dentro del nombre de un archivo, y un clic con un trozo de nombre cae donde no
/// es. La pulsación y la soltura van en frames distintos porque egui solo cuenta un clic si las
/// dos caen dentro del mismo widget.
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

/// Un clic y una soltura sobre el texto `texto`, y lo que MiniIDE ha hecho con ello.
///
/// Es el camino que hace el usuario y es el que hay que comprobar: el clic se convierte en
/// comando, el comando lo ejecuta la ventana y lo que salga se ve en la pantalla siguiente.
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

/// Al abrir un proyecto de WinForms, su árbol aparece y se puede diseñar. FE-057.
///
/// Lo primero que se ve de un proyecto es su árbol, y es la diferencia entre abrir un proyecto
/// y abrir MiniIDE: sin él no hay forma de llegar a un archivo. Con el árbol delante se
/// comprueba también lo que habilita el framework del proyecto, que es la otra mitad de
/// FE-057: sus vistas y los controles de su diseñador.
#[test]
fn al_abrir_un_proyecto_de_winforms_se_ve_su_arbol_y_puede_disenarse() {
    let context = eframe::egui::Context::default();
    let (mut app, raiz) = miniide_con_un_proyecto_de_winforms("miniide-fe057-integra");

    let salida = dibujar(&context, &mut app, &[]);

    for archivo in ["Form1.cs", "Program.cs"] {
        assert!(
            se_ve(&salida, archivo),
            "el árbol tiene que enseñar {archivo}: {:?}",
            pintado(&salida)
        );
    }

    assert!(
        app.vistas().diseniable(),
        "un proyecto de WinForms habilita su vista de diseño: {:?}",
        app.vistas()
    );
    assert!(
        app.vistas()
            .controles()
            .iter()
            .any(|control| control == "Button"),
        "y su toolbox ofrece los controles de su framework: {:?}",
        app.vistas().controles()
    );

    let _ = std::fs::remove_dir_all(&raiz);
}

/// Pulsar un `.cs` del árbol lo abre en una pestaña con su texto y se puede escribir en él.
/// FE-058.
///
/// Es el flujo de todos los días de MiniIDE con C# y el que pide FE-058: un archivo del
/// proyecto se abre en una pestaña y su contenido es editable. Se comprueba hasta el final
/// —escribir, guardar y leer el archivo del disco— porque un editor que se ve pero no guarda
/// es medio editor, y el texto escrito se tiene que encontrar donde el usuario lo dejó.
#[test]
fn pulsar_un_archivo_csharp_lo_abre_en_una_pestana_y_se_puede_escribir() {
    let context = eframe::egui::Context::default();
    let (mut app, raiz) = miniide_con_un_proyecto_de_winforms("miniide-fe058-integra");

    // 1. Se abre el archivo del formulario desde el árbol.
    pulsar_y_avanzar(&context, &mut app, "Form1.cs");

    assert_eq!(
        app.state().pestanas().len(),
        1,
        "pulsar un archivo del árbol abre una pestaña: {:?}",
        app.state().pestanas()
    );
    assert_eq!(app.state().pestanas()[0].ruta(), "Form1.cs");
    assert_eq!(
        app.state().pestana_activa(),
        Some("Form1.cs"),
        "y la pestaña que se abre es la que se ve"
    );

    // 2. Su texto está en pantalla, no solo su nombre.
    let salida = dibujar(&context, &mut app, &[]);
    assert!(
        se_ve(&salida, "InitializeComponent"),
        "el texto del documento tiene que verse en el editor: {:?}",
        pintado(&salida)
    );

    // 3. Se escribe en él. Con un clic dentro del editor primero —el editor toma el foco con
    //    el clic, no solo con que el ratón pase por encima— y en frames aparte porque el foco
    //    se aplica al terminar el frame en que se ha pedido.
    escribir(&context, &mut app, "// escrito desde la ventana\n");

    assert!(
        app.state().pestanas()[0].esta_modificada(),
        "escribir tiene que marcar el documento como modificado: {:?}",
        app.state().pestanas()
    );

    // 4. Se guarda, y lo escrito está en el archivo del disco.
    app.emitir(Command::Save);
    app.avanzar(&context);

    let en_disco =
        std::fs::read_to_string(raiz.join("Form1.cs")).expect("el archivo se puede leer");
    assert!(
        en_disco.starts_with("// escrito desde la ventana"),
        "lo escrito tiene que estar en el archivo: {en_disco}"
    );

    let _ = std::fs::remove_dir_all(&raiz);
}

/// Con un formulario abierto se puede pasar a su vista de diseño y volver. FE-059.
///
/// Es lo que hace útil al diseñador: el formulario que se coloca en el canvas y el código del
/// mismo formulario, en la misma ventana. Se comprueba que el diseñador aparece con el nombre
/// de la clase del archivo abierto y con los controles del framework del proyecto, que es lo
/// que el toolbox puede ofrecer sin inventarse nada.
#[test]
fn un_formulario_winforms_se_puede_ver_en_codigo_y_en_disenio() {
    let context = eframe::egui::Context::default();
    let (mut app, raiz) = miniide_con_un_proyecto_de_winforms("miniide-fe059-integra");

    pulsar_y_avanzar(&context, &mut app, "Form1.cs");

    let con_codigo = dibujar(&context, &mut app, &[]);
    assert!(
        se_ve(&con_codigo, "Designer"),
        "con un formulario abierto tiene que haber forma de pasar al diseño: {:?}",
        pintado(&con_codigo)
    );

    pulsar_y_avanzar(&context, &mut app, "Designer");

    let salida = dibujar(&context, &mut app, &[]);
    assert!(
        se_ve(&salida, "Button"),
        "el toolbox ofrece los controles del framework del proyecto: {:?}",
        pintado(&salida)
    );
    assert!(
        !se_ve(&salida, "InitializeComponent"),
        "y el código del formulario ya no se está viendo: {:?}",
        pintado(&salida)
    );

    pulsar_y_avanzar(&context, &mut app, "Code");
    assert!(
        se_ve(&dibujar(&context, &mut app, &[]), "InitializeComponent"),
        "y se vuelve al código del mismo formulario"
    );

    let _ = std::fs::remove_dir_all(&raiz);
}

/// Un proyecto sin diseño detrás no ofrece un diseñador que no lleva a ninguna parte.
///
/// El otro lado de FE-059: el conmutor aparece porque el archivo abierto tiene un formulario y
/// el proyecto tiene dónde escribirlo, y desaparece cuando no lo hay. Con un archivo sin diseño
/// el interruptor sería un botón que acepta el clic y no enseña nada.
#[test]
fn un_proyecto_sin_archivo_de_diseno_no_ofrece_el_diseniador() {
    let context = eframe::egui::Context::default();
    let raiz = std::env::temp_dir().join("miniide-fe059-sin-diseno-integra");
    let _ = std::fs::remove_dir_all(&raiz);
    std::fs::create_dir_all(&raiz).expect("directorio del proyecto");
    std::fs::write(raiz.join("Form1.cs"), "public partial class Form1 { }\n")
        .expect("se escribe el archivo");

    let proyecto = miniide::project::Project::new(
        "sin-diseno",
        &raiz,
        ProjectType::CSharpWinForms,
        miniide::project::BuildConfiguration::new(
            miniide::project::ProjectRelativePath::new("bin").expect("ruta válida"),
        ),
    )
    .expect("el proyecto de prueba es válido");

    let mut app = App::new();
    app.abrir(proyecto, Arc::new(DotNetToolchain));

    pulsar_y_avanzar(&context, &mut app, "Form1.cs");

    assert_eq!(
        app.state().pestanas().len(),
        1,
        "el archivo se abre igual: lo que no hay es un diseño detrás"
    );
    let salida = dibujar(&context, &mut app, &[]);
    assert!(
        !se_ve(&salida, "Designer"),
        "sin diseño no hay conmutor que enseñe una vista vacía: {:?}",
        pintado(&salida)
    );

    let _ = std::fs::remove_dir_all(&raiz);
}
