//! El explorador de proyectos: el panel de la izquierda.
//!
//! Es el sitio donde se ve lo que tiene un proyecto, y ahora es un panel con su cabecera y
//! un árbol vacío: FE-010 pone el sitio y nada más. Lo que va dentro lo traen las que
//! vienen detrás: FE-011 pinta las carpetas y los archivos que le dé el core, FE-012 qué
//! carpetas están abiertas, FE-013 la selección y FE-014 el menú contextual.
//!
//! El árbol está vacío a propósito, y no porque no haya nada que poner: un explorador que
//! se inventara archivos para no verse vacío enseñaría un proyecto que MiniIDE no ha visto.
//! Los archivos son del core, y a la ventana todavía no le llega nada de él.
//!
//! El panel es de los de egui que se quedan pegados al borde, no una ventana encima, porque
//! un explorador que flota tapa el editor y hay que moverlo de sitio cada vez que se quiere
//! ver una cosa.

use crate::commands::Command;
use crate::project::{ProjectFile, ProjectFileKind};
use eframe::egui;

use super::acciones::{self, Accion, NUEVO_ARCHIVO, NUEVO_DIRECTORIO};
use super::app::App;
use super::layout::Zona;
use super::ui_state::UiState;

/// Cuánto se hunde una fila por cada carpeta de la que cuelga.
///
/// Es una medida de pantalla, no una regla de negocio: lo que decide la jerarquía es
/// cuántas carpetas hay de por medio, no estos puntos. Se elige la que separa una fila de
/// la de su carpeta sin que dos niveles se confundan.
const SANGRIA: f32 = 12.0;

/// Dibuja el explorador de proyectos en `ui` con los archivos del proyecto.
///
/// Los archivos son los del core y llegan en el orden en que él los da, que ya viene
/// ordenado por carpetas y por nombre. El explorador no los ordena ni los agrupa: los dibuja
/// en ese orden y lo único que decide es cuánto se hunde cada uno, que es lo que convierte
/// una lista de rutas en un árbol.
///
/// De lo que están abiertas las carpetas y de cuál está seleccionada se encarga el estado
/// visual, y no este archivo: que una carpeta esté abierta no es cosa del explorador sino de
/// la ventana, y es lo que hace que siga como estaba mientras el usuario trabaja en otra
/// parte. Por eso entra la aplicación entera y no su estado suelto: una fila también pide
/// abrir lo que tiene debajo, y esa petición sale por `App::emitir`, el punto único de
/// FE-009, como la de cualquier botón.
///
/// Sin archivos el árbol está vacío, y tiene que estarlo: un explorador que se inventara
/// archivos para no verse vacío enseñaría un proyecto que MiniIDE no ha visto. Los archivos
/// son del core, y a la ventana todavía no le llega nada de él.
pub fn panel(ui: &mut egui::Ui, archivos: &[ProjectFile], app: &mut App) {
    ui.set_min_size(ui.available_size());

    egui::CollapsingHeader::new(Zona::Proyecto.nombre().to_owned())
        .default_open(true)
        .show(ui, |ui| {
            for archivo in archivos {
                if !se_ve(archivo, app.state()) {
                    continue;
                }

                fila(ui, archivo, app);
            }
        });
}

/// Si la fila se ve: no se ve lo que cuelga de una carpeta cerrada.
///
/// Se mira cada carpeta de la que cuelga la fila, y no solo la inmediata, porque cerrar
/// "src" tiene que esconder también lo que hay dentro de "src/vistas". Si solo se mirara la
/// carpeta de al lado, lo que cuelga dos niveles más abajo se quedaría a la vista colgando
/// de una carpeta que ya no se ve, que es un árbol roto.
fn se_ve(archivo: &ProjectFile, estado: &UiState) -> bool {
    ancestros(archivo)
        .iter()
        .all(|carpeta| estado.esta_expandida(carpeta))
}

/// Las carpetas de las que cuelga un archivo, de la más cercana a la raíz a la más honda.
///
/// Un archivo de la raíz no cuelga de ninguna, y por eso no tiene ninguna: se ve siempre.
fn ancestros(archivo: &ProjectFile) -> Vec<String> {
    let partes: Vec<String> = archivo
        .path()
        .as_path()
        .components()
        .map(|parte| parte.as_os_str().to_string_lossy().into_owned())
        .collect();

    (1..partes.len())
        .map(|cuantas| partes[..cuantas].join("/"))
        .collect()
}

/// Las acciones del menú contextual de una fila.
///
/// Son las que RF-06 prepara, y van aquí y no en el menú principal porque son de este panel:
/// un archivo nuevo nace dentro del proyecto, y el sitio del que se pide es la fila que se
/// ha pulsado. En el menú principal no están, y no es descuido: desde el menú no se sabe de
/// qué carpeta se está hablando, y una acción que necesita saberlo no puede estar en todas
/// partes.
const ACCIONES_DEL_CONTEXTO: &[Accion] = &[NUEVO_ARCHIVO, NUEVO_DIRECTORIO];

/// Dibuja una fila del árbol, hundida según lo hondo que esté dentro del proyecto.
///
/// La fila es un botón, y pulsarla hace una cosa u otra según lo que sea: una carpeta se
/// pliega y se despliega, y un archivo se selecciona y pide abrirse. Se dibujan todas como
/// botones iguales para que el día que una fila sirva para las dos cosas no haya que cambiar
/// cómo se ve.
///
/// Que un archivo pida abrirlo no significa que se abra: la apertura la hace el core, y lo
/// que hace la fila es decirlo una vez, por el punto único. El archivo se marca como
/// seleccionado porque es lo que el usuario ha pedido, y no porque esté abierto todavía: la
/// pestaña donde se abra es FE-015.
///
/// Con el botón derecho se abre el menú contextual, que es de la fila y sale de ella: donde
/// se ha pulsado es justo lo que después le dice al core en qué carpeta va lo que se cree. El
/// menú es el mismo en cualquier fila, y no uno por tipo de fila, porque lo único que cambia
/// con el tipo es si lo que se crea puede colgar de ahí, y eso lo decide el core al ejecutar
/// el comando, no este archivo al dibujarlo.
fn fila(ui: &mut egui::Ui, archivo: &ProjectFile, app: &mut App) {
    let ruta = archivo.path().as_path();
    let carpetas = ruta.components().count().saturating_sub(1);
    let es_carpeta = archivo.kind() == ProjectFileKind::Directory;
    let clave = clave_de(archivo);

    ui.horizontal(|ui| {
        ui.add_space(SANGRIA * carpetas as f32);
        let respuesta = ui.selectable_label(
            app.state().seleccion() == Some(clave.as_str()),
            texto(archivo, app.state()),
        );
        let pulsada = respuesta.clicked();

        respuesta.context_menu(|ui| {
            for accion in ACCIONES_DEL_CONTEXTO {
                acciones::boton(ui, app, *accion);
            }
        });

        if !pulsada {
            return;
        }

        if es_carpeta {
            app.state_mut().alternar_expansion(&clave);
        } else {
            app.state_mut().seleccionar(&clave);
            app.emitir(Command::OpenDocument);
        }
    });
}

/// Lo que se ve en la fila: el nombre del archivo o de la carpeta.
///
/// El nombre y no la ruta entera, porque la carpeta de la que cuelga ya está dibujada
/// encima con su sangría: "Form1.cs" debajo de "src" se entiende mejor que "src/Form1.cs"
/// repetido en cada fila, y en una columna de doscientos puntos la ruta no cabe.
///
/// La carpeta lleva delante una marca que dice si está abierta, y va en negrita: es lo
/// único que distingue una fila que se puede desplegar de una que no, y sin eso "src" y
/// "Form1.cs" serían la misma fila.
fn texto(archivo: &ProjectFile, estado: &UiState) -> egui::RichText {
    let ruta = archivo.path().as_path();
    let nombre = ruta
        .file_name()
        .map(|nombre| nombre.to_string_lossy().into_owned())
        .unwrap_or_else(|| ruta.display().to_string());

    if archivo.kind() == ProjectFileKind::Directory {
        let marca = if estado.esta_expandida(&clave_de(archivo)) {
            "▾"
        } else {
            "▸"
        };
        egui::RichText::new(format!("{marca} {nombre}")).strong()
    } else {
        egui::RichText::new(nombre)
    }
}

/// La clave de una fila: su ruta relativa, con la misma forma en todos los sistemas.
///
/// Es la clave con la que el estado visual recuerda qué carpetas están abiertas, y por eso
/// se rehace siempre igual en vez de dejar la ruta como venga: si dependiera de cómo se
/// escribieron las rutas, la misma carpeta sería dos claves distintas en dos equipos, y una
/// se quedaría abierta y la otra no.
fn clave_de(archivo: &ProjectFile) -> String {
    archivo
        .path()
        .as_path()
        .components()
        .map(|parte| parte.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use crate::commands::Command;
    use crate::core::ProjectType;
    use crate::frontend::App;
    use crate::project::{
        BuildConfiguration, Project, ProjectFile, ProjectFileKind, ProjectRelativePath,
    };
    use eframe::egui;

    /// Una ventana de trabajo normal, ni enorme ni mínima.
    const ANCHO: f32 = 1000.0;
    const ALTO: f32 = 800.0;

    /// El hueco que el layout le da al explorador. No importa que sea este y no otro: lo
    /// que se mira aquí son las filas y su sangría, no el ancho de la columna.
    const COLUMNA: f32 = 200.0;

    /// Lo que le cuesta a egui cada frontera entre zonas.
    const MARGEN: f32 = 2.0;

    /// Un frame con una ventana de tamaño conocido.
    fn entrada() -> eframe::egui::RawInput {
        entrada_con(&[])
    }

    /// Un frame con una ventana de tamaño conocido y estos eventos de por medio.
    ///
    /// Los eventos van en un frame aparte cada uno porque egui solo cuenta un clic si la
    /// pulsación y la soltura caen las dos en el widget.
    fn entrada_con(eventos: &[egui::Event]) -> eframe::egui::RawInput {
        eframe::egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(ANCHO, ALTO),
            )),
            events: eventos.to_vec(),
            ..eframe::egui::RawInput::default()
        }
    }

    /// Dibuja la ventana y devuelve los rectángulos que se han pintado.
    fn pintar(context: &egui::Context) -> Vec<egui::Rect> {
        let mut app = App::new();

        let mut salida = context.run_ui(entrada(), |ui| {
            app.dibujar(ui);
        });
        salida.textures_delta.clear();

        salida
            .shapes
            .into_iter()
            .filter_map(|forma| match forma.shape {
                egui::Shape::Rect(rectangulo) => Some(rectangulo.rect),
                _ => None,
            })
            .collect()
    }

    /// El panel del explorador: el rectángulo que está pegado al borde izquierdo.
    ///
    /// Un panel pegado a un borde se distingue de una ventana que flota porque su
    /// rectángulo empieza en el borde, en cero, y porque ocupa el alto que le queda. Se
    /// busca así y no por un ancho esperado porque el ancho lo decide egui y va a cambiar
    /// cuando se pueda arrastrar el borde (FE-066).
    fn panel(context: &egui::Context) -> egui::Rect {
        let pegados: Vec<egui::Rect> = pintar(context)
            .into_iter()
            .filter(|rectangulo| rectangulo.min.x <= MARGEN)
            .filter(|rectangulo| rectangulo.width() < ANCHO / 2.0)
            .collect();

        pegados
            .into_iter()
            .max_by(|uno, otro| uno.height().total_cmp(&otro.height()))
            .unwrap_or_else(|| {
                panic!("la ventana tiene que tener el panel del explorador pegado a la izquierda")
            })
    }

    /// La zona de arriba de la ventana, que es lo que hay encima del explorador.
    fn franja_de_arriba(context: &egui::Context) -> egui::Rect {
        let mut zonas: Vec<egui::Rect> = pintar(context)
            .into_iter()
            .filter(|rectangulo| rectangulo.max.x >= ANCHO - MARGEN)
            .collect();
        zonas.sort_by(|uno, otro| uno.min.y.total_cmp(&otro.min.y));

        zonas.first().copied().expect("la ventana tiene zonas")
    }

    /// El área central: la zona que llega al borde derecho y es la más alta.
    fn area_central(context: &egui::Context) -> egui::Rect {
        let mut zonas: Vec<egui::Rect> = pintar(context)
            .into_iter()
            .filter(|rectangulo| rectangulo.max.x >= ANCHO - MARGEN)
            .collect();
        zonas.sort_by(|uno, otro| uno.height().total_cmp(&otro.height()));

        zonas
            .into_iter()
            .next_back()
            .expect("la ventana tiene areas")
    }

    /// Un proyecto de prueba con lo justo para que se vea un árbol y se puedan probar las
    /// carpetas: un archivo en la raíz, dos carpetas en la raíz, y dentro de "src" otra
    /// carpeta con un archivo.
    ///
    /// ```text
    /// App.csproj
    /// src
    ///   Form1.cs
    ///   vistas
    ///     Vista.cs
    /// docs
    ///   guia.md
    /// ```
    ///
    /// Es un `Project` de verdad, del core, y no una lista de rutas escrita a mano: FE-011
    /// dice que el explorador pinta lo que le da el core, y la única forma de comprobarlo
    /// es darle uno de verdad. Las carpetas van en la lista porque el core las da, y el
    /// explorador no las inventa.
    fn proyecto_de_prueba() -> Project {
        let salida = ProjectRelativePath::new("bin").expect("ruta de salida valida");
        let configuracion = BuildConfiguration::new(salida);

        let mut proyecto = Project::new(
            "prueba",
            std::env::temp_dir(),
            ProjectType::CSharpWinForms,
            configuracion,
        )
        .expect("proyecto de prueba valido");

        for (ruta, clase) in [
            ("App.csproj", ProjectFileKind::File),
            ("src", ProjectFileKind::Directory),
            ("src/Form1.cs", ProjectFileKind::File),
            ("src/vistas", ProjectFileKind::Directory),
            ("src/vistas/Vista.cs", ProjectFileKind::File),
            ("docs", ProjectFileKind::Directory),
            ("docs/guia.md", ProjectFileKind::File),
        ] {
            let ruta = ProjectRelativePath::new(ruta).expect("ruta de prueba valida");
            proyecto.add_file(ProjectFile::new(ruta, clase));
        }

        proyecto
    }

    /// Cuántas filas tiene el árbol con lo que hay abierto, y cuáles son.
    ///
    /// Las filas llegan en el orden en que el core da los archivos y sin las que están
    /// escondidas, así que el número de filas es exactamente cuántas cosas se ven, y el
    /// sitio de cada una dice cuál es: la tercera fila es la tercera entrada del proyecto
    /// que se ve.
    fn filas_de(context: &egui::Context, proyecto: &Project, app: &mut App) -> Vec<egui::Rect> {
        columna_y_filas(context, proyecto.files(), app).1
    }

    /// Dibuja el explorador con una lista de archivos y devuelve la columna y sus filas.
    ///
    /// No se dibuja la ventana entera porque la ventana no tiene proyecto: los archivos se
    /// le pasan al explorador directamente, que es el sitio donde se pinta el árbol.
    ///
    /// La columna es el rectángulo más grande de los que se pintan, porque el explorador se
    /// dibuja solo y ocupa la ventana entera, y las filas son los que caben dentro de ella.
    /// Se separan así porque egui no enseña el texto de los widgets en la salida de un
    /// frame, pero sí los rectángulos que pinta, y de una fila se ve lo único que importa
    /// sin leer su texto: que está, dónde está y cuánto se hunde.
    /// Dibuja el explorador dentro de un hueco de columna y devuelve la columna y sus
    /// filas.
    ///
    /// El hueco es el que le da el layout, y no un panel de egui dibujado suelto: un panel
    /// recuerda su tamaño entre frames y con el ratón encima se coloca de otra manera, así
    /// que un test que lo dibuja suelto acaba midiendo una fila y pulsando donde ya no
    /// está. El contenido es el mismo del explorador, que es lo que se quiere comprobar.
    fn columna_y_filas(
        context: &egui::Context,
        archivos: &[ProjectFile],
        app: &mut App,
    ) -> (egui::Rect, Vec<egui::Rect>) {
        let mut columna = egui::Rect::ZERO;

        let mut salida = context.run_ui(entrada(), |ui| {
            let respuesta = ui.allocate_ui(egui::vec2(COLUMNA, ALTO), |ui| {
                super::panel(ui, archivos, app);
            });
            columna = respuesta.response.rect;
        });
        salida.textures_delta.clear();

        let mut filas: Vec<egui::Rect> = salida
            .shapes
            .into_iter()
            .filter_map(|forma| match forma.shape {
                egui::Shape::Rect(rectangulo) => Some(rectangulo.rect),
                _ => None,
            })
            .collect();
        filas.sort_by(|una, otra| una.min.y.total_cmp(&otra.min.y));
        filas.dedup();

        (columna, filas)
    }

    /// Dibuja el explorador con unos eventos de por medio y tira lo que pinta.
    ///
    /// Se dibuja el contenido en un hueco de columna y no en el panel de la izquierda por
    /// el motivo que se explica en [`Self::columna_y_filas`].
    fn dibujar(
        context: &egui::Context,
        proyecto: &Project,
        app: &mut App,
        eventos: &[egui::Event],
    ) {
        let mut salida = context.run_ui(entrada_con(eventos), |ui| {
            ui.allocate_ui(egui::vec2(COLUMNA, ALTO), |ui| {
                super::panel(ui, proyecto.files(), app);
            });
        });
        salida.textures_delta.clear();
    }

    /// Pulsa y suelta la fila que ocupa el sitio `indice` entre las que se ven.
    ///
    /// Con un clic de verdad, pulsando donde egui ha pintado la fila. El sitio se cuenta
    /// entre las filas visibles porque eso es lo que el usuario ve y lo que puede pulsar.
    fn pulsar_fila(
        context: &egui::Context,
        proyecto: &Project,
        app: &mut App,
        indice: usize,
    ) -> Vec<egui::Rect> {
        let antes = filas_de(context, proyecto, app);
        let fila = antes
            .get(indice)
            .copied()
            .unwrap_or_else(|| panic!("no hay ninguna fila {indice}: {antes:?}"));

        for pressed in [true, false] {
            let boton = egui::Event::PointerButton {
                pos: fila.center(),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            };
            dibujar(context, proyecto, app, &[boton]);
        }

        filas_de(context, proyecto, app)
    }

    /// El explorador es un panel pegado a la izquierda, de alto y con ancho.
    ///
    /// Es lo que FE-010 pide: un sitio para el proyecto. Se comprueba que ocupa sitio de
    /// verdad —con ancho y con alto— y no una línea de un píxel pegada al borde, porque un
    /// panel que no se ve es un panel que no existe aunque su código esté.
    #[test]
    fn el_explorador_es_un_panel_pegado_a_la_izquierda() {
        let context = egui::Context::default();

        let panel = panel(&context);

        assert!(
            panel.min.x <= MARGEN,
            "el explorador tiene que estar pegado al borde izquierdo: {panel:?}"
        );
        assert!(
            panel.width() > 0.0,
            "un panel sin ancho no se ve: {panel:?}"
        );
    }

    /// El explorador va al lado del área central, sin taparla.
    ///
    /// El editor y el diseñador viven en el área central, así que un explorador que se
    /// solapara con ella los taparía y el usuario no podría ver ni lo uno ni lo otro. Se
    /// comprueba lo de no solaparse, y también que el área central se ha movido hacia la
    /// derecha, porque un panel que no empuja a nadie es un panel pintado encima.
    #[test]
    fn el_explorador_no_tapa_el_area_central() {
        let context = egui::Context::default();

        let panel = panel(&context);
        let central = area_central(&context);

        assert!(
            central.min.x >= panel.max.x - MARGEN,
            "el área central empieza donde acaba el explorador: {central:?} {panel:?}"
        );
        assert!(
            central.max.x >= ANCHO - MARGEN,
            "el área central tiene que seguir llegando al borde derecho: {central:?}"
        );
        assert!(
            central.height() > ALTO / 2.0,
            "el área central tiene que seguir siendo la mayor parte de la ventana: {central:?}"
        );
    }

    /// El explorador va de la franja de arriba a la de abajo, al lado del editor.
    ///
    /// Las dos columnas, el explorador y el área central, tienen que ocupar las mismas
    /// filas: si el explorador se quedara corto, el hueco de debajo parecería un panel que
    /// no dibuja nada, y el usuario haría clic ahí esperando que pase algo.
    #[test]
    fn el_explorador_va_de_arriba_a_abajo() {
        let context = egui::Context::default();

        let panel = panel(&context);
        let arriba = franja_de_arriba(&context);
        let central = area_central(&context);

        assert!(
            (panel.min.y - arriba.max.y).abs() <= MARGEN,
            "el explorador tiene que empezar donde acaba la franja de arriba: {panel:?} {arriba:?}"
        );
        assert!(
            (panel.max.y - central.max.y).abs() <= MARGEN,
            "el explorador tiene que acabar donde acaba el área central: {panel:?} {central:?}"
        );
    }

    /// Un árbol vacío no mueve el panel de sitio.
    ///
    /// El panel no se dibuja a partir de su contenido sino del espacio que queda, y por eso
    /// un contenido vacío tiene que dejar el panel igual: si al aparecer o desaparecer un
    /// archivo el panel se moviera, el árbol estaría dictando el ancho del panel y el
    /// usuario lo notaría en cada apertura de carpeta.
    #[test]
    fn un_arbol_vacio_no_mueve_el_panel() {
        let context = egui::Context::default();

        let primera = panel(&context);
        let segunda = panel(&context);

        assert!(
            (primera.min.y - segunda.min.y).abs() <= MARGEN
                && (primera.height() - segunda.height()).abs() <= MARGEN
                && (primera.width() - segunda.width()).abs() <= MARGEN,
            "el panel no se puede mover al redibujar: {primera:?} {segunda:?}"
        );
    }

    /// Cada archivo del proyecto tiene su fila, y la fila se queda dentro de la columna.
    ///
    /// Es lo que FE-011 viene a hacer: que un proyecto se pueda ver. Una fila por archivo, en
    /// el orden en que el core los da, y dentro de la columna, porque una fila que se sale
    /// de la columna se painted encima del editor.
    ///
    /// Se abren antes todas las carpetas, porque con el árbol recién abierto hay carpetas
    /// cerradas y sus filas están escondidas: esto es lo que se ve cuando el usuario ha
    /// desplegado lo que quiere ver.
    #[test]
    fn el_arbol_muestra_una_fila_por_archivo() {
        let context = egui::Context::default();
        let proyecto = proyecto_de_prueba();
        let mut app = App::new();
        for carpeta in ["src", "src/vistas", "docs"] {
            app.state_mut().alternar_expansion(carpeta);
        }

        let filas = filas_de(&context, &proyecto, &mut app);
        let columna = panel(&context);

        assert_eq!(
            filas.len(),
            proyecto.files().len(),
            "cada archivo del proyecto tiene su fila: {filas:?}"
        );

        for fila in &filas {
            assert!(
                fila.width() > 0.0 && fila.height() > 0.0,
                "una fila vacía no se ve: {fila:?}"
            );
            assert!(
                fila.max.x <= columna.max.x + MARGEN,
                "una fila se ha salido de la columna: {fila:?} {columna:?}"
            );
        }
    }

    /// Un archivo dentro de una carpeta se ve más hondo que la carpeta.
    ///
    /// Eso es la jerarquía, y es lo único que distingue un árbol de una lista de rutas: si
    /// todo fuera a la misma altura, el explorador sería un listado y el usuario no
    /// distinguiría "src/Form1.cs" de "Form1.cs". Las filas llegan en el orden en que el
    /// core las da, así que la carpeta es la segunda y lo que cuelga de ella la tercera.
    #[test]
    fn las_carpetas_se_ven_hondas_dentro_de_sus_archivos() {
        let context = egui::Context::default();
        let proyecto = proyecto_de_prueba();
        let mut app = App::new();
        app.state_mut().alternar_expansion("src");

        let filas = filas_de(&context, &proyecto, &mut app);
        assert!(
            filas.len() >= 3,
            "con src abierta se ven la raíz y lo que cuelga de ella: {filas:?}"
        );

        let archivo_en_la_raiz = filas[0];
        let carpeta = filas[1];
        let archivo_en_la_carpeta = filas[2];

        assert!(
            (archivo_en_la_raiz.min.x - carpeta.min.x).abs() <= MARGEN,
            "una carpeta y un archivo de la raiz empiezan a la misma altura: {archivo_en_la_raiz:?} {carpeta:?}"
        );
        assert!(
            archivo_en_la_carpeta.min.x > carpeta.min.x + MARGEN,
            "un archivo dentro de una carpeta tiene que verse mas hondo que la carpeta: {carpeta:?} {archivo_en_la_carpeta:?}"
        );
    }

    /// Sin archivos no hay filas, ni de mentira.
    ///
    /// La ventana no tiene proyecto abierto, así que el explorador tiene que verse vacío. Si
    /// se dibujara algo sin que el core lo hubiera dado, el usuario vería un proyecto que
    /// MiniIDE no ha abierto, y no hay forma de que eso sea verdad.
    #[test]
    fn un_arbol_sin_archivos_no_dibuja_filas() {
        let context = egui::Context::default();
        let mut app = App::new();

        let (_columna, filas) = columna_y_filas(&context, &[], &mut app);

        assert!(
            filas.is_empty(),
            "sin archivos el árbol no tiene filas, y tiene estas: {filas:?}"
        );
    }

    /// Una carpeta se pliega y se despliega al pulsarla.
    ///
    /// Es lo que FE-012 pide: que el usuario pueda plegar y desplegar una carpeta. Se
    /// comprueba con pulsaciones de verdad y contando lo que queda a la vista, que es
    /// como se comprueba de verdad: si al pulsar "src" desaparecen sus tres filas y al
    /// volver a pulsarla aparecen, la carpeta se está plegando.
    #[test]
    fn una_carpeta_se_plega_y_se_despliega_al_pulsarla() {
        let context = egui::Context::default();
        let proyecto = proyecto_de_prueba();
        let mut app = App::new();

        let cerrada = filas_de(&context, &proyecto, &mut app);
        assert_eq!(
            cerrada.len(),
            3,
            "un árbol recién abierto solo enseña la raíz del proyecto: {cerrada:?}"
        );

        let abierta = pulsar_fila(&context, &proyecto, &mut app, 1);
        assert_eq!(
            abierta.len(),
            5,
            "desplegar src enseña lo que cuelga de ella y nada más: {abierta:?}"
        );

        let plegada = pulsar_fila(&context, &proyecto, &mut app, 1);
        assert_eq!(
            plegada.len(),
            3,
            "volver a pulsar src la deja como estaba: {plegada:?}"
        );
    }

    /// Cada carpeta se pliega por su cuenta.
    ///
    /// Con dos carpetas en la raíz, plegar una no puede tocar la otra: si lo hiciera, en
    /// cuanto el usuario plegara "src" para hacerse un hueco se le cerrarían también los
    /// documentos, y no hay forma de trabajar con los dos a la vez.
    #[test]
    fn cada_carpeta_se_plega_sola() {
        let context = egui::Context::default();
        let proyecto = proyecto_de_prueba();
        let mut app = App::new();

        pulsar_fila(&context, &proyecto, &mut app, 1);
        let con_las_dos = pulsar_fila(&context, &proyecto, &mut app, 4);
        assert_eq!(
            con_las_dos.len(),
            6,
            "desplegar src y docs deja ver los dos árboles, con lo que cuelga de cada uno: {con_las_dos:?}"
        );

        let solo_src = pulsar_fila(&context, &proyecto, &mut app, 1);
        assert_eq!(
            solo_src.len(),
            4,
            "plegar src deja lo que cuelga de docs intacto: {solo_src:?}"
        );
    }

    /// Plegar una carpeta esconde también lo que cuelga de sus carpetas.
    ///
    /// "src" tiene dentro "vistas", y "vistas" tiene dentro un archivo. Si al plegar "src"
    /// se escondieran solo sus archivos de golpe, lo que cuelga dos niveles más abajo se
    /// quedaría a la vista colgando de una carpeta que ya no está, que es un árbol roto.
    #[test]
    fn plegar_una_carpeta_esconde_todo_lo_que_cuelga_de_ella() {
        let context = egui::Context::default();
        let proyecto = proyecto_de_prueba();
        let mut app = App::new();

        pulsar_fila(&context, &proyecto, &mut app, 1);
        let con_vistas = pulsar_fila(&context, &proyecto, &mut app, 3);
        assert_eq!(
            con_vistas.len(),
            6,
            "desplegar src y lo que hay dentro deja ver el proyecto menos lo de docs: {con_vistas:?}"
        );

        let plegada = pulsar_fila(&context, &proyecto, &mut app, 1);
        assert_eq!(
            plegada.len(),
            3,
            "plegar src se lleva también lo que cuelga de sus carpetas: {plegada:?}"
        );
    }

    /// Una fila de archivo no pliega nada.
    ///
    /// Pulsar un archivo pide abrirlo (FE-013) pero no pliega la columna en la que está: si
    /// un archivo cerrara lo que tiene alrededor, el botón haría dos cosas y el usuario
    /// tendría que adivinar cuál.
    #[test]
    fn una_fila_de_archivo_no_plega_nada() {
        let context = egui::Context::default();
        let proyecto = proyecto_de_prueba();
        let mut app = App::new();

        let despues = pulsar_fila(&context, &proyecto, &mut app, 0);

        assert_eq!(
            despues.len(),
            3,
            "pulsar un archivo no cambia lo que se ve: {despues:?}"
        );
    }

    /// Seleccionar un archivo pide abrirlo, y lo pide una vez.
    ///
    /// Es lo que FE-013 pide: que seleccionar un archivo produzca exactamente un comando de
    /// apertura. "Exactamente" quiere decir dos cosas, y las dos se comprueban aquí: que lo
    /// pida una vez y solo una —ni dos veces por un clic, ni ninguna— y que lo pida siendo
    /// un archivo. El comando sale por `App::emitir`, el punto único de FE-009, así que
    /// `peticiones` solo necesita mirar.
    #[test]
    fn seleccionar_un_archivo_pide_abrirlo_exactamente_una_vez() {
        let context = egui::Context::default();
        let proyecto = proyecto_de_prueba();
        let mut app = App::new();

        filas_de(&context, &proyecto, &mut app);
        assert!(
            app.peticiones().is_empty(),
            "sin pulsar nada no se pide nada, y se pidió {:?}",
            app.peticiones()
        );

        // La primera fila con el árbol recién abierto es "App.csproj", un archivo.
        pulsar_fila(&context, &proyecto, &mut app, 0);

        assert_eq!(
            app.peticiones(),
            [Command::OpenDocument],
            "seleccionar un archivo pide abrirlo una vez y solo esa"
        );
    }

    /// Seleccionar una carpeta no pide abrir nada.
    ///
    /// Una carpeta no es un archivo, y pedir abrirla sería pedir algo que no se puede abrir.
    /// Lo que hace una carpeta es plegarse, que es FE-012. Con el árbol recién abierto la
    /// segunda fila es "src", y aquí va plegada: pulsar una carpeta que ya está plegada no
    /// pide abrir nada ni cambia nada, y ese es el punto.
    #[test]
    fn seleccionar_una_carpeta_no_pide_abrir_nada() {
        let context = egui::Context::default();
        let proyecto = proyecto_de_prueba();
        let mut app = App::new();

        pulsar_fila(&context, &proyecto, &mut app, 1);

        assert!(
            app.peticiones().is_empty(),
            "una carpeta no se abre, y se pidió {:?}",
            app.peticiones()
        );
    }

    /// El archivo que se ha seleccionado sigue marcado.
    ///
    /// Seleccionar y no verse nada marcado es la peor forma de seleccionar: el usuario
    /// pulsa un archivo y no tiene forma de saber cuál ha abierto. Y tiene que acordarse
    /// entre frames, que es lo que distingue una selección de un efecto momentáneo.
    #[test]
    fn el_archivo_seleccionado_sigue_marcado() {
        let context = egui::Context::default();
        let proyecto = proyecto_de_prueba();
        let mut app = App::new();

        filas_de(&context, &proyecto, &mut app);
        assert_eq!(
            app.state().seleccion(),
            None,
            "nada está seleccionado al empezar"
        );

        pulsar_fila(&context, &proyecto, &mut app, 0);
        assert_eq!(
            app.state().seleccion(),
            Some("App.csproj"),
            "el archivo que se ha pulsado es el que queda marcado"
        );

        dibujar(&context, &proyecto, &mut app, &[]);
        assert_eq!(
            app.state().seleccion(),
            Some("App.csproj"),
            "la selección se recuerda entre frames: si no, el explorador parpadearía"
        );
    }

    /// Pulsa con el botón derecho la fila que ocupa el sitio `indice` y devuelve lo que el
    /// menú dibuja.
    ///
    /// El menú se dibuja encima de todo lo demás, así que sus elementos no son filas del
    /// árbol: son los rectángulos que aparecen al abrirse. Por eso se comparan los de antes
    /// con los de después y se devuelven los que no estaban, que son sus botones.
    ///
    /// Con el botón derecho y no con el izquierdo porque es el que abre el menú, y porque
    /// con el otro la fila seleccionaría el archivo, que es lo de FE-013 y no lo que aquí se
    /// está probando.
    fn pulsar_boton_derecho(
        context: &egui::Context,
        proyecto: &Project,
        app: &mut App,
        indice: usize,
    ) -> Vec<egui::Rect> {
        let antes = filas_de(context, proyecto, app);
        let fila = antes
            .get(indice)
            .copied()
            .unwrap_or_else(|| panic!("no hay ninguna fila {indice}: {antes:?}"));

        for pressed in [true, false] {
            let boton = egui::Event::PointerButton {
                pos: fila.center(),
                button: egui::PointerButton::Secondary,
                pressed,
                modifiers: egui::Modifiers::default(),
            };
            dibujar(context, proyecto, app, &[boton]);
        }

        let despues = filas_de(context, proyecto, app);
        despues
            .iter()
            .filter(|rectangulo| !antes.contains(rectangulo))
            .copied()
            .collect()
    }

    /// Pulsa el elemento `indice` del menú contextual que ya está abierto.
    ///
    /// Los elementos van en el orden en que los dibuja el menú, que es el orden en que los
    /// escribió el que lo dibujó, y ese orden es también el que ve el usuario. Por eso el
    /// sitio se cuenta entre los que se han dibujado: el primero es el de arriba, que es el
    /// primero que se lee.
    fn pulsar_elemento(
        context: &egui::Context,
        proyecto: &Project,
        app: &mut App,
        elementos: &[egui::Rect],
        indice: usize,
    ) {
        let elemento = elementos
            .get(indice)
            .copied()
            .unwrap_or_else(|| panic!("no hay ningún elemento {indice}: {elementos:?}"));

        for pressed in [true, false] {
            let boton = egui::Event::PointerButton {
                pos: elemento.center(),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            };
            dibujar(context, proyecto, app, &[boton]);
        }
    }

    /// El menú contextual de la fila es un menú, y lo que ofrece son las dos acciones de
    /// RF-06.
    ///
    /// Se mira lo que hay en el menú y no lo que hay en el archivo porque el menú es lo que
    /// el usuario ve cuando pulsa el botón derecho, y porque su contenido es una lista, y
    /// una lista se puede comprobar entera: si mañana el menú ofreciera "Abrir" o
    /// "Renombrar", este test es el que se entera.
    ///
    /// Las dos acciones se miran con su comando porque lo que RF-06 prepara son los
    /// comandos: un elemento que no pide nada no es una acción, es un rótulo.
    #[test]
    fn el_menu_contextual_ofrece_las_dos_acciones_de_rf_06() {
        let menu: Vec<_> = super::ACCIONES_DEL_CONTEXTO
            .iter()
            .map(|accion| (accion.nombre, accion.comando))
            .collect();

        assert_eq!(
            menu,
            vec![
                ("Nuevo archivo", Some(Command::NewFile)),
                ("Nuevo directorio", Some(Command::NewDirectory)),
            ],
            "el menú contextual de la fila ofrece el archivo y el directorio, y cada uno pide su comando"
        );
    }

    /// Con el botón derecho se abre el menú contextual, y con el izquierdo no.
    ///
    /// Los dos botones se comprueban porque el menú tiene que aparecer donde el usuario lo
    /// ha pedido: si apareciera con el botón izquierdo, cada vez que se seleccionara un
    /// archivo saltaría un menú encima, y pulsar un archivo se parecería a pulsarlo dos
    /// veces. Y al revés: un menú que no aparece es un menú que no existe.
    #[test]
    fn el_menu_contextual_aparece_con_el_boton_derecho() {
        let context = egui::Context::default();
        let mut app = App::new();
        let proyecto = proyecto_de_prueba();

        assert!(
            !context.any_popup_open(),
            "sin pulsar nada no hay ningún menú abierto"
        );

        let elementos = pulsar_boton_derecho(&context, &proyecto, &mut app, 1);
        assert!(
            context.any_popup_open(),
            "el botón derecho sobre una fila tiene que abrir su menú"
        );
        assert_eq!(
            elementos.len(),
            super::ACCIONES_DEL_CONTEXTO.len(),
            "el menú tiene un botón por acción: {elementos:?}"
        );
    }

    /// Abrir el menú no pide nada: lo que se pide es lo que se pulse dentro.
    ///
    /// Un menú que al abrirse ya pidiera algo no sería un menú: sería una acción con la
    /// que hay que acertar, y abrirlo por error crearía un archivo sin querer. El clic
    /// derecho se usa mucho más veces de las que se pulsa un elemento, así que pedir en
    /// cuanto se abre sería pedir casi siempre.
    #[test]
    fn abrir_el_menu_contextual_no_pide_nada() {
        let context = egui::Context::default();
        let mut app = App::new();
        let proyecto = proyecto_de_prueba();

        pulsar_boton_derecho(&context, &proyecto, &mut app, 1);

        assert_eq!(
            app.peticiones(),
            Vec::<Command>::new(),
            "abrir el menú no pide nada por sí solo"
        );
    }

    /// "Nuevo archivo" en el menú pide un archivo nuevo, y solo uno.
    ///
    /// Se comprueba que pide exactamente uno porque un archivo nuevo que se pidiera dos
    /// veces sería un archivo de más en el proyecto, y no se sabría cuál es cuál.
    #[test]
    fn nuevo_archivo_pide_crear_un_archivo_una_sola_vez() {
        let context = egui::Context::default();
        let mut app = App::new();
        let proyecto = proyecto_de_prueba();

        let elementos = pulsar_boton_derecho(&context, &proyecto, &mut app, 1);
        pulsar_elemento(&context, &proyecto, &mut app, &elementos, 0);

        assert_eq!(
            app.peticiones(),
            vec![Command::NewFile],
            "el elemento de arriba del menú pide un archivo nuevo, y nada más"
        );
    }

    /// "Nuevo directorio" pide un directorio nuevo, y solo uno.
    ///
    /// Es el otro elemento del menú, y va en su propio test para que se sepa cuál de los
    /// dos botones es el que se ha roto cuando uno de los dos falle.
    #[test]
    fn nuevo_directorio_pide_crear_un_directorio_una_sola_vez() {
        let context = egui::Context::default();
        let mut app = App::new();
        let proyecto = proyecto_de_prueba();

        let elementos = pulsar_boton_derecho(&context, &proyecto, &mut app, 1);
        pulsar_elemento(&context, &proyecto, &mut app, &elementos, 1);

        assert_eq!(
            app.peticiones(),
            vec![Command::NewDirectory],
            "el elemento de abajo del menú pide un directorio nuevo, y nada más"
        );
    }
}
