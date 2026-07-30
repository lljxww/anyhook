use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, SqlitePool};
use anyhook_core::error::Result;
use std::str::FromStr;
use tracing::info;

/// 数据库连接管理封装
/// 知识点: `sqlx::SqlitePool` 是一个纯 Rust 实现的异步数据库连接池。
/// 与 Diesel 等 ORM 不同，sqlx 甚至可以在编译期校验你的 SQL 语句是否合法！
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new(db_url: &str) -> Result<Self> {
        // 知识点: `create_if_missing(true)` 会在指定的 sqlite 数据库文件不存在时自动创建它。
        let opts = SqliteConnectOptions::from_str(db_url)
            .map_err(|e| anyhook_core::error::AnyhookError::Execution(format!("Invalid DB URL: {}", e)))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await
            .map_err(|e| anyhook_core::error::AnyhookError::Execution(format!("DB Connect error: {}", e)))?;
        
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS event_log (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                event_type TEXT NOT NULL,
                payload TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            );"
        )
        .execute(&pool)
        .await
        .map_err(|e| anyhook_core::error::AnyhookError::Execution(format!("DB Init error (event_log): {}", e)))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS action_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL,
                action_name TEXT NOT NULL,
                status TEXT NOT NULL,
                result TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            );"
        )
        .execute(&pool)
        .await
        .map_err(|e| anyhook_core::error::AnyhookError::Execution(format!("DB Init error (action_log): {}", e)))?;

        info!("Database initialized at {}", db_url);

        Ok(Self { pool })
    }
}
