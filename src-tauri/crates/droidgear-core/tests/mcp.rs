use droidgear_core::mcp::{self, McpServer, McpServerConfig, McpServerType};
use std::collections::HashMap;
use tempfile::TempDir;

#[test]
fn mcp_save_load_and_toggle_roundtrip() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();

    let mut env = HashMap::new();
    env.insert("EXAMPLE".to_string(), "1".to_string());

    let server = McpServer {
        name: "test".to_string(),
        config: McpServerConfig {
            server_type: McpServerType::Stdio,
            disabled: false,
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "exa-mcp-server".to_string()]),
            env: Some(env),
            url: None,
            headers: None,
        },
    };

    mcp::save_mcp_server_for_home(home, server).unwrap();

    let loaded = mcp::load_mcp_servers_for_home(home).unwrap();
    let loaded = loaded.into_iter().find(|s| s.name == "test").unwrap();
    assert_eq!(loaded.config.server_type, McpServerType::Stdio);
    assert_eq!(loaded.config.command.as_deref(), Some("npx"));
    assert_eq!(
        loaded.config.args.as_deref(),
        Some(&["-y".to_string(), "exa-mcp-server".to_string()][..])
    );
    assert_eq!(
        loaded
            .config
            .env
            .as_ref()
            .and_then(|m| m.get("EXAMPLE"))
            .map(String::as_str),
        Some("1")
    );

    mcp::toggle_mcp_server_for_home(home, "test", true).unwrap();
    let loaded2 = mcp::load_mcp_servers_for_home(home).unwrap();
    let loaded2 = loaded2.into_iter().find(|s| s.name == "test").unwrap();
    assert!(loaded2.config.disabled);
}

#[test]
fn baizhi_preset_preserves_unknown_top_level_and_other_records() {
    let temp = TempDir::new().unwrap();
    let factory = temp.path().join(".factory");
    std::fs::create_dir_all(&factory).unwrap();
    let original = serde_json::json!({
        "vendorOption": {"nested": [1, null, "synthetic"]},
        "mcpServers": {
            "other": {"type": "future-transport", "oauth": {"clientId": "synthetic-client"}, "headers": {"X-Example": "fixture"}},
            "disabled-other": {"type": "http", "disabled": true, "url": "https://example.test/mcp"}
        }
    });
    let path = factory.join("mcp.json");
    std::fs::write(&path, original.to_string()).unwrap();
    mcp::save_baizhi_preset_for_home(temp.path()).unwrap();
    let actual: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(actual["vendorOption"], original["vendorOption"]);
    assert_eq!(
        actual["mcpServers"]["other"],
        original["mcpServers"]["other"]
    );
    assert_eq!(
        actual["mcpServers"]["disabled-other"],
        original["mcpServers"]["disabled-other"]
    );
    assert_eq!(
        actual["mcpServers"]["baizhi"],
        serde_json::json!({
            "type": "http", "disabled": true, "oauth": false,
            "url": "https://agent-toolkit.app.baizhi.cloud/mcp",
            "headers": {"Authorization": "Bearer ${BAIZHI_API_KEY}"}
        })
    );
    let loaded = mcp::load_mcp_servers_for_home(temp.path()).unwrap();
    let baizhi = loaded
        .iter()
        .find(|server| server.name == "baizhi")
        .unwrap();
    assert!(baizhi.config.disabled);
    assert!(baizhi.config.env.is_none());
    assert!(baizhi.config.command.is_none());
    mcp::toggle_mcp_server_for_home(temp.path(), "baizhi", false).unwrap();
    let toggled: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(toggled["mcpServers"]["baizhi"]["oauth"], false);
    assert_eq!(toggled["mcpServers"]["baizhi"]["disabled"], false);
}

#[test]
fn baizhi_preset_refuses_every_existing_raw_name_without_changing_bytes() {
    for record in [
        serde_json::Value::Null,
        serde_json::json!({"type":"unknown"}),
        serde_json::json!({"type":"http","disabled":false,"url":"https://example.test/custom","oauth":{"clientId":"synthetic-client"}}),
    ] {
        let temp = TempDir::new().unwrap();
        let factory = temp.path().join(".factory");
        std::fs::create_dir_all(&factory).unwrap();
        let contents =
            format!("{{\n  \"mcpServers\": {{\"baizhi\": {record}}},\n  \"unknown\": true\n}}\n");
        let path = factory.join("mcp.json");
        std::fs::write(&path, &contents).unwrap();
        assert!(mcp::save_baizhi_preset_for_home(temp.path())
            .unwrap_err()
            .contains("already exists"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), contents);
    }
}

#[test]
fn baizhi_preset_rejects_malformed_file_or_shape_without_changing_bytes() {
    for contents in [
        "{malformed",
        "[]",
        "{\"mcpServers\":null}",
        "{\"mcpServers\":[]}",
    ] {
        let temp = TempDir::new().unwrap();
        let factory = temp.path().join(".factory");
        std::fs::create_dir_all(&factory).unwrap();
        let path = factory.join("mcp.json");
        std::fs::write(&path, contents).unwrap();
        assert!(mcp::save_baizhi_preset_for_home(temp.path()).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), contents);
    }
}

#[test]
fn baizhi_preset_write_failure_preserves_original_file() {
    let temp = TempDir::new().unwrap();
    let factory = temp.path().join(".factory");
    std::fs::create_dir_all(factory.join("mcp.tmp")).unwrap();
    let contents = "{\"mcpServers\":{},\"unknown\":true}\n";
    let path = factory.join("mcp.json");
    std::fs::write(&path, contents).unwrap();
    assert!(mcp::save_baizhi_preset_for_home(temp.path()).is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), contents);
}

#[test]
fn ordinary_save_can_still_update_an_existing_baizhi_entry() {
    let temp = TempDir::new().unwrap();
    mcp::save_baizhi_preset_for_home(temp.path()).unwrap();
    let mut server = mcp::load_mcp_servers_for_home(temp.path())
        .unwrap()
        .remove(0);
    server.config.disabled = false;
    server.config.headers = Some(HashMap::from([(
        "X-Custom".to_string(),
        "fixture".to_string(),
    )]));
    mcp::save_mcp_server_for_home(temp.path(), server).unwrap();
    let actual = mcp::load_mcp_servers_for_home(temp.path()).unwrap();
    assert!(!actual[0].config.disabled);
    assert_eq!(
        actual[0]
            .config
            .headers
            .as_ref()
            .unwrap()
            .get("X-Custom")
            .map(String::as_str),
        Some("fixture")
    );
}
