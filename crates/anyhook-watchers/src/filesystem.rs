use anyhook_core::traits::Watcher;
use anyhook_core::event::Event;
use anyhook_core::error::{Result, AnyhookError};
use async_trait::async_trait;
use notify::{Watcher as NotifyWatcher, RecursiveMode, Event as NotifyEvent, EventKind};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::{error, info};

/// FilesystemConfig (文件系统监控配置)
/// 通过 derive `Deserialize` 可以让 serde 自动将 YAML/JSON 中的配置解析为该结构体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    pub path: String,
    #[serde(default)]
    pub recursive: bool,
}

/// FilesystemWatcher (文件系统监听器)
/// 使用 `notify` 库来跨平台(inotify/FSEvents/ReadDirectoryChangesW)监听文件变更。
#[derive(Debug)]
pub struct FilesystemWatcher {
    name: String,
    config: FilesystemConfig,
}

impl FilesystemWatcher {
    pub fn new(name: String, config: FilesystemConfig) -> Self {
        Self { name, config }
    }
}

#[async_trait]
impl Watcher for FilesystemWatcher {
    fn watcher_type(&self) -> &'static str {
        "filesystem"
    }

    async fn start(&self, sender: mpsc::Sender<Event>) -> Result<()> {
        let path = PathBuf::from(&self.config.path);
        let recursive_mode = if self.config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        let watcher_name = self.name.clone();

        info!("Starting filesystem watcher '{}' on {:?}", watcher_name, path);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<NotifyEvent>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        }).map_err(|e| AnyhookError::Other(e.into()))?;

        watcher.watch(&path, recursive_mode).map_err(|e| AnyhookError::Other(e.into()))?;

        tokio::spawn(async move {
            // Keep the watcher alive in this async task
            let _watcher = watcher; 
            while let Some(notify_event) = rx.recv().await {
                let event_type = match notify_event.kind {
                    EventKind::Create(_) => "filesystem.create",
                    EventKind::Modify(_) => "filesystem.modify",
                    EventKind::Remove(_) => "filesystem.remove",
                    _ => continue,
                };
                
                let paths: Vec<String> = notify_event.paths.iter().map(|p| p.to_string_lossy().to_string()).collect();

                let payload = json!({
                    "paths": paths,
                    "kind": format!("{:?}", notify_event.kind)
                });

                let event = Event::new(
                    format!("watcher.{}", watcher_name),
                    event_type,
                    payload
                );

                if let Err(e) = sender.send(event).await {
                    error!("Failed to send event from watcher '{}': {}", watcher_name, e);
                    break;
                }
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping filesystem watcher '{}'", self.name);
        Ok(())
    }
}
