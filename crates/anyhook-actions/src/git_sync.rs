use anyhook_core::traits::{Action, ActionContext};
use anyhook_core::error::{Result, AnyhookError};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use std::process::Command;
use std::path::Path;

/// 知识点: GitSyncAction 专为自动部署设计。
/// 它会判断目标目录是否为有效的 Git 仓库，如果不存在就 clone，如果存在就 pull。
#[derive(Debug)]
pub struct GitSyncAction;

impl GitSyncAction {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct GitSyncConfig {
    repo_url: String,
    target_dir: String,
    #[serde(default = "default_branch")]
    branch: String,
}

fn default_branch() -> String { "main".to_string() }

#[async_trait]
impl Action for GitSyncAction {
    fn action_type(&self) -> &'static str { "git_sync" }

    async fn execute(&self, ctx: ActionContext) -> Result<serde_json::Value> {
        let config: GitSyncConfig = serde_json::from_value(ctx.config)
            .map_err(|e| AnyhookError::Execution(format!("Invalid git_sync config: {}", e)))?;
            
        let is_repo = std::path::Path::new(&config.target_dir).join(".git").exists();
        
        let result = tokio::task::spawn_blocking(move || {
            let dir = std::path::PathBuf::from(&config.target_dir);
            if is_repo {
                // 已存在，执行 git pull
                Command::new("git")
                    .current_dir(&dir)
                    .args(["pull", "origin", &config.branch])
                    .output()
            } else {
                // 不存在，执行 git clone
                Command::new("git")
                    .args(["clone", "-b", &config.branch, &config.repo_url, &config.target_dir])
                    .output()
            }
        }).await.map_err(|e| AnyhookError::Execution(e.to_string()))?;
        
        let output = result.map_err(|e| AnyhookError::Execution(e.to_string()))?;
        
        if !output.status.success() {
            let err_str = String::from_utf8_lossy(&output.stderr);
            return Err(AnyhookError::Execution(format!("Git command failed: {}", err_str)));
        }
        
        Ok(json!({
            "status": "success",
            "operation": if is_repo { "pull" } else { "clone" },
            "output": String::from_utf8_lossy(&output.stdout).to_string()
        }))
    }
}
