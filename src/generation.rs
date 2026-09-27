use crate::core::{CoreError, CoreResult};

/// El generador de un framework concreto.
mod swing;
mod winforms;
pub use swing::{SwingGenerator, BUILD_METHOD};
pub use winforms::WinFormsGenerator;

/// Marcador con el que empieza la zona que escribe MiniIDE en el archivo del
/// disenador.
///
/// Vive aqui y no en la plantilla porque es el generador quien decide donde
/// escribe, y los dos tienen que usar el mismo texto: si no, el generador
/// escribiria una zona que la plantilla no tiene.
///
/// El mismo texto sirve para los dos generadores porque `//` es un comentario
/// tanto en C# como en Java.
pub const MARKER_BEGIN: &str = "// <MiniIDE>";

/// Marcador con el que termina la zona que escribe MiniIDE.
pub const MARKER_END: &str = "// </MiniIDE>";

/// La ventana o el formulario raiz del disenador.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSpec {
    name: String,
    title: String,
    width: u32,
    height: u32,
}

impl WindowSpec {
    pub fn new(name: impl Into<String>, title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            name: name.into(),
            title: title.into(),
            width,
            height,
        }
    }

    /// Nombre de la clase que genera el codigo.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Texto que se ve en el titulo de la ventana.
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

/// Un control colocado en la ventana.
///
/// `kind` es el nombre del control en el framework, que es lo que declara
/// `FrameworkCapabilities::components`, y no el nombre de un lenguaje.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignerComponent {
    name: String,
    kind: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    properties: Vec<(String, String)>,
}

impl DesignerComponent {
    pub fn new(
        name: impl Into<String>,
        kind: impl Into<String>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            name: name.into(),
            kind: kind.into(),
            x,
            y,
            width,
            height,
            properties: Vec::new(),
        }
    }

    /// Anade una propiedad. Las propiedades se guardan en orden, para que el
    /// codigo generado sea siempre el mismo.
    pub fn with_property(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.push((name.into(), value.into()));

        self
    }

    /// Nombre del control en el codigo generado.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Tipo del control en el framework.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Propiedades, en el orden en que se anadieron.
    pub fn properties(&self) -> &[(String, String)] {
        &self.properties
    }

    /// Mueve el control a otra posicion.
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// Cambia el tamano del control.
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Cambia el nombre del control.
    pub fn rename(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Pone una propiedad, o cambia la que ya habia con ese nombre.
    ///
    /// No se guardan dos propiedades con el mismo nombre: la segunda
    /// machacaria a la primera al generar el codigo, y el generador tendria que
    /// decidir cual de las dos gana.
    pub fn set_property(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        let value = value.into();

        match self
            .properties
            .iter_mut()
            .find(|(property, _)| *property == name)
        {
            Some(entry) => entry.1 = value,
            None => self.properties.push((name, value)),
        }
    }

    /// Valor de una propiedad, si el control la tiene.
    pub fn property(&self, name: &str) -> Option<&str> {
        self.properties
            .iter()
            .find(|(property, _)| property == name)
            .map(|(_, value)| value.as_str())
    }

    /// Valor de una propiedad que es un si o un no.
    ///
    /// Una propiedad que no esta puesta vale `default_value`, que es lo que hace
    /// el control cuando MiniIDE no dice nada de ella.
    pub fn flag_property(&self, name: &str, default_value: bool) -> bool {
        match self.property(name) {
            Some("true") => true,
            Some("false") => false,
            _ => default_value,
        }
    }
}

/// Un booleano como lo escribe el toolkit: en minusculas y como texto.
///
/// Los dos toolkit lo escriben igual, asi que es una regla del modelo y no de un
/// framework: C# y Java ponen `true` y `false`.
pub fn flag(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

/// La interfaz tal y como la ve el disenador, sin nada de codigo.
///
/// No sabe de ningun lenguaje: los controles son los del framework y el
/// generador es quien los traduce. Es la entrada de la generacion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignerModel {
    window: WindowSpec,
    components: Vec<DesignerComponent>,
}

impl DesignerModel {
    pub fn new(
        name: impl Into<String>,
        title: impl Into<String>,
        width: u32,
        height: u32,
        components: Vec<DesignerComponent>,
    ) -> Self {
        Self {
            window: WindowSpec::new(name, title, width, height),
            components,
        }
    }

    pub fn window(&self) -> &WindowSpec {
        &self.window
    }

    pub fn components(&self) -> &[DesignerComponent] {
        &self.components
    }

    /// Anade un control al final.
    ///
    /// Quien decide si el control vale para su framework es el framework, no
    /// el modelo: aqui solo se guarda.
    pub fn add_component(&mut self, component: DesignerComponent) {
        self.components.push(component);
    }

    /// Quita el control con ese nombre y lo devuelve, si estaba.
    pub fn remove_component(&mut self, name: &str) -> Option<DesignerComponent> {
        let index = self
            .components
            .iter()
            .position(|control| control.name() == name)?;

        Some(self.components.remove(index))
    }

    /// Control con ese nombre, si lo hay.
    pub fn component(&self, name: &str) -> Option<&DesignerComponent> {
        self.components
            .iter()
            .find(|control| control.name() == name)
    }

    /// Control con ese nombre, para poder cambiarlo, si lo hay.
    pub fn component_mut(&mut self, name: &str) -> Option<&mut DesignerComponent> {
        self.components
            .iter_mut()
            .find(|control| control.name() == name)
    }
}

/// La zona del archivo que genera el disenador, entre un marcador de inicio y
/// uno de fin.
///
/// El codigo del usuario se mantiene fuera de la zona, y por eso el generador
/// nunca devuelve un archivo entero sino esto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedRegion {
    begin: String,
    end: String,
    content: String,
}

impl GeneratedRegion {
    pub fn new(
        begin: impl Into<String>,
        end: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            begin: begin.into(),
            end: end.into(),
            content: content.into(),
        }
    }

    /// Marcador con el que empieza la zona.
    pub fn begin(&self) -> &str {
        &self.begin
    }

    /// Marcador con el que termina la zona.
    pub fn end(&self) -> &str {
        &self.end
    }

    /// Codigo generado, sin los marcadores.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// La zona completa, con sus marcadores.
    pub fn block(&self) -> String {
        format!("{}\n{}\n{}", self.begin, self.content, self.end)
    }

    /// Pone la zona en `source` y devuelve el resultado.
    ///
    /// Si ya habia una zona, la sustituye; si no, la anade al final. El codigo
    /// de fuera de la zona se queda como estaba, que es lo unico que garantiza
    /// que el generador no pise lo que ha escrito el usuario.
    ///
    /// Devuelve `CoreError::MalformedSource` si el archivo tiene el marcador de
    /// inicio pero no el de fin: en ese caso no se toca el archivo, porque
    /// cortarlo por donde toca se llevaria codigo del usuario.
    pub fn apply_to(&self, source: &str) -> CoreResult<String> {
        match (source.find(&self.begin), source.find(&self.end)) {
            (None, _) => Ok(format!("{}{}", source, self.separated_block())),
            (Some(_), None) => Err(CoreError::MalformedSource(format!(
                "{} sin {}",
                self.begin, self.end
            ))),
            (Some(begin), Some(end)) if end < begin => Err(CoreError::MalformedSource(format!(
                "{} despues de {}",
                self.end, self.begin
            ))),
            (Some(begin), Some(end)) => {
                let mut result = String::with_capacity(source.len() + self.content.len());

                result.push_str(&source[..begin]);
                result.push_str(&self.block());
                result.push_str(&source[end + self.end.len()..]);

                Ok(result)
            }
        }
    }

    /// La zona con una linea en blanco delante, para no pegarla al final.
    fn separated_block(&self) -> String {
        if self.content.is_empty() {
            return format!("\n{}\n", self.begin);
        }

        format!("\n{}\n", self.block())
    }
}

/// El codigo que produce un generador, con la zona que lo delimita.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedCode {
    region: GeneratedRegion,
}

impl GeneratedCode {
    pub fn new(
        begin: impl Into<String>,
        end: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            region: GeneratedRegion::new(begin, end, content),
        }
    }

    pub fn region(&self) -> &GeneratedRegion {
        &self.region
    }
}

/// Quien traduce el modelo del disenador al codigo de un framework.
///
/// No recibe ningun lenguaje: el framework ya esta en el tipo que implementa el
/// contrato, y el core elige el generador a partir del framework del proyecto.
pub trait CodeGenerator {
    /// Genera el codigo de `model` dentro de su zona delimitada.
    fn generate(&self, model: &DesignerModel) -> CoreResult<GeneratedCode>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::CoreResult;

    struct FakeGenerator {
        begin: &'static str,
        end: &'static str,
        refuse: bool,
    }

    impl FakeGenerator {
        fn winforms() -> Self {
            Self {
                begin: "// <MiniIDE>",
                end: "// </MiniIDE>",
                refuse: false,
            }
        }

        fn swing() -> Self {
            Self {
                begin: "/* <MiniIDE> */",
                end: "/* </MiniIDE> */",
                refuse: false,
            }
        }
    }

    impl CodeGenerator for FakeGenerator {
        fn generate(&self, model: &DesignerModel) -> CoreResult<GeneratedCode> {
            if self.refuse {
                return Err(CoreError::Unsupported("sin soporte".to_string()));
            }

            let mut content = format!("// ventana {}", model.window().name());

            for component in model.components() {
                content.push_str(&format!("\n// {} {}", component.name(), component.kind()));
            }

            Ok(GeneratedCode::new(self.begin, self.end, content))
        }
    }

    fn model() -> DesignerModel {
        DesignerModel::new(
            "MainForm",
            "Formulario principal",
            640,
            480,
            vec![DesignerComponent::new("okButton", "Button", 8, 8, 120, 30)],
        )
    }

    #[test]
    fn the_model_exposes_its_window_and_its_components() {
        let model = model();

        assert_eq!(model.window().name(), "MainForm");
        assert_eq!(model.window().title(), "Formulario principal");
        assert_eq!(model.window().width(), 640);
        assert_eq!(model.window().height(), 480);
        assert_eq!(model.components().len(), 1);
        assert_eq!(model.components()[0].name(), "okButton");
        assert_eq!(model.components()[0].kind(), "Button");
    }

    #[test]
    fn the_generated_code_carries_its_region() {
        let code = FakeGenerator::winforms().generate(&model()).unwrap();

        assert_eq!(code.region().begin(), "// <MiniIDE>");
        assert_eq!(code.region().end(), "// </MiniIDE>");
        assert!(code.region().content().contains("ventana MainForm"));
    }

    #[test]
    fn the_generated_content_comes_from_the_model() {
        let code = FakeGenerator::swing().generate(&model()).unwrap();

        assert!(code.region().content().contains("okButton Button"));
    }

    #[test]
    fn a_region_is_appended_to_a_source_that_does_not_have_it() {
        let code = FakeGenerator::winforms().generate(&model()).unwrap();

        let result = code.region().apply_to("class MainForm { }\n").unwrap();

        assert!(result.starts_with("class MainForm { }\n"));
        assert!(result.contains("// <MiniIDE>"));
        assert!(result.contains("// ventana MainForm"));
        assert!(result.contains("// </MiniIDE>"));
    }

    #[test]
    fn a_region_replaces_the_previous_region_and_keeps_the_user_code() {
        let generator = FakeGenerator::winforms();
        let code = generator.generate(&model()).unwrap();
        let source = "using System;\n\nclass MainForm\n{\n    // codigo del usuario\n}\n";

        let first = code.region().apply_to(source).unwrap();
        let second = code.region().apply_to(&first).unwrap();

        assert!(second.contains("using System;"));
        assert!(second.contains("// codigo del usuario"));
        assert_eq!(first, second);
    }

    #[test]
    fn a_region_that_changes_replaces_only_its_own_content() {
        let first = FakeGenerator::winforms().generate(&model()).unwrap();
        let bigger = DesignerModel::new(
            "MainForm",
            "Formulario principal",
            640,
            480,
            vec![DesignerComponent::new(
                "cancelButton",
                "Button",
                140,
                8,
                120,
                30,
            )],
        );
        let second = FakeGenerator::winforms().generate(&bigger).unwrap();
        let source = "class MainForm { }\n";

        let applied = first.region().apply_to(source).unwrap();
        let updated = second.region().apply_to(&applied).unwrap();

        assert!(updated.contains("cancelButton"));
        assert!(!updated.contains("okButton"));
        assert!(updated.starts_with("class MainForm { }\n"));
    }

    #[test]
    fn a_source_with_a_begin_marker_and_no_end_marker_is_not_modified() {
        let code = FakeGenerator::winforms().generate(&model()).unwrap();
        let source = "class MainForm\n{\n    // <MiniIDE>\n    algo a medias\n}\n";

        let result = code.region().apply_to(source);

        assert!(matches!(result, Err(CoreError::MalformedSource(_))));
    }

    #[test]
    fn a_generator_can_refuse_to_generate() {
        let refusing = FakeGenerator {
            begin: "// <MiniIDE>",
            end: "// </MiniIDE>",
            refuse: true,
        };

        let result = refusing.generate(&model());

        assert!(matches!(result, Err(CoreError::Unsupported(_))));
    }

    #[test]
    fn several_generators_can_be_queried_through_the_same_interface() {
        let generators: [&dyn CodeGenerator; 2] =
            [&FakeGenerator::winforms(), &FakeGenerator::swing()];

        let begins: Vec<String> = generators
            .iter()
            .map(|g| g.generate(&model()).unwrap().region().begin().to_string())
            .collect();

        assert_eq!(begins, vec!["// <MiniIDE>", "/* <MiniIDE> */"]);
    }
}
