use anyhook_core::traits::{Action, ActionContext};
use anyhook_core::error::{Result, AnyhookError};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use reqwest::Client;

/// 知识点: DiscordAction 用于推送富文本消息到 Discord 频道。是一个非常有用的内置通知组件。
#[derive(Debug)]
pub struct DiscordAction {
    client: Client,
}

impl DiscordAction {
    pub fn new() -> Self { 
        Self { client: Client::builder().use_rustls_tls().build().unwrap() }
    }
}

#[derive(Deserialize)]
struct DiscordConfig {
    webhook_url: String,
    content: String,
    username: Option<String>,
}

#[async_trait]
impl Action for DiscordAction {
    fn action_type(&self) -> &'static str { "discord" }

    async fn execute(&self, ctx: ActionContext) -> Result<serde_json::Value> {
        let config: DiscordConfig = serde_json::from_value(ctx.config)
            .map_err(|e| AnyhookError::Execution(format!("Invalid discord config: {}", e)))?;
            
        let mut payload = json!({
            "content": config.content
        });
        
        if let Some(user) = config.username {
            payload.as_object_mut().unwrap().insert("username".to_string(), json!(user));
        }
            
        let res = self.client.post(&config.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| AnyhookError::Execution(e.to_string()))?;
            
        if !res.status().is_success() {
            return Err(AnyhookError::Execution(format!("Discord webhook failed with status: {}", res.status())));
        }
        
        Ok(json!({ "status": "success", "notified": true }))
    }
}
