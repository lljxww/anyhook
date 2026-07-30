use anyhook_core::traits::Action;
use anyhook_core::error::{Result, AnyhookError};
use anyhook_core::traits::ActionContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::path::PathBuf;
use flate2::write::GzEncoder;
use flate2::Compression;
use tracing::{info, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveConfig {
    pub source: String,
    pub output: String,
}

#[derive(Debug)]
pub struct ArchiveAction;

impl ArchiveAction {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Action for ArchiveAction {
    fn action_type(&self) -> &'static str {
        "archive"
    }

    async fn execute(&self, ctx: ActionContext) -> Result<serde_json::Value> {
        let config: ArchiveConfig = serde_json::from_value(ctx.config.clone())
            .map_err(|e| AnyhookError::Config(format!("Invalid archive config: {}", e)))?;

        let source = PathBuf::from(&config.source);
        let output = PathBuf::from(&config.output);

        info!("Archive Action: compressing {:?} to {:?}", source, output);

        if !source.exists() {
            return Err(AnyhookError::Execution(format!("Source path does not exist: {:?}", source)));
        }

        let result = tokio::task::spawn_blocking(move || -> std::result::Result<(), String> {
            let tar_gz = File::create(&output).map_err(|e| e.to_string())?;
            let enc = GzEncoder::new(tar_gz, Compression::default());
            let mut tar = tar::Builder::new(enc);
            
            if source.is_dir() {
                tar.append_dir_all(".", &source).map_err(|e| e.to_string())?;
            } else {
                let file_name = source.file_name().unwrap_or_default();
                tar.append_path_with_name(&source, file_name).map_err(|e| e.to_string())?;
            }
            
            tar.finish().map_err(|e| e.to_string())?;
            Ok(())
        }).await;

        match result {
            Ok(Ok(_)) => {
                info!("Archive Action: successfully created {:?}", config.output);
                Ok(json!({
                    "status": "success",
                    "output": config.output
                }))
            }
            Ok(Err(e)) => {
                error!("Archive Action failed: {}", e);
                Err(AnyhookError::Execution(format!("Archive error: {}", e)))
            }
            Err(e) => {
                error!("Archive Action thread panicked: {}", e);
                Err(AnyhookError::Execution(format!("Archive panic: {}", e)))
            }
        }
    }
}
