use crate::core::{CoreError, CoreResult};
use crate::generation::{
    CodeGenerator, DesignerComponent, DesignerModel, GeneratedCode, MARKER_BEGIN, MARKER_END,
};

/// Genera el codigo C# de un formulario de Windows Forms.
///
/// Escribe una zona delimitada, no el archivo entero: lo que el usuario haya
/// escrito fuera de la zona se queda, que es lo unico que permite ir y volver
/// entre el disenador y el codigo sin pisar lo hecho a mano.
///
/// No usa `using` porque los tipos van con su namespace completo. El codigo
/// generado es entonces autonomo: compila igual aunque el archivo donde se
/// escribe no tenga los `using` que el usuario haya puesto o quitado.
pub struct WinFormsGenerator;

impl WinFormsGenerator {
    /// Aplica el codigo de `model` al contenido actual del archivo del
    /// disenador y devuelve el contenido nuevo.
    ///
    /// No escribe en disco: quien tiene el archivo decide cuando guardarlo.
    pub fn apply_to(&self, model: &DesignerModel, source: &str) -> CoreResult<String> {
        self.generate(model)?.region().apply_to(source)
    }
}

impl CodeGenerator for WinFormsGenerator {
    fn generate(&self, model: &DesignerModel) -> CoreResult<GeneratedCode> {
        Ok(GeneratedCode::new(
            MARKER_BEGIN,
            MARKER_END,
            self.content(model)?,
        ))
    }
}

impl WinFormsGenerator {
    /// El codigo de la zona, con sus miembros y su `InitializeComponent`.
    fn content(&self, model: &DesignerModel) -> CoreResult<String> {
        for component in model.components() {
            validate_name(component.name())?;
        }

        let mut content = String::new();

        for component in model.components() {
            content.push_str(&format!(
                "        private System.Windows.Forms.{} {};\n\n",
                component.kind(),
                field_name(component.name()),
            ));
        }

        content.push_str("        private void InitializeComponent()\n        {\n");
        content.push_str("            this.SuspendLayout();\n");
        content.push_str(&format!(
            "            this.ClientSize = new System.Drawing.Size({}, {});\n",
            model.window().width(),
            model.window().height(),
        ));
        content.push_str(&format!(
            "            this.Text = \"{}\";\n",
            literal(model.window().title()),
        ));

        for component in model.components() {
            content.push_str(&component_body(component));
        }

        content.push_str("            this.ResumeLayout(false);\n");
        content.push_str("        }");

        Ok(content)
    }
}

/// Lo que se escribe para un control: se crea, se coloca y se anade al
/// formulario.
fn component_body(component: &DesignerComponent) -> String {
    let name = field_name(component.name());
    let mut body = String::new();

    body.push_str(&format!(
        "            this.{name} = new System.Windows.Forms.{}();\n",
        component.kind()
    ));
    body.push_str(&format!(
        "            this.{name}.Location = new System.Drawing.Point({}, {});\n",
        component.x(),
        component.y(),
    ));
    body.push_str(&format!(
        "            this.{name}.Name = \"{}\";\n",
        literal(component.name())
    ));
    body.push_str(&format!(
        "            this.{name}.Size = new System.Drawing.Size({}, {});\n",
        component.width(),
        component.height(),
    ));

    // Las demas propiedades solo se escriben si el usuario las ha puesto: lo
    // que no se ha tocado en el disenador se queda con el valor que ya tiene el
    // control, y el codigo generado no se llena de lineas que no dicen nada.
    for (property, value) in component.properties() {
        body.push_str(&format!(
            "            this.{name}.{property} = \"{}\";\n",
            literal(value)
        ));
    }

    body.push_str(&format!("            this.Controls.Add(this.{name});\n"));

    body
}

/// Comprueba que un nombre de control puede ser el nombre de un campo de C#.
///
/// El nombre del control es el nombre del campo, asi que un nombre que no valga
/// como identificador daria codigo que no compila. Se comprueba aqui y no en el
/// modelo porque es el generador quien sabe que es un identificador de C#.
fn validate_name(name: &str) -> CoreResult<()> {
    let mut characters = name.chars();

    let starts_well = characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_');

    let rest_is_well =
        characters.all(|character| character.is_ascii_alphanumeric() || character == '_');

    if !starts_well || !rest_is_well {
        return Err(CoreError::InvalidName(format!(
            "{name} no puede ser el nombre de un campo de C#"
        )));
    }

    Ok(())
}

/// Como se escribe un nombre de control al que hay que anteponer una arroba.
fn field_name(name: &str) -> String {
    if is_keyword(name) {
        return format!("@{name}");
    }

    name.to_string()
}

/// Si `word` es una palabra reservada de C#.
///
/// Un control que se llame `class` es legitimo en el disenador, pero `class` a
/// solas no es un campo: se escribe `@class`, que si lo es. Negarselo al
/// usuario seria quitarle un nombre que puede usar en cualquier otro sitio.
fn is_keyword(word: &str) -> bool {
    const KEYWORDS: [&str; 77] = [
        "abstract",
        "as",
        "base",
        "bool",
        "break",
        "byte",
        "case",
        "catch",
        "char",
        "checked",
        "class",
        "const",
        "continue",
        "decimal",
        "default",
        "delegate",
        "do",
        "double",
        "else",
        "enum",
        "event",
        "explicit",
        "extern",
        "false",
        "finally",
        "fixed",
        "float",
        "for",
        "foreach",
        "goto",
        "if",
        "implicit",
        "in",
        "int",
        "interface",
        "internal",
        "is",
        "lock",
        "long",
        "namespace",
        "new",
        "null",
        "object",
        "operator",
        "out",
        "override",
        "params",
        "private",
        "protected",
        "public",
        "readonly",
        "ref",
        "return",
        "sbyte",
        "sealed",
        "short",
        "sizeof",
        "stackalloc",
        "static",
        "string",
        "struct",
        "switch",
        "this",
        "throw",
        "true",
        "try",
        "typeof",
        "uint",
        "ulong",
        "unchecked",
        "unsafe",
        "ushort",
        "using",
        "virtual",
        "void",
        "volatile",
        "while",
    ];

    KEYWORDS.contains(&word)
}

/// Un texto de C# entrecomillado, con lo que haya que escapar escapado.
///
/// Sin esto, un texto con una comilla o un salto de linea partiria el literal
/// y el archivo no compilaria, y el usuario no tendria forma de saber por que.
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
    use crate::framework::WinFormsModel;

    /// Formulario minimo con un boton, el caso que se repite en casi todas las
    /// pruebas.
    fn form() -> WinFormsModel {
        let mut form = WinFormsModel::new("MainForm", "Formulario principal", 640, 480);
        form.add_control("okButton", "Button", 8, 8, 120, 30)
            .unwrap();

        form
    }

    /// El codigo de la zona de un formulario.
    fn code_of(form: &WinFormsModel) -> String {
        WinFormsGenerator
            .generate(form.model())
            .unwrap()
            .region()
            .content()
            .to_string()
    }

    #[test]
    fn the_generated_code_comes_with_the_markers_of_the_zone() {
        let code = WinFormsGenerator.generate(form().model()).unwrap();

        assert_eq!(code.region().begin(), MARKER_BEGIN);
        assert_eq!(code.region().end(), MARKER_END);
    }

    #[test]
    fn a_minimum_form_generates_an_initialize_component() {
        let code = code_of(&form());

        assert!(code.contains("private void InitializeComponent()"));
        assert!(code.contains("this.SuspendLayout();"));
        assert!(code.contains("this.ResumeLayout(false);"));
    }

    #[test]
    fn a_minimum_form_generates_its_title_and_its_size() {
        let code = code_of(&form());

        assert!(code.contains(r#"this.Text = "Formulario principal";"#));
        assert!(code.contains("this.ClientSize = new System.Drawing.Size(640, 480);"));
    }

    #[test]
    fn a_control_is_declared_as_a_field() {
        let code = code_of(&form());

        assert!(
            code.contains("private System.Windows.Forms.Button okButton;"),
            "{code}"
        );
    }

    #[test]
    fn a_control_is_created_and_added_to_the_form() {
        let code = code_of(&form());

        assert!(code.contains("this.okButton = new System.Windows.Forms.Button();"));
        assert!(code.contains("this.Controls.Add(this.okButton);"));
    }

    #[test]
    fn the_position_and_the_size_of_a_control_are_generated() {
        let code = code_of(&form());

        assert!(code.contains("this.okButton.Location = new System.Drawing.Point(8, 8);"));
        assert!(code.contains("this.okButton.Size = new System.Drawing.Size(120, 30);"));
        assert!(code.contains(r#"this.okButton.Name = "okButton";"#));
    }

    #[test]
    fn an_empty_form_generates_code_without_controls() {
        let empty = WinFormsModel::new("MainForm", "Formulario", 640, 480);

        let code = code_of(&empty);

        assert!(code.contains("private void InitializeComponent()"));
        assert!(!code.contains("Controls.Add"));
        assert!(!code.contains("private System.Windows.Forms."));
    }

    #[test]
    fn the_four_types_of_control_generate_their_own_type() {
        let mut form = WinFormsModel::new("MainForm", "Formulario", 640, 480);
        form.add_control("okButton", "Button", 0, 0, 10, 10)
            .unwrap();
        form.add_control("titleLabel", "Label", 0, 20, 10, 10)
            .unwrap();
        form.add_control("nameTextBox", "TextBox", 0, 40, 10, 10)
            .unwrap();
        form.add_control("rootPanel", "Panel", 0, 60, 10, 10)
            .unwrap();

        let code = code_of(&form);

        assert!(code.contains("private System.Windows.Forms.Button okButton;"));
        assert!(code.contains("private System.Windows.Forms.Label titleLabel;"));
        assert!(code.contains("private System.Windows.Forms.TextBox nameTextBox;"));
        assert!(code.contains("private System.Windows.Forms.Panel rootPanel;"));
    }

    #[test]
    fn the_text_of_a_control_is_generated() {
        let mut form = form();
        form.set_text("okButton", "Aceptar").unwrap();

        assert!(code_of(&form).contains(r#"this.okButton.Text = "Aceptar";"#));
    }

    #[test]
    fn a_control_without_text_does_not_generate_a_text_line() {
        let code = code_of(&form());

        assert!(!code.contains("okButton.Text"), "{code}");
    }

    #[test]
    fn a_hidden_control_generates_its_visibility() {
        let mut form = form();
        form.set_visible("okButton", false).unwrap();

        assert!(code_of(&form).contains(r#"this.okButton.Visible = "false";"#));
    }

    #[test]
    fn a_disabled_control_generates_that_it_is_disabled() {
        let mut form = form();
        form.set_enabled("okButton", false).unwrap();

        assert!(code_of(&form).contains(r#"this.okButton.Enabled = "false";"#));
    }

    #[test]
    fn a_moved_and_resized_control_generates_its_new_place() {
        let mut form = form();
        form.move_control("okButton", 40, 60).unwrap();
        form.resize_control("okButton", 200, 80).unwrap();

        let code = code_of(&form);

        assert!(code.contains("new System.Drawing.Point(40, 60)"));
        assert!(code.contains("new System.Drawing.Size(200, 80)"));
    }

    #[test]
    fn a_negative_position_is_generated_as_it_is() {
        let mut form = form();
        form.move_control("okButton", -20, -10).unwrap();

        let code = code_of(&form);

        assert!(
            code.contains("new System.Drawing.Point(-20, -10)"),
            "{code}"
        );
    }

    #[test]
    fn a_quote_in_the_text_does_not_break_the_generated_literal() {
        let mut form = form();
        form.set_text("okButton", r#"Di "hola""#).unwrap();

        let code = code_of(&form);

        assert!(
            code.contains(r#"this.okButton.Text = "Di \"hola\"";"#),
            "{code}"
        );
    }

    #[test]
    fn a_new_line_in_the_text_does_not_break_the_generated_literal() {
        let mut form = form();
        form.set_text("okButton", "linea1\nlinea2").unwrap();

        let code = code_of(&form);

        assert!(
            code.contains(r#"this.okButton.Text = "linea1\nlinea2";"#),
            "{code}"
        );
    }

    #[test]
    fn a_backslash_in_the_title_does_not_break_the_generated_literal() {
        let form = WinFormsModel::new("MainForm", r"C:\proyectos", 640, 480);

        assert!(code_of(&form).contains(r#"this.Text = "C:\\proyectos";"#));
    }

    #[test]
    fn a_control_whose_name_is_not_an_identifier_is_rejected() {
        for name in ["2botones", "mi-boton", "con espacio", "", "punto.ok"] {
            let model = DesignerModel::new(
                "MainForm",
                "Formulario",
                640,
                480,
                vec![DesignerComponent::new(name, "Button", 0, 0, 10, 10)],
            );

            let result = WinFormsGenerator.generate(&model);

            assert!(
                matches!(result, Err(CoreError::InvalidName(_))),
                "{name} deberia rechazarse"
            );
        }
    }

    #[test]
    fn a_control_named_like_a_keyword_is_generated_with_the_escape() {
        let mut form = WinFormsModel::new("MainForm", "Formulario", 640, 480);
        form.add_control("class", "Button", 0, 0, 10, 10).unwrap();
        form.set_text("class", "Aceptar").unwrap();

        let code = code_of(&form);

        assert!(code.contains("private System.Windows.Forms.Button @class;"));
        assert!(code.contains("this.@class = new System.Windows.Forms.Button();"));
        assert!(code.contains(r#"this.@class.Name = "class";"#));
    }

    #[test]
    fn the_generator_can_be_used_through_the_contract() {
        let generators: [&dyn CodeGenerator; 1] = [&WinFormsGenerator];

        for generator in generators {
            assert_eq!(
                generator.generate(form().model()).unwrap().region().begin(),
                MARKER_BEGIN
            );
        }
    }
}
