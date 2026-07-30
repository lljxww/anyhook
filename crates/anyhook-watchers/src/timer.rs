use anyhook_core::traits::Watcher;
use anyhook_core::event::Event;
use anyhook_core::error::{Result, AnyhookError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use tokio::sync::mpsc;
use tracing::{error, info};
use cron::Schedule;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    pub cron: String,
}

#[derive(Debug)]
pub struct TimerWatcher {
    name: String,
    config: TimerConfig,
}

impl TimerWatcher {
    pub fn new(name: String, config: TimerConfig) -> Self {
        Self { name, config }
    }
}

#[async_trait]
impl Watcher for TimerWatcher {
    fn watcher_type(&self) -> &'static str {
        "timer"
    }

    async fn start(&self, sender: mpsc::Sender<Event>) -> Result<()> {
        let schedule = Schedule::from_str(&self.config.cron)
            .map_err(|e| AnyhookError::Config(format!("Invalid cron expression: {}", e)))?;
        
        let watcher_name = self.name.clone();
        let cron_expr = self.config.cron.clone();

        info!("Starting timer watcher '{}' with cron: {}", watcher_name, cron_expr);

        tokio::spawn(async move {
            for datetime in schedule.upcoming(Utc) {
                let now = Utc::now();
                if let Ok(duration) = (datetime - now).to_std() {
                    tokio::time::sleep(duration).await;
                    
                    let event = Event::new(
                        format!("watcher.{}", watcher_name),
                        "timer.trigger",
                        json!({"scheduled_for": datetime.to_rfc3339()})
                    );

                    if let Err(e) = sender.send(event).await {
                        error!("Failed to send event from timer '{}': {}", watcher_name, e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping timer watcher '{}'", self.name);
        Ok(())
    }
}
