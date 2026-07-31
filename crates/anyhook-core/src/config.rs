use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnyhookConfig {
    #[serde(default)]
    pub plugins: Vec<PluginConfig>,
    #[serde(default)]
    pub watchers: Vec<WatcherConfig>,
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub path: Option<String>,
    #[serde(default)]
    pub config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub watcher_type: String,
    pub config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub watcher: String,
    pub actions: Vec<ActionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionConfig {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub action_type: String,
    pub config: Value,
}
