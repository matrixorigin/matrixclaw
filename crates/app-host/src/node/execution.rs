use std::io;
use std::path::PathBuf;

use matrixclaw_manifests::config::ExecutionSettings;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::execution::route_isolated_command;
use crate::local_command::{execute_local_command, LocalCommandRequest};
use crate::sandbox_backend::{LocalSandboxBackend, SandboxExecutionRequest};

const EXECUTION_NODE_CAPABILITY_REQUEST_KIND: &str = "execution-node.capability-request";
const EXECUTION_NODE_CAPABILITY_RESULT_KIND: &str = "execution-node.capability-result";
const HOST_COMMAND_CAPABILITY: &str = "host.command";
const NODE_BACKEND_LABEL: &str = "node";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionNodeCapabilityRequest {
    pub kind: String,
    pub capability: String,
    #[serde(default)]
    pub policy: Option<ExecutionSettings>,
    pub request: ExecutionNodeCommandRequest,
}

impl ExecutionNodeCapabilityRequest {
    pub fn host_command(
        policy: Option<ExecutionSettings>,
        command: impl Into<String>,
        args: Vec<String>,
        cwd: Option<PathBuf>,
    ) -> Self {
        Self {
            kind: EXECUTION_NODE_CAPABILITY_REQUEST_KIND.to_string(),
            capability: HOST_COMMAND_CAPABILITY.to_string(),
            policy,
            request: ExecutionNodeCommandRequest {
                command: command.into(),
                args,
                cwd,
            },
        }
    }

    pub fn host_command_from_tool_arguments(
        arguments: &str,
        policy: Option<ExecutionSettings>,
    ) -> io::Result<Self> {
        let mut parts = arguments
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let command = parts.next().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "host.command requires a command name",
            )
        })?;

        Ok(Self::host_command(
            policy,
            command.to_string(),
            parts.map(str::to_string).collect(),
            None,
        ))
    }

    pub fn execute(&self) -> io::Result<ExecutionNodeCapabilityResponse> {
        if self.kind != EXECUTION_NODE_CAPABILITY_REQUEST_KIND {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unsupported execution node request kind: {}", self.kind),
            ));
        }

        if self.capability != HOST_COMMAND_CAPABILITY {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unsupported execution node capability: {}", self.capability),
            ));
        }

        if let Some(policy) = &self.policy {
            return self.execute_with_policy(policy);
        }

        self.execute_legacy()
    }

    fn execute_legacy(&self) -> io::Result<ExecutionNodeCapabilityResponse> {
        let mut command =
            LocalCommandRequest::new(self.request.command.clone(), self.request.args.clone());
        if let Some(cwd) = &self.request.cwd {
            command = command.with_cwd(cwd.clone());
        }

        let result = execute_local_command(&command)?;
        Ok(ExecutionNodeCapabilityResponse {
            request: self.clone(),
            result: ExecutionNodeCapabilityResult {
                kind: EXECUTION_NODE_CAPABILITY_RESULT_KIND.to_string(),
                backend: NODE_BACKEND_LABEL.to_string(),
                exit_code: result.exit_code,
                stdout: trim_trailing_newlines(&result.stdout),
                stderr: trim_trailing_newlines(&result.stderr),
            },
        })
    }

    fn execute_with_policy(
        &self,
        policy: &ExecutionSettings,
    ) -> io::Result<ExecutionNodeCapabilityResponse> {
        let request = SandboxExecutionRequest {
            command: self.request.command.clone(),
            args: self.request.args.clone(),
            cwd: self.request.cwd.clone(),
        };
        let mut sandbox_backend = LocalSandboxBackend::from_settings(policy);
        let structured = route_isolated_command(policy, Some(&mut sandbox_backend), &request)?;

        Ok(ExecutionNodeCapabilityResponse {
            request: self.clone(),
            result: ExecutionNodeCapabilityResult {
                kind: EXECUTION_NODE_CAPABILITY_RESULT_KIND.to_string(),
                backend: structured.backend.label,
                exit_code: structured.exit_code,
                stdout: trim_trailing_newlines(&structured.stdout),
                stderr: trim_trailing_newlines(&structured.stderr),
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionNodeCommandRequest {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionNodeCapabilityResult {
    pub kind: String,
    pub backend: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionNodeCapabilityResponse {
    pub request: ExecutionNodeCapabilityRequest,
    pub result: ExecutionNodeCapabilityResult,
}

impl ExecutionNodeCapabilityResponse {
    pub fn into_tool_output(self) -> io::Result<String> {
        serde_json::to_string(&self.result)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}

fn trim_trailing_newlines(value: &str) -> String {
    value.trim_end_matches(['\r', '\n']).to_string()
}
