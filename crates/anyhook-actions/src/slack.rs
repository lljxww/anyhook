use anyhook_core::traits::{Action, ActionContext};
use anyhook_core::error::{Result, AnyhookError};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use reqwest::Client;

/// 知识点: SlackAction 用于推送消息到 Slack 频道的 Webhook，常用于开发者的团队通知。
#[derive(Debug)]
pub struct SlackAction {
    client: Client,
}

impl SlackAction {
    pub fn new() -> Self { 
        Self { client: Client::builder().use_rustls_tls().build().unwrap() }
    }
}

#[derive(Deserialize)]
struct SlackConfig {
    webhook_url: String,
    text: String,
}

#[async_trait]
impl Action for SlackAction {
    fn action_type(&self) -> &'static str { "slack" }

    async fn execute(&self, ctx: ActionContext) -> Result<serde_json::Value> {
        let config: SlackConfig = serde_json::from_value(ctx.config)
            .map_err(|e| AnyhookError::Execution(format!("Invalid slack config: {}", e)))?;
            
        let payload = json!({
            "text": config.text
        });
            
        let res = self.client.post(&config.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| AnyhookError::Execution(e.to_string()))?;
            
        if !res.status().is_success() {
            return Err(AnyhookError::Execution(format!("Slack webhook failed with status: {}", res.status())));
        }
        
        Ok(json!({ "status": "success", "notified": true }))
    }
}
