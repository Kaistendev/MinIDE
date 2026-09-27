use std::path::Path;

use crate::core::{CoreError, CoreResult, ProjectType};
use crate::generation::{MARKER_BEGIN, MARKER_END};
use crate::project::{BuildConfiguration, Project, ProjectRelativePath};

/// Framework al que apunta la plantilla de C#.
///
/// Es una decision que se nota: si el SDK instalado no tiene el paquete de
/// referencia de esta version, el proyecto generado no compilara hasta que se
/// cambie aqui.
const TARGET_FRAMEWORK: &str = "net8.0-windows";

/// Nombre del archivo donde el generador del disenador escribe.
///
/// Vive aqui porque es esta plantilla quien lo crea, y quien lo tiene que volver
/// a encontrar para actualizarlo.
pub const DESIGNER_FILE: &str = "Form1.Designer.cs";

/// Nombre de un proyecto como namespace de C#.
///
/// El nombre del proyecto es un directorio, y un directorio vale `mi-proyecto`
/// mientras que un namespace no: `namespace mi-proyecto;` no compila. Se
/// cambian los caracteres que no valen por `_`, y se antepone una `_` si el
/// nombre empieza por un digito.
///
/// El namespace no es el nombre del proyecto, y a proposito: el nombre es lo
/// que ve el usuario en el explorador y en la carpeta, y el namespace es lo que
/// tiene que entender el compilador.
fn namespace_for(name: &str) -> String {
    let mut namespace: String = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect();

    if namespace.starts_with(|character: char| character.is_ascii_digit()) {
        namespace.insert(0, '_');
    }

    namespace
}

/// Crea en `root` un proyecto nuevo del tipo `project_type` y lo devuelve.
///
/// `root` es el directorio del propio proyecto, no el de la carpeta que lo
/// contiene: si el proyecto se llama "App" y va en `C:\proyectos`, `root` es
/// `C:\proyectos\App`. El nombre del proyecto sale del directorio, igual que al
/// abrirlo, para que no puedan contradecirse.
///
/// El directorio se crea si no existe, y tiene que estar vacio: no se escribe
/// nunca encima de algo que ya esta ahi.
///
/// Ahora solo hay plantilla de C# con Windows Forms. Anadir otro tipo es anadir
/// su brazo aqui, y nada mas.
pub fn create_project(project_type: ProjectType, root: &Path) -> CoreResult<Project> {
    validate_root_directory(root)?;

    match project_type {
        ProjectType::CSharpWinForms => create_csharp_winforms(root),
        _ => Err(CoreError::Unsupported(format!(
            "todavia no hay plantilla para {project_type}"
        ))),
    }
}

/// Comprueba que `root` se puede usar como directorio de proyecto nuevo.
fn validate_root_directory(root: &Path) -> CoreResult<()> {
    if root.as_os_str().is_empty() || !root.is_absolute() {
        return Err(CoreError::InvalidPath(root.display().to_string()));
    }

    if !root.exists() {
        return Ok(());
    }

    if !root.is_dir() {
        return Err(CoreError::InvalidPath(root.display().to_string()));
    }

    let is_empty = std::fs::read_dir(root)
        .map_err(|error| CoreError::from_io(root, &error))?
        .next()
        .is_none();

    if !is_empty {
        return Err(CoreError::AlreadyExists(root.display().to_string()));
    }

    Ok(())
}

/// Escribe un proyecto de C# con Windows Forms y devuelve su modelo.
fn create_csharp_winforms(root: &Path) -> CoreResult<Project> {
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CoreError::InvalidName(root.display().to_string()))?
        .to_string();

    // El modelo se construye antes de escribir nada, para no dejar un
    // directorio a medias con un proyecto que despues no se puede abrir.
    let project = Project::new(
        &name,
        root,
        ProjectType::CSharpWinForms,
        BuildConfiguration::new(ProjectRelativePath::new(
            BuildConfiguration::DEFAULT_OUTPUT_DIRECTORY,
        )?),
    )?;

    // El namespace se deriva del nombre, no se copia: un directorio con guion
    // o con espacio genera un namespace que el compilador acepta.
    let namespace = namespace_for(&name);

    std::fs::create_dir_all(root).map_err(|error| CoreError::from_io(root, &error))?;

    write_file(root, &format!("{name}.csproj"), &csharp_project_file())?;
    write_file(root, "Program.cs", &program_file(&namespace))?;
    write_file(root, "Form1.cs", &form_file(&namespace))?;
    write_file(root, DESIGNER_FILE, &form_designer_file(&namespace))?;

    Ok(project)
}

fn write_file(root: &Path, name: &str, content: &str) -> CoreResult<()> {
    let path = root.join(name);

    std::fs::write(&path, content).map_err(|error| CoreError::from_io(&path, &error))
}

/// El archivo de proyecto. `UseWindowsForms` no es decorativo: sin el,
/// `Project::open` no reconoce el proyecto y MiniIDE no lo podria abrir.
fn csharp_project_file() -> String {
    format!(
        r#"<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <OutputType>WinExe</OutputType>
    <TargetFramework>{TARGET_FRAMEWORK}</TargetFramework>
    <UseWindowsForms>true</UseWindowsForms>
    <Nullable>enable</Nullable>
  </PropertyGroup>

</Project>
"#
    )
}

fn program_file(namespace: &str) -> String {
    format!(
        r#"using System;
using System.Windows.Forms;

namespace {namespace};

static class Program
{{
    [STAThread]
    static void Main()
    {{
        Application.EnableVisualStyles();
        Application.SetCompatibleTextRenderingDefault(false);
        Application.Run(new Form1());
    }}
}}
"#
    )
}

fn form_file(namespace: &str) -> String {
    format!(
        r#"using System.Windows.Forms;

namespace {namespace};

public partial class Form1 : Form
{{
    public Form1()
    {{
        InitializeComponent();
    }}
}}
"#
    )
}

/// La otra mitad del formulario. El generador del disenador escribe aqui, y
/// por eso la clase es `partial` y el constructor no hace nada mas.
///
/// Lleva el namespace del proyecto: si no, el archivo no casaria con
/// `Form1.cs` y el proyecto no compilaria.
///
/// La zona que escribe MiniIDE viene ya marcada y con un formulario vacio
/// dentro. Si la plantilla no trajera la zona, la primera generacion la
/// anadiria al final del archivo, fuera de la clase, y el proyecto dejaria de
/// compilar. Y si viniera vacia, el proyecto recien creado no tendria
/// `InitializeComponent` que ejecutar.
fn form_designer_file(namespace: &str) -> String {
    format!(
        r#"using System.Windows.Forms;

namespace {namespace};

partial class Form1
{{
    {MARKER_BEGIN}
    private void InitializeComponent()
    {{
        this.SuspendLayout();
        this.ClientSize = new System.Drawing.Size(640, 480);
        this.ResumeLayout(false);
    }}
    {MARKER_END}
}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::WinFormsModel;
    use crate::generation::WinFormsGenerator;

    #[test]
    fn the_project_file_declares_windows_forms_and_a_framework() {
        let csproj = csharp_project_file();

        assert!(csproj.contains("<UseWindowsForms>true</UseWindowsForms>"));
        assert!(csproj.contains("<OutputType>WinExe</OutputType>"));
        assert!(csproj.contains(TARGET_FRAMEWORK));
    }

    #[test]
    fn every_generated_file_uses_the_namespace_of_the_project() {
        let namespace = "JuegoDePrueba";

        for content in [
            program_file(namespace),
            form_file(namespace),
            form_designer_file(namespace),
        ] {
            assert!(
                content.contains(&format!("namespace {namespace};")),
                "{content}"
            );
        }
    }

    #[test]
    fn the_form_is_partial_and_calls_the_designer() {
        let form = form_file("App");
        let designer = form_designer_file("App");

        assert!(form.contains("public partial class Form1 : Form"));
        assert!(form.contains("InitializeComponent();"));
        assert!(designer.contains("partial class Form1"));
        assert!(designer.contains("private void InitializeComponent()"));
    }

    #[test]
    fn every_generated_file_imports_the_namespaces_it_uses() {
        // Los tres archivos usan tipos de System o de Windows Forms, y el
        // proyecto no activate `ImplicitUsings`, asi que cada uno tiene que
        // importarlos. Sin esto el proyecto generado no compila.
        for (name, content) in [
            ("Program.cs", program_file("App")),
            ("Form1.cs", form_file("App")),
            ("Form1.Designer.cs", form_designer_file("App")),
        ] {
            assert!(
                content.contains("using System.Windows.Forms;"),
                "{name} no importa System.Windows.Forms"
            );
        }

        assert!(program_file("App").contains("using System;"));
    }

    #[test]
    fn a_relative_root_is_not_a_project_directory() {
        let result = validate_root_directory(Path::new("App"));

        assert!(matches!(result, Err(CoreError::InvalidPath(_))));
    }

    #[test]
    fn a_root_that_does_not_exist_can_be_a_project_directory() {
        let root = std::env::temp_dir().join("miniide-plantilla-no-existe");

        assert!(validate_root_directory(&root).is_ok());
    }

    #[test]
    fn there_is_no_template_for_java_yet() {
        let root = std::env::temp_dir().join("miniide-plantilla-java");

        let result = create_project(ProjectType::JavaSwing, &root);

        assert!(matches!(result, Err(CoreError::Unsupported(_))));
    }

    #[test]
    fn the_designer_file_has_the_region_of_the_generator() {
        let designer = form_designer_file("App");

        assert!(
            designer.contains(MARKER_BEGIN),
            "sin el marcador de inicio la primera generacion escribe fuera de la clase"
        );
        assert!(designer.contains(MARKER_END));
    }

    #[test]
    fn the_region_of_the_designer_file_is_inside_the_class() {
        let designer = form_designer_file("App");

        let class = designer.find("partial class Form1").expect("la clase");
        let begin = designer.find(MARKER_BEGIN).expect("el marcador de inicio");
        let end = designer.find(MARKER_END).expect("el marcador de fin");
        let close = designer.rfind('}').expect("la llave de cierre");

        assert!(class < begin, "la zona empieza antes de la clase");
        assert!(begin < end, "la zona esta al reves");
        assert!(end < close, "la zona se sale de la clase");
    }

    /// El proyecto recien creado tiene que compilar sin haber generado nada, y
    /// para eso la zona tiene que traer un formulario valido dentro.
    #[test]
    fn the_designer_file_has_a_usable_form_before_any_generation() {
        let designer = form_designer_file("App");

        assert!(designer.contains("private void InitializeComponent()"));
        assert!(designer.contains("this.SuspendLayout();"));
        assert!(designer.contains("this.ResumeLayout(false);"));
    }

    /// El codigo que el usuario escribe en el archivo del disenador, fuera de la
    /// zona, se simulando con un campo que no genera nadie.
    fn with_user_code() -> String {
        let user_code = "    private int contadorDelUsuario;\n\n    // codigo del usuario\n    private void Refrescar() { }\n\n    private int Contador() { return this.contadorDelUsuario; }\n\n";

        form_designer_file("App").replace(
            &format!("    {MARKER_BEGIN}"),
            &format!("{user_code}    {MARKER_BEGIN}"),
        )
    }

    #[test]
    fn the_first_generation_fills_the_region_of_the_template() {
        let mut form = WinFormsModel::new("MainForm", "Formulario principal", 640, 480);
        form.add_control("okButton", "Button", 8, 8, 120, 30)
            .unwrap();

        let result = WinFormsGenerator
            .apply_to(form.model(), &form_designer_file("App"))
            .unwrap();

        assert!(result.contains("private System.Windows.Forms.Button okButton;"));
        assert_eq!(
            result.matches(MARKER_BEGIN).count(),
            1,
            "no se anade una segunda zona"
        );
    }

    #[test]
    fn regenerating_the_design_keeps_the_code_the_user_wrote() {
        let mut form = WinFormsModel::new("MainForm", "Formulario principal", 640, 480);
        form.add_control("okButton", "Button", 8, 8, 120, 30)
            .unwrap();

        let generated = WinFormsGenerator
            .apply_to(form.model(), &with_user_code())
            .unwrap();

        for expected in [
            "private int contadorDelUsuario;",
            "// codigo del usuario",
            "private void Refrescar() { }",
            "return this.contadorDelUsuario;",
        ] {
            assert!(generated.contains(expected), "se ha perdido: {expected}");
        }
    }

    #[test]
    fn a_control_that_is_deleted_disappears_from_the_generated_code() {
        let mut form = WinFormsModel::new("MainForm", "Formulario principal", 640, 480);
        form.add_control("okButton", "Button", 8, 8, 120, 30)
            .unwrap();
        form.add_control("titleLabel", "Label", 8, 44, 200, 20)
            .unwrap();

        let first = WinFormsGenerator
            .apply_to(form.model(), &form_designer_file("App"))
            .unwrap();

        form.remove_control("titleLabel");

        let second = WinFormsGenerator.apply_to(form.model(), &first).unwrap();

        assert!(second.contains("okButton"));
        assert!(
            !second.contains("titleLabel"),
            "el control quitado sigue ahi"
        );
    }

    /// Regenerar sin cambiar nada no puede ir moviendo el codigo de sitio: si
    /// fuera asi, un archivo iria creciendo cada vez que se abre el disenador.
    #[test]
    fn regenerating_the_same_design_twice_gives_the_same_file() {
        let mut form = WinFormsModel::new("MainForm", "Formulario principal", 640, 480);
        form.add_control("okButton", "Button", 8, 8, 120, 30)
            .unwrap();

        let first = WinFormsGenerator
            .apply_to(form.model(), &with_user_code())
            .unwrap();
        let second = WinFormsGenerator.apply_to(form.model(), &first).unwrap();

        assert_eq!(first, second);
    }

    /// Si el usuario ha dejado la zona sin cerrar, no se toca el archivo: cortar
    /// por donde toca se llevaria su codigo.
    #[test]
    fn a_region_left_open_by_hand_is_refused() {
        let mut form = WinFormsModel::new("MainForm", "Formulario principal", 640, 480);
        form.add_control("okButton", "Button", 8, 8, 120, 30)
            .unwrap();

        let broken = form_designer_file("App").replace(
            &format!("    {MARKER_END}\n"),
            "    // se ha borrado el cierre de la zona\n",
        );

        let result = WinFormsGenerator.apply_to(form.model(), &broken);

        assert!(matches!(result, Err(CoreError::MalformedSource(_))));
    }

    /// Un directorio puede llamarse como quiera, y un namespace de C# no puede.
    /// Si se copiase el nombre tal cual, un proyecto creado en `mi-proyecto`
    /// generaria un `namespace mi-proyecto;` que no compila.
    #[test]
    fn a_project_name_that_is_not_a_csharp_identifier_needs_its_own_namespace() {
        for (name, expected) in [
            ("App", "App"),
            ("JuegoDePrueba", "JuegoDePrueba"),
            ("mi-proyecto", "mi_proyecto"),
            ("Mi Proyecto", "Mi_Proyecto"),
            ("mi.app", "mi_app"),
            ("1App", "_1App"),
            ("juego 2024", "juego_2024"),
            ("2024", "_2024"),
        ] {
            assert_eq!(namespace_for(name), expected, "namespace de {name}");
        }
    }

    #[test]
    fn the_namespace_of_the_project_is_always_usable() {
        for name in ["mi-proyecto", "Mi Proyecto", "1App", "app.cs", "año-nuevo"] {
            let namespace = namespace_for(name);

            let first = namespace.chars().next().expect("namespace no vacio");

            assert!(
                first.is_ascii_alphabetic() || first == '_',
                "{namespace} no empieza por una letra"
            );
            assert!(
                namespace
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_'),
                "{namespace} tiene caracteres que no valen en un namespace"
            );
        }
    }

    /// Los tres archivos tienen que llevar el namespace derivado. Si uno llevara
    /// el nombre del directorio y otro el derivado, el proyecto no compilaria.
    #[test]
    fn every_file_of_a_project_with_a_dashed_directory_shares_its_namespace() {
        let namespace = namespace_for("mi-proyecto");

        for content in [
            program_file(&namespace),
            form_file(&namespace),
            form_designer_file(&namespace),
        ] {
            assert!(
                content.contains(&format!("namespace {namespace};")),
                "{content}"
            );
        }
    }
}
