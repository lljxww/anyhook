use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Anyhook 的核心配置文件映射结构 (对应 `anyhook.yaml`)
/// 知识点: `#[serde(default)]` 会在反序列化时，如果 YAML 中缺失该字段，
/// 则自动调用 `Vec::default()` 也就是空数组来填充，防止报错。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnyhookConfig {
    #[serde(default)]
    pub watchers: Vec<WatcherConfig>,
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    pub name: String,
    
    // 知识点: 因为 `type` 是 Rust 的保留关键字（用于定义类型别名），
    // 无法直接作为结构体字段名。所以我们用 `watcher_type`，
    // 并通过 `#[serde(rename = "type")]` 宏让解析器知道 YAML 里的字段名叫 "type"。
    #[serde(rename = "type")]
    pub watcher_type: String,
    
    // 知识点: 使用 `serde_json::Value` 可以存储任何格式的动态 JSON 内容，
    // 方便后续由具体的 Watcher 拿走并二次反序列化。
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
