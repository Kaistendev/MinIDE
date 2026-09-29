use std::fmt;

use crate::core::CoreError;

/// Las operaciones que se pueden pedir al core.
///
/// Un comando es el nombre de una operación y nada más: no lleva datos ni dice cómo se
/// hace. Los datos van aparte, en la petición que lo acompaña, y la forma de hacerlo es del
/// core, que es el único que sabe. Así una operación es la misma se pida desde un botón,
/// desde un menú o desde un atajo, y solo hay un sitio donde se ejecuta.
///
/// Los que hablan de documentos están al final porque son los que ha ido pidiendo la
/// interfaz (T-096): sin ellos la interfaz no tenía forma de pedir abrir un documento y se
/// lo habría inventado por su cuenta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    NewProject,
    OpenProject,
    CloseProject,
    Save,
    Build,
    Run,
    Stop,
    Undo,
    Redo,
    Find,
    OpenDocument,
    CloseDocument,
    SaveAll,
    ActivateDocument,
    NewFile,
    NewDirectory,
    Replace,
}

impl Command {
    pub const ALL: [Command; 17] = [
        Command::NewProject,
        Command::OpenProject,
        Command::CloseProject,
        Command::Save,
        Command::Build,
        Command::Run,
        Command::Stop,
        Command::Undo,
        Command::Redo,
        Command::Find,
        Command::OpenDocument,
        Command::CloseDocument,
        Command::SaveAll,
        Command::ActivateDocument,
        Command::NewFile,
        Command::NewDirectory,
        Command::Replace,
    ];

    /// El identificador del comando, que es como se nombra fuera del core.
    ///
    /// Es texto y no el nombre de la variante porque el nombre de una variante se cambia
    /// al reorganizar el archivo y esto no: sale en los mensajes al usuario y en el registro
    /// de lo que se ha pedido, y eso ya está escrito con el identificador antiguo.
    pub fn as_str(&self) -> &'static str {
        match self {
            Command::NewProject => "new_project",
            Command::OpenProject => "open_project",
            Command::CloseProject => "close_project",
            Command::Save => "save",
            Command::Build => "build",
            Command::Run => "run",
            Command::Stop => "stop",
            Command::Undo => "undo",
            Command::Redo => "redo",
            Command::Find => "find",
            Command::OpenDocument => "open_document",
            Command::CloseDocument => "close_document",
            Command::SaveAll => "save_all",
            Command::ActivateDocument => "activate_document",
            Command::NewFile => "new_file",
            Command::NewDirectory => "new_directory",
            Command::Replace => "replace",
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
    Completed,
    Cancelled,
    Rejected(CoreError),
}

impl CommandOutcome {
    pub fn is_completed(&self) -> bool {
        matches!(self, CommandOutcome::Completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_is_the_only_successful_outcome() {
        assert!(CommandOutcome::Completed.is_completed());
        assert!(!CommandOutcome::Cancelled.is_completed());
        assert!(!CommandOutcome::Rejected(CoreError::NotFound("a".to_string())).is_completed());
    }

    #[test]
    fn rejected_keeps_the_core_error() {
        let outcome = CommandOutcome::Rejected(CoreError::Unsupported("WPF".to_string()));

        assert_eq!(
            outcome,
            CommandOutcome::Rejected(CoreError::Unsupported("WPF".to_string()))
        );
    }

    #[test]
    fn the_command_set_covers_the_main_actions() {
        assert_eq!(
            Command::ALL,
            [
                Command::NewProject,
                Command::OpenProject,
                Command::CloseProject,
                Command::Save,
                Command::Build,
                Command::Run,
                Command::Stop,
                Command::Undo,
                Command::Redo,
                Command::Find,
                Command::OpenDocument,
                Command::CloseDocument,
                Command::SaveAll,
                Command::ActivateDocument,
                Command::NewFile,
                Command::NewDirectory,
                Command::Replace,
            ]
        );
    }

    /// Los comandos que la interfaz necesita para hablar de documentos existen.
    ///
    /// T-096 los fija uno por uno, y sin ellos la interfaz no tiene forma de pedir abrir
    /// un documento, cerrarlo, guardar todos, crear un archivo o un directorio, o
    /// reemplazar texto. Y como no puede pedirlos, se los inventaría: la interfaz
    /// definiendo un comando que el core no tiene es la forma más rápida de tener dos
    /// verdades sobre lo que significa "abrir un documento".
    ///
    /// El identificador de cada uno se fija aquí porque es parte de lo que el core
    /// publica: sale en los mensajes y en el registro, y cambiarlo después rompe lo que ya
    /// esté escrito con el antiguo.
    #[test]
    fn the_commands_the_interface_needs_exist() {
        for (comando, identificador) in [
            (Command::OpenDocument, "open_document"),
            (Command::CloseDocument, "close_document"),
            (Command::SaveAll, "save_all"),
            (Command::NewFile, "new_file"),
            (Command::NewDirectory, "new_directory"),
            (Command::Replace, "replace"),
        ] {
            assert!(
                Command::ALL.contains(&comando),
                "{comando:?} tiene que estar en la lista de comandos: {:?}",
                Command::ALL
            );
            assert_eq!(
                comando.as_str(),
                identificador,
                "el identificador de {comando:?}"
            );
        }
    }

    /// La interfaz puede pedir que se vea otro documento.
    ///
    /// FE-017 es un clic en una pestaña, y un clic se convierte en comando como
    /// cualquier otro. Sin este comando la ventana no tiene forma de decir cuál es el
    /// documento que se está viendo, y se queda con su propia idea: el usuario ve una
    /// pestaña resaltada y el core cree que se está viendo otra, que es la forma más
    /// corta de que lo que se guarda y lo que se ve sean cosas distintas.
    ///
    /// Va con los de documentos y no al principio porque es de los que ha ido
    /// pidiendo la interfaz, como ellos (T-097).
    #[test]
    fn the_interface_can_ask_for_another_active_document() {
        assert!(
            Command::ALL.contains(&Command::ActivateDocument),
            "hay que poder pedir que se vea otro documento: {:?}",
            Command::ALL
        );
        assert_eq!(
            Command::ActivateDocument.as_str(),
            "activate_document",
            "el identificador de ActivateDocument"
        );
    }

    #[test]
    fn every_command_has_a_unique_identifier() {
        let mut identifiers: Vec<&str> = Command::ALL.iter().map(Command::as_str).collect();

        for identifier in &identifiers {
            assert!(!identifier.is_empty());
        }

        identifiers.sort_unstable();
        identifiers.dedup();

        assert_eq!(identifiers.len(), Command::ALL.len());
    }

    #[test]
    fn a_command_displays_its_own_identifier() {
        for command in Command::ALL {
            assert_eq!(command.to_string(), command.as_str());
        }
    }
}
