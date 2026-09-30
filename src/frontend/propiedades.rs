//! El panel de propiedades: lo que se ve del control seleccionado.
//!
//! Es la otra mitad del diseñador. El canvas enseña la forma y aquí se enseña lo que no cabe
//! en un rectángulo: el nombre, dónde está, cuánto mide y qué propiedades tiene.
//!
//! No guarda copia de nada. Lo que se está editando sale del modelo del diseñador y se
//! vuelve a él, y todo cambio entra por [`App::pedir`] como un comando del diseñador. Un
//! panel que guardara el valor mientras se escribe tendría dos verdades sobre el mismo
//! control, y la copia se quedaría vieja en cuanto otra cosa tocara el modelo.
//!
//! Las propiedades se enseñan todas y no una en concreto porque el nombre de la del texto
//! depende del framework: WinForms la llama `Text` y Swing la llama `text`, y este panel no
//! sabe cuál de las dos está usando el proyecto. Enseñar todas evita tener que escribir ese
//! nombre en la ventana, que es el `match` por framework que AGENTS.md §2.4 prohíbe.

use eframe::egui;

use super::diseniador::Comando;
use crate::frontend::App;

/// Una fila de las propiedades: un nombre y el valor que tiene.
///
/// Es un tipo y no un `String` porque el nombre es lo que va al comando y el valor es lo que
/// se enseña. Juntarlos en un texto habría que partirlo otra vez para poder mandar el
/// comando, y partirlo es donde se cuela un cambio en la propiedad que no era el que se
/// quería.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Campo {
    pub nombre: String,
    pub valor: String,
}

/// Las propiedades del control seleccionado.
///
/// Vacío cuando no hay nada seleccionado: el panel se queda sin filas en vez de enseñar un
/// hueco, porque un panel de propiedades sin control seleccionado no tiene nada que decir y
/// un marco vacío parece que falta algo.
pub fn campos(diseniador: &super::diseniador::Diseniador) -> Vec<Campo> {
    let Some(nombre) = diseniador.seleccion() else {
        return Vec::new();
    };
    let Some(control) = diseniador.modelo().component(nombre) else {
        return Vec::new();
    };

    let mut campos = vec![
        Campo {
            nombre: "Nombre".to_owned(),
            valor: control.name().to_owned(),
        },
        Campo {
            nombre: "Tipo".to_owned(),
            valor: control.kind().to_owned(),
        },
        Campo {
            nombre: "X".to_owned(),
            valor: control.x().to_string(),
        },
        Campo {
            nombre: "Y".to_owned(),
            valor: control.y().to_string(),
        },
        Campo {
            nombre: "Ancho".to_owned(),
            valor: control.width().to_string(),
        },
        Campo {
            nombre: "Alto".to_owned(),
            valor: control.height().to_string(),
        },
    ];

    for (nombre, valor) in control.properties() {
        campos.push(Campo {
            nombre: nombre.clone(),
            valor: valor.clone(),
        });
    }

    campos
}

/// Dibuja el panel de propiedades del control que esté seleccionado.
pub fn panel(ui: &mut egui::Ui, app: &mut App) {
    let campos = campos(app.diseniador());

    if campos.is_empty() {
        ui.weak("Sin selección");
        return;
    }

    let seleccion = app.diseniador().seleccion().map(str::to_owned);
    ui.vertical(|ui| {
        for campo in campos {
            fila(ui, app, &campo, seleccion.as_deref());
        }
    });
}

/// Una fila: el nombre de la propiedad y su valor, editable.
fn fila(ui: &mut egui::Ui, app: &mut App, campo: &Campo, seleccion: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(&campo.nombre);
        editable(ui, app, seleccion, campo);
    });
}

/// Dibuja el valor de una propiedad y lo devuelve al modelo si ha cambiado.
///
/// Los números van con `DragValue` y el resto con `TextEdit`, y no al revés, porque arrastrar
/// un número es la manera de colocar un control sin tener que acertar el valor: el usuario
/// quiere "un poco más a la derecha", no "x = 137".
///
/// El valor sale del modelo y se copia a una variable local porque los dos widgets escriben
/// mientras el usuario arrastra o teclea: si escribieran en el modelo, cada paso del ratón
/// sería un cambio del modelo y el control se movería mientras todavía se está moviendo.
///
/// Lo que cambia se manda al modelo en cuanto cambia y no al soltar. Esperar a soltar
/// dejaría el modelo viejo durante todo el arrastre, y el canvas se dibujaría desde el
/// modelo: el control iría por detrás del ratón en vez de ir con él.
fn editable(ui: &mut egui::Ui, app: &mut App, seleccion: Option<&str>, campo: &Campo) {
    let texto = campo.valor.clone();

    match campo.nombre.as_str() {
        "X" | "Y" => {
            let mut valor: i32 = texto.parse().unwrap_or(0);
            let antes = valor;

            ui.add(egui::DragValue::new(&mut valor).speed(1.0));

            if valor != antes {
                mover(app, seleccion, campo, valor);
            }
        }
        "Ancho" | "Alto" => {
            let mut valor: u32 = texto.parse().unwrap_or(0);
            let antes = valor;

            ui.add(egui::DragValue::new(&mut valor).speed(1.0));

            if valor != antes {
                redimensionar(app, seleccion, campo, valor);
            }
        }
        "Nombre" => {
            let mut valor = texto;
            let salida = egui::TextEdit::singleline(&mut valor)
                .desired_width(ANCHO_DEL_VALOR)
                .show(ui);

            // Solo al perder el foco, y no en cada tecla: mientras se escribe un nombre
            // casi nunca es un nombre válido, y renombrarlo en cada tecla dejaría el modelo
            // lleno de controles con nombres a medias que después habría que limpiar.
            if salida.response.lost_focus() {
                renombrar(app, seleccion, valor);
            }
        }
        _ => {
            let mut valor = texto;
            let salida = egui::TextEdit::singleline(&mut valor)
                .desired_width(ANCHO_DEL_VALOR)
                .show(ui);

            if salida.response.lost_focus() {
                propiedad(app, seleccion, campo, valor);
            }
        }
    }
}

/// Cuánto ancho ocupa el campo de valor de una propiedad, en puntos.
///
/// Con el ancho escrito, el panel enseña la misma cosa se mire en la ventana que mida, y un
/// panel que encogiera sus campos al redimensionar la ventana dejaría de poder leerse el
/// nombre del control justo cuando se ha hecho sitio para verlo.
const ANCHO_DEL_VALOR: f32 = 90.0;

/// Mueve el control al cambiar su X o su Y.
///
/// Cambia la posición entera y no solo una coordenada: el modelo guarda la posición como un
/// par, así que mandar solo la que ha cambiado obligaría a leer la otra del modelo en mitad de
/// un gesto, y si en ese momento el control ya no estuviera, se movería el otro.
fn mover(app: &mut App, seleccion: Option<&str>, campo: &Campo, nuevo: i32) {
    let Some((x, y, _, _)) = geometria(app, seleccion) else {
        return;
    };
    let (x, y) = if campo.nombre == "X" {
        (nuevo, y)
    } else {
        (x, nuevo)
    };

    app.pedir(Comando::Mover {
        control: seleccion.map(str::to_owned).unwrap_or_default(),
        x,
        y,
    });
}

/// Cambia el ancho o el alto del control.
fn redimensionar(app: &mut App, seleccion: Option<&str>, campo: &Campo, nuevo: u32) {
    let Some((_, _, ancho, alto)) = geometria(app, seleccion) else {
        return;
    };
    let (ancho, alto) = if campo.nombre == "Ancho" {
        (nuevo, alto)
    } else {
        (ancho, nuevo)
    };

    app.pedir(Comando::Redimensionar {
        control: seleccion.map(str::to_owned).unwrap_or_default(),
        ancho,
        alto,
    });
}

/// Renombra el control.
fn renombrar(app: &mut App, seleccion: Option<&str>, nombre: String) {
    let Some(control) = seleccion.map(str::to_owned) else {
        return;
    };

    app.pedir(Comando::Renombrar { control, nombre });
}

/// Cambia una propiedad del control.
fn propiedad(app: &mut App, seleccion: Option<&str>, campo: &Campo, valor: String) {
    let Some(control) = seleccion.map(str::to_owned) else {
        return;
    };

    app.pedir(Comando::Propiedad {
        control,
        propiedad: campo.nombre.clone(),
        valor,
    });
}

/// Dónde está y cuánto mide el control seleccionado.
///
/// Se busca en el modelo y no se da por hecho que existe: entre que se seleccionó y que se
/// cambia una propiedad el control puede haberse borrado, y mandar un comando con un nombre
/// que ya no está dejaría que el modelo lo ignorara por su cuenta.
fn geometria(app: &App, seleccion: Option<&str>) -> Option<(i32, i32, u32, u32)> {
    let nombre = seleccion?;

    app.diseniador()
        .modelo()
        .component(nombre)
        .map(|control| (control.x(), control.y(), control.width(), control.height()))
}

#[cfg(test)]
mod tests {
    use crate::frontend::diseniador::{Comando as Gesto, Diseniador};
    use crate::frontend::App;
    use eframe::egui;

    use super::*;

    const ANCHO: f32 = 1000.0;
    const ALTO: f32 = 800.0;

    /// Un diseñador con un botón con texto, que es el caso con el que se mira un panel de
    /// propiedades: un control sin propiedades no enseña para qué sirve el panel.
    fn diseniador_con_texto() -> Diseniador {
        let mut diseniador = Diseniador::vacio("Principal", "Principal", 400, 300);

        diseniador.modelo_mut().add_component(
            crate::generation::DesignerComponent::new("boton1", "Button", 20, 30, 100, 40)
                .with_property("Text", "Aceptar"),
        );
        diseniador.aplicar(Gesto::Seleccionar(Some("boton1".to_owned())));

        diseniador
    }

    fn app_con_seleccion() -> App {
        let mut app = App::new();
        *app.diseniador_mut() = diseniador_con_texto();
        app.state_mut().mostrar_diseniador();

        app
    }

    fn entrada() -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(ANCHO, ALTO),
            )),
            ..egui::RawInput::default()
        }
    }

    /// Seleccionar un control muestra su nombre, su texto, su posición y su tamaño.
    /// FE-051.
    ///
    /// Los cuatro campos que la tarea nombra se comprueban uno a uno en vez de mirar que la
    /// lista "tiene algo": un panel que enseñara el nombre y el tipo pero no la posición
    /// dejaría al usuario sin forma de colocar el control con precisión, que es la mitad de
    /// para qué está un panel de propiedades.
    #[test]
    fn seleccionar_un_control_muestra_su_nombre_texto_posicion_y_tamano() {
        let campos = campos(&diseniador_con_texto());
        let valor = |nombre: &str| {
            campos
                .iter()
                .find(|campo| campo.nombre == nombre)
                .map(|campo| campo.valor.clone())
        };

        assert_eq!(valor("Nombre").as_deref(), Some("boton1"), "el nombre");
        assert_eq!(valor("Text").as_deref(), Some("Aceptar"), "el texto");
        assert_eq!(valor("X").as_deref(), Some("20"), "la posición horizontal");
        assert_eq!(valor("Y").as_deref(), Some("30"), "la posición vertical");
        assert_eq!(valor("Ancho").as_deref(), Some("100"), "el ancho");
        assert_eq!(valor("Alto").as_deref(), Some("40"), "el alto");
    }

    /// Sin selección no hay propiedades que enseñar.
    ///
    /// El panel tiene que poder quedarse vacío sin que eso parezca un fallo: es lo que se
    /// ve antes de que el usuario haya tocado nada, que es la mayor parte del tiempo.
    #[test]
    fn sin_seleccion_no_hay_propiedades() {
        let mut diseniador = diseniador_con_texto();
        diseniador.aplicar(Gesto::Seleccionar(None));

        assert!(
            campos(&diseniador).is_empty(),
            "sin control seleccionado no hay nada que enseñar"
        );
    }

    /// El texto del control sale con el nombre que le pone su framework.
    ///
    /// Se enseñan todas las propiedades y no solo el texto, porque aquí no se sabe cuál es
    /// el nombre del texto: `Text` en WinForms y `text` en Swing. Si el panel enseñara solo
    /// una, el otro framework no tendría forma de editar su control.
    #[test]
    fn las_propiedades_se_enseñan_con_el_nombre_que_ponen() {
        let mut diseniador = Diseniador::vacio("Principal", "Principal", 400, 300);
        diseniador.modelo_mut().add_component(
            crate::generation::DesignerComponent::new("campo1", "JTextField", 0, 0, 80, 20)
                .with_property("text", "Nombre"),
        );
        diseniador.aplicar(Gesto::Seleccionar(Some("campo1".to_owned())));

        let campos = campos(&diseniador);

        assert!(
            campos
                .iter()
                .any(|campo| campo.nombre == "text" && campo.valor == "Nombre"),
            "la propiedad del texto tiene que salir con el nombre que le pone el framework: \
             {campos:?}"
        );
    }

    /// El panel vive en la ventana, al lado del diseñador.
    ///
    /// Las filas pueden estar bien escritas y no verse nunca, que es lo que pasa con un
    /// panel que no llega a dibujarse: se comprueba pintando la ventana entera y mirando que
    /// hay algo en el canto derecho, que es donde el layout reserva el panel de propiedades.
    #[test]
    fn el_panel_de_propiedades_se_dibuja_en_la_ventana() {
        let contexto = egui::Context::default();
        let mut app = app_con_seleccion();

        let mut salida = contexto.run_ui(entrada(), |ui| app.dibujar(ui));
        salida.textures_delta.clear();

        let a_la_derecha = salida
            .shapes
            .into_iter()
            .filter_map(|forma| match forma.shape {
                egui::Shape::Rect(rectangulo) => Some(rectangulo.rect),
                _ => None,
            })
            .filter(|rectangulo| rectangulo.min.x > ANCHO / 2.0 && rectangulo.width() < ANCHO / 2.0)
            .count();

        assert!(
            a_la_derecha > 0,
            "el panel de propiedades tiene que dibujarse en su sitio de la ventana"
        );
    }

    /// Cambiar una propiedad la cambia en el modelo. FE-052.
    ///
    /// Lo que se mide es el modelo y no lo que se ve, porque lo que se ve se redibuja a cada
    /// frame y lo que tiene que sobrevivir es lo que hay en el modelo: de ahí se generan el
    /// código y el resto de la aplicación.
    #[test]
    fn cambiar_una_propiedad_la_cambia_en_el_modelo() {
        let mut app = app_con_seleccion();

        app.pedir(Comando::Propiedad {
            control: "boton1".to_owned(),
            propiedad: "Text".to_owned(),
            valor: "Cancelar".to_owned(),
        });

        let boton = app
            .diseniador()
            .modelo()
            .component("boton1")
            .expect("el botón sigue en el modelo");
        assert_eq!(
            boton.property("Text"),
            Some("Cancelar"),
            "la propiedad tiene que quedar cambiada en el modelo, no solo en el panel"
        );
    }

    /// Cambiar la posición desde las propiedades mueve el control. FE-052.
    ///
    /// Mover con el panel y mover arrastrando tienen que acabar en lo mismo. Si el panel se
    /// guardara la posición por su cuenta, un control movido con el ratón y luego tocado en
    /// las propiedades volvería a donde estaba.
    #[test]
    fn cambiar_la_posicion_mueve_el_control_en_el_modelo() {
        let mut app = app_con_seleccion();
        let campos = campos(app.diseniador());
        let campo_x = campos
            .iter()
            .find(|campo| campo.nombre == "X")
            .expect("el panel tiene un campo X")
            .clone();
        let seleccion = app.diseniador().seleccion().map(str::to_owned);

        let mut salida = egui::Context::default().run_ui(entrada(), |ui| {
            ui.horizontal(|ui| {
                fila(ui, &mut app, &campo_x, seleccion.as_deref());
            });
        });
        salida.textures_delta.clear();

        // El `DragValue` se ha dibujado con el valor del modelo; lo que se comprueba aquí es
        // que el camino del comando es el mismo que usa el canvas y que llega al modelo.
        mover(&mut app, seleccion.as_deref(), &campo_x, 55);

        let boton = app
            .diseniador()
            .modelo()
            .component("boton1")
            .expect("el botón sigue en el modelo");
        assert_eq!(
            (boton.x(), boton.y()),
            (55, 30),
            "cambiar la X mueve el control sin tocar la Y"
        );
    }

    /// Cambiar el tamaño desde las propiedades redimensiona el control. FE-052.
    #[test]
    fn cambiar_el_tamano_redimensiona_el_control_en_el_modelo() {
        let mut app = app_con_seleccion();
        let campos = campos(app.diseniador());
        let campo_ancho = campos
            .iter()
            .find(|campo| campo.nombre == "Ancho")
            .expect("el panel tiene un campo de ancho")
            .clone();
        let seleccion = app.diseniador().seleccion().map(str::to_owned);

        redimensionar(&mut app, seleccion.as_deref(), &campo_ancho, 250);

        let boton = app
            .diseniador()
            .modelo()
            .component("boton1")
            .expect("el botón sigue en el modelo");
        assert_eq!(
            (boton.width(), boton.height()),
            (250, 40),
            "cambiar el ancho redimensiona sin tocar el alto"
        );
    }

    /// El panel pide los cambios por el mismo camino que el canvas. FE-052.
    ///
    /// Es lo que evita que el panel se guarde los valores por su cuenta: si los dos caminos
    /// fueran distintos habría dos sitios donde tocar el modelo y basta con que uno se
    /// olvide de hacerlo para que el canvas y las propiedades dejen de cuadrar. Se comprueba
    /// leyendo el código porque las dos rutas llevan a lo mismo y solo se distinguen en cómo
    /// se escribieron.
    #[test]
    fn el_panel_pide_los_cambios_por_el_camino_del_diseniador() {
        let fuente =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file!()))
                .expect("el fuente del panel tiene que poder leerse");

        let codigo = fuente
            .split("#[cfg(test)]")
            .next()
            .expect("el panel tiene que tener código antes de los tests");

        assert!(
            codigo.contains("app.pedir("),
            "toda propiedad que cambie tiene que entrar por `App::pedir`, que es el único \
             camino al modelo del diseñador"
        );
        assert!(
            !codigo.contains("set_property(") && !codigo.contains("set_position("),
            "el panel no puede tocar el modelo por su cuenta: lo que cambia el modelo es el \
             diseñador"
        );
    }
}
