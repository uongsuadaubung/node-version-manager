use node_version_manager::config::AppConfig;

#[test]
fn test_default_config() {
    let config = AppConfig::default();
    assert!(config.base_dir.to_string_lossy().contains(".nvm-rust"));
    assert!(config.current_version.is_none());
    assert_eq!(config.language, "en");
    assert!(config.installed_versions.is_empty());
}

#[test]
fn test_config_paths() {
    let config = AppConfig::default();
    let v_dir = config.versions_dir();
    let m_dir = config.modules_dir();

    assert!(v_dir.ends_with("versions"));
    assert!(m_dir.ends_with("modules"));
}

#[test]
fn test_config_toml_roundtrip() {
    let mut config = AppConfig {
        current_version: Some("v20.10.0".to_string()),
        language: "vi".to_string(),
        installed_versions: vec!["v18.19.0".to_string(), "v20.10.0".to_string()],
        ..AppConfig::default()
    };
    config
        .version_configs
        .insert("v20.10.0".to_string(), true);
    config
        .version_configs
        .insert("v18.19.0".to_string(), false);

    let toml_str = config.to_toml_string();
    let parsed = AppConfig::from_toml_str(&toml_str).expect("Failed to parse generated toml");

    assert_eq!(parsed.base_dir, config.base_dir);
    assert_eq!(parsed.current_version, Some("v20.10.0".to_string()));
    assert_eq!(parsed.language, "vi");
    assert_eq!(
        parsed.installed_versions,
        vec!["v18.19.0".to_string(), "v20.10.0".to_string()]
    );
    assert_eq!(parsed.version_configs.get("v20.10.0"), Some(&true));
    assert_eq!(parsed.version_configs.get("v18.19.0"), Some(&false));
}

#[test]
fn test_config_from_multiline_toml() {
    let sample = r#"
# Comment line
base_dir = "C:\\Users\\test\\.nvm-rust"
current_version = "v22.0.0"
language = "en"
installed_versions = [
    "v18.0.0",
    "v20.0.0",
    "v22.0.0"
]

[version_configs]
"v22.0.0" = true
v18.0.0 = false
"#;

    let parsed = AppConfig::from_toml_str(sample).expect("Failed to parse sample");
    assert_eq!(parsed.current_version, Some("v22.0.0".to_string()));
    assert_eq!(parsed.language, "en");
    assert_eq!(
        parsed.installed_versions,
        vec!["v18.0.0".to_string(), "v20.0.0".to_string(), "v22.0.0".to_string()]
    );
    assert_eq!(parsed.version_configs.get("v22.0.0"), Some(&true));
    assert_eq!(parsed.version_configs.get("v18.0.0"), Some(&false));
}

