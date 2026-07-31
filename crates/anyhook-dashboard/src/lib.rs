use axum::{
    routing::get,
    Router,
    response::{IntoResponse, Response},
    http::{StatusCode, header, Uri, Request},
    extract::State,
    middleware::{self, Next},
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
    pub event_sender: tokio::sync::mpsc::Sender<anyhook_core::event::Event>,
    pub plugins_dir: String,
    pub config_error: Arc<RwLock<Option<String>>>,
    pub auth: Option<String>,
    pub registered_actions: Vec<String>,
}

/// 启动 Axum Web 框架服务器
pub async fn start_dashboard(port: u16, state: Arc<DashboardState>) -> anyhow::Result<()> {
    // 知识点: Axum 的 Router 支持通过 `.with_state()` 将全局状态注入进每个请求上下文中
    let app = Router::new()
        .route("/api/status", get(api_status))
        .route("/api/logs", get(api_logs).delete(api_clear_logs))
        .route("/api/trigger", axum::routing::post(api_trigger))
        .route("/api/plugins", get(api_plugins))
        .fallback(static_handler) // 如果找不到路由，就返回静态文件 (SPA 逻辑)
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Web Dashboard running on http://{}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

#[allow(clippy::collapsible_if)]
pub async fn auth_middleware(
    State(state): State<Arc<DashboardState>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    if let Some(auth_str) = &state.auth {
        let auth_header = req.headers().get(header::AUTHORIZATION)
            .and_then(|val| val.to_str().ok());
            
        let mut authorized = false;
        if let Some(auth_header) = auth_header {
            if let Some(b64) = auth_header.strip_prefix("Basic ") {
                use base64::{Engine as _, engine::general_purpose};
                
                if let Ok(decoded) = general_purpose::STANDARD.decode(b64) {
                    if let Ok(decoded_str) = String::from_utf8(decoded) {
                        if decoded_str == *auth_str {
                            authorized = true;
                        }
                    }
                }
            }
        }
        
        if !authorized {
            return Err((
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Basic realm=\"Anyhook Dashboard\"")],
                "Unauthorized",
            ));
        }
    }
    
    Ok(next.run(req).await)
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
    config_error: Option<String>,
    registered_actions: Vec<String>,
}

async fn api_status(State(state): State<Arc<DashboardState>>) -> Json<StatusResponse> {
    let mut watchers = Vec::new();
    let config = state.config.read().unwrap();
    let config_error = state.config_error.read().unwrap().clone();
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

    Json(StatusResponse { watchers, hooks, config_error, registered_actions: state.registered_actions.clone() })
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

#[derive(serde::Deserialize)]
struct TriggerPayload {
    watcher: String,
    payload: Option<serde_json::Value>,
}

async fn api_trigger(
    State(state): State<Arc<DashboardState>>,
    Json(body): Json<TriggerPayload>,
) -> Result<StatusCode, StatusCode> {
    let event = anyhook_core::event::Event::new(
        format!("watcher.{}", body.watcher),
        "api.trigger",
        body.payload.unwrap_or_else(|| serde_json::json!({}))
    );

    if let Err(e) = state.event_sender.send(event).await {
        tracing::error!("Failed to send api event: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    Ok(StatusCode::OK)
}

#[derive(Serialize)]
struct PluginInfo {
    name: String,
    path: String,
    is_configured: bool,
    markdown_doc: Option<String>,
}

async fn api_plugins(State(state): State<Arc<DashboardState>>) -> Json<Vec<PluginInfo>> {
    let mut plugins = Vec::new();
    let mut loaded_names = std::collections::HashSet::new();
    
    // First, process explicitly configured plugins
    {
        let config = state.config.read().unwrap();
        for p in &config.plugins {
            let wasm_path = if let Some(path) = &p.path {
                std::path::PathBuf::from(path)
            } else {
                std::path::Path::new(&state.plugins_dir).join(format!("{}.wasm", p.name))
            };
            
            let md_path = wasm_path.with_extension("md");
            let markdown_doc = std::fs::read_to_string(&md_path).ok();
            
            plugins.push(PluginInfo {
                name: p.name.clone(),
                path: wasm_path.to_string_lossy().into_owned(),
                is_configured: true,
                markdown_doc,
            });
            loaded_names.insert(p.name.clone());
        }
    }
    
    // Next, auto-discover plugins in plugins_dir
    let plugins_dir = std::path::Path::new(&state.plugins_dir);
    #[allow(clippy::collapsible_if)]
    if plugins_dir.exists() && plugins_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        if !loaded_names.contains(name) {
                            let md_path = path.with_extension("md");
                            let markdown_doc = std::fs::read_to_string(&md_path).ok();
                            
                            plugins.push(PluginInfo {
                                name: name.to_string(),
                                path: path.to_string_lossy().to_string(),
                                is_configured: false,
                                markdown_doc,
                            });
                        }
                    }
                }
            }
        }
    }
    
    Json(plugins)
}
