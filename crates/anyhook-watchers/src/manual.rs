use anyhook_core::traits::Watcher;
use anyhook_core::event::Event;
use anyhook_core::error::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tracing::{error, info};
use axum::{routing::post, Router, extract::{State, Json}};
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Debug)]
pub struct ManualWatcher {
    name: String,
}

impl ManualWatcher {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

struct AppState {
    sender: mpsc::Sender<Event>,
}

#[async_trait]
impl Watcher for ManualWatcher {
    fn watcher_type(&self) -> &'static str {
        "manual"
    }

    async fn start(&self, sender: mpsc::Sender<Event>) -> Result<()> {
        let watcher_name = self.name.clone();
        info!("Starting manual watcher '{}' on 127.0.0.1:9099", watcher_name);

        let state = Arc::new(AppState { sender });

        let app = Router::new()
            .route("/trigger", post(handle_trigger))
            .with_state(state);

        tokio::spawn(async move {
            let addr = "127.0.0.1:9099";
            match TcpListener::bind(&addr).await {
                Ok(listener) => {
                    info!("Manual trigger server listening on {}", addr);
                    if let Err(e) = axum::serve(listener, app).await {
                        error!("Manual trigger server error: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to bind manual trigger server on {}: {}", addr, e);
                }
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping manual watcher '{}'", self.name);
        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct TriggerPayload {
    watcher: String,
    payload: Option<Value>,
}

async fn handle_trigger(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TriggerPayload>,
) -> &'static str {
    let event = Event::new(
        format!("watcher.{}", body.watcher),
        "manual.trigger",
        body.payload.unwrap_or_else(|| json!({}))
    );

    if let Err(e) = state.sender.send(event).await {
        error!("Failed to send manual event: {}", e);
        return "Internal Error";
    }
    
    "OK"
}
