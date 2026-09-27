//! Tests del generador de codigo Swing.

use crate::core::{CoreError, CoreResult};
use crate::generation::{
    CodeGenerator, DesignerComponent, DesignerModel, GeneratedCode, MARKER_BEGIN, MARKER_END,
};

/// Metodo que genera la zona de una ventana.
///
/// La zona va dentro de la clase de la ventana, y la zona no contiene el
/// constructor: el constructor llama a `super(...)` y a este metodo, que son
/// cosas de la plantilla, no del disenador. Los componentes son campos y no
/// variables del metodo para que un manejador de evento pueda usarlos.
///
/// Vive aqui y no en la plantilla por lo mismo que los marcadores: es el
/// generador quien decide el nombre, y los dos tienen que usar el mismo o el
/// proyecto generado no compilaria.
pub const BUILD_METHOD: &str = "buildContent";

/// Genera el codigo Java de una ventana de Swing.
///
/// Escribe una zona delimitada, no el archivo entero: lo que el usuario haya
/// escrito fuera de la zona se queda, que es lo unico que permite ir y volver
/// entre el disenador y el codigo sin pisar lo hecho a mano.
///
/// A diferencia del generador de Windows Forms, aqui si hacen falta `import`:
/// Java no tiene un tipo que se pueda escribir con su nombre completo y sin
/// importar. El generador no los escribe: quien decide el `import` de cada tipo
/// es la plantilla del archivo, que es la que sabe que usa el archivo entero.
pub struct SwingGenerator;

impl SwingGenerator {
    /// Aplica el codigo de `model` al contenido actual del archivo de la ventana y
    /// devuelve el contenido nuevo.
    ///
    /// No escribe en disco: quien tiene el archivo decide cuando guardarlo.
    pub fn apply_to(&self, model: &DesignerModel, source: &str) -> CoreResult<String> {
        self.generate(model)?.region().apply_to(source)
    }
}

impl CodeGenerator for SwingGenerator {
    fn generate(&self, model: &DesignerModel) -> CoreResult<GeneratedCode> {
        Ok(GeneratedCode::new(
            MARKER_BEGIN,
            MARKER_END,
            self.content(model)?,
        ))
    }
}

impl SwingGenerator {
    /// El codigo de la zona: los campos de los componentes y el metodo que los
    /// monta en la ventana.
    fn content(&self, model: &DesignerModel) -> CoreResult<String> {
        for component in model.components() {
            validate_name(component.name())?;
        }

        let mut content = String::new();

        for component in model.components() {
            content.push_str(&format!(
                "        private {} {};\n",
                component.kind(),
                variable_of(component.name()),
            ));
        }

        content.push_str(&format!("\n        private void {BUILD_METHOD}() {{\n"));
        content.push_str(&format!(
            "            setSize({}, {});\n",
            model.window().width(),
            model.window().height(),
        ));
        // El titulo se pone aqui y no en el `super(...)` del constructor porque
        // esa llamada esta fuera de la zona y no es de MiniIDE.
        content.push_str(&format!(
            "            setTitle(\"{}\");\n",
            literal(model.window().title()),
        ));
        // Sin un gestor de disposicion que no sea el por defecto, Swing coloca los
        // componentes donde le place y hace caso de `setBounds`.
        content.push_str("            getContentPane().setLayout(null);\n");

        for component in model.components() {
            content.push('\n');
            content.push_str(&component_body(component)?);
        }

        content.push_str("        }");

        Ok(content)
    }
}

/// Lo que se escribe para un componente: se crea, se coloca, se le ponen sus
/// propiedades y se anade a la ventana.
fn component_body(component: &DesignerComponent) -> CoreResult<String> {
    let name = variable_of(component.name());
    let mut body = String::new();

    body.push_str(&format!(
        "            {name} = new {}();\n",
        component.kind()
    ));
    body.push_str(&format!(
        "            {name}.setBounds({}, {}, {}, {});\n",
        component.x(),
        component.y(),
        component.width(),
        component.height(),
    ));

    for (property, value) in component.properties() {
        body.push_str(&property_line(&name, property, value)?);
    }

    body.push_str(&format!("            getContentPane().add({name});\n"));

    Ok(body)
}

/// La linea que pone una propiedad del componente.
///
/// Swing no tiene asignacion de propiedades como Windows Forms: cada cosa se pone
/// con su metodo, `setText` y `setVisible`, y con el tipo que le corresponde. Un
/// texto va entrecomillado y un booleano no.
fn property_line(name: &str, property: &str, value: &str) -> CoreResult<String> {
    let line = match property {
        "text" => format!("{name}.setText(\"{}\");\n", literal(value)),
        "visible" | "enabled" => {
            let setter = if property == "visible" {
                "setVisible"
            } else {
                "setEnabled"
            };

            format!("{name}.{setter}({value});\n")
        }
        other => {
            return Err(CoreError::Unsupported(format!(
                "MiniIDE no sabe generar la propiedad {other} de un componente de Swing"
            )))
        }
    };

    Ok(format!("            {line}"))
}

/// Comprueba que un nombre de componente puede ser el de una variable de Java.
///
/// El nombre del componente es el de la variable, asi que un nombre que no valga
/// como identificador daria codigo que no compila. Se comprueba aqui y no en el
/// modelo porque es el generador quien sabe que es un identificador de Java.
fn validate_name(name: &str) -> CoreResult<()> {
    let mut characters = name.chars();

    let starts_well = characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_' || first == '$');

    let rest_is_well = characters
        .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '$');

    if !starts_well || !rest_is_well {
        return Err(CoreError::InvalidName(format!(
            "{name} no puede ser el nombre de una variable de Java"
        )));
    }

    Ok(())
}

/// Como se escribe el nombre de un componente en el codigo generado.
///
/// Java no tiene forma de marcar un identificador reservado como el `@` de C#,
/// asi que un componente llamado `class` se escribe `class_`: el disenador sigue
/// mostrando el nombre que el usuario puso y el codigo compila.
fn variable_of(name: &str) -> String {
    if is_keyword(name) {
        return format!("{name}_");
    }

    name.to_string()
}

/// Si `word` es una palabra reservada de Java.
fn is_keyword(word: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "abstract",
        "assert",
        "boolean",
        "break",
        "byte",
        "case",
        "catch",
        "char",
        "class",
        "const",
        "continue",
        "default",
        "do",
        "double",
        "else",
        "enum",
        "extends",
        "false",
        "final",
        "finally",
        "float",
        "for",
        "goto",
        "if",
        "implements",
        "import",
        "instanceof",
        "int",
        "interface",
        "long",
        "native",
        "new",
        "null",
        "package",
        "private",
        "protected",
        "public",
        "return",
        "short",
        "static",
        "strictfp",
        "super",
        "switch",
        "synchronized",
        "this",
        "throw",
        "throws",
        "transient",
        "true",
        "try",
        "void",
        "volatile",
        "while",
        "_",
    ];

    KEYWORDS.contains(&word)
}

/// Un texto de Java entrecomillado, con lo que haya que escapar escapado.
///
/// Sin esto, un texto con una comilla o un salto de linea partiria el literal y el
/// archivo no compilaria, y el usuario no tendria forma de saber por que.
fn literal(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());

    for character in text.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(character),
        }
    }

    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::SwingModel;

    /// Ventana minima con un boton, el caso que se repite en casi todas las
    /// pruebas.
    fn window() -> SwingModel {
        let mut window = SwingModel::new("MainWindow", "Ventana principal", 640, 480);
        window
            .add_component("okButton", "JButton", 8, 8, 120, 30)
            .unwrap();

        window
    }

    /// El codigo de la zona de una ventana.
    fn code_of(window: &SwingModel) -> String {
        SwingGenerator
            .generate(window.model())
            .expect("codigo generado")
            .region()
            .content()
            .to_string()
    }

    #[test]
    fn the_generated_code_comes_with_the_markers_of_the_zone() {
        let code = SwingGenerator.generate(window().model()).unwrap();

        assert_eq!(code.region().begin(), MARKER_BEGIN);
        assert_eq!(code.region().end(), MARKER_END);
    }

    /// La zona va dentro de la clase, asi que el codigo que genera son miembros:
    /// campos para los componentes y un metodo que los monta.
    #[test]
    fn a_minimum_window_generates_the_method_that_builds_its_content() {
        let code = code_of(&window());

        assert!(code.contains("private void buildContent()"), "{code}");
    }

    #[test]
    fn a_minimum_window_generates_its_title_and_its_size() {
        let code = code_of(&window());

        assert!(code.contains(r#"setTitle("Ventana principal");"#), "{code}");
        assert!(code.contains("setSize(640, 480);"), "{code}");
    }

    /// Sin esto el gestor de disposicion de Swing ignora los `setBounds` y los
    /// componentes se apilarian en el centro en vez de donde dice el disenador.
    #[test]
    fn the_components_are_placed_by_bounds_so_the_layout_is_disabled() {
        let code = code_of(&window());

        assert!(code.contains("getContentPane().setLayout(null);"), "{code}");
    }

    #[test]
    fn a_component_is_declared_as_a_field() {
        let code = code_of(&window());

        assert!(code.contains("private JButton okButton;"), "{code}");
    }

    #[test]
    fn a_component_is_created_and_added_to_the_content_pane() {
        let code = code_of(&window());

        assert!(code.contains("okButton = new JButton();"), "{code}");
        assert!(code.contains("getContentPane().add(okButton);"), "{code}");
    }

    #[test]
    fn the_position_and_the_size_of_a_component_are_generated() {
        let code = code_of(&window());

        assert!(
            code.contains("okButton.setBounds(8, 8, 120, 30);"),
            "{code}"
        );
    }

    #[test]
    fn an_empty_window_generates_code_without_components() {
        let empty = SwingModel::new("MainWindow", "Ventana", 640, 480);

        let code = code_of(&empty);

        assert!(code.contains("private void buildContent()"), "{code}");
        assert!(!code.contains("getContentPane().add"), "{code}");
        assert!(!code.contains("private J"), "{code}");
    }

    #[test]
    fn the_four_types_of_component_generate_their_own_type() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);
        window
            .add_component("okButton", "JButton", 0, 0, 10, 10)
            .unwrap();
        window
            .add_component("titleLabel", "JLabel", 0, 20, 10, 10)
            .unwrap();
        window
            .add_component("nameText", "JTextField", 0, 40, 10, 10)
            .unwrap();
        window
            .add_component("rootPanel", "JPanel", 0, 60, 10, 10)
            .unwrap();

        let code = code_of(&window);

        assert!(code.contains("private JButton okButton;"), "{code}");
        assert!(code.contains("private JLabel titleLabel;"), "{code}");
        assert!(code.contains("private JTextField nameText;"), "{code}");
        assert!(code.contains("private JPanel rootPanel;"), "{code}");
    }

    /// Las propiedades de Swing se montan con los metodos de su API, no con una
    /// asignacion como en Windows Forms: `okButton.text = "Aceptar"` no compila.
    #[test]
    fn the_properties_are_generated_with_the_setters_of_swing() {
        let mut window = window();
        window.set_text("okButton", "Aceptar").unwrap();
        window.set_visible("okButton", false).unwrap();
        window.set_enabled("okButton", false).unwrap();

        let code = code_of(&window);

        assert!(code.contains(r#"okButton.setText("Aceptar");"#), "{code}");
        assert!(code.contains("okButton.setVisible(false);"), "{code}");
        assert!(code.contains("okButton.setEnabled(false);"), "{code}");
        assert!(!code.contains("okButton.text"), "{code}");
        assert!(!code.contains("okButton.visible"), "{code}");
    }

    #[test]
    fn a_component_without_text_does_not_generate_a_text_line() {
        let code = code_of(&window());

        assert!(!code.contains("setText"), "{code}");
    }

    #[test]
    fn a_moved_and_resized_component_generates_its_new_place() {
        let mut window = window();
        window.move_component("okButton", 40, 60).unwrap();
        window.resize_component("okButton", 200, 80).unwrap();

        assert!(code_of(&window).contains("okButton.setBounds(40, 60, 200, 80);"));
    }

    #[test]
    fn a_negative_position_is_generated_as_it_is() {
        let mut window = window();
        window.move_component("okButton", -20, -10).unwrap();

        assert!(code_of(&window).contains("okButton.setBounds(-20, -10, 120, 30);"));
    }

    #[test]
    fn a_quote_in_the_text_does_not_break_the_generated_literal() {
        let mut window = window();
        window.set_text("okButton", r#"Di "hola""#).unwrap();

        assert!(code_of(&window).contains(r#"okButton.setText("Di \"hola\"");"#));
    }

    #[test]
    fn a_new_line_in_the_text_does_not_break_the_generated_literal() {
        let mut window = window();
        window.set_text("okButton", "linea1\nlinea2").unwrap();

        assert!(code_of(&window).contains(r#"okButton.setText("linea1\nlinea2");"#));
    }

    #[test]
    fn a_backslash_in_the_title_does_not_break_the_generated_literal() {
        let window = SwingModel::new("MainWindow", r"C:\proyectos", 640, 480);

        assert!(code_of(&window).contains(r#"setTitle("C:\\proyectos");"#));
    }

    /// El nombre del componente es el de la variable, asi que tiene que ser un
    /// identificador de Java o el archivo no compilaria.
    #[test]
    fn a_component_whose_name_is_not_an_identifier_is_rejected() {
        for name in ["2botones", "mi-boton", "con espacio", "", "punto.ok"] {
            let model = DesignerModel::new(
                "MainWindow",
                "Ventana",
                640,
                480,
                vec![DesignerComponent::new(name, "JButton", 0, 0, 10, 10)],
            );

            let result = SwingGenerator.generate(&model);

            assert!(
                matches!(result, Err(CoreError::InvalidName(_))),
                "{name} deberia rechazarse"
            );
        }
    }

    /// Java no tiene forma de escapar un identificador reservado como en C#, asi
    /// que un componente llamado `class` genera una variable `class_`.
    #[test]
    fn a_component_named_like_a_java_keyword_is_generated_with_the_escape() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);
        window
            .add_component("class", "JButton", 0, 0, 10, 10)
            .unwrap();
        window.set_text("class", "Aceptar").unwrap();

        let code = code_of(&window);

        assert!(code.contains("private JButton class_;"), "{code}");
        assert!(code.contains("class_ = new JButton();"), "{code}");
        assert!(code.contains(r#"class_.setText("Aceptar");"#), "{code}");
    }

    /// Una propiedad que no es de Swing no se puede traducir a un metodo: es mejor
    /// no generar codigo que no se sabe escribir que inventarse algo.
    #[test]
    fn a_property_that_is_not_of_swing_is_rejected() {
        let mut model = window().model().clone();
        model
            .component_mut("okButton")
            .unwrap()
            .set_property("text_align", "CENTER");

        let result = SwingGenerator.generate(&model);

        assert!(matches!(result, Err(CoreError::Unsupported(_))));
    }

    #[test]
    fn the_generated_code_goes_into_the_region_of_the_file() {
        let source =
            "public class MainWindow extends JFrame {\n    // <MiniIDE>\n    // </MiniIDE>\n}\n";

        let applied = SwingGenerator.apply_to(window().model(), source).unwrap();

        assert!(applied.contains("private JButton okButton;"), "{applied}");
        assert!(
            applied.contains("public class MainWindow extends JFrame {"),
            "{applied}"
        );
        assert_eq!(applied.matches(MARKER_BEGIN).count(), 1, "{applied}");
    }

    #[test]
    fn the_generator_can_be_used_through_the_contract() {
        let generators: [&dyn CodeGenerator; 1] = [&SwingGenerator];

        for generator in generators {
            assert_eq!(
                generator
                    .generate(window().model())
                    .unwrap()
                    .region()
                    .begin(),
                MARKER_BEGIN
            );
        }
    }
}
