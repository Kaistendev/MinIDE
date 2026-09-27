use std::fs;
use std::path::{Path, PathBuf};

fn read_repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);

    match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => panic!("could not read {}: {error}", path.display()),
    }
}

fn logic_modules() -> Vec<PathBuf> {
    fn visit(directory: &Path, root: bool, found: &mut Vec<PathBuf>) {
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) => panic!("could not read {}: {error}", directory.display()),
        };

        for entry in entries {
            let path = entry.expect("readable directory entry").path();

            if path.is_dir() {
                visit(&path, false, found);
                continue;
            }

            if path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }

            if root
                && matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("lib.rs" | "main.rs")
                )
            {
                continue;
            }

            found.push(path);
        }
    }

    let mut found = Vec::new();
    visit(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        true,
        &mut found,
    );
    found.sort();

    found
}

#[test]
fn every_logic_module_has_unit_tests() {
    let modules = logic_modules();

    assert!(!modules.is_empty(), "no logic modules found under src/");

    for module in &modules {
        let content = fs::read_to_string(module).expect("readable module source");

        assert!(
            content.contains("#[cfg(test)]"),
            "{} must contain a `#[cfg(test)] mod tests` unit test module",
            module.display()
        );
    }
}

#[test]
fn crate_documentation_includes_a_compiled_example() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_source =
        fs::read_to_string(manifest_dir.join("src").join("lib.rs")).expect("readable lib.rs");

    assert!(
        lib_source.contains("use miniide::"),
        "the crate documentation must contain a runnable example using the public API"
    );
}

#[test]
fn testing_strategy_is_documented() {
    let strategy = read_repo_file("docs/testing.md");

    for expected in [
        "#[cfg(test)]",
        "tests/",
        "#[ignore]",
        "golden",
        "regresión",
        "cargo test",
        "T-004",
    ] {
        assert!(
            strategy.contains(expected),
            "docs/testing.md must document `{expected}`"
        );
    }
}

#[test]
fn filesystem_and_workspace_tests_do_not_depend_on_the_ui() {
    for file in ["tests/filesystem.rs", "tests/editor_flow.rs"] {
        let source = read_repo_file(file);

        assert!(
            !source.contains("miniide::frontend"),
            "{file} must exercise the filesystem and the workspace without the UI"
        );
    }
}

/// Identificadores de tarea citados en un documento, como `FE-013` o `T-055`.
///
/// Se separan por todo lo que no sea una letra, un digito o un guion, y se
/// ignoran los que no llevan los tres digitos de la numeracion.
fn cited_task_ids(document: &str, prefix: &str) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();

    for token in
        document.split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
    {
        let digits = token.strip_prefix(prefix).unwrap_or_default();

        let is_task_id = digits.len() == 3 && digits.chars().all(|digit| digit.is_ascii_digit());

        if is_task_id && !ids.contains(&token.to_string()) {
            ids.push(token.to_string());
        }
    }

    ids.sort();

    ids
}

#[test]
fn the_frontend_plan_declares_the_stack_it_uses() {
    let plan = read_repo_file("docs/frontend-plan.md");
    let tasks = read_repo_file("docs/frontend-tasks.md");

    assert!(
        plan.contains("eframe") && plan.contains("egui"),
        "docs/frontend-plan.md must declare egui/eframe as the frontend stack"
    );
    assert!(
        tasks.contains("egui/eframe"),
        "docs/frontend-tasks.md must state the stack it implements"
    );
}

#[test]
fn the_two_task_documents_reference_each_other() {
    let tasks = read_repo_file("docs/tasks.md");
    let frontend = read_repo_file("docs/frontend-tasks.md");

    assert!(
        tasks.contains("frontend-tasks.md"),
        "docs/tasks.md must point at the frontend tasks"
    );
    assert!(
        frontend.contains("tasks.md"),
        "docs/frontend-tasks.md must point at the core tasks"
    );
}

#[test]
fn every_frontend_task_cited_by_the_core_plan_exists() {
    let core = read_repo_file("docs/tasks.md");
    let frontend = read_repo_file("docs/frontend-tasks.md");

    let known = cited_task_ids(&frontend, "FE-");

    for cited in cited_task_ids(&core, "FE-") {
        assert!(
            known.contains(&cited),
            "docs/tasks.md cites {cited}, which is not a task in docs/frontend-tasks.md"
        );
    }
}

#[test]
fn every_core_task_cited_by_the_frontend_plan_exists() {
    let core = read_repo_file("docs/tasks.md");
    let frontend = read_repo_file("docs/frontend-tasks.md");

    let known = cited_task_ids(&core, "T-");

    for cited in cited_task_ids(&frontend, "T-") {
        assert!(
            known.contains(&cited),
            "docs/frontend-tasks.md cites {cited}, which is not a task in docs/tasks.md"
        );
    }
}
