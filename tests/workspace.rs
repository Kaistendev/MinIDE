use miniide::{APP_NAME, APP_VERSION};

#[test]
fn app_name_is_miniide() {
    assert_eq!(APP_NAME, "MiniIDE");
}

#[test]
fn app_version_matches_package_metadata() {
    assert_eq!(APP_VERSION, env!("CARGO_PKG_VERSION"));
}
