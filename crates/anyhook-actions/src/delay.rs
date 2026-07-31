use anyhook_core::traits::{Action, ActionContext};
use anyhook_core::error::{Result, AnyhookError};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

/// 知识点: DelayAction 用于在流水线中故意停顿一段时间，比如等待文件系统落盘、或限制请求频率。
#[derive(Debug)]
pub struct DelayAction;

impl DelayAction {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct DelayConfig {
    milliseconds: u64,
}

#[async_trait]
impl Action for DelayAction {
    fn action_type(&self) -> &'static str { "delay" }

    async fn execute(&self, ctx: ActionContext) -> Result<serde_json::Value> {
        let config: DelayConfig = serde_json::from_value(ctx.config)
            .map_err(|e| AnyhookError::Execution(format!("Invalid delay config: {}", e)))?;
            
        tracing::info!("Delay Action: sleeping for {} ms", config.milliseconds);
        tokio::time::sleep(std::time::Duration::from_millis(config.milliseconds)).await;
        
        Ok(json!({
            "status": "success",
            "delayed_ms": config.milliseconds
        }))
    }
}
