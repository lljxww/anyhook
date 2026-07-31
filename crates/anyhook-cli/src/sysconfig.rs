#![allow(clippy::collapsible_if)]
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct SystemConfig {
    #[serde(default = "default_dashboard_port")]
    pub dashboard_port: u16,
    #[serde(default = "default_plugins_dir")]
    pub plugins_dir: String,
    #[serde(default = "default_db_url")]
    pub db_url: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    pub dashboard_auth: Option<String>,
}

fn default_dashboard_port() -> u16 { 3000 }
fn default_plugins_dir() -> String { "plugins".to_string() }
fn default_db_url() -> String { "sqlite://.anyhook.db?mode=rwc".to_string() }
fn default_log_level() -> String { "info".to_string() }

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            dashboard_port: default_dashboard_port(),
            plugins_dir: default_plugins_dir(),
            db_url: default_db_url(),
            log_level: default_log_level(),
            dashboard_auth: None,
        }
    }
}

pub fn load_system_config() -> SystemConfig {
    let mut search_paths = vec![PathBuf::from("anyhook.conf")];

    if let Some(config_dir) = dirs::config_dir() {
        search_paths.push(config_dir.join("anyhook/anyhook.conf"));
    }

    if cfg!(target_os = "windows") {
        search_paths.push(PathBuf::from("C:\\ProgramData\\anyhook\\anyhook.conf"));
    } else {
        search_paths.push(PathBuf::from("/etc/anyhook/anyhook.conf"));
    }

    for path in search_paths {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_yaml::from_str::<SystemConfig>(&content) {
                    println!("Loaded system config from {}", path.display());
                    return config;
                } else {
                    eprintln!("Failed to parse system config at {}", path.display());
                }
            }
        }
    }

    println!("No system config found, using defaults");
    SystemConfig::default()
}
