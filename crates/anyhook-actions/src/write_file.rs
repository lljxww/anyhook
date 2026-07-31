use anyhook_core::traits::{Action, ActionContext};
use anyhook_core::error::{Result, AnyhookError};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

/// 知识点: WriteFileAction 提供原生的文件写入功能，可以用于保存日志、转存 Webhook 负载等。
#[derive(Debug)]
pub struct WriteFileAction;

impl WriteFileAction {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct WriteFileConfig {
    path: String,
    content: String,
    #[serde(default)]
    append: bool,
}

#[async_trait]
impl Action for WriteFileAction {
    fn action_type(&self) -> &'static str { "write_file" }

    async fn execute(&self, ctx: ActionContext) -> Result<serde_json::Value> {
        let config: WriteFileConfig = serde_json::from_value(ctx.config)
            .map_err(|e| AnyhookError::Execution(format!("Invalid write_file config: {}", e)))?;
            
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(config.append)
            .truncate(!config.append)
            .open(&config.path)
            .await
            .map_err(|e| AnyhookError::Execution(e.to_string()))?;
            
        file.write_all(config.content.as_bytes())
            .await
            .map_err(|e| AnyhookError::Execution(e.to_string()))?;
        
        Ok(json!({
            "status": "success",
            "path": config.path,
            "bytes_written": config.content.len()
        }))
    }
}
