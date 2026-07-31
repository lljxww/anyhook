use anyhook_core::traits::{Action, ActionContext};
use anyhook_core::error::{Result, AnyhookError};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use reqwest::Client;

/// 知识点: GithubDispatchAction 用于向 GitHub 仓库发送 repository_dispatch 事件，
/// 是打通 Anyhook 自动化流与 GitHub Actions CI/CD 流水线的完美桥梁。
#[derive(Debug)]
pub struct GithubDispatchAction {
    client: Client,
}

impl GithubDispatchAction {
    pub fn new() -> Self { 
        Self { client: Client::builder().use_rustls_tls().build().unwrap() }
    }
}

#[derive(Deserialize)]
struct GithubDispatchConfig {
    repo: String,
    token: String,
    event_type: String,
    #[serde(default)]
    client_payload: Option<serde_json::Value>,
}

#[async_trait]
impl Action for GithubDispatchAction {
    fn action_type(&self) -> &'static str { "github_dispatch" }

    async fn execute(&self, ctx: ActionContext) -> Result<serde_json::Value> {
        let config: GithubDispatchConfig = serde_json::from_value(ctx.config)
            .map_err(|e| AnyhookError::Execution(format!("Invalid github_dispatch config: {}", e)))?;
            
        let mut payload = json!({
            "event_type": config.event_type
        });
        
        if let Some(cp) = config.client_payload {
            payload.as_object_mut().unwrap().insert("client_payload".to_string(), cp);
        }
        
        let url = format!("https://api.github.com/repos/{}/dispatches", config.repo);
            
        let res = self.client.post(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("token {}", config.token))
            .header("User-Agent", "anyhook-engine")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AnyhookError::Execution(e.to_string()))?;
            
        if !res.status().is_success() {
            return Err(AnyhookError::Execution(format!("GitHub dispatch failed with status: {}, maybe check token or repo name", res.status())));
        }
        
        Ok(json!({ "status": "success", "event_type": config.event_type }))
    }
}
