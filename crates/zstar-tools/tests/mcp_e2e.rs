use std::io::Write;
use std::sync::Arc;

fn create_mock_server(dir: &std::path::Path) -> std::path::PathBuf {
    let script = dir.join("mock_mcp.sh");
    let mut f = std::fs::File::create(&script).unwrap();
    writeln!(f, "#!/bin/sh").unwrap();
    // Read initialize request, respond
    writeln!(f, "read LINE").unwrap();
    writeln!(f, r#"echo '{{"jsonrpc":"2.0","id":1,"result":{{"protocolVersion":"2024-11-05","capabilities":{{}},"serverInfo":{{"name":"mock","version":"1.0.0"}}}}}}'"#).unwrap();
    // Read initialized notification (fire-and-forget), then read tools/list
    writeln!(f, "read LINE").unwrap();
    writeln!(f, r#"echo '{{"jsonrpc":"2.0","id":2,"result":{{"tools":[{{"name":"echo","description":"Echoes back input","inputSchema":{{"type":"object","properties":{{"message":{{"type":"string","description":"Text to echo"}}}},"required":["message"]}}}}]}}}}'"#).unwrap();
    // Read tools/call request, respond
    writeln!(f, "read LINE").unwrap();
    writeln!(f, r#"echo '{{"jsonrpc":"2.0","id":3,"result":{{"content":[{{"type":"text","text":"mocked output"}}],"isError":false}}}}'"#).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    script
}

#[tokio::test]
async fn mcp_e2e_connect_list_and_call() {
    let dir = tempfile::tempdir().unwrap();
    let script = create_mock_server(dir.path());

    let client = zstar_tools::mcp::client::McpClient::connect(script.to_str().unwrap(), &[], &[])
        .await
        .unwrap();

    let client = Arc::new(client.initialize().await.unwrap());
    let tools = client.list_tools().await.unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "echo");

    let result = client
        .call_tool("echo", serde_json::json!({"message": "hello"}))
        .await
        .unwrap();
    assert!(!result.is_error());
    assert_eq!(result.text_output(), "mocked output");
}

#[tokio::test]
async fn mcp_adapter_registers_and_executes() {
    let dir = tempfile::tempdir().unwrap();
    let script = create_mock_server(dir.path());

    let registry = zstar_tools::ToolRegistry::new();

    let client = zstar_tools::mcp::client::McpClient::connect(script.to_str().unwrap(), &[], &[])
        .await
        .unwrap();

    let client = Arc::new(client.initialize().await.unwrap());
    let tools = client.list_tools().await.unwrap();

    let tool = &tools[0];
    let mut namespaced = tool.clone();
    namespaced.name = format!("mcp__mock__{}", tool.name);
    let adapter = zstar_tools::mcp::adapter::McpToolAdapter::new(&namespaced, Arc::clone(&client));
    registry.register(Arc::new(adapter)).await;

    assert!(registry.has("mcp__mock__echo").await);

    let call = zstar_tools::ToolCall::new(
        "test-id".to_string(),
        "mcp__mock__echo".to_string(),
        serde_json::json!({"message": "test"}),
    );
    let result = registry.execute(call).await;
    assert!(!result.is_error);
    assert_eq!(result.output, "mocked output");
}
