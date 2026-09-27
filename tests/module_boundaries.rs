use std::path::Path;

use miniide::build::BuildResult;
use miniide::commands::{Command, CommandOutcome};
use miniide::core::{CoreError, FrameworkId, LanguageId, ProjectType};
use miniide::diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticLocation};
use miniide::document::{TextBuffer, TextPosition, TextRange};
use miniide::editor::{Cursor, Document, DocumentPath, OpenTabs, Selection, Tab};
use miniide::framework::{FrameworkCapabilities, FrameworkProvider, WinFormsModel};
use miniide::generation::{CodeGenerator, DesignerComponent, DesignerModel, GeneratedCode};
use miniide::language::{EditingConfiguration, LanguageProvider};
use miniide::project::{
    BuildConfiguration, Project, ProjectFile, ProjectFileKind, ProjectRelativePath,
};
use miniide::runtime::{ProcessState, RunResult};
use miniide::supports::Supports;
use miniide::toolchain::{Invocation, ToolchainProvider};
use miniide::ui::UiState;
use miniide::workspace::Workspace;

#[test]
fn core_modules_are_reachable_from_outside_the_crate() {
    let _: TextPosition = TextPosition::new(0, 0);
    let _: TextRange = TextRange::collapsed(TextPosition::new(0, 0));
    let _: ProjectRelativePath = ProjectRelativePath::new("src/main.rs").unwrap();
    let _: CommandOutcome = CommandOutcome::Completed;
    let _: UiState = UiState::new();
}

#[test]
fn commands_reuse_the_core_error_type() {
    let outcome = CommandOutcome::Rejected(CoreError::NotFound("MainForm.cs".to_string()));

    assert!(matches!(
        outcome,
        CommandOutcome::Rejected(CoreError::NotFound(_))
    ));
}

#[test]
fn every_supported_language_round_trips_through_its_identifier() {
    let languages: Vec<LanguageId> = LanguageId::ALL.to_vec();

    assert_eq!(languages, vec![LanguageId::CSharp, LanguageId::Java]);

    for language in languages {
        assert_eq!(language.as_str().parse::<LanguageId>(), Ok(language));
    }
}

#[test]
fn every_supported_framework_round_trips_through_its_identifier() {
    let frameworks: Vec<FrameworkId> = FrameworkId::ALL.to_vec();

    assert_eq!(frameworks, vec![FrameworkId::WinForms, FrameworkId::Swing]);

    for framework in frameworks {
        assert_eq!(framework.as_str().parse::<FrameworkId>(), Ok(framework));
    }
}

#[test]
fn every_supported_project_type_combines_supported_parts() {
    for project_type in ProjectType::ALL {
        assert!(LanguageId::ALL.contains(&project_type.language()));
        assert!(FrameworkId::ALL.contains(&project_type.framework()));
    }
}

#[test]
fn project_entries_are_addressed_by_a_path_inside_the_project() {
    let path = ProjectRelativePath::new("src/forms/MainForm.cs").unwrap();
    let file = ProjectFile::new(path, ProjectFileKind::File);

    assert_eq!(
        file.path().as_path().to_str(),
        Some("src/forms/MainForm.cs")
    );
    assert_eq!(file.kind(), ProjectFileKind::File);
}

#[test]
fn a_project_is_built_from_core_types_only() {
    let build_configuration = BuildConfiguration::new(ProjectRelativePath::new("out").unwrap());
    let mut project = Project::new(
        "Demo",
        Path::new("C:/projects/Demo"),
        ProjectType::CSharpWinForms,
        build_configuration,
    )
    .unwrap();

    project.add_file(ProjectFile::new(
        ProjectRelativePath::new("src/Program.cs").unwrap(),
        ProjectFileKind::File,
    ));

    assert_eq!(project.name(), "Demo");
    assert_eq!(project.language(), LanguageId::CSharp);
    assert_eq!(project.framework(), FrameworkId::WinForms);
    assert_eq!(project.files().len(), 1);
    assert_eq!(
        project.build_configuration().output_directory().as_path(),
        Path::new("out")
    );
}

#[test]
fn an_unsupported_platform_combination_is_rejected() {
    let result = ProjectType::from_parts(LanguageId::CSharp, FrameworkId::Swing);

    assert_eq!(
        result,
        Err(CoreError::Unsupported("C# / Swing".to_string()))
    );
}

#[test]
fn a_diagnostic_points_at_a_file_inside_the_project() {
    let location = DiagnosticLocation::new(
        ProjectRelativePath::new("src/Program.cs").unwrap(),
        TextPosition::new(11, 4),
    );
    let diagnostic = Diagnostic::new(
        DiagnosticLevel::Error,
        "CS1002: ; expected",
        Some(location.clone()),
    );

    assert_eq!(diagnostic.level(), DiagnosticLevel::Error);
    assert_eq!(diagnostic.location(), Some(&location));
}

#[test]
fn a_build_result_carries_the_diagnostics_of_a_failed_compilation() {
    let location = DiagnosticLocation::new(
        ProjectRelativePath::new("src/Program.cs").unwrap(),
        TextPosition::new(11, 4),
    );
    let result = BuildResult::new(
        false,
        Some(1),
        "",
        "Build FAILED.",
        vec![Diagnostic::new(
            DiagnosticLevel::Error,
            "CS1002: ; expected",
            Some(location.clone()),
        )],
    );

    assert!(!result.succeeded());
    assert_eq!(result.exit_code(), Some(1));
    assert_eq!(result.standard_error(), "Build FAILED.");
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].level(), DiagnosticLevel::Error);
    assert_eq!(result.diagnostics()[0].location(), Some(&location));
}

#[test]
fn a_finished_run_reports_its_state_exit_code_and_output() {
    let result = RunResult::new(
        ProcessState::Exited,
        Some(3),
        "iniciando...",
        "principal no encontrada",
        None,
    );

    assert_eq!(result.state(), ProcessState::Exited);
    assert_eq!(result.exit_code(), Some(3));
    assert_eq!(result.standard_output(), "iniciando...");
    assert_eq!(result.standard_error(), "principal no encontrada");
    assert_eq!(result.error(), None);
}

#[test]
fn the_main_commands_are_reachable_from_outside_the_crate() {
    assert_ne!(Command::Build, Command::Run);
    assert_eq!(Command::Stop.as_str(), "stop");
    assert!(Command::ALL.contains(&Command::Find));
}

#[test]
fn a_document_buffer_keeps_its_text_outside_the_ui() {
    let buffer = TextBuffer::new("class Main { static void Main() {} }");

    assert_eq!(buffer.text(), "class Main { static void Main() {} }");
}

#[test]
fn text_can_be_inserted_at_a_position_from_outside_the_crate() {
    let mut buffer = TextBuffer::new("class Main {}");

    buffer.insert(TextPosition::new(0, 6), "static ").unwrap();

    assert_eq!(buffer.text(), "class static Main {}");
}

#[test]
fn a_range_of_text_can_be_removed_from_outside_the_crate() {
    let mut buffer = TextBuffer::new("class Main {}");
    let range = TextRange::new(TextPosition::new(0, 0), TextPosition::new(0, 6));

    buffer.remove(range).unwrap();

    assert_eq!(buffer.text(), "Main {}");
}

#[test]
fn a_cursor_moves_inside_a_document_from_outside_the_crate() {
    let buffer = TextBuffer::new("uno\ndos");
    let mut cursor = Cursor::new(&buffer, TextPosition::new(0, 3));

    cursor.move_right(&buffer);

    assert_eq!(cursor.at(), TextPosition::new(1, 0));
}

#[test]
fn a_selection_can_be_read_and_replaced_from_outside_the_crate() {
    let mut buffer = TextBuffer::new("uno\ndos");
    let mut selection = Selection::new(Cursor::new(&buffer, TextPosition::new(0, 1)));
    selection.extend_to(&buffer, TextPosition::new(1, 1));

    assert_eq!(selection.selected_text(&buffer).unwrap(), Some("no\nd"));

    selection.replace(&mut buffer, "X").unwrap();

    assert_eq!(buffer.text(), "uXos");
}

#[test]
fn a_document_reports_and_clears_its_modified_state_from_outside_the_crate() {
    let mut document = Document::new("uno");

    document.insert(TextPosition::new(0, 3), "!").unwrap();

    assert!(document.is_modified());

    document.mark_saved();

    assert!(!document.is_modified());
}

#[test]
fn a_document_edit_can_be_undone_from_outside_the_crate() {
    let mut document = Document::new("uno");

    document.insert(TextPosition::new(0, 3), "!").unwrap();

    assert_eq!(document.buffer().text(), "uno!");

    assert!(document.undo());

    assert_eq!(document.buffer().text(), "uno");
    assert!(!document.is_modified());
}

#[test]
fn an_undone_document_edit_can_be_redone_from_outside_the_crate() {
    let mut document = Document::new("uno");

    document.insert(TextPosition::new(0, 3), "!").unwrap();

    assert!(document.undo());
    assert_eq!(document.buffer().text(), "uno");

    assert!(document.redo());
    assert_eq!(document.buffer().text(), "uno!");
    assert!(document.is_modified());
    assert!(!document.can_redo());
}

#[test]
fn a_document_search_gives_ranges_from_outside_the_crate() {
    let document = Document::new("uno dos uno");

    let found = document.buffer().find_all("uno");

    assert_eq!(
        found,
        vec![
            TextRange::new(TextPosition::new(0, 0), TextPosition::new(0, 3)),
            TextRange::new(TextPosition::new(0, 8), TextPosition::new(0, 11)),
        ]
    );
}

#[test]
fn a_document_can_replace_every_match_from_outside_the_crate() {
    let mut document = Document::new("uno dos uno");

    let replaced = document.replace_all("uno", "tres").unwrap();

    assert_eq!(replaced, 2);
    assert_eq!(document.buffer().text(), "tres dos tres");

    assert!(document.undo());

    assert_eq!(document.buffer().text(), "uno dos uno");
}

#[test]
fn two_documents_can_be_open_at_once_from_outside_the_crate() {
    let mut tabs = OpenTabs::new();

    let main = tabs.open(DocumentPath::new("src/main.rs").unwrap(), "uno");
    let other = tabs.open(DocumentPath::new("src/other.rs").unwrap(), "dos");
    tabs.get_mut(main)
        .unwrap()
        .document_mut()
        .insert(TextPosition::new(0, 3), "!")
        .unwrap();

    assert_eq!(tabs.len(), 2);
    assert!(tabs.get(main).unwrap().is_modified());
    assert!(!tabs.get(other).unwrap().is_modified());
    assert_eq!(tabs.get(other).unwrap().document().buffer().text(), "dos");
}

#[test]
fn a_tab_can_be_built_on_its_own_from_outside_the_crate() {
    let tab = Tab::new(
        DocumentPath::new("notes.txt").unwrap(),
        Document::new("uno"),
    );

    assert_eq!(tab.file_name(), "notes.txt");
    assert_eq!(tab.document().buffer().text(), "uno");
}

#[test]
fn a_workspace_holds_its_root_and_its_active_project_from_outside_the_crate() {
    let mut workspace = Workspace::new("C:\\proyectos").unwrap();

    assert_eq!(workspace.active_project(), None);

    let project = Project::new(
        "app",
        "C:\\proyectos\\app",
        ProjectType::CSharpWinForms,
        BuildConfiguration::new(ProjectRelativePath::new("bin").unwrap()),
    )
    .unwrap();

    workspace.set_active_project(project);

    assert_eq!(workspace.root(), Path::new("C:\\proyectos"));
    assert_eq!(workspace.active_project().unwrap().name(), "app");
}

#[test]
fn a_project_enumerates_its_files_from_outside_the_crate() {
    let root = std::env::temp_dir().join(format!("miniide-boundary-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).expect("temporary directory");
    std::fs::write(root.join("app.csproj"), "<Project />").expect("file written");
    std::fs::write(root.join("src/Program.cs"), "class Program {}").expect("file written");

    let project = Project::new(
        "app",
        &root,
        ProjectType::CSharpWinForms,
        BuildConfiguration::new(ProjectRelativePath::new("bin").unwrap()),
    )
    .unwrap();

    let found = project.discover_files().expect("files discovered");

    let _ = std::fs::remove_dir_all(&root);

    assert_eq!(found.len(), 3);
    assert_eq!(found[0].path().as_path(), Path::new("app.csproj"));
    assert_eq!(found[0].kind(), ProjectFileKind::File);
    assert_eq!(found[1].path().as_path(), Path::new("src"));
    assert_eq!(found[1].kind(), ProjectFileKind::Directory);
    assert_eq!(
        found[2].path().as_path(),
        Path::new("src").join("Program.cs")
    );
}

/// Un lenguaje definido fuera del crate, para comprobar que el contrato de
/// lenguaje se puede implementar y consultar desde fuera.
struct ExternalLanguage;

impl LanguageProvider for ExternalLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::CSharp
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["cs"]
    }

    fn editing(&self) -> EditingConfiguration {
        EditingConfiguration::new(Some("//"), Some(("/*", "*/")), "    ")
    }
}

#[test]
fn a_language_contract_can_be_implemented_and_queried_from_outside_the_crate() {
    let languages: [&dyn LanguageProvider; 1] = [&ExternalLanguage];
    let csharp = languages[0];

    assert_eq!(csharp.id(), LanguageId::CSharp);
    assert_eq!(csharp.extensions(), &["cs"]);
    assert_eq!(csharp.editing().indent(), "    ");
    assert!(csharp.supports(Path::new("src/Main.cs")));
    assert!(!csharp.supports(Path::new("src/Main.java")));
}

/// Un framework definido fuera del crate.
struct ExternalFramework;

impl FrameworkProvider for ExternalFramework {
    fn id(&self) -> FrameworkId {
        FrameworkId::Swing
    }

    fn capabilities(&self) -> FrameworkCapabilities {
        FrameworkCapabilities::new("JFrame", &["JButton", "JLabel"], true)
    }
}

#[test]
fn a_framework_contract_can_be_implemented_and_queried_from_outside_the_crate() {
    let frameworks: [&dyn FrameworkProvider; 1] = [&ExternalFramework];
    let swing = frameworks[0];

    assert_eq!(swing.id(), FrameworkId::Swing);
    assert_eq!(swing.capabilities().root(), "JFrame");
    assert_eq!(swing.capabilities().components(), &["JButton", "JLabel"]);
    assert!(swing.capabilities().generates_code());
}

/// Una toolchain definida fuera del crate.
struct ExternalToolchain;

impl ToolchainProvider for ExternalToolchain {
    fn project_type(&self) -> ProjectType {
        ProjectType::CSharpWinForms
    }

    fn tool(&self) -> &'static str {
        ".NET SDK"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn build_invocation(&self, project: &Project) -> miniide::core::CoreResult<Invocation> {
        Ok(Invocation::new(
            "dotnet",
            vec!["build".to_string()],
            project.root().to_path_buf(),
        ))
    }

    fn run_invocation(&self, project: &Project) -> miniide::core::CoreResult<Invocation> {
        Ok(Invocation::new(
            "bin/app.exe".to_string(),
            Vec::new(),
            project.root().to_path_buf(),
        ))
    }

    fn parse_diagnostics(
        &self,
        _output: &miniide::runtime::ProcessOutput,
        _root: &Path,
    ) -> Vec<miniide::diagnostics::Diagnostic> {
        Vec::new()
    }
}

#[test]
fn a_toolchain_contract_can_be_implemented_and_queried_from_outside_the_crate() {
    let toolchains: [&dyn ToolchainProvider; 1] = [&ExternalToolchain];
    let dotnet = toolchains[0];
    let project = Project::new(
        "app",
        "C:\\proyectos\\app",
        ProjectType::CSharpWinForms,
        BuildConfiguration::new(ProjectRelativePath::new("bin").unwrap()),
    )
    .unwrap();

    assert_eq!(dotnet.project_type(), ProjectType::CSharpWinForms);
    assert_eq!(dotnet.tool(), ".NET SDK");
    assert!(dotnet.is_available());
    assert_eq!(
        dotnet.build_invocation(&project).unwrap().program(),
        "dotnet"
    );
    assert_eq!(
        dotnet.run_invocation(&project).unwrap().working_directory(),
        Path::new("C:\\proyectos\\app")
    );
}

/// Un generador definido fuera del crate.
struct ExternalGenerator;

impl CodeGenerator for ExternalGenerator {
    fn generate(&self, model: &DesignerModel) -> miniide::core::CoreResult<GeneratedCode> {
        let content = model
            .components()
            .iter()
            .map(|c| format!("{} {}", c.name(), c.kind()))
            .collect::<Vec<String>>()
            .join("\n");

        Ok(GeneratedCode::new(
            "// <generado>",
            "// </generado>",
            content,
        ))
    }
}

#[test]
fn a_generation_contract_can_be_used_from_outside_the_crate_without_touching_user_code() {
    let generator = ExternalGenerator;
    let model = DesignerModel::new(
        "MainForm",
        "Formulario",
        100,
        100,
        vec![DesignerComponent::new("ok", "Button", 0, 0, 10, 10)],
    );
    let source = "class MainForm\n{\n    // usuario\n}\n";

    let applied = generator
        .generate(&model)
        .unwrap()
        .region()
        .apply_to(source)
        .unwrap();

    assert!(applied.contains("// usuario"));
    assert!(applied.contains("ok Button"));
}

#[test]
fn the_initial_supports_are_reachable_from_outside_the_crate() {
    let supports = Supports::initial();

    for project_type in ProjectType::ALL {
        let language = supports
            .language(project_type.language())
            .expect("language registered");
        let framework = supports
            .framework(project_type.framework())
            .expect("framework registered");

        assert!(!language.extensions().is_empty());
        assert!(!framework.capabilities().components().is_empty());
    }
}

#[test]
fn a_file_is_associated_with_its_language_from_outside_the_crate() {
    let supports = Supports::initial();

    let csharp = supports
        .language_for(Path::new("src/Program.cs"))
        .expect("a .cs file belongs to C#");
    let java = supports
        .language_for(Path::new("src/Main.java"))
        .expect("a .java file belongs to Java");

    assert_eq!(csharp.id(), LanguageId::CSharp);
    assert_eq!(csharp.editing().line_comment(), Some("//"));
    assert_eq!(java.id(), LanguageId::Java);
    assert!(supports.language_for(Path::new("notas.txt")).is_none());
}

#[test]
fn a_winforms_form_can_be_built_from_outside_the_crate() {
    let mut form = WinFormsModel::new("MainForm", "Formulario principal", 640, 480);

    form.add_control("okButton", "Button", 8, 8, 120, 30)
        .unwrap();
    form.add_control("nameTextBox", "TextBox", 8, 48, 200, 24)
        .unwrap();

    assert!(matches!(
        form.add_control("no", "JButton", 0, 0, 10, 10),
        Err(CoreError::Unsupported(_))
    ));
    assert_eq!(form.component("okButton").unwrap().width(), 120);
    assert_eq!(form.model().components().len(), 2);
}
