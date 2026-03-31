use matrixclaw_tools::mcp::types::{CallToolResult, McpTool, ToolContent};

#[test]
fn call_tool_result_extracts_text() {
    let result = CallToolResult {
        content: vec![ToolContent::Text {
            text: "hello world".to_string(),
        }],
        is_error: Some(false),
    };
    assert_eq!(result.text_output(), "hello world");
    assert!(!result.is_error());
}

#[test]
fn call_tool_result_joins_multiple_text() {
    let result = CallToolResult {
        content: vec![
            ToolContent::Text {
                text: "part1".to_string(),
            },
            ToolContent::Text {
                text: "part2".to_string(),
            },
        ],
        is_error: None,
    };
    assert_eq!(result.text_output(), "part1\npart2");
    assert!(!result.is_error());
}

#[test]
fn call_tool_result_detects_error() {
    let result = CallToolResult {
        content: vec![ToolContent::Text {
            text: "something went wrong".to_string(),
        }],
        is_error: Some(true),
    };
    assert!(result.is_error());
}

#[test]
fn mcp_tool_descriptor_name_preserved() {
    let tool = McpTool {
        name: "my_tool".to_string(),
        description: Some("Does stuff".to_string()),
        input_schema: None,
    };
    assert_eq!(tool.name, "my_tool");
    assert_eq!(tool.description.as_deref(), Some("Does stuff"));
}

#[test]
fn json_rpc_request_serializes_correctly() {
    let req = matrixclaw_tools::mcp::types::JsonRpcRequest::new(
        42,
        "tools/list",
        Some(serde_json::json!({})),
    );
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"id\":42"));
    assert!(json.contains("\"method\":\"tools/list\""));
}

#[test]
fn json_rpc_response_parses_result() {
    let json = r#"{"jsonrpc":"2.0","id":1,"result":{"tools":[]}}"#;
    let resp: matrixclaw_tools::mcp::types::JsonRpcResponse = serde_json::from_str(json).unwrap();
    assert!(resp.result.is_some());
    assert!(resp.error.is_none());
}

#[test]
fn json_rpc_response_parses_error() {
    let json = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid Request"}}"#;
    let resp: matrixclaw_tools::mcp::types::JsonRpcResponse = serde_json::from_str(json).unwrap();
    assert!(resp.result.is_none());
    let err = resp.error.unwrap();
    assert_eq!(err.code, -32600);
    assert_eq!(err.message, "Invalid Request");
}
