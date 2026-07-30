use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use crate::event::Event;
use crate::error::Result;

/// ActionContext (动作上下文)
/// 包含了触发该动作时的所有信息。
/// 知识点: Rust 中通常把跨组件传递的大对象打包成 Context 结构体，
/// 这样未来即使需要增加新的参数，也不用修改所有的特征(Trait)方法签名。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContext {
    pub event: Event,         // 触发该动作的原始事件
    pub config: Value,        // 针对该动作的自定义配置项 (来自 anyhook.yaml)
    pub runtime_env: Value,   // 运行时的环境变量等信息
}

/// Watcher (监听器特征)
/// 知识点: `async_trait` 宏用于在 trait 中支持 async fn (异步函数)。
/// Rust 原生 trait 在较老版本不支持 async，虽然新版已经支持，但为了兼容对象安全，通常还是会使用此宏。
/// `Send + Sync` 保证了 Watcher 可以在多线程环境中安全地传递和共享。
#[async_trait::async_trait]
pub trait Watcher: Send + Sync + Debug {
    /// 返回该监听器的类型标识，例如 "filesystem", "timer"
    fn watcher_type(&self) -> &'static str;
    
    /// 启动监听器。
    /// 知识点: 使用 `tokio::sync::mpsc::Sender` (多生产者单消费者 channel) 
    /// 向外发送事件。每当监听器侦测到变化，就会把生成的 Event 发送到这个通道中。
    async fn start(&self, sender: tokio::sync::mpsc::Sender<Event>) -> Result<()>;
    
    /// 停止监听器 (常用于优雅退出)
    async fn stop(&self) -> Result<()>;
}

/// Action (动作执行器特征)
/// 所有具体的动作 (如 Shell脚本, HTTP请求, WASM调用) 都需要实现此 Trait。
#[async_trait::async_trait]
pub trait Action: Send + Sync + Debug {
    /// 返回该动作的类型标识，例如 "shell", "http"
    fn action_type(&self) -> &'static str;
    
    /// 执行具体的动作逻辑。
    /// 入参是 ActionContext，返回处理结果 (通常是 JSON Value)。
    async fn execute(&self, ctx: ActionContext) -> Result<Value>;
}
