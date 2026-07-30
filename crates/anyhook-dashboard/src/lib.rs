use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    http::{StatusCode, header, Uri},
    extract::State,
    Json,
};
use rust_embed::RustEmbed;
use std::sync::{Arc, RwLock};
use serde::Serialize;
use anyhook_core::config::AnyhookConfig;
use sqlx::SqlitePool;

/// 知识点: `rust_embed` 宏能在编译期将前端文件 (HTML/CSS/JS) 
/// 直接作为二进制数组打包进最后生成的执行文件中，
/// 这样分发程序时就不需要额外带上一堆静态资源文件夹了。
#[derive(RustEmbed)]
#[folder = "frontend/"]
struct Assets;

/// Dashboard 全局状态
/// 知识点: `RwLock` 读写锁允许多个线程同时读取配置，但在需要修改（热重载）时能独占写入。
/// `SqlitePool` 是 sqlx 的异步连接池，本身已经包裹了 Arc，可以直接安全克隆。
pub struct DashboardState {
    pub config: Arc<RwLock<AnyhookConfig>>,
    pub pool: SqlitePool,
}

/// 启动 Axum Web 框架服务器
pub async fn start_dashboard(port: u16, state: Arc<DashboardState>) -> anyhow::Result<()> {
    // 知识点: Axum 的 Router 支持通过 `.with_state()` 将全局状态注入进每个请求上下文中
    let app = Router::new()
        .route("/api/status", get(api_status))
        .route("/api/logs", get(api_logs).delete(api_clear_logs))
        .fallback(static_handler) // 如果找不到路由，就返回静态文件 (SPA 逻辑)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Web Dashboard running on http://{}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Assets::get(path.as_str()) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            (StatusCode::NOT_FOUND, "404 Not Found").into_response()
        }
    }
}

#[derive(Serialize)]
struct StatusResponse {
    watchers: Vec<serde_json::Value>,
    hooks: Vec<serde_json::Value>,
}

async fn api_status(State(state): State<Arc<DashboardState>>) -> Json<StatusResponse> {
    let mut watchers = Vec::new();
    let config = state.config.read().unwrap();
    for w in &config.watchers {
        watchers.push(serde_json::json!({
            "name": w.name,
            "type": w.watcher_type
        }));
    }
    
    let mut hooks = Vec::new();
    for h in &config.hooks {
        let actions = h.actions.iter().map(|a| {
            serde_json::json!({
                "name": a.name.clone().unwrap_or_default(),
                "type": a.action_type
            })
        }).collect::<Vec<_>>();
        
        hooks.push(serde_json::json!({
            "watcher": h.watcher,
            "actions": actions
        }));
    }

    Json(StatusResponse { watchers, hooks })
}

#[derive(Serialize)]
struct LogEntry {
    id: i32,
    action_name: String,
    status: String,
    timestamp: String,
}

async fn api_logs(State(state): State<Arc<DashboardState>>) -> Result<Json<Vec<LogEntry>>, StatusCode> {
    use sqlx::Row;
    let query = "SELECT id, action_name, status, timestamp FROM action_log ORDER BY id DESC LIMIT 50";
    
    let rows = sqlx::query(query)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error fetching logs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut logs = Vec::new();
    for row in rows {
        logs.push(LogEntry {
            id: row.get("id"),
            action_name: row.get("action_name"),
            status: row.get("status"),
            timestamp: row.get("timestamp"),
        });
    }

    Ok(Json(logs))
}

async fn api_clear_logs(State(state): State<Arc<DashboardState>>) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM action_log")
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error clearing action_log: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    sqlx::query("DELETE FROM event_log")
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error clearing event_log: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}
