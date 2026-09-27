use crate::core::{CoreError, CoreResult};
use crate::framework::FrameworkProvider;
use crate::generation::{flag, DesignerComponent, DesignerModel, WindowSpec};
use crate::supports::Swing;

/// Propiedad de Swing con el texto que se ve en el componente.
///
/// Se llama como la API del toolkit (`setText`), no como en Windows Forms, donde
/// la propiedad se llama `Text`. Un generador de Java buscaria `text` y no
/// encontraria nada si se guardara con el nombre de la otra parte.
const TEXT: &str = "text";

/// Propiedad de Swing con si el componente se ve.
const VISIBLE: &str = "visible";

/// Propiedad de Swing con si el componente responde.
const ENABLED: &str = "enabled";

/// Modelo visual de una ventana de Swing.
///
/// Envuelve el modelo del disenador, que no sabe de frameworks, y le anade las
/// reglas de Swing: que la raiz es un `JFrame` y que los componentes tienen que
/// ser de los que Swing declara.
///
/// El modelo de dentro solo se puede tocar por aqui, y cada cambio pasa por las
/// comprobaciones, para que el generador no reciba una ventana con componentes
/// inventados.
///
/// Swing dice `component` donde Windows Forms dice `control`, y aqui tambien:
/// los metodos usan el nombre del toolkit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwingModel {
    model: DesignerModel,
    selected: Option<String>,
}

impl SwingModel {
    /// Tipo de la ventana raiz de Swing.
    pub const ROOT_TYPE: &'static str = "JFrame";

    /// Ventana vacia con su nombre de clase, su titulo y su tamano.
    pub fn new(name: impl Into<String>, title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            model: DesignerModel::new(name, title, width, height, Vec::new()),
            selected: None,
        }
    }

    /// El modelo sin reglas de Swing, que es lo que lee el generador.
    pub fn model(&self) -> &DesignerModel {
        &self.model
    }

    pub fn window(&self) -> &WindowSpec {
        self.model.window()
    }

    pub fn components(&self) -> &[DesignerComponent] {
        self.model.components()
    }

    /// Componente con ese nombre, si lo hay.
    pub fn component(&self, name: &str) -> Option<&DesignerComponent> {
        self.model.component(name)
    }

    /// Selecciona un componente y devuelve si ha podido.
    ///
    /// Seleccionar un componente que no esta es un `false`, no un error: en el
    /// disenador se hace clic en un punto, y ahi no siempre hay algo.
    pub fn select(&mut self, name: &str) -> bool {
        if self.component(name).is_none() {
            return false;
        }

        self.selected = Some(name.to_string());

        true
    }

    /// Componente seleccionado, si hay alguno.
    pub fn selection(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Quita la seleccion.
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    /// Mueve un componente a otra posicion de la ventana.
    ///
    /// No se comprueba que la posicion este dentro de la ventana: en Swing un
    /// componente se puede sacar del todo, y es el generador el que decide donde
    /// acaba cada cosa. Si no, un boton medio fuera de la ventana no se podria
    /// ni seleccionar ni mover.
    pub fn move_component(&mut self, name: &str, x: i32, y: i32) -> CoreResult<()> {
        self.component_to_change(name)?.set_position(x, y);

        Ok(())
    }

    /// Cambia el tamano de un componente.
    pub fn resize_component(&mut self, name: &str, width: u32, height: u32) -> CoreResult<()> {
        self.component_to_change(name)?.set_size(width, height);

        Ok(())
    }

    /// Cambia el nombre de un componente.
    ///
    /// El nombre es con el que se generara el codigo, asi que no puede quedar
    /// vacio ni repetido, y el nuevo nombre no puede ser el mismo de antes.
    pub fn rename_component(&mut self, name: &str, new_name: impl Into<String>) -> CoreResult<()> {
        let new_name = new_name.into();

        if new_name.trim().is_empty() {
            return Err(CoreError::InvalidName(
                "un componente no puede quedar sin nombre".into(),
            ));
        }

        if new_name == name {
            return Err(CoreError::InvalidName(format!(
                "{new_name} ya es el nombre de ese componente"
            )));
        }

        if self.component(&new_name).is_some() {
            return Err(CoreError::InvalidName(format!(
                "ya hay un componente llamado {new_name}"
            )));
        }

        self.component_to_change(name)?.rename(new_name.clone());

        // La seleccion se guarda por nombre, asi que tiene que ir con el.
        if self.selected.as_deref() == Some(name) {
            self.selected = Some(new_name);
        }

        Ok(())
    }

    /// Texto que se ve en el componente.
    ///
    /// `None` si el usuario no ha puesto ninguno, que no es lo mismo que un texto
    /// vacio: vacio es una decision, y no poner nada es no haber decidido.
    pub fn text(&self, name: &str) -> Option<&str> {
        self.component(name)?.property(TEXT)
    }

    /// Pone el texto que se ve en el componente.
    pub fn set_text(&mut self, name: &str, text: impl Into<String>) -> CoreResult<()> {
        self.component_to_change(name)?.set_property(TEXT, text);

        Ok(())
    }

    /// Si el componente se ve. Un componente sin nada puesto se ve.
    pub fn is_visible(&self, name: &str) -> bool {
        self.component(name)
            .is_some_and(|component| component.flag_property(VISIBLE, true))
    }

    /// Muestra u oculta un componente.
    pub fn set_visible(&mut self, name: &str, visible: bool) -> CoreResult<()> {
        self.component_to_change(name)?
            .set_property(VISIBLE, flag(visible));

        Ok(())
    }

    /// Si el componente responde. Un componente sin nada puesto responde.
    pub fn is_enabled(&self, name: &str) -> bool {
        self.component(name)
            .is_some_and(|component| component.flag_property(ENABLED, true))
    }

    /// Habilita o deshabilita un componente.
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> CoreResult<()> {
        self.component_to_change(name)?
            .set_property(ENABLED, flag(enabled));

        Ok(())
    }

    /// El componente a cambiar, o el error que explica que no se puede.
    fn component_to_change(&mut self, name: &str) -> CoreResult<&mut DesignerComponent> {
        self.model.component_mut(name).ok_or_else(|| {
            CoreError::NotFound(format!("la ventana no tiene ningun componente {name}"))
        })
    }

    /// Anade un componente del tipo `kind`, que tiene que ser uno de los que Swing
    /// declara.
    ///
    /// El nombre no se puede repetir, porque es el nombre con el que se generara
    /// el codigo: dos componentes con el mismo nombre no se podrian generar.
    pub fn add_component(
        &mut self,
        name: impl Into<String>,
        kind: impl Into<String>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> CoreResult<()> {
        let name = name.into();
        let kind = kind.into();

        if !self.supports_component(&kind) {
            return Err(CoreError::Unsupported(format!(
                "Swing no tiene un componente {kind}"
            )));
        }

        if self.component(&name).is_some() {
            return Err(CoreError::InvalidName(format!(
                "ya hay un componente llamado {name}"
            )));
        }

        self.model
            .add_component(DesignerComponent::new(name, kind, x, y, width, height));

        Ok(())
    }

    /// Quita un componente y lo devuelve, si estaba.
    ///
    /// Si el componente quitado era el seleccionado, la seleccion se pierde: no
    /// puede quedar apuntando a un componente que ya no esta.
    pub fn remove_component(&mut self, name: &str) -> Option<DesignerComponent> {
        let removed = self.model.remove_component(name)?;

        if self.selected.as_deref() == Some(name) {
            self.selected = None;
        }

        Some(removed)
    }

    /// Si Swing tiene un componente de ese tipo.
    fn supports_component(&self, kind: &str) -> bool {
        Swing.capabilities().components().contains(&kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ventana con un unico boton, que es el caso que se repite en casi todas las
    /// pruebas de seleccion y movimiento.
    fn window_with_a_button() -> SwingModel {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);
        window
            .add_component("okButton", "JButton", 8, 8, 120, 30)
            .unwrap();

        window
    }

    #[test]
    fn a_new_window_has_its_name_title_and_size() {
        let window = SwingModel::new("MainWindow", "Ventana principal", 640, 480);

        assert_eq!(window.window().name(), "MainWindow");
        assert_eq!(window.window().title(), "Ventana principal");
        assert_eq!(window.window().width(), 640);
        assert_eq!(window.window().height(), 480);
        assert!(window.components().is_empty());
    }

    #[test]
    fn the_four_components_of_the_minimum_window_can_be_added() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);

        for (name, kind) in [
            ("okButton", "JButton"),
            ("titleLabel", "JLabel"),
            ("nameText", "JTextField"),
            ("rootPanel", "JPanel"),
        ] {
            window
                .add_component(name, kind, 8, 8, 120, 30)
                .expect("componente de Swing");
        }

        assert_eq!(window.components().len(), 4);
        assert_eq!(window.component("okButton").unwrap().kind(), "JButton");
        assert_eq!(window.component("titleLabel").unwrap().kind(), "JLabel");
        assert_eq!(window.component("nameText").unwrap().kind(), "JTextField");
        assert_eq!(window.component("rootPanel").unwrap().kind(), "JPanel");
    }

    #[test]
    fn a_component_keeps_its_position_and_size() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);

        window
            .add_component("okButton", "JButton", 12, 34, 120, 30)
            .unwrap();

        let component = window.component("okButton").unwrap();

        assert_eq!(component.name(), "okButton");
        assert_eq!(component.x(), 12);
        assert_eq!(component.y(), 34);
        assert_eq!(component.width(), 120);
        assert_eq!(component.height(), 30);
    }

    #[test]
    fn a_window_knows_that_its_root_is_a_swing_jframe() {
        assert_eq!(SwingModel::ROOT_TYPE, "JFrame");
        assert_eq!(SwingModel::ROOT_TYPE, Swing.capabilities().root());
    }

    #[test]
    fn a_component_of_another_framework_is_rejected() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);

        let result = window.add_component("okButton", "Button", 8, 8, 120, 30);

        assert!(
            matches!(result, Err(CoreError::Unsupported(_))),
            "{result:?}"
        );
        assert!(
            window.components().is_empty(),
            "un componente rechazado no se anade"
        );
    }

    #[test]
    fn a_component_of_an_unknown_kind_is_rejected() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);

        let result = window.add_component("raro", "JHyperlink", 0, 0, 10, 10);

        assert!(matches!(result, Err(CoreError::Unsupported(_))));
        assert!(window.components().is_empty());
    }

    #[test]
    fn a_component_with_a_repeated_name_is_rejected() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);
        window
            .add_component("okButton", "JButton", 8, 8, 120, 30)
            .unwrap();

        let result = window.add_component("okButton", "JLabel", 0, 0, 10, 10);

        assert!(matches!(result, Err(CoreError::InvalidName(_))));
        assert_eq!(window.components().len(), 1, "no se anade el repetido");
    }

    #[test]
    fn a_component_can_be_removed_from_a_window() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);
        window
            .add_component("okButton", "JButton", 8, 8, 120, 30)
            .unwrap();
        window
            .add_component("titleLabel", "JLabel", 8, 40, 200, 20)
            .unwrap();

        let removed = window.remove_component("okButton");

        assert_eq!(removed.unwrap().name(), "okButton");
        assert_eq!(window.components().len(), 1);
        assert!(window.component("okButton").is_none());
    }

    #[test]
    fn removing_a_component_that_is_not_there_gives_nothing() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);

        assert!(window.remove_component("okButton").is_none());
    }

    #[test]
    fn a_component_that_is_not_there_is_not_found() {
        let window = SwingModel::new("MainWindow", "Ventana", 640, 480);

        assert!(window.component("okButton").is_none());
    }

    /// El generador Swing leera esto, y no debe saber nada de Swing.
    #[test]
    fn a_window_can_be_read_as_a_model_for_the_generator() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);
        window
            .add_component("okButton", "JButton", 8, 8, 120, 30)
            .unwrap();

        let model = window.model();

        assert_eq!(model.window().name(), "MainWindow");
        assert_eq!(model.window().title(), "Ventana");
        assert_eq!(model.components().len(), 1);
        assert_eq!(model.components()[0].kind(), "JButton");
    }

    #[test]
    fn a_new_window_has_nothing_selected() {
        let window = SwingModel::new("MainWindow", "Ventana", 640, 480);

        assert_eq!(window.selection(), None);
    }

    #[test]
    fn a_component_can_be_selected() {
        let mut window = window_with_a_button();

        assert!(window.select("okButton"));
        assert_eq!(window.selection(), Some("okButton"));
    }

    /// Seleccionar es hacer clic en un sitio, y en un sitio vacio no hay nada que
    /// seleccionar: eso no puede ser un error.
    #[test]
    fn a_component_that_is_not_there_is_not_selected() {
        let mut window = window_with_a_button();

        assert!(!window.select("titleLabel"));
        assert_eq!(window.selection(), None);
    }

    #[test]
    fn selecting_another_component_replaces_the_selection() {
        let mut window = window_with_a_button();
        window
            .add_component("titleLabel", "JLabel", 0, 40, 100, 20)
            .unwrap();
        window.select("okButton");

        window.select("titleLabel");

        assert_eq!(window.selection(), Some("titleLabel"));
    }

    #[test]
    fn the_selection_can_be_cleared() {
        let mut window = window_with_a_button();
        window.select("okButton");

        window.clear_selection();

        assert_eq!(window.selection(), None);
    }

    #[test]
    fn a_component_can_be_moved() {
        let mut window = window_with_a_button();

        window.move_component("okButton", 40, 60).unwrap();

        let component = window.component("okButton").unwrap();
        assert_eq!(component.x(), 40);
        assert_eq!(component.y(), 60);
        assert_eq!(component.width(), 120, "mover no cambia el tamano");
        assert_eq!(component.height(), 30);
    }

    #[test]
    fn a_component_can_be_resized() {
        let mut window = window_with_a_button();

        window.resize_component("okButton", 200, 80).unwrap();

        let component = window.component("okButton").unwrap();
        assert_eq!(component.width(), 200);
        assert_eq!(component.height(), 80);
        assert_eq!(component.x(), 8, "redimensionar no cambia la posicion");
        assert_eq!(component.y(), 8);
    }

    #[test]
    fn a_selected_component_can_be_moved() {
        let mut window = window_with_a_button();
        window.select("okButton");

        window.move_component("okButton", 12, 12).unwrap();

        assert_eq!(window.component("okButton").unwrap().x(), 12);
        assert_eq!(
            window.selection(),
            Some("okButton"),
            "mover no deselecciona"
        );
    }

    #[test]
    fn a_component_can_be_moved_outside_the_window() {
        let mut window = window_with_a_button();

        window.move_component("okButton", 900, 700).unwrap();

        let component = window.component("okButton").unwrap();
        assert_eq!(component.x(), 900);
        assert_eq!(component.y(), 700);
    }

    #[test]
    fn a_component_that_is_not_there_is_not_moved() {
        let mut window = window_with_a_button();

        let result = window.move_component("titleLabel", 10, 10);

        assert!(matches!(result, Err(CoreError::NotFound(_))));
    }

    #[test]
    fn a_component_that_is_not_there_is_not_resized() {
        let mut window = window_with_a_button();

        let result = window.resize_component("titleLabel", 10, 10);

        assert!(matches!(result, Err(CoreError::NotFound(_))));
    }

    /// La seleccion se guarda por nombre, asi que no puede quedar apuntando a un
    /// componente que ya no esta.
    #[test]
    fn removing_the_selected_component_clears_the_selection() {
        let mut window = window_with_a_button();
        window.select("okButton");

        window.remove_component("okButton");

        assert_eq!(window.selection(), None);
    }

    #[test]
    fn removing_another_component_keeps_the_selection() {
        let mut window = window_with_a_button();
        window
            .add_component("titleLabel", "JLabel", 0, 40, 100, 20)
            .unwrap();
        window.select("okButton");

        window.remove_component("titleLabel");

        assert_eq!(window.selection(), Some("okButton"));
    }

    #[test]
    fn a_component_can_be_renamed() {
        let mut window = window_with_a_button();

        window.rename_component("okButton", "acceptButton").unwrap();

        assert!(window.component("okButton").is_none());
        assert_eq!(window.component("acceptButton").unwrap().kind(), "JButton");
    }

    /// La seleccion se guarda por nombre, asi que tiene que ir con el componente.
    #[test]
    fn renaming_a_selected_component_keeps_it_selected() {
        let mut window = window_with_a_button();
        window.select("okButton");

        window.rename_component("okButton", "acceptButton").unwrap();

        assert_eq!(window.selection(), Some("acceptButton"));
    }

    #[test]
    fn a_component_that_is_not_there_is_not_renamed() {
        let mut window = window_with_a_button();

        let result = window.rename_component("titleLabel", "otro");

        assert!(matches!(result, Err(CoreError::NotFound(_))));
    }

    #[test]
    fn a_component_is_not_renamed_to_a_name_that_is_taken() {
        let mut window = window_with_a_button();
        window
            .add_component("titleLabel", "JLabel", 0, 40, 100, 20)
            .unwrap();

        let result = window.rename_component("okButton", "titleLabel");

        assert!(matches!(result, Err(CoreError::InvalidName(_))));
        assert!(window.component("okButton").is_some(), "no se renombra");
    }

    #[test]
    fn a_component_is_not_renamed_to_its_own_name() {
        let mut window = window_with_a_button();

        let result = window.rename_component("okButton", "okButton");

        assert!(matches!(result, Err(CoreError::InvalidName(_))));
    }

    #[test]
    fn a_component_is_not_renamed_to_an_empty_name() {
        let mut window = window_with_a_button();

        let result = window.rename_component("okButton", "");

        assert!(matches!(result, Err(CoreError::InvalidName(_))));
    }

    #[test]
    fn the_text_of_a_component_can_be_set() {
        let mut window = window_with_a_button();

        window.set_text("okButton", "Aceptar").unwrap();

        assert_eq!(window.text("okButton"), Some("Aceptar"));
    }

    /// No poner texto y ponerlo vacio son cosas distintas: vacio es una decision
    /// del usuario, y no poner nada es no haber decidido.
    #[test]
    fn a_component_without_text_has_none() {
        let window = window_with_a_button();

        assert_eq!(window.text("okButton"), None);
    }

    #[test]
    fn the_text_of_a_component_can_be_changed() {
        let mut window = window_with_a_button();
        window.set_text("okButton", "Aceptar").unwrap();

        window.set_text("okButton", "Cancelar").unwrap();

        assert_eq!(window.text("okButton"), Some("Cancelar"));
    }

    #[test]
    fn the_text_of_a_component_can_be_emptied() {
        let mut window = window_with_a_button();
        window.set_text("okButton", "Aceptar").unwrap();

        window.set_text("okButton", "").unwrap();

        assert_eq!(window.text("okButton"), Some(""));
    }

    #[test]
    fn a_component_is_visible_and_enabled_by_default() {
        let window = window_with_a_button();

        assert!(window.is_visible("okButton"));
        assert!(window.is_enabled("okButton"));
    }

    #[test]
    fn a_component_can_be_hidden() {
        let mut window = window_with_a_button();

        window.set_visible("okButton", false).unwrap();

        assert!(!window.is_visible("okButton"));
    }

    #[test]
    fn a_hidden_component_can_be_shown_again() {
        let mut window = window_with_a_button();
        window.set_visible("okButton", false).unwrap();

        window.set_visible("okButton", true).unwrap();

        assert!(window.is_visible("okButton"));
    }

    #[test]
    fn a_component_can_be_disabled() {
        let mut window = window_with_a_button();

        window.set_enabled("okButton", false).unwrap();

        assert!(!window.is_enabled("okButton"));
    }

    #[test]
    fn a_property_of_a_component_that_is_not_there_is_not_changed() {
        let mut window = window_with_a_button();

        assert!(matches!(
            window.set_text("titleLabel", "Hola"),
            Err(CoreError::NotFound(_))
        ));
        assert!(matches!(
            window.set_visible("titleLabel", false),
            Err(CoreError::NotFound(_))
        ));
        assert!(matches!(
            window.set_enabled("titleLabel", false),
            Err(CoreError::NotFound(_))
        ));
    }

    #[test]
    fn a_property_is_not_kept_twice() {
        let mut window = window_with_a_button();
        window.set_text("okButton", "Aceptar").unwrap();

        window.set_text("okButton", "Cancelar").unwrap();

        let properties = window.component("okButton").unwrap().properties();

        assert_eq!(properties, &[("text".to_string(), "Cancelar".to_string())]);
    }

    /// Swing escribe las propiedades como las llama su propia API: `setText`, no
    /// `Text`. Un generador de Java no puede buscarlas en `Text`.
    #[test]
    fn the_properties_of_a_component_use_java_names() {
        let mut window = window_with_a_button();
        window.set_text("okButton", "Aceptar").unwrap();
        window.set_visible("okButton", false).unwrap();
        window.set_enabled("okButton", false).unwrap();

        let properties = window.component("okButton").unwrap().properties();

        assert!(properties.contains(&("text".to_string(), "Aceptar".to_string())));
        assert!(properties.contains(&("visible".to_string(), "false".to_string())));
        assert!(properties.contains(&("enabled".to_string(), "false".to_string())));
    }

    /// Lo que hace el panel de propiedades: los seis cambios, y el modelo detras
    /// de los seis.
    #[test]
    fn everything_the_properties_panel_edits_is_reflected_in_the_model() {
        let mut window = window_with_a_button();

        window.rename_component("okButton", "acceptButton").unwrap();
        window.set_text("acceptButton", "Aceptar").unwrap();
        window.move_component("acceptButton", 20, 30).unwrap();
        window.resize_component("acceptButton", 150, 40).unwrap();
        window.set_visible("acceptButton", false).unwrap();
        window.set_enabled("acceptButton", false).unwrap();

        let component = window.model().component("acceptButton").unwrap();

        assert_eq!(component.name(), "acceptButton");
        assert_eq!(window.text("acceptButton"), Some("Aceptar"));
        assert_eq!((component.x(), component.y()), (20, 30));
        assert_eq!((component.width(), component.height()), (150, 40));
        assert!(!window.is_visible("acceptButton"));
        assert!(!window.is_enabled("acceptButton"));
    }

    #[test]
    fn the_edited_properties_are_in_the_model_the_generator_reads() {
        let mut window = window_with_a_button();
        window.set_text("okButton", "Aceptar").unwrap();
        window.set_visible("okButton", false).unwrap();
        window.set_enabled("okButton", false).unwrap();

        let properties = window.model().component("okButton").unwrap().properties();

        assert!(properties.contains(&("text".to_string(), "Aceptar".to_string())));
        assert!(properties.contains(&("visible".to_string(), "false".to_string())));
        assert!(properties.contains(&("enabled".to_string(), "false".to_string())));
    }
}
