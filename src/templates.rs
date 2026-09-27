use std::path::Path;

use crate::core::{CoreError, CoreResult, ProjectType};
use crate::generation::{BUILD_METHOD, MARKER_BEGIN, MARKER_END};
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

/// Punto de entrada de la plantilla de Java.
///
/// El nombre de la clase va en el nombre del archivo, como en cualquier clase
/// publica de Java: una clase `Main` en `main.java` no compila.
const JAVA_MAIN_FILE: &str = "src/main/java/Main.java";

/// Nombre del archivo de la ventana, que es donde escribe el generador del
/// disenador.
///
/// Publico por lo mismo que `DESIGNER_FILE`: quien tiene que encontrarlo otra vez
/// para escribir el diseño, o para comprobar que ha llegado al disco, no puede
/// adivinar el nombre.
///
/// Tambien tiene que coincidir con el nombre de la clase, como en cualquier clase
/// publica de Java.
pub const JAVA_WINDOW_FILE: &str = "src/main/java/MainWindow.java";

const JAVA_MAIN_SOURCE: &str = r#"import javax.swing.SwingUtilities;

public class Main {
    public static void main(String[] args) {
        SwingUtilities.invokeLater(() -> new MainWindow().setVisible(true));
    }
}
"#;

/// La ventana de la plantilla: la clase donde escribe el generador del disenador.
///
/// Lleva los cuatro `import` de los componentes que Swing declara, porque el
/// generador escribe el tipo del componente tal cual y sin nombre completo: sin
/// ellos, en cuanto se anadiera un `JButton` al disenador el proyecto dejaria de
/// compilar.
///
/// La zona que escribe MiniIDE viene ya marcada y con una ventana usable dentro.
/// Si la plantilla no trajera la zona, la primera generacion la anadiria al final
/// del archivo, fuera de la clase, y el proyecto dejaria de compilar. Y si viniera
/// vacia, el proyecto recien creado no tendria el metodo que el constructor llama.
///
/// El nombre del metodo y los marcadores vienen del generador y no de aqui: los
/// dos tienen que escribir el mismo texto.
fn java_window_file() -> String {
    format!(
        r#"import javax.swing.JButton;
import javax.swing.JFrame;
import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.JTextField;

public class MainWindow extends JFrame {{
    public MainWindow() {{
        super("MiniIDE");
        {BUILD_METHOD}();
    }}

    {MARKER_BEGIN}
    private void {BUILD_METHOD}() {{
        setSize(640, 480);
        setLocationRelativeTo(null);
    }}
    {MARKER_END}
}}
"#
    )
}

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
/// Cada tipo de proyecto tiene su brazo aqui, y anadir otro es anadir el suyo y
/// nada mas.
pub fn create_project(project_type: ProjectType, root: &Path) -> CoreResult<Project> {
    validate_root_directory(root)?;

    match project_type {
        ProjectType::CSharpWinForms => create_csharp_winforms(root),
        ProjectType::JavaSwing => create_java_swing(root),
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
    let name = project_name_in(root)?;

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

/// Escribe un proyecto de Java con Swing y devuelve su modelo.
///
/// Los fuentes van en `src/main/java`, que es donde los espera el `pom.xml`, y las
/// clases van en el paquete por defecto: un paquete obligaria a que el nombre del
/// directorio del proyecto fuera un identificador, y el nombre lo elige el usuario.
fn create_java_swing(root: &Path) -> CoreResult<Project> {
    let name = project_name_in(root)?;

    // El modelo se construye antes de escribir nada, para no dejar un
    // directorio a medias con un proyecto que despues no se puede abrir.
    let project = Project::new(
        &name,
        root,
        ProjectType::JavaSwing,
        BuildConfiguration::new(ProjectRelativePath::new(
            BuildConfiguration::DEFAULT_OUTPUT_DIRECTORY,
        )?),
    )?;

    std::fs::create_dir_all(root).map_err(|error| CoreError::from_io(root, &error))?;

    write_file(root, "pom.xml", &java_pom(&name))?;
    write_file(root, JAVA_MAIN_FILE, JAVA_MAIN_SOURCE)?;
    write_file(root, JAVA_WINDOW_FILE, &java_window_file())?;

    Ok(project)
}

/// El nombre del proyecto es el del directorio donde vive.
fn project_name_in(root: &Path) -> CoreResult<String> {
    root.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .ok_or_else(|| CoreError::InvalidName(root.display().to_string()))
}

/// El archivo de proyecto de Java.
///
/// `Project::open` reconoce un `pom.xml` como Java con Swing, asi que sin este
/// archivo el proyecto creado no se podria volver a abrir. No lleva configuracion
/// de compilacion porque MiniIDE compila con el JDK y no con Maven: la
/// configuracion que cuenta es la que prepara la toolchain al compilar.
fn java_pom(name: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">

  <modelVersion>4.0.0</modelVersion>
  <groupId>miniide</groupId>
  <artifactId>{artifact}</artifactId>
  <version>1.0.0</version>
  <packaging>jar</packaging>

</project>
"#,
        artifact = artifact_for(name)
    )
}

/// Un `artifactId` de Maven a partir del nombre del directorio.
///
/// El nombre es lo que ve el usuario y puede llevar espacios o acentos, y un
/// `artifactId` solo admite letras ASCII, digitos, `-`, `_` y `.`. Lo que no vale
/// se cambia por `-`, en vez de recortarse: un nombre recortado puede deixar dos
/// proyectos con el mismo `artifactId`.
fn artifact_for(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect()
}

/// Escribe `content` en `relative` dentro de `root`, creando los directorios que
/// falten.
///
/// Los crea porque la plantilla de Java escribe en `src/main/java`, y escribir sin
/// crear antes fallaria en un directorio que todavia no existe.
fn write_file(root: &Path, relative: &str, content: &str) -> CoreResult<()> {
    let path = root.join(relative);
    let parent = path.parent().unwrap_or(root);

    std::fs::create_dir_all(parent).map_err(|error| CoreError::from_io(parent, &error))?;
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
    use crate::framework::{SwingModel, WinFormsModel};
    use crate::generation::{SwingGenerator, WinFormsGenerator};

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
    fn the_java_entry_point_opens_the_window_from_the_event_thread() {
        let main = JAVA_MAIN_SOURCE;

        assert!(main.contains("public class Main "), "{main}");
        assert!(
            main.contains("public static void main(String[] args)"),
            "{main}"
        );
        assert!(
            main.contains("import javax.swing.SwingUtilities;"),
            "{main}"
        );
        // La ventana se muestra en el hilo de Swing, no en el principal: hacerlo
        // al revés es el error clasico de una aplicacion Swing.
        assert!(main.contains("SwingUtilities.invokeLater("), "{main}");
        assert!(main.contains("new MainWindow().setVisible(true)"), "{main}");
    }

    #[test]
    fn the_java_window_is_a_swing_frame_that_can_be_shown() {
        let window = java_window_file();

        assert!(window.contains("import javax.swing.JFrame;"), "{window}");
        assert!(
            window.contains("public class MainWindow extends JFrame"),
            "{window}"
        );
        assert!(window.contains("super("), "{window}");
        assert!(window.contains("setSize(640, 480);"), "{window}");
        assert!(
            window.contains("setLocationRelativeTo(null);"),
            "sin esto la ventana sale en una esquina: {window}"
        );
    }

    /// En Java, una clase publica tiene que vivir en un archivo con su mismo
    /// nombre. Si no, el archivo no compila.
    #[test]
    fn every_java_source_declares_the_class_that_names_its_file() {
        for (path, source) in [
            (JAVA_MAIN_FILE, JAVA_MAIN_SOURCE.to_string()),
            (JAVA_WINDOW_FILE, java_window_file()),
        ] {
            let class = path
                .rsplit('/')
                .next()
                .and_then(|name| name.strip_suffix(".java"))
                .expect("archivo .java");

            assert!(
                source.contains(&format!("public class {class} ")),
                "{path} no declara la clase {class}"
            );
        }
    }

    #[test]
    fn the_java_project_file_declares_the_project_name() {
        let pom = java_pom("App");

        assert!(pom.contains("<modelVersion>4.0.0</modelVersion>"), "{pom}");
        assert!(pom.contains("<artifactId>App</artifactId>"), "{pom}");
        assert!(pom.contains("<packaging>jar</packaging>"), "{pom}");
    }

    /// El nombre del proyecto es un directorio, y un directorio vale `mi proyecto`
    /// mientras que un `artifactId` no: un nombre con espacios haria que el
    /// `pom.xml` generado no valiera.
    #[test]
    fn the_artifact_of_a_directory_that_is_not_an_artifact_is_still_one() {
        for (name, expected) in [
            ("App", "App"),
            ("mi-proyecto", "mi-proyecto"),
            ("Mi Proyecto", "Mi-Proyecto"),
            ("mi.app", "mi.app"),
            ("2024", "2024"),
        ] {
            assert_eq!(artifact_for(name), expected, "artifact de {name}");
        }

        for name in ["año nuevo", "juego 2024", "100%"] {
            let artifact = artifact_for(name);

            assert!(!artifact.is_empty(), "artifact vacio para {name}");

            for character in artifact.chars() {
                assert!(
                    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'),
                    "{artifact} tiene {character:?}, que no vale en un artifact"
                );
            }
        }
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

    #[test]
    fn the_java_window_file_has_the_region_of_the_generator() {
        let window = java_window_file();

        assert!(window.contains(MARKER_BEGIN), "{window}");
        assert!(window.contains(MARKER_END), "{window}");
    }

    #[test]
    fn the_region_of_the_java_window_file_is_inside_the_class() {
        let window = java_window_file();
        let class = window.find("public class MainWindow").expect("la clase");
        let begin = window.find(MARKER_BEGIN).expect("el marcador de inicio");
        let end = window.find(MARKER_END).expect("el marcador de fin");
        let close = window.rfind('}').expect("la llave de cierre");

        assert!(class < begin, "la zona empieza antes de la clase");
        assert!(begin < end, "la zona esta al reves");
        assert!(end < close, "la zona se sale de la clase");
    }

    /// El generador pone los componentes en `buildContent`, asi que el
    /// constructor tiene que llamarla: si no, la ventana se abriria en blanco y el
    /// proyecto recien creado no tendria ni el titulo ni el tamano del disenador.
    #[test]
    fn the_java_window_builds_its_content_from_the_constructor() {
        let window = java_window_file();

        assert!(window.contains(&format!("{BUILD_METHOD}();")), "{window}");
        assert!(
            window.contains(&format!("private void {BUILD_METHOD}()")),
            "{window}"
        );
    }

    /// El proyecto recien creado tiene que compilar y abrir una ventana antes de
    /// que se haya generado nada.
    #[test]
    fn the_java_window_file_has_a_usable_window_before_any_generation() {
        let window = java_window_file();

        assert!(window.contains("setSize(640, 480);"), "{window}");
        assert!(window.contains("setLocationRelativeTo(null);"), "{window}");
    }

    #[test]
    fn the_first_generation_fills_the_region_of_the_java_template() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);
        window
            .add_component("okButton", "JButton", 8, 8, 120, 30)
            .unwrap();

        let result = SwingGenerator
            .apply_to(window.model(), &java_window_file())
            .unwrap();

        assert!(result.contains("private JButton okButton;"), "{result}");
        assert!(result.contains(&format!("{BUILD_METHOD}()")), "{result}");
        assert_eq!(
            result.matches(MARKER_BEGIN).count(),
            1,
            "no se anade una segunda zona"
        );
    }

    /// El codigo que el usuario escribe en el archivo de la ventana, fuera de la
    /// zona, se simulando con un campo que no genera nadie.
    fn java_window_with_user_code() -> String {
        let user_code = "    private int contadorDelUsuario;\n\n    // codigo del usuario\n    private void refrescar() { }\n\n";

        java_window_file().replace(
            &format!("    {MARKER_BEGIN}"),
            &format!("{user_code}    {MARKER_BEGIN}"),
        )
    }

    #[test]
    fn regenerating_the_java_design_keeps_the_code_the_user_wrote() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);
        window
            .add_component("okButton", "JButton", 8, 8, 120, 30)
            .unwrap();

        let generated = SwingGenerator
            .apply_to(window.model(), &java_window_with_user_code())
            .unwrap();

        for expected in [
            "private int contadorDelUsuario;",
            "// codigo del usuario",
            "private void refrescar() { }",
        ] {
            assert!(generated.contains(expected), "se ha perdido: {expected}");
        }
    }

    #[test]
    fn a_component_that_is_deleted_disappears_from_the_java_code() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);
        window
            .add_component("okButton", "JButton", 8, 8, 120, 30)
            .unwrap();
        window
            .add_component("titleLabel", "JLabel", 8, 44, 200, 20)
            .unwrap();

        let first = SwingGenerator
            .apply_to(window.model(), &java_window_file())
            .unwrap();

        window.remove_component("titleLabel");

        let second = SwingGenerator.apply_to(window.model(), &first).unwrap();

        assert!(second.contains("okButton"), "{second}");
        assert!(
            !second.contains("titleLabel"),
            "el componente quitado sigue ahi"
        );
    }

    /// Regenerar sin cambiar nada no puede ir moviendo el codigo de sitio: si fuera
    /// asi, el archivo iria creciendo cada vez que se abre el disenador.
    #[test]
    fn regenerating_the_same_java_design_twice_gives_the_same_file() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);
        window
            .add_component("okButton", "JButton", 8, 8, 120, 30)
            .unwrap();

        let first = SwingGenerator
            .apply_to(window.model(), &java_window_with_user_code())
            .unwrap();
        let second = SwingGenerator.apply_to(window.model(), &first).unwrap();

        assert_eq!(first, second);
    }

    /// Si el usuario ha dejado la zona sin cerrar, no se toca el archivo: cortar
    /// por donde toca se llevaria su codigo.
    #[test]
    fn a_java_region_left_open_by_hand_is_refused() {
        let mut window = SwingModel::new("MainWindow", "Ventana", 640, 480);
        window
            .add_component("okButton", "JButton", 8, 8, 120, 30)
            .unwrap();

        let broken = java_window_file().replace(
            &format!("    {MARKER_END}\n"),
            "    // se ha borrado el cierre de la zona\n",
        );

        let result = SwingGenerator.apply_to(window.model(), &broken);

        assert!(matches!(result, Err(CoreError::MalformedSource(_))));
    }

    #[test]
    fn the_java_window_file_imports_every_component_it_can_generate() {
        // Un componente del disenador se genera con su tipo y sin nombre completo,
        // asi que el archivo tiene que importar de antemano los cuatro tipos que
        // Swing declara: si no, en cuanto se anadiera uno el proyecto dejaria de
        // compilar.
        for tipo in ["JButton", "JLabel", "JTextField", "JPanel"] {
            assert!(
                java_window_file().contains(&format!("import javax.swing.{tipo};")),
                "falta el import de {tipo}"
            );
        }
    }

    #[test]
    fn the_java_window_class_and_its_file_have_the_same_name() {
        assert!(java_window_file().contains("public class MainWindow extends JFrame"));
        assert!(JAVA_WINDOW_FILE.ends_with("MainWindow.java"));
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
