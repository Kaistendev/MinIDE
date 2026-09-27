use std::path::{Path, PathBuf};

use crate::core::CoreResult;
use crate::project::{validate_root, Project};

/// Lo que esta abierto en la sesion: una raiz de trabajo y, si la hay, el
/// proyecto activo.
///
/// La raiz pertenece a la sesion, no al proyecto, y por eso existe tambien sin
/// proyecto activo: al arrancar y al cerrarlo sigue habiendo una raiz que
/// resolver. La raiz del proyecto es la suya y se consulta en `Project::root`.
///
/// Abrir y cerrar un proyecto, con lo que eso implica en disco, no esta aqui:
/// este tipo solo guarda el estado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    root: PathBuf,
    active_project: Option<Project>,
}

impl Workspace {
    /// Workspace con la raiz `root` y sin proyecto activo.
    ///
    /// Devuelve `CoreError::InvalidPath` si la raiz esta vacia o no es
    /// absoluta, igual que la raiz de un proyecto.
    pub fn new(root: impl Into<PathBuf>) -> CoreResult<Self> {
        let root = root.into();

        validate_root(&root)?;

        Ok(Self {
            root,
            active_project: None,
        })
    }

    /// Raiz de la sesion.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Proyecto activo, si lo hay.
    pub fn active_project(&self) -> Option<&Project> {
        self.active_project.as_ref()
    }

    /// Deja `project` como proyecto activo, sustituyendo al que hubiera.
    ///
    /// No comprueba nada en disco ni decide si se puede abrir: de eso se ocupa
    /// la apertura de proyecto.
    pub fn set_active_project(&mut self, project: Project) {
        self.active_project = Some(project);
    }

    /// Cierra el proyecto activo y lo devuelve, dejando el workspace solo con
    /// su raiz.
    ///
    /// Devuelve `None` si no habia proyecto activo. Cerrar no toca el disco: el
    /// proyecto sigue ahi y se puede volver a abrir.
    pub fn close_project(&mut self) -> Option<Project> {
        self.active_project.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CoreError, ProjectType};
    use crate::project::{BuildConfiguration, ProjectRelativePath};

    fn project(name: &str, root: &str) -> Project {
        Project::new(
            name,
            root,
            ProjectType::CSharpWinForms,
            BuildConfiguration::new(
                ProjectRelativePath::new(BuildConfiguration::DEFAULT_OUTPUT_DIRECTORY).unwrap(),
            ),
        )
        .unwrap()
    }

    #[test]
    fn a_workspace_keeps_its_root() {
        let workspace = Workspace::new("C:\\proyectos").unwrap();

        assert_eq!(workspace.root(), Path::new("C:\\proyectos"));
    }

    #[test]
    fn a_new_workspace_has_no_active_project() {
        let workspace = Workspace::new("C:\\proyectos").unwrap();

        assert_eq!(workspace.active_project(), None);
    }

    #[test]
    fn a_workspace_with_a_relative_root_is_rejected() {
        let result = Workspace::new("proyectos");

        assert!(matches!(result, Err(CoreError::InvalidPath(_))));
    }

    #[test]
    fn a_workspace_with_an_empty_root_is_rejected() {
        let result = Workspace::new("");

        assert!(matches!(result, Err(CoreError::InvalidPath(_))));
    }

    #[test]
    fn a_workspace_can_hold_the_active_project() {
        let mut workspace = Workspace::new("C:\\proyectos").unwrap();

        workspace.set_active_project(project("app", "C:\\proyectos\\app"));

        let active = workspace.active_project().unwrap();

        assert_eq!(active.name(), "app");
        assert_eq!(active.root(), Path::new("C:\\proyectos\\app"));
    }

    #[test]
    fn the_active_project_can_be_replaced() {
        let mut workspace = Workspace::new("C:\\proyectos").unwrap();
        workspace.set_active_project(project("app", "C:\\proyectos\\app"));

        workspace.set_active_project(project("otro", "C:\\proyectos\\otro"));

        assert_eq!(workspace.active_project().unwrap().name(), "otro");
    }

    #[test]
    fn the_workspace_root_does_not_change_with_the_active_project() {
        let mut workspace = Workspace::new("C:\\proyectos").unwrap();

        workspace.set_active_project(project("app", "C:\\proyectos\\app"));

        assert_eq!(workspace.root(), Path::new("C:\\proyectos"));
    }

    #[test]
    fn a_workspace_without_a_project_still_has_its_root() {
        let workspace = Workspace::new("C:\\proyectos").unwrap();

        assert_eq!(workspace.active_project(), None);
        assert_eq!(workspace.root(), Path::new("C:\\proyectos"));
    }
}
