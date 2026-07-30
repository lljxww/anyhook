use anyhook_core::traits::Action;
use anyhook_core::error::{Result, AnyhookError};
use anyhook_core::traits::ActionContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub body: Option<Value>,
}

fn default_method() -> String {
    "POST".to_string()
}

#[derive(Debug)]
pub struct HttpAction {
    client: reqwest::Client,
}

impl HttpAction {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Action for HttpAction {
    fn action_type(&self) -> &'static str {
        "http"
    }

    async fn execute(&self, ctx: ActionContext) -> Result<Value> {
        let config: HttpConfig = serde_json::from_value(ctx.config.clone())
            .map_err(|e| AnyhookError::Config(format!("Invalid HTTP config: {}", e)))?;

        info!("HTTP Action: sending {} request to {}", config.method, config.url);

        let method = match config.method.to_uppercase().as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            _ => return Err(AnyhookError::Config(format!("Unsupported HTTP method: {}", config.method))),
        };

        let mut builder = self.client.request(method, &config.url);

        if let Some(headers) = config.headers {
            for (k, v) in headers {
                builder = builder.header(k, v);
            }
        }

        if let Some(body) = config.body {
            builder = builder.json(&body);
        }

        let response = builder.send().await
            .map_err(|e| AnyhookError::Execution(format!("HTTP request failed: {}", e)))?;

        let status = response.status().as_u16();
        let resp_text = response.text().await
            .unwrap_or_else(|_| "".to_string());

        if status >= 200 && status < 300 {
            info!("HTTP Action succeeded with status: {}", status);
            Ok(json!({
                "status": "success",
                "http_status": status,
                "response": resp_text
            }))
        } else {
            error!("HTTP Action failed with status {}: {}", status, resp_text);
            Err(AnyhookError::Execution(format!("HTTP error {}: {}", status, resp_text)))
        }
    }
}
