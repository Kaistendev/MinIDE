use crate::core::{CoreError, CoreResult};
use crate::framework::FrameworkProvider;
use crate::generation::{flag, DesignerComponent, DesignerModel, WindowSpec};
use crate::supports::WinForms;

/// Propiedad de WinForms con el texto que se ve en el control.
const TEXT: &str = "Text";

/// Propiedad de WinForms con la visibilidad del control.
const VISIBLE: &str = "Visible";

/// Propiedad de WinForms con si el control responde.
const ENABLED: &str = "Enabled";

/// Modelo visual de un formulario de Windows Forms.
///
/// Envuelve el modelo del disenador, que no sabe de frameworks, y le anade las
/// reglas de WinForms: que la raiz es un `Form` y que los controles tienen que
/// ser de los que WinForms tiene.
///
/// El modelo de dentro solo se puede tocar por aqui, y cada cambio pasa por las
/// comprobaciones, para que el generador no reciba un formulario con controles
/// inventados.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WinFormsModel {
    model: DesignerModel,
    selected: Option<String>,
}

impl WinFormsModel {
    /// Tipo de la ventana raiz de WinForms.
    pub const ROOT_TYPE: &'static str = "Form";

    /// Formulario vacio con su nombre de clase, su titulo y su tamano.
    pub fn new(name: impl Into<String>, title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            model: DesignerModel::new(name, title, width, height, Vec::new()),
            selected: None,
        }
    }

    /// El modelo sin reglas de WinForms, que es lo que lee el generador.
    pub fn model(&self) -> &DesignerModel {
        &self.model
    }

    pub fn window(&self) -> &WindowSpec {
        self.model.window()
    }

    pub fn components(&self) -> &[DesignerComponent] {
        self.model.components()
    }

    /// Control con ese nombre, si lo hay.
    pub fn component(&self, name: &str) -> Option<&DesignerComponent> {
        self.model
            .components()
            .iter()
            .find(|control| control.name() == name)
    }

    /// Selecciona un control y devuelve si ha podido.
    ///
    /// Seleccionar un control que no esta es un `false`, no un error: en el
    /// disenador se hace clic en un punto, y ahi no siempre hay algo.
    pub fn select(&mut self, name: &str) -> bool {
        if self.component(name).is_none() {
            return false;
        }

        self.selected = Some(name.to_string());

        true
    }

    /// Control seleccionado, si hay alguno.
    pub fn selection(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Quita la seleccion.
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    /// Mueve un control a otra posicion del formulario.
    pub fn move_control(&mut self, name: &str, x: i32, y: i32) -> CoreResult<()> {
        self.control_to_change(name)?.set_position(x, y);

        Ok(())
    }

    /// Cambia el tamano de un control.
    pub fn resize_control(&mut self, name: &str, width: u32, height: u32) -> CoreResult<()> {
        self.control_to_change(name)?.set_size(width, height);

        Ok(())
    }

    /// El control a cambiar, o el error que explica que no se puede.
    fn control_to_change(&mut self, name: &str) -> CoreResult<&mut DesignerComponent> {
        self.model.component_mut(name).ok_or_else(|| {
            CoreError::NotFound(format!("el formulario no tiene ningun control {name}"))
        })
    }

    /// Cambia el nombre de un control.
    ///
    /// El nombre es el campo que generara el codigo, asi que no puede quedar
    /// vacio ni repetido, y el nuevo nombre no puede ser el mismo de antes.
    pub fn rename_control(&mut self, name: &str, new_name: impl Into<String>) -> CoreResult<()> {
        let new_name = new_name.into();

        if new_name.trim().is_empty() {
            return Err(CoreError::InvalidName(
                "un control no puede quedar sin nombre".into(),
            ));
        }

        if new_name == name {
            return Err(CoreError::InvalidName(format!(
                "{new_name} ya es el nombre de ese control"
            )));
        }

        if self.component(&new_name).is_some() {
            return Err(CoreError::InvalidName(format!(
                "ya hay un control llamado {new_name}"
            )));
        }

        self.control_to_change(name)?.rename(new_name.clone());

        // La seleccion se guarda por nombre, asi que tiene que ir con el.
        if self.selected.as_deref() == Some(name) {
            self.selected = Some(new_name);
        }

        Ok(())
    }

    /// Texto que se ve en el control.
    ///
    /// `None` si el usuario no ha puesto ninguno, que no es lo mismo que un
    /// texto vacio: vacio es una decision, y no poner nada es no haber decidido.
    pub fn text(&self, name: &str) -> Option<&str> {
        self.component(name)?.property(TEXT)
    }

    /// Pone el texto que se ve en el control.
    pub fn set_text(&mut self, name: &str, text: impl Into<String>) -> CoreResult<()> {
        self.control_to_change(name)?.set_property(TEXT, text);

        Ok(())
    }

    /// Si el control se ve. Un control sin nada puesto se ve.
    pub fn is_visible(&self, name: &str) -> bool {
        self.component(name)
            .is_some_and(|control| control.flag_property(VISIBLE, true))
    }

    /// Muestra u oculta un control.
    pub fn set_visible(&mut self, name: &str, visible: bool) -> CoreResult<()> {
        self.control_to_change(name)?
            .set_property(VISIBLE, flag(visible));

        Ok(())
    }

    /// Si el control responde. Un control sin nada puesto responde.
    pub fn is_enabled(&self, name: &str) -> bool {
        self.component(name)
            .is_some_and(|control| control.flag_property(ENABLED, true))
    }

    /// Habilita o deshabilita un control.
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> CoreResult<()> {
        self.control_to_change(name)?
            .set_property(ENABLED, flag(enabled));

        Ok(())
    }

    /// Anade un control del tipo `kind`, que tiene que ser uno de los que
    /// WinForms declara.
    ///
    /// El nombre no se puede repetir, porque es el nombre del campo que
    /// generara el codigo: dos controles con el mismo nombre no se podrian
    /// generar.
    pub fn add_control(
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

        if !self.supports_control(&kind) {
            return Err(CoreError::Unsupported(format!(
                "WinForms no tiene un control {kind}"
            )));
        }

        if self.component(&name).is_some() {
            return Err(CoreError::InvalidName(format!(
                "ya hay un control llamado {name}"
            )));
        }

        self.model
            .add_component(DesignerComponent::new(name, kind, x, y, width, height));

        Ok(())
    }

    /// Quita un control y lo devuelve, si estaba.
    ///
    /// Si el control quitaro era el seleccionado, la seleccion se pierde: no
    /// puede quedar apuntando a un control que ya no esta.
    pub fn remove_control(&mut self, name: &str) -> Option<DesignerComponent> {
        let removed = self.model.remove_component(name)?;

        if self.selected.as_deref() == Some(name) {
            self.selected = None;
        }

        Some(removed)
    }

    /// Si WinForms tiene un control de ese tipo.
    fn supports_control(&self, kind: &str) -> bool {
        WinForms.capabilities().components().contains(&kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::TextPosition;

    /// Formulario con un unico boton, que es el caso que se repite en casi
    /// todas las pruebas de seleccion y movimiento.
    fn form_with_a_button() -> WinFormsModel {
        let mut form = WinFormsModel::new("MainForm", "Formulario", 640, 480);
        form.add_control("okButton", "Button", 8, 8, 120, 30)
            .unwrap();

        form
    }

    #[test]
    fn a_new_form_has_its_name_title_and_size() {
        let form = WinFormsModel::new("MainForm", "Formulario principal", 640, 480);

        assert_eq!(form.window().name(), "MainForm");
        assert_eq!(form.window().title(), "Formulario principal");
        assert_eq!(form.window().width(), 640);
        assert_eq!(form.window().height(), 480);
        assert!(form.components().is_empty());
    }

    #[test]
    fn the_four_controls_of_the_minimum_form_can_be_added() {
        let mut form = WinFormsModel::new("MainForm", "Formulario", 640, 480);

        for (name, kind) in [
            ("okButton", "Button"),
            ("titleLabel", "Label"),
            ("nameTextBox", "TextBox"),
            ("rootPanel", "Panel"),
        ] {
            form.add_control(name, kind, 8, 8, 120, 30).unwrap();
        }

        assert_eq!(form.components().len(), 4);
        assert_eq!(form.component("okButton").unwrap().kind(), "Button");
        assert_eq!(form.component("nameTextBox").unwrap().kind(), "TextBox");
    }

    #[test]
    fn a_control_keeps_its_position_and_size() {
        let mut form = WinFormsModel::new("MainForm", "Formulario", 640, 480);

        form.add_control("okButton", "Button", 12, 34, 120, 30)
            .unwrap();

        let control = form.component("okButton").unwrap();

        assert_eq!(control.x(), 12);
        assert_eq!(control.y(), 34);
        assert_eq!(control.width(), 120);
        assert_eq!(control.height(), 30);
    }

    #[test]
    fn a_control_of_another_framework_is_rejected() {
        let mut form = WinFormsModel::new("MainForm", "Formulario", 640, 480);

        let result = form.add_control("okButton", "JButton", 8, 8, 120, 30);

        assert!(
            matches!(result, Err(CoreError::Unsupported(_))),
            "{result:?}"
        );
        assert!(
            form.components().is_empty(),
            "un control rechazado no se anade"
        );
    }

    #[test]
    fn a_control_of_an_unknown_kind_is_rejected() {
        let mut form = WinFormsModel::new("MainForm", "Formulario", 640, 480);

        let result = form.add_control("raro", "HyperLink", 0, 0, 10, 10);

        assert!(matches!(result, Err(CoreError::Unsupported(_))));
    }

    #[test]
    fn a_control_with_a_repeated_name_is_rejected() {
        let mut form = WinFormsModel::new("MainForm", "Formulario", 640, 480);
        form.add_control("okButton", "Button", 8, 8, 120, 30)
            .unwrap();

        let result = form.add_control("okButton", "Label", 0, 0, 10, 10);

        assert!(matches!(result, Err(CoreError::InvalidName(_))));
        assert_eq!(form.components().len(), 1, "no se anade el repetido");
    }

    #[test]
    fn a_control_can_be_removed_from_a_form() {
        let mut form = WinFormsModel::new("MainForm", "Formulario", 640, 480);
        form.add_control("okButton", "Button", 8, 8, 120, 30)
            .unwrap();
        form.add_control("titleLabel", "Label", 8, 40, 200, 20)
            .unwrap();

        let removed = form.remove_control("okButton");

        assert_eq!(removed.unwrap().name(), "okButton");
        assert_eq!(form.components().len(), 1);
        assert!(form.component("okButton").is_none());
    }

    #[test]
    fn removing_a_control_that_is_not_there_gives_nothing() {
        let mut form = WinFormsModel::new("MainForm", "Formulario", 640, 480);

        assert!(form.remove_control("okButton").is_none());
    }

    #[test]
    fn a_control_that_is_not_there_is_not_found() {
        let form = WinFormsModel::new("MainForm", "Formulario", 640, 480);

        assert!(form.component("okButton").is_none());
    }

    #[test]
    fn a_form_can_be_read_as_a_model_for_the_generator() {
        let mut form = WinFormsModel::new("MainForm", "Formulario", 640, 480);
        form.add_control("okButton", "Button", 8, 8, 120, 30)
            .unwrap();

        let model = form.model();

        assert_eq!(model.window().name(), "MainForm");
        assert_eq!(model.components().len(), 1);
        assert_eq!(model.components()[0].kind(), "Button");
    }

    #[test]
    fn a_form_knows_that_its_root_is_a_winforms_form() {
        assert_eq!(WinFormsModel::ROOT_TYPE, "Form");
        assert_eq!(WinFormsModel::ROOT_TYPE, WinForms.capabilities().root());
    }

    #[test]
    fn the_model_of_a_form_can_be_rebuilt_as_a_designer_model() {
        let form = WinFormsModel::new("MainForm", "Formulario", 640, 480);
        let model = DesignerModel::new(
            form.window().name(),
            form.window().title(),
            form.window().width(),
            form.window().height(),
            Vec::new(),
        );

        assert_eq!(model.window().name(), "MainForm");
        assert_eq!(model.window().title(), "Formulario");
    }

    #[test]
    fn a_control_name_is_used_as_is() {
        let mut form = WinFormsModel::new("MainForm", "Formulario", 640, 480);
        form.add_control("okButton", "Button", 0, 0, 10, 10)
            .unwrap();

        assert_eq!(form.component("okButton").unwrap().name(), "okButton");
    }

    #[test]
    fn a_control_carries_the_position_of_the_designer_model() {
        let model = DesignerModel::new(
            "MainForm",
            "Formulario",
            100,
            100,
            vec![DesignerComponent::new("okButton", "Button", 4, 5, 60, 20)],
        );

        assert_eq!(model.components()[0].x(), 4);
        assert_eq!(model.components()[0].y(), 5);
        let _ = TextPosition::new(0, 0);
    }

    #[test]
    fn a_new_form_has_nothing_selected() {
        let form = WinFormsModel::new("MainForm", "Formulario", 640, 480);

        assert_eq!(form.selection(), None);
    }

    #[test]
    fn a_control_can_be_selected() {
        let mut form = form_with_a_button();

        assert!(form.select("okButton"));
        assert_eq!(form.selection(), Some("okButton"));
    }

    #[test]
    fn a_control_that_is_not_there_is_not_selected() {
        let mut form = form_with_a_button();

        assert!(!form.select("titleLabel"));
        assert_eq!(form.selection(), None);
    }

    #[test]
    fn selecting_another_control_replaces_the_selection() {
        let mut form = form_with_a_button();
        form.add_control("titleLabel", "Label", 0, 40, 100, 20)
            .unwrap();
        form.select("okButton");

        form.select("titleLabel");

        assert_eq!(form.selection(), Some("titleLabel"));
    }

    #[test]
    fn the_selection_can_be_cleared() {
        let mut form = form_with_a_button();
        form.select("okButton");

        form.clear_selection();

        assert_eq!(form.selection(), None);
    }

    #[test]
    fn a_control_can_be_moved() {
        let mut form = form_with_a_button();

        form.move_control("okButton", 40, 60).unwrap();

        let control = form.component("okButton").unwrap();
        assert_eq!(control.x(), 40);
        assert_eq!(control.y(), 60);
        assert_eq!(control.width(), 120, "mover no cambia el tamano");
        assert_eq!(control.height(), 30);
    }

    #[test]
    fn a_control_can_be_resized() {
        let mut form = form_with_a_button();

        form.resize_control("okButton", 200, 80).unwrap();

        let control = form.component("okButton").unwrap();
        assert_eq!(control.width(), 200);
        assert_eq!(control.height(), 80);
        assert_eq!(control.x(), 8, "redimensionar no cambia la posicion");
        assert_eq!(control.y(), 8);
    }

    #[test]
    fn a_selected_control_can_be_moved() {
        let mut form = form_with_a_button();
        form.select("okButton");

        form.move_control("okButton", 12, 12).unwrap();

        assert_eq!(form.component("okButton").unwrap().x(), 12);
    }

    #[test]
    fn a_control_that_is_not_there_is_not_moved() {
        let mut form = form_with_a_button();

        let result = form.move_control("titleLabel", 10, 10);

        assert!(matches!(result, Err(CoreError::NotFound(_))));
    }

    #[test]
    fn a_control_that_is_not_there_is_not_resized() {
        let mut form = form_with_a_button();

        let result = form.resize_control("titleLabel", 10, 10);

        assert!(matches!(result, Err(CoreError::NotFound(_))));
    }

    #[test]
    fn removing_the_selected_control_clears_the_selection() {
        let mut form = form_with_a_button();
        form.select("okButton");

        form.remove_control("okButton");

        assert_eq!(form.selection(), None);
    }

    #[test]
    fn removing_another_control_keeps_the_selection() {
        let mut form = form_with_a_button();
        form.add_control("titleLabel", "Label", 0, 40, 100, 20)
            .unwrap();
        form.select("okButton");

        form.remove_control("titleLabel");

        assert_eq!(form.selection(), Some("okButton"));
    }

    #[test]
    fn a_control_can_be_renamed() {
        let mut form = form_with_a_button();

        form.rename_control("okButton", "acceptButton").unwrap();

        assert!(form.component("okButton").is_none());
        assert_eq!(form.component("acceptButton").unwrap().kind(), "Button");
    }

    #[test]
    fn renaming_a_selected_control_keeps_it_selected() {
        let mut form = form_with_a_button();
        form.select("okButton");

        form.rename_control("okButton", "acceptButton").unwrap();

        assert_eq!(form.selection(), Some("acceptButton"));
    }

    #[test]
    fn a_control_that_is_not_there_is_not_renamed() {
        let mut form = form_with_a_button();

        let result = form.rename_control("titleLabel", "otro");

        assert!(matches!(result, Err(CoreError::NotFound(_))));
    }

    #[test]
    fn a_control_is_not_renamed_to_a_name_that_is_taken() {
        let mut form = form_with_a_button();
        form.add_control("titleLabel", "Label", 0, 40, 100, 20)
            .unwrap();

        let result = form.rename_control("okButton", "titleLabel");

        assert!(matches!(result, Err(CoreError::InvalidName(_))));
        assert!(form.component("okButton").is_some(), "no se renombra");
    }

    #[test]
    fn a_control_is_not_renamed_to_its_own_name() {
        let mut form = form_with_a_button();

        let result = form.rename_control("okButton", "okButton");

        assert!(matches!(result, Err(CoreError::InvalidName(_))));
    }

    #[test]
    fn a_control_is_not_renamed_to_an_empty_name() {
        let mut form = form_with_a_button();

        let result = form.rename_control("okButton", "");

        assert!(matches!(result, Err(CoreError::InvalidName(_))));
    }

    #[test]
    fn the_text_of_a_control_can_be_set() {
        let mut form = form_with_a_button();

        form.set_text("okButton", "Aceptar").unwrap();

        assert_eq!(form.text("okButton"), Some("Aceptar"));
    }

    #[test]
    fn a_control_without_text_has_none() {
        let form = form_with_a_button();

        assert_eq!(form.text("okButton"), None);
    }

    #[test]
    fn the_text_of_a_control_can_be_changed() {
        let mut form = form_with_a_button();
        form.set_text("okButton", "Aceptar").unwrap();

        form.set_text("okButton", "Cancelar").unwrap();

        assert_eq!(form.text("okButton"), Some("Cancelar"));
    }

    #[test]
    fn the_text_of_a_control_can_be_emptied() {
        let mut form = form_with_a_button();
        form.set_text("okButton", "Aceptar").unwrap();

        form.set_text("okButton", "").unwrap();

        assert_eq!(form.text("okButton"), Some(""));
    }

    #[test]
    fn a_control_is_visible_and_enabled_by_default() {
        let form = form_with_a_button();

        assert!(form.is_visible("okButton"));
        assert!(form.is_enabled("okButton"));
    }

    #[test]
    fn a_control_can_be_hidden() {
        let mut form = form_with_a_button();

        form.set_visible("okButton", false).unwrap();

        assert!(!form.is_visible("okButton"));
    }

    #[test]
    fn a_hidden_control_can_be_shown_again() {
        let mut form = form_with_a_button();
        form.set_visible("okButton", false).unwrap();

        form.set_visible("okButton", true).unwrap();

        assert!(form.is_visible("okButton"));
    }

    #[test]
    fn a_control_can_be_disabled() {
        let mut form = form_with_a_button();

        form.set_enabled("okButton", false).unwrap();

        assert!(!form.is_enabled("okButton"));
    }

    #[test]
    fn a_property_of_a_control_that_is_not_there_is_not_changed() {
        let mut form = form_with_a_button();

        assert!(matches!(
            form.set_text("titleLabel", "Hola"),
            Err(CoreError::NotFound(_))
        ));
        assert!(matches!(
            form.set_visible("titleLabel", false),
            Err(CoreError::NotFound(_))
        ));
        assert!(matches!(
            form.set_enabled("titleLabel", false),
            Err(CoreError::NotFound(_))
        ));
    }

    #[test]
    fn a_property_is_not_kept_twice() {
        let mut form = form_with_a_button();

        form.set_text("okButton", "Aceptar").unwrap();
        form.set_text("okButton", "Cancelar").unwrap();

        let properties = form.component("okButton").unwrap().properties();

        assert_eq!(properties, &[("Text".to_string(), "Cancelar".to_string())]);
    }

    /// Lo que hace el panel de propiedades: los seis cambios, y el modelo
    /// detrás de los seis.
    #[test]
    fn everything_the_properties_panel_edits_is_reflected_in_the_model() {
        let mut form = form_with_a_button();

        form.rename_control("okButton", "acceptButton").unwrap();
        form.set_text("acceptButton", "Aceptar").unwrap();
        form.move_control("acceptButton", 20, 30).unwrap();
        form.resize_control("acceptButton", 150, 40).unwrap();
        form.set_visible("acceptButton", false).unwrap();
        form.set_enabled("acceptButton", false).unwrap();

        let control = form.model().component("acceptButton").unwrap();

        assert_eq!(control.name(), "acceptButton");
        assert_eq!(form.text("acceptButton"), Some("Aceptar"));
        assert_eq!((control.x(), control.y()), (20, 30));
        assert_eq!((control.width(), control.height()), (150, 40));
        assert!(!form.is_visible("acceptButton"));
        assert!(!form.is_enabled("acceptButton"));
    }

    #[test]
    fn the_edited_properties_are_in_the_model_the_generator_reads() {
        let mut form = form_with_a_button();
        form.set_text("okButton", "Aceptar").unwrap();
        form.set_visible("okButton", false).unwrap();
        form.set_enabled("okButton", false).unwrap();

        let properties = form.model().component("okButton").unwrap().properties();

        assert!(properties.contains(&("Text".to_string(), "Aceptar".to_string())));
        assert!(properties.contains(&("Visible".to_string(), "false".to_string())));
        assert!(properties.contains(&("Enabled".to_string(), "false".to_string())));
    }
}
