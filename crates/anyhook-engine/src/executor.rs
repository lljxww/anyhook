use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, error, warn};
use anyhook_core::traits::{Action, ActionContext};
use crate::db::Database;

/// Executor (任务执行器)
/// 负责实际派发和运行 Action，并控制并发数。
/// 知识点: 在高并发的网络/事件系统中，如果不加限制地启动任务，可能会耗尽系统资源。
/// 这里使用 `tokio::sync::Semaphore` (信号量) 来限制同时运行的 Action 数量。
pub struct Executor {
    semaphore: Arc<Semaphore>, // 用于控制最大并发量的信号量
    db: Arc<Database>,         // 共享的数据库连接池，用于记录执行日志
}

impl Executor {
    pub fn new(max_concurrent: usize, db: Arc<Database>) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            db,
        }
    }

    /// 执行一个动作 (Action)
    /// 注意此方法是同步签名的，但它内部使用了 `tokio::spawn` 来派发后台异步任务。
    /// 知识点: `tokio::spawn` 类似于 Go 语言中的 `go func() {}`，它会将一个 async block 
    /// 提交给 tokio 运行时进行调度执行。通过把上下文变量 `clone` 进去，可以转移所有权 (`move`)。
    pub fn execute_action(
        &self, 
        action_name: String, 
        action: Arc<dyn Action>, 
        ctx: ActionContext
    ) {
        let sem = self.semaphore.clone();
        let db = self.db.clone();
        
        tokio::spawn(async move {
            // 获取执行许可 (Permit)。如果并发达到上限，这里会异步等待阻塞，直到有其他任务完成释放许可。
            let _permit = sem.acquire().await.unwrap();
            let mut attempts = 0;
            let max_retries = 3;
            let mut success = false;
            let mut final_result = String::new();

            while attempts <= max_retries {
                attempts += 1;
                info!("Executing action: {} (Attempt {}/{})", action_name, attempts, max_retries + 1);
                
                match action.execute(ctx.clone()).await {
                    Ok(res) => {
                        info!("Action '{}' succeeded: {}", action_name, res);
                        success = true;
                        final_result = res.to_string();
                        break;
                    }
                    Err(e) => {
                        warn!("Action '{}' failed on attempt {}: {}", action_name, attempts, e);
                        final_result = e.to_string();
                        if attempts <= max_retries {
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        }
                    }
                }
            }

            if !success {
                error!("Action '{}' completely failed after {} attempts.", action_name, max_retries + 1);
            }

            let status_str = if success { "success" } else { "failed" };
            let _ = sqlx::query("INSERT INTO action_log (event_id, action_name, status, result) VALUES (?, ?, ?, ?)")
                .bind(ctx.event.id.to_string())
                .bind(&action_name)
                .bind(status_str)
                .bind(&final_result)
                .execute(&db.pool)
                .await;
        });
    }
}
