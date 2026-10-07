use node_version_manager::version_service::NodeVersion;

#[test]
fn test_parse_node_version_lts_named() {
    let json = r#"{"version":"v22.14.0","date":"2025-02-11","lts":"Jod"}"#;
    let v: NodeVersion = serde_json::from_str(json).unwrap();
    assert_eq!(v.version, "v22.14.0");
    assert!(v.is_lts());
    assert_eq!(v.lts_name(), Some("Jod"));
}

#[test]
fn test_parse_node_version_non_lts() {
    let json = r#"{"version":"v23.8.0","date":"2025-02-18","lts":false}"#;
    let v: NodeVersion = serde_json::from_str(json).unwrap();
    assert_eq!(v.version, "v23.8.0");
    assert!(!v.is_lts());
    assert_eq!(v.lts_name(), None);
}

#[test]
fn test_fetch_node_versions() {
    let res = node_version_manager::version_service::fetch_node_versions();
    assert!(res.is_ok(), "Fetch failed: {:?}", res.err());
    let list = res.unwrap();
    assert!(!list.is_empty(), "Node versions list should not be empty");
}

