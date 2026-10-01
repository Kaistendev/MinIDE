//! FE-073 a FE-075: la ventana y el core juntos, de principio a fin.
//!
//! Los tests de dentro del crate miran una pieza cada vez: que un atajo pida su comando, que la
//! barra enseñe un estado, que el diálogo avise. Este archivo mira lo contrario —que lo que
//! pide la ventana lo ejecuta el core y lo que contesta el core se ve en la ventana—, y eso es
//! lo que ningún test de una sola pieza puede ver. Sin esto, una ventana puede pasar todos sus
//! tests y no guardar nunca un archivo: cada pieza respondería lo suyo y el camino entero
//! estaría roto justo en la unión.
//!
//! Todo lo que aquí se hace se hace como lo haría el usuario: la ventana se dibuja de verdad
//! con egui, se pulsan sus textos, se escriben sus teclas y se espera a lo que conteste el
//! core. No hay ventana abierta —eframe solo aporta los widgets— y no hace falta que estén
//! instalados el SDK de .NET ni el de Java: el proveedor de toolchain es de prueba y lanza el
//! intérprete de comandos de Windows, que está en cualquier máquina donde corre MiniIDE. Con
//! él el camino de compilar y ejecutar es el de verdad —el core lanza el proceso y recoge su
//! código de salida— y no una respuesta inventada.
//!
//! Los tests que lanzan procesos de verdad se ignoran con `#[ignore]`, que es como se marcan
//! en el resto del repositorio. Lo que se comprueba es el camino entero, y ese camino no se
//! puede comprobar sin lanzar nada.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use miniide::commands::Command;
use miniide::core::{CoreResult, ProjectType};
use miniide::diagnostics::Diagnostic;
use miniide::frontend::App;
use miniide::project::Project;
use miniide::runtime::ProcessOutput;
use miniide::templates::create_project;
use miniide::toolchain::{Invocation, ToolchainProvider};

/// Una ventana de trabajo normal, ni enorme ni mínima.
const ANCHO: f32 = 1000.0;
const ALTO: f32 = 800.0;

/// Cuánto se espera como mucho a que conteste el core antes de rendirse.
///
/// Es un tope y no una espera: los tests pasan en cuanto el core contesta y esta cifra solo
/// sirve para que un flujo colgado falle con un mensaje en vez de quedarse esperando siempre.
const PACIENCIA: Duration = Duration::from_secs(20);

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
fn pulsar(context: &eframe::egui::Context, app: &mut App, texto: &str) {
    pulsar_el(context, app, texto, 0)
}

/// Un clic de verdad sobre la `ocurrencia`-ésima vez que aparece `texto`.
///
/// Hace falta porque hay palabras que están dos veces en pantalla: "Compilar" es un elemento
/// del menú y un botón de la barra, y el primero de los dos que se encuentra al leer los textos
/// de arriba abajo es el del menú —pulsarlo abre el menú en vez de compilar—. Con la ocurrencia
/// se apunta al botón sin tener que saber de dónde sale el texto que se busca.
fn pulsar_el(context: &eframe::egui::Context, app: &mut App, texto: &str, ocurrencia: usize) {
    let posicion = pintado(&dibujar(context, app, &[]))
        .into_iter()
        .filter(|(escrito, _)| escrito == texto)
        .nth(ocurrencia)
        .map(|(_, posicion)| posicion + eframe::egui::vec2(4.0, 4.0))
        .unwrap_or_else(|| panic!("{texto:?} no aparece {ocurrencia} veces en la ventana"));

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

/// Una tecla de verdad: la pulsación y la soltura, y lo que la ventana ha hecho con ellas.
///
/// Van en frames distintos porque egui solo mira lo que se ha pulsado en el frame en que
/// ocurre, y una tecla que se aprieta y se suelta en el mismo frame no se ve.
fn pulsar_tecla(
    context: &eframe::egui::Context,
    app: &mut App,
    tecla: eframe::egui::Key,
    ctrl: bool,
) {
    let modifiers = eframe::egui::Modifiers {
        ctrl,
        ..eframe::egui::Modifiers::default()
    };

    for pressed in [true, false] {
        let _ = dibujar(
            context,
            app,
            &[eframe::egui::Event::Key {
                key: tecla,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            }],
        );
    }
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

/// Da vueltas a la ventana hasta que lo que se ve deje de decir `texto`.
///
/// Son las vueltas que hace el usuario mientras espera: dibujar, avanzar y mirar. No espera a
/// que el core conteste con un `sleep` largo —eso congelaría la ventana, que es justo lo que
/// estos tests quieren comprobar que no pasa— sino que va mirando con un tope de tiempo.
///
/// Espera a lo que se ve y no al estado interno porque esto es un test de integración: lo que
/// tiene que cambiar para el usuario es la barra de estado, y un estado que cambia y no se ve
/// no le sirve de nada a nadie.
fn esperar_a_que_deje_de_decir(context: &eframe::egui::Context, app: &mut App, texto: &str) {
    let limite = Instant::now() + PACIENCIA;

    while Instant::now() < limite {
        if !se_ve(&dibujar(context, app, &[]), texto) {
            return;
        }

        app.avanzar(context);
        std::thread::sleep(Duration::from_millis(10));
    }

    panic!("el core no ha terminado: la ventana sigue diciendo {texto:?}");
}

/// Da vueltas a la ventana hasta que lo que se ve dice `texto`.
fn esperar_a_que_diga(context: &eframe::egui::Context, app: &mut App, texto: &str) {
    let limite = Instant::now() + PACIENCIA;

    while Instant::now() < limite {
        if se_ve(&dibujar(context, app, &[]), texto) {
            return;
        }

        app.avanzar(context);
        std::thread::sleep(Duration::from_millis(10));
    }

    panic!("el core no ha llegado: la ventana no dice {texto:?}");
}

/// Un toolchain de prueba que lanza el intérprete de comandos de Windows.
///
/// Existe para que el camino entero —ventana, comando, core, proceso, estado— se pruebe sin el
/// SDK de .NET ni el de Java instalados, que es lo que hace que estos tests puedan correr en
/// cualquier máquina. Lo que lanza es `cmd`: para compilar, algo que sale bien al momento, y
/// para ejecutar, algo que se queda vivo lo suficiente para poder pararlo.
struct HerramientaDePrueba {
    arguments_de_build: Vec<String>,
    arguments_de_run: Vec<String>,
    espera: Option<Duration>,
}

impl HerramientaDePrueba {
    /// Compila con `cmd /C exit 0`, que sale bien al instante, y ejecuta lo mismo.
    fn que_compila() -> Self {
        Self {
            arguments_de_build: vec!["/C".to_owned(), "exit 0".to_owned()],
            arguments_de_run: vec!["/C".to_owned(), "exit 0".to_owned()],
            espera: None,
        }
    }

    /// Ejecuta con `cmd /C ping`, que se queda esperando lo suficiente para poder pararlo.
    fn que_ejecuta() -> Self {
        Self {
            arguments_de_build: vec!["/C".to_owned(), "exit 0".to_owned()],
            arguments_de_run: vec!["/C".to_owned(), "ping -n 60 127.0.0.1 >NUL".to_owned()],
            espera: None,
        }
    }

    /// Tarda `espera` antes de preparar la llamada, y no lanza ningún proceso.
    ///
    /// Es para el test que comprueba que la ventana no se congela: mientras el core prepara
    /// la llamada, la ventana tiene que seguir dibujando, y esto es lo que la tiene ocupada
    /// sin necesitar ni el sistema operativo ni un SDK.
    fn que_tarda(espera: Duration) -> Self {
        Self {
            arguments_de_build: Vec::new(),
            arguments_de_run: Vec::new(),
            espera: Some(espera),
        }
    }

    fn llamada(&self, argumentos: Vec<String>, proyecto: &Project) -> CoreResult<Invocation> {
        if let Some(espera) = self.espera {
            std::thread::sleep(espera);
        }

        Ok(Invocation::new(
            "cmd",
            argumentos,
            proyecto.root().to_path_buf(),
        ))
    }
}

impl ToolchainProvider for HerramientaDePrueba {
    fn project_type(&self) -> ProjectType {
        ProjectType::CSharpWinForms
    }

    fn tool(&self) -> &'static str {
        "cmd"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn build_invocation(&self, project: &Project) -> CoreResult<Invocation> {
        self.llamada(self.arguments_de_build.clone(), project)
    }

    fn run_invocation(&self, project: &Project) -> CoreResult<Invocation> {
        self.llamada(self.arguments_de_run.clone(), project)
    }

    fn parse_diagnostics(&self, _output: &ProcessOutput, _root: &Path) -> Vec<Diagnostic> {
        Vec::new()
    }
}

/// Un proyecto de C# con Windows Forms con sus archivos en el disco, y MiniIDE abierto en él.
///
/// El directorio lleva el nombre del test porque los tests corren a la vez y dos proyectos con
/// el mismo nombre se pisarían. Lo que se crea no se borra al final en los helpers: es cada
/// test quien lo borra, al terminar y no antes, para que un fallo deje poder mirar qué pasó.
fn miniide_con_un_proyecto(
    nombre: &str,
    herramienta: Arc<dyn ToolchainProvider>,
) -> (App, PathBuf) {
    let raiz = std::env::temp_dir().join(nombre);
    let _ = std::fs::remove_dir_all(&raiz);

    let proyecto = create_project(ProjectType::CSharpWinForms, &raiz)
        .expect("la plantilla de WinForms se crea");

    let mut app = App::new();
    app.abrir(proyecto, herramienta);

    (app, raiz)
}

/// Guardar desde el atajo de teclado escribe el documento en su archivo. FE-073.
///
/// FE-073 pide que el guardado se compruebe "sin depender de un botón concreto", y por eso
/// aquí se guarda con Ctrl+S y no pulsando nada: el botón es una de las maneras de llegar al
/// mismo comando, y atajo, menú y botón no tienen por qué guardar distinto.
///
/// Lo que se comprueba es el final del camino —el texto escrito está en el archivo del disco— y
/// no que la ventana haya emitido un comando, porque un comando que nadie ejecuta también
/// pasaría un test de comandos.
#[test]
fn el_atajo_de_guardar_escribe_el_documento_en_su_archivo() {
    let context = eframe::egui::Context::default();
    let (mut app, raiz) = miniide_con_un_proyecto(
        "miniide-fe073-atajo-de-guardar",
        Arc::new(HerramientaDePrueba::que_compila()),
    );

    pulsar(&context, &mut app, "Form1.cs");
    app.avanzar(&context);
    escribir(&context, &mut app, "// escrito con el atajo\n");
    assert!(
        app.state().pestanas()[0].esta_modificada(),
        "escribir tiene que marcar el documento: {:?}",
        app.state().pestanas()
    );

    pulsar_tecla(&context, &mut app, eframe::egui::Key::S, true);
    app.avanzar(&context);

    let en_disco =
        std::fs::read_to_string(raiz.join("Form1.cs")).expect("el archivo se puede leer");
    assert!(
        en_disco.starts_with("// escrito con el atajo"),
        "lo escrito tiene que estar en el archivo: {en_disco}"
    );
    assert!(
        en_disco.contains("InitializeComponent"),
        "y el archivo que ya había no se ha perdido: {en_disco}"
    );
    assert!(
        !app.state().pestanas()[0].esta_modificada(),
        "guardado lo que se ha escrito, la pestaña deja de estar modificada: {:?}",
        app.state().pestanas()
    );

    let _ = std::fs::remove_dir_all(&raiz);
}

/// Guardar no depende de un botón: los caminos distintos llegan al mismo core. FE-073.
///
/// Es el otro lado de FE-073. El test anterior comprueba que el atajo guarda, y este que lo que
/// guarda es el core y no la ventana: si el atajo guardara por su cuenta y el botón por la
/// suya, el segundo guardado dejaría el documento en un estado distinto del primero. Se pide
/// guardar dos veces —una con el atajo y otra con el comando que emiten el menú y el botón— y
/// las dos tienen que acabar en el mismo archivo.
#[test]
fn guardar_dos_veces_por_caminos_distintos_llega_al_mismo_core() {
    let context = eframe::egui::Context::default();
    let (mut app, raiz) = miniide_con_un_proyecto(
        "miniide-fe073-dos-caminos",
        Arc::new(HerramientaDePrueba::que_compila()),
    );

    pulsar(&context, &mut app, "Form1.cs");
    app.avanzar(&context);
    escribir(&context, &mut app, "// una vez\n");

    pulsar_tecla(&context, &mut app, eframe::egui::Key::S, true);
    assert!(
        app.peticiones() == [Command::Save],
        "el atajo tiene que pedir el guardado y solo eso: {:?}",
        app.peticiones()
    );
    app.avanzar(&context);
    let tras_el_atajo =
        std::fs::read_to_string(raiz.join("Form1.cs")).expect("el archivo se puede leer");

    // El segundo guardado va por el comando, que es lo que emiten el menú y el botón: se pide
    // igual que si lo hubiera pulsado el usuario.
    app.emitir(Command::Save);
    app.avanzar(&context);
    let tras_el_comando =
        std::fs::read_to_string(raiz.join("Form1.cs")).expect("el archivo se puede leer");

    assert_eq!(
        tras_el_atajo, tras_el_comando,
        "los dos caminos escriben lo mismo porque los dos llegan al mismo core"
    );
    assert!(
        !app.state().pestanas()[0].esta_modificada(),
        "guardar dos veces no deja el documento modificado: {:?}",
        app.state().pestanas()
    );

    let _ = std::fs::remove_dir_all(&raiz);
}

/// La ventana sigue dibujando y escribiendo mientras compila. FE-074.
///
/// "Sin bloqueo de UI" es la mitad de FE-074, y es la mitad que los estados no demuestran: que
/// los estados cambien bien no dice que la ventana no se quedara esperando mientras tanto.
/// Aquí se mide eso —varios frames seguidos mientras el core todavía no ha contestado, y
/// escribir en el documento que se está editando—, que es lo que un IDE tiene que poder hacer
/// mientras compila.
#[test]
fn la_ventana_se_sigue_dibujando_mientras_compila() {
    let context = eframe::egui::Context::default();
    let (mut app, raiz) = miniide_con_un_proyecto(
        "miniide-fe074-sin-bloqueo",
        Arc::new(HerramientaDePrueba::que_tarda(Duration::from_millis(600))),
    );

    pulsar(&context, &mut app, "Form1.cs");
    app.avanzar(&context);
    escribir(&context, &mut app, "// mientras compila\n");

    pulsar_el(&context, &mut app, "Compilar", 1);
    app.avanzar(&context);

    for vuelta in 0..5 {
        let salida = dibujar(&context, &mut app, &[]);
        assert!(
            se_ve(&salida, "Compilación: Compilando"),
            "el frame {vuelta} tiene que verse mientras compila: {:?}",
            pintado(&salida)
        );
    }

    escribir(&context, &mut app, "// y se sigue escribiendo\n");
    assert!(
        app.state().pestanas()[0].esta_modificada(),
        "escribir mientras compila tiene que seguir funcionando: {:?}",
        app.state().pestanas()
    );

    esperar_a_que_deje_de_decir(&context, &mut app, "Compilación: Compilando");
    assert!(
        !dibujar(&context, &mut app, &[]).shapes.is_empty(),
        "la ventana se sigue dibujando después de compilar"
    );

    let _ = std::fs::remove_dir_all(&raiz);
}

/// Compilar y ejecutar desde la barra pasan por el core y la ventana cuenta lo que pasa.
/// FE-074.
///
/// Es el camino que no se puede fingir: la ventana pide compilar con su botón, el core lanza el
/// proceso, recoge su código de salida y la ventana se entera. Se mira lo que ve el usuario —
/// que compila, que compila bien, que ejecuta y que se para— y no el estado interno, porque un
/// estado correcto que no se enseña no sirve de nada.
///
/// Se ignora porque el core lanza procesos de verdad, y no hay forma de comprobar el camino
/// entero sin lanzarlos.
#[test]
#[ignore = "lanza procesos de verdad"]
fn compilar_y_ejecutar_desde_la_barra_pasan_por_el_core() {
    let context = eframe::egui::Context::default();
    let (mut app, raiz) = miniide_con_un_proyecto(
        "miniide-fe074-compilar-y-ejecutar",
        Arc::new(HerramientaDePrueba::que_ejecuta()),
    );

    let inactiva = dibujar(&context, &mut app, &[]);
    assert!(
        se_ve(&inactiva, "Compilación: Inactivo"),
        "una ventana recién abierta no está compilando: {:?}",
        pintado(&inactiva)
    );

    pulsar_el(&context, &mut app, "Compilar", 1);
    app.avanzar(&context);
    let compilando = dibujar(&context, &mut app, &[]);
    assert!(
        se_ve(&compilando, "Compilación: Compilando"),
        "pulsar Compilar tiene que ponerse a compilar: {:?}",
        pintado(&compilando)
    );

    esperar_a_que_deje_de_decir(&context, &mut app, "Compilación: Compilando");
    let compilada = dibujar(&context, &mut app, &[]);
    assert!(
        se_ve(&compilada, "Compilación: Correcta"),
        "una compilación que ha salido bien se dice: {:?}",
        pintado(&compilada)
    );

    pulsar(&context, &mut app, "Ejecutar");
    app.avanzar(&context);
    let ejecutando = dibujar(&context, &mut app, &[]);
    assert!(
        se_ve(&ejecutando, "Ejecución: Ejecutando"),
        "pulsar Ejecutar tiene que poner algo en marcha: {:?}",
        pintado(&ejecutando)
    );

    pulsar(&context, &mut app, "Detener");
    app.avanzar(&context);
    esperar_a_que_diga(&context, &mut app, "Ejecución: Terminado");
    let parado = dibujar(&context, &mut app, &[]);
    assert!(
        se_ve(&parado, "Ejecución: Terminado"),
        "parar tiene que terminar el proceso: {:?}",
        pintado(&parado)
    );

    let _ = std::fs::remove_dir_all(&raiz);
}

/// El flujo principal de MiniIDE se puede repetir entero y sin fallos. FE-075.
///
/// Abrir un proyecto, abrir un archivo, escribir, guardar, compilar, ejecutar y parar, todo por
/// la ventana y todo como lo haría el usuario. Es un test de humo y por eso no comprueba cómo
/// se ve cada paso —de eso se encargan los de FE-057 a FE-072—, sino que el camino entero se
/// puede recorrer y acaba como se espera: el archivo del disco con lo escrito, la compilación
/// correcta, el proceso parado y la ventana en pie.
///
/// Se ignora porque compilar y ejecutar lanzan procesos de verdad, que es justo lo que hay que
/// comprobar: un flujo que no lanza nada no es el flujo.
#[test]
#[ignore = "lanza procesos de verdad"]
fn el_flujo_principal_se_puede_repetir_de_principio_a_fin() {
    let context = eframe::egui::Context::default();
    let (mut app, raiz) = miniide_con_un_proyecto(
        "miniide-fe075-flujo-principal",
        Arc::new(HerramientaDePrueba::que_ejecuta()),
    );

    assert!(
        !app.archivos().is_empty(),
        "un proyecto abierto tiene archivos que enseñar"
    );

    pulsar(&context, &mut app, "Form1.cs");
    app.avanzar(&context);
    assert_eq!(app.state().pestana_activa(), Some("Form1.cs"));
    assert!(
        se_ve(&dibujar(&context, &mut app, &[]), "InitializeComponent"),
        "y su contenido está en el editor"
    );

    escribir(&context, &mut app, "// del flujo principal\n");
    assert!(app.state().pestanas()[0].esta_modificada());

    pulsar_tecla(&context, &mut app, eframe::egui::Key::S, true);
    app.avanzar(&context);
    let en_disco =
        std::fs::read_to_string(raiz.join("Form1.cs")).expect("el archivo se puede leer");
    assert!(
        en_disco.starts_with("// del flujo principal"),
        "el archivo del disco tiene lo escrito: {en_disco}"
    );

    pulsar_el(&context, &mut app, "Compilar", 1);
    app.avanzar(&context);
    esperar_a_que_deje_de_decir(&context, &mut app, "Compilación: Compilando");
    assert!(
        se_ve(&dibujar(&context, &mut app, &[]), "Compilación: Correcta"),
        "una compilación que ha salido bien deja la ventana contenta"
    );

    pulsar(&context, &mut app, "Ejecutar");
    app.avanzar(&context);
    assert!(
        se_ve(&dibujar(&context, &mut app, &[]), "Ejecución: Ejecutando"),
        "ejecutar pone algo en marcha"
    );

    pulsar(&context, &mut app, "Detener");
    app.avanzar(&context);
    esperar_a_que_diga(&context, &mut app, "Ejecución: Terminado");

    assert_eq!(
        app.state().pestana_activa(),
        Some("Form1.cs"),
        "el archivo sigue abierto al terminar"
    );
    assert!(
        !app.state().pestanas()[0].esta_modificada(),
        "y sin nada sin guardar: {:?}",
        app.state().pestanas()
    );
    assert!(
        !dibujar(&context, &mut app, &[]).shapes.is_empty(),
        "la ventana se sigue dibujando"
    );

    let _ = std::fs::remove_dir_all(&raiz);
}
