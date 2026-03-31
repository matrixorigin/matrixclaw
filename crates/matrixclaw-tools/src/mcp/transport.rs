use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use super::types::{JsonRpcRequest, JsonRpcResponse};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub struct StdioTransport {
    stdin: Mutex<tokio::process::ChildStdin>,
    stdout: Mutex<BufReader<tokio::process::ChildStdout>>,
    #[allow(dead_code)]
    child: Child,
}

impl std::fmt::Debug for StdioTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StdioTransport").finish_non_exhaustive()
    }
}

impl StdioTransport {
    pub async fn spawn(
        command: &str,
        args: &[String],
        env: &[(String, String)],
    ) -> Result<Self, String> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .envs(env.iter().cloned())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn MCP server '{}': {e}", command))?;

        let stdin = child
            .stdin
            .take()
            .ok_or("failed to get stdin of MCP server")?;
        let stdout = child
            .stdout
            .take()
            .ok_or("failed to get stdout of MCP server")?;

        Ok(Self {
            stdin: Mutex::new(stdin),
            stdout: Mutex::new(BufReader::new(stdout)),
            child,
        })
    }

    pub async fn send_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<JsonRpcResponse, String> {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let request = JsonRpcRequest::new(id, method, params);
        let mut line = serde_json::to_string(&request)
            .map_err(|e| format!("failed to serialize JSON-RPC request: {e}"))?;
        line.push('\n');

        let mut stdin = self.stdin.lock().await;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| format!("failed to write to MCP server stdin: {e}"))?;
        stdin
            .flush()
            .await
            .map_err(|e| format!("failed to flush MCP server stdin: {e}"))?;
        drop(stdin);

        let mut stdout = self.stdout.lock().await;
        let mut response_line = String::new();
        stdout
            .read_line(&mut response_line)
            .await
            .map_err(|e| format!("failed to read from MCP server stdout: {e}"))?;

        let response: JsonRpcResponse = serde_json::from_str(response_line.trim()).map_err(|e| {
            format!(
                "failed to parse JSON-RPC response: {e}; line={response_line}"
            )
        })?;

        Ok(response)
    }

    pub async fn send_notification(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<(), String> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params.unwrap_or(serde_json::json!({})),
        });
        let mut line = serde_json::to_string(&notification)
            .map_err(|e| format!("failed to serialize JSON-RPC notification: {e}"))?;
        line.push('\n');

        let mut stdin = self.stdin.lock().await;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| format!("failed to write notification to MCP server stdin: {e}"))?;
        stdin
            .flush()
            .await
            .map_err(|e| format!("failed to flush MCP server stdin: {e}"))?;

        Ok(())
    }
}
