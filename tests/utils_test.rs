use node_version_manager::utils;

#[test]
fn test_get_version_dir_name() {
    let dir = utils::get_version_dir_name("v22.14.0");
    assert!(dir.starts_with("node-v22.14.0-"));
    assert!(dir.contains("x64") || dir.contains("arm64"));
}

#[test]
fn test_platform_suffix() {
    let suffix = utils::platform_suffix();
    assert!(!suffix.is_empty());
}

