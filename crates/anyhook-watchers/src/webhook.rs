use anyhook_core::traits::Watcher;
use anyhook_core::event::Event;
use anyhook_core::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;
use tracing::{error, info};
use axum::{
    routing::post,
    Router,
    extract::{State, Json},
};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub port: u16,
    pub path: String,
}

#[derive(Debug)]
pub struct WebhookWatcher {
    name: String,
    config: WebhookConfig,
}

impl WebhookWatcher {
    pub fn new(name: String, config: WebhookConfig) -> Self {
        Self { name, config }
    }
}

struct AppState {
    sender: mpsc::Sender<Event>,
    watcher_name: String,
}

#[async_trait]
impl Watcher for WebhookWatcher {
    fn watcher_type(&self) -> &'static str {
        "webhook"
    }

    async fn start(&self, sender: mpsc::Sender<Event>) -> Result<()> {
        let watcher_name = self.name.clone();
        let port = self.config.port;
        let path = self.config.path.clone();

        info!("Starting webhook watcher '{}' on port {} at path {}", watcher_name, port, path);

        let state = Arc::new(AppState {
            sender,
            watcher_name: watcher_name.clone(),
        });

        let app = Router::new()
            .route(&path, post(handle_webhook))
            .with_state(state);

        tokio::spawn(async move {
            let addr = format!("0.0.0.0:{}", port);
            let listener_res = tokio::net::TcpListener::bind(&addr).await;
            match listener_res {
                Ok(listener) => {
                    info!("Webhook server listening on {}", addr);
                    if let Err(e) = axum::serve(listener, app).await {
                        error!("Webhook server error for watcher '{}': {}", watcher_name, e);
                    }
                }
                Err(e) => {
                    error!("Failed to bind webhook server port {}: {}", port, e);
                }
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping webhook watcher '{}'", self.name);
        Ok(())
    }
}

async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> &'static str {
    let event = Event::new(
        format!("watcher.{}", state.watcher_name),
        "webhook.received",
        payload
    );

    if let Err(e) = state.sender.send(event).await {
        error!("Failed to send webhook event from '{}': {}", state.watcher_name, e);
        return "Internal Error";
    }
    
    "OK"
}
