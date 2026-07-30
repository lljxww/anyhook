use anyhook_core::config::{HookConfig, ActionConfig};
use anyhook_core::event::Event;
use std::sync::RwLock;

/// Router (事件路由器)
/// 负责根据收到的 Event，查找所有匹配的 Action 配置。
/// 知识点: 使用 `std::sync::RwLock` (读写锁) 包装配置，
/// 这允许在执行热重载时安全地替换内部的 Hooks 规则，而不影响正在读取路由规则的其他线程。
pub struct Router {
    hooks: RwLock<Vec<HookConfig>>,
}

impl Router {
    pub fn new(hooks: Vec<HookConfig>) -> Self {
        Self {
            hooks: RwLock::new(hooks),
        }
    }

    pub fn update_hooks(&self, new_hooks: Vec<HookConfig>) {
        if let Ok(mut hooks) = self.hooks.write() {
            *hooks = new_hooks;
        }
    }

    /// 路由查找逻辑
    /// 知识点: `.read().unwrap()` 会获取读锁，多个线程可以同时获取读锁，性能很高。
    /// 只有在调用 `update_hooks` 获取 `.write()` 时，才会阻塞新的读操作。
    pub fn route(&self, event: &Event) -> Vec<ActionConfig> {
        let mut matched = Vec::new();
        let hooks = self.hooks.read().unwrap();
        
        for hook in hooks.iter() {
            if event.source == format!("watcher.{}", hook.watcher) {
                matched.extend(hook.actions.clone());
            }
        }
        matched
    }
}
