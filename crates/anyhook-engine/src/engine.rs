use anyhook_core::config::AnyhookConfig;
use anyhook_core::event::Event;
use anyhook_core::traits::{Action, Watcher, ActionContext};
use anyhook_core::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::signal;
use tracing::{info, error};
use crate::router::Router;
use crate::executor::Executor;
use crate::db::Database;

/// Engine (自动化引擎核心)
/// 整个系统的大脑。负责管理所有的事件源 (Watchers) 和处理模块 (Actions)，
/// 并通过 Router (路由器) 将事件正确地分发。
pub struct Engine {
    _config: AnyhookConfig, // 全局配置
    
    // 知识点: Box<dyn Watcher> 使用了 trait object (特征对象)，
    // 允许在 HashMap 中存储实现了 Watcher 特征的任意类型实例。这也是 Rust 实现“面向对象编程/多态”的一种方式。
    watchers: HashMap<String, Box<dyn Watcher>>,
    
    // 知识点: Arc<dyn Action> 同样是特征对象，但包了一层 Arc (原子引用计数)，
    // 使得 Action 可以被安全、廉价地 clone 给多线程并发执行。
    actions: HashMap<String, Arc<dyn Action>>,
    
    router: Arc<Router>, // 路由表
    reload_rx: Option<mpsc::Receiver<AnyhookConfig>>, // 用于接收热重载信号的通道
    pub db_url: String, // 数据库连接字符串
    event_tx: mpsc::Sender<Event>, // 内部事件发送器
    event_rx: Option<mpsc::Receiver<Event>>, // 内部事件接收器
}

impl Engine {
    pub fn new(config: AnyhookConfig, db_url: String) -> Self {
        let router = Arc::new(Router::new(config.hooks.clone()));
        let (event_tx, event_rx) = mpsc::channel(1024);
        Self {
            _config: config,
            watchers: HashMap::new(),
            actions: HashMap::new(),
            router,
            reload_rx: None,
            db_url,
            event_tx,
            event_rx: Some(event_rx),
        }
    }

    pub fn get_event_sender(&self) -> mpsc::Sender<Event> {
        self.event_tx.clone()
    }

    pub fn set_reload_receiver(&mut self, rx: mpsc::Receiver<AnyhookConfig>) {
        self.reload_rx = Some(rx);
    }

    pub fn register_watcher(&mut self, name: String, watcher: Box<dyn Watcher>) {
        self.watchers.insert(name, watcher);
    }

    pub fn register_action(&mut self, action_type: String, action: Arc<dyn Action>) {
        self.actions.insert(action_type, action);
    }

    pub async fn start(mut self) -> Result<()> {
        let db = Arc::new(Database::new(&self.db_url).await?);
        let executor = Arc::new(Executor::new(10, db.clone()));
        let mut rx = self.event_rx.take().expect("Engine started without event_rx!");
        let tx = self.event_tx.clone();

        for (name, watcher) in &self.watchers {
            info!("Starting watcher: {}", name);
            if let Err(e) = watcher.start(tx.clone()).await {
                error!("Failed to start watcher {}: {}", name, e);
            }
        }

        let router = self.router.clone();
        let actions = Arc::new(self.actions);

        info!("Engine started, waiting for events (Press Ctrl+C to stop)...");
        let mut shutdown_signal = std::pin::pin!(signal::ctrl_c());
        let mut reload_rx = self.reload_rx.unwrap_or_else(|| mpsc::channel(1).1);

        loop {
            tokio::select! {
                Some(new_config) = reload_rx.recv() => {
                    info!("Hot-reloading Engine hooks configuration...");
                    router.update_hooks(new_config.hooks);
                }
                Some(event) = rx.recv() => {
                    info!("Received event: [{}] from {}", event.event_type, event.source);
                    
                    // Log event to DB
                    let payload_str = serde_json::to_string(&event.payload).unwrap_or_default();
                    let _ = sqlx::query("INSERT INTO event_log (id, source, event_type, payload) VALUES (?, ?, ?, ?)")
                        .bind(event.id.to_string())
                        .bind(&event.source)
                        .bind(&event.event_type)
                        .bind(payload_str)
                        .execute(&db.pool)
                        .await;

                    let matched_actions = router.route(&event);
                    
                    for action_cfg in matched_actions {
                        if let Some(action) = actions.get(&action_cfg.action_type) {
                            let ctx = ActionContext {
                                event: event.clone(),
                                config: action_cfg.config.clone(),
                                runtime_env: serde_json::json!({}),
                            };
                            
                            let action_name = action_cfg.name.clone().unwrap_or_else(|| action_cfg.action_type.clone());
                            executor.execute_action(action_name, action.clone(), ctx);
                        } else {
                            error!("Action type '{}' not found in registry", action_cfg.action_type);
                        }
                    }
                }
                _ = &mut shutdown_signal => {
                    info!("Received shutdown signal. Waiting for running tasks to complete...");
                    // Graceful shutdown MVP: give tasks a few seconds to finish
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    info!("Graceful shutdown complete.");
                    break;
                }
            }
        }

        Ok(())
    }
}
