//! MiniIDE - núcleo del IDE minimalista para Windows.
//!
//! El crate expone módulos independientes entre sí: `core`, `document`,
//! `editor`, `language`, `framework`, `generation`, `project`, `workspace`,
//! `build`, `runtime`, `toolchain`, `supports`, `templates`, `diagnostics`,
//! `commands` y `ui`. Ninguno depende de C#, Java, WinForms o Swing.
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
pub mod generation;
pub mod language;
pub mod project;
pub mod runtime;
pub mod supports;
pub mod templates;
pub mod toolchain;
pub mod ui;
pub mod workspace;

pub const APP_NAME: &str = "MiniIDE";

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
