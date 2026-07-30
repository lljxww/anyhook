use anyhook_core::traits::Action;
use anyhook_core::error::{Result, AnyhookError};
use anyhook_core::traits::ActionContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::path::PathBuf;
use flate2::read::GzDecoder;
use tracing::{info, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractConfig {
    pub source: String,
    pub output: String,
}

#[derive(Debug)]
pub struct ExtractAction;

impl ExtractAction {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Action for ExtractAction {
    fn action_type(&self) -> &'static str {
        "extract"
    }

    async fn execute(&self, ctx: ActionContext) -> Result<serde_json::Value> {
        let config: ExtractConfig = serde_json::from_value(ctx.config.clone())
            .map_err(|e| AnyhookError::Config(format!("Invalid extract config: {}", e)))?;

        let source = PathBuf::from(&config.source);
        let output = PathBuf::from(&config.output);

        info!("Extract Action: extracting {:?} to {:?}", source, output);

        if !source.exists() {
            return Err(AnyhookError::Execution(format!("Source archive does not exist: {:?}", source)));
        }

        let result = tokio::task::spawn_blocking(move || -> std::result::Result<(), String> {
            let tar_gz = File::open(&source).map_err(|e| e.to_string())?;
            let dec = GzDecoder::new(tar_gz);
            let mut archive = tar::Archive::new(dec);
            
            std::fs::create_dir_all(&output).map_err(|e| e.to_string())?;
            archive.unpack(&output).map_err(|e| e.to_string())?;
            
            Ok(())
        }).await;

        match result {
            Ok(Ok(_)) => {
                info!("Extract Action: successfully extracted to {:?}", config.output);
                Ok(json!({
                    "status": "success",
                    "output": config.output
                }))
            }
            Ok(Err(e)) => {
                error!("Extract Action failed: {}", e);
                Err(AnyhookError::Execution(format!("Extract error: {}", e)))
            }
            Err(e) => {
                error!("Extract Action thread panicked: {}", e);
                Err(AnyhookError::Execution(format!("Extract panic: {}", e)))
            }
        }
    }
}
