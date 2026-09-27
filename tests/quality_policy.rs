use std::fs;
use std::path::Path;

fn read_repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);

    match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => panic!("could not read {}: {error}", path.display()),
    }
}

#[test]
fn format_config_pins_edition_and_width() {
    let config = read_repo_file("rustfmt.toml");

    assert!(config.contains("edition = \"2021\""));
    assert!(config.contains("max_width = 100"));
}

#[test]
fn lint_config_declares_msrv() {
    let config = read_repo_file("clippy.toml");

    assert!(config.contains("msrv = \"1.75\""));
}

#[test]
fn manifest_declares_the_warning_policy() {
    let manifest = read_repo_file("Cargo.toml");

    for expected in [
        "[lints.rust]",
        "unsafe_code = \"forbid\"",
        "[lints.clippy]",
        "all = \"deny\"",
    ] {
        assert!(
            manifest.contains(expected),
            "Cargo.toml must declare `{expected}`"
        );
    }
}

#[test]
fn quality_policy_is_documented() {
    let policy = read_repo_file("docs/quality.md");

    for expected in ["cargo fmt --check", "cargo clippy", "cargo test", "T-003"] {
        assert!(
            policy.contains(expected),
            "docs/quality.md must mention `{expected}`"
        );
    }
}
