//! MiniIDE - núcleo del IDE minimalista para Windows.
//!
//! El crate expone módulos independientes entre sí: `core`, `document`,
//! `editor`, `language`, `framework`, `generation`, `project`, `workspace`,
//! `build`, `runtime`, `toolchain`, `supports`, `templates`, `diagnostics`,
//! `commands` y `frontend`. Ninguno depende de C#, Java, WinForms o Swing.
//!
//! `frontend` es la excepción a propósito, y solo en un sentido: es el único módulo
//! que puede usar a todos los demás, porque la interfaz consume el core y le emite
//! comandos. El core no depende del frontend y ni lo menciona.
//!
//! # Ejemplo
//!
//! ```
//! use miniide::core::CoreError;
//! use miniide::project::ProjectRelativePath;
//!
//! // Toda ruta de proyecto es relativa a su raíz.
//! let path = ProjectRelativePath::new("src/forms/MainForm.cs").unwrap();
//! assert_eq!(path.as_path().to_str(), Some("src/forms/MainForm.cs"));
//!
//! // Un camino que sale del proyecto se rechaza con un error del core.
//! let escaped = ProjectRelativePath::new("../secrets.txt");
//! assert_eq!(escaped, Err(CoreError::InvalidPath("../secrets.txt".to_string())));
//! ```

pub mod build;
pub mod commands;
pub mod core;
pub mod diagnostics;
pub mod document;
pub mod editor;
pub mod framework;
pub mod frontend;
pub mod generation;
pub mod language;
pub mod project;
pub mod runtime;
pub mod supports;
pub mod templates;
pub mod toolchain;
pub mod workspace;

pub const APP_NAME: &str = "MiniIDE";

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
