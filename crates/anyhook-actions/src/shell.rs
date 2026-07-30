use anyhook_core::traits::Action;
use anyhook_core::error::{Result, AnyhookError};
use anyhook_core::traits::ActionContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, error};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    pub command: String,
}

#[derive(Debug)]
pub struct ShellAction;

impl ShellAction {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Action for ShellAction {
    fn action_type(&self) -> &'static str {
        "shell"
    }

    async fn execute(&self, ctx: ActionContext) -> Result<serde_json::Value> {
        let config: ShellConfig = serde_json::from_value(ctx.config.clone())
            .map_err(|e| AnyhookError::Config(format!("Invalid shell config: {}", e)))?;

        info!("Shell Action: executing command: {}", config.command);

        let (shell, arg) = if cfg!(target_os = "windows") {
            ("powershell", "-Command")
        } else {
            ("bash", "-c")
        };

        let output = Command::new(shell)
            .arg(arg)
            .arg(&config.command)
            .output()
            .await
            .map_err(|e| AnyhookError::Execution(format!("Failed to spawn shell command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            info!("Shell Action succeeded.");
            Ok(json!({
                "status": "success",
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": output.status.code(),
            }))
        } else {
            error!("Shell Action failed with exit code: {:?}", output.status.code());
            Err(AnyhookError::Execution(format!("Shell command failed: stdout: {}, stderr: {}", stdout, stderr)))
        }
    }
}
