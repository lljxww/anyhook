pub mod sysconfig;

use clap::{Parser, Subcommand};
use tracing::{info, Level};
use std::fs;
use std::sync::Arc;
use anyhook_core::config::AnyhookConfig;
use anyhook_engine::engine::Engine;
use anyhook_watchers::filesystem::{FilesystemWatcher, FilesystemConfig};
use anyhook_watchers::timer::{TimerWatcher, TimerConfig};
use anyhook_watchers::webhook::{WebhookWatcher, WebhookConfig};
use anyhook_watchers::manual::ManualWatcher;
use anyhook_actions::archive::ArchiveAction;
use anyhook_actions::shell::ShellAction;
use anyhook_actions::extract::ExtractAction;
use anyhook_actions::http::HttpAction;
use anyhook_actions::wasm::WasmAction;
use anyhook_dashboard::{start_dashboard, DashboardState};
use std::sync::RwLock;
use std::path::Path;

/// CLI 命令解析结构体
/// 知识点: 使用 `clap` 宏可以非常方便地生成强大的命令行交互界面，
/// 包括自动生成 --help 文档、参数校验等。
#[derive(Parser)]
#[command(name = "anyhook")]
#[command(about = "A cross-platform event-driven automation engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new anyhook.yaml in the current directory
    Init,
    /// Start the anyhook engine
    Start {
        /// Path to config file
        #[arg(short, long, default_value = "anyhook.yaml")]
        config: String,
    },
    /// List all configured watchers and hooks
    List {
        #[arg(short, long, default_value = "anyhook.yaml")]
        config: String,
    },
    /// View recent action execution status
    Status,
    /// Manually trigger a watcher
    Trigger {
        watcher: String,
        #[arg(short, long)]
        payload: Option<String>,
    }
}

/// 知识点: `#[tokio::main]` 是一个宏，它会在底层自动生成一个异步运行时 (Runtime)，
/// 并将 `async fn main` 转换成标准的同步 `fn main` 去启动。
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sys_cfg = sysconfig::load_system_config();
    let level = match sys_cfg.log_level.to_lowercase().as_str() {
        "error" => Level::ERROR,
        "warn" => Level::WARN,
        "info" => Level::INFO,
        "debug" => Level::DEBUG,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            let template = r#"
watchers:
  - name: project_monitor
    type: filesystem
    config:
      path: ./source
      recursive: true
hooks:
  - watcher: project_monitor
    actions:
      - name: backup_project
        type: archive
        config:
          source: ./source
          output: ./backup/project.tar.gz
"#;
            let path = "anyhook.yaml";
            if fs::metadata(path).is_ok() {
                tracing::warn!("{} already exists, skip initialization.", path);
            } else {
                fs::write(path, template.trim())?;
                info!("Generated template {}", path);
            }
        }
        Commands::Start { config } => {
            info!("Starting anyhook with config: {}", config);
            let config_content = fs::read_to_string(config)?;
            let app_config: AnyhookConfig = serde_yaml::from_str(&config_content)?;
            
            let mut engine = Engine::new(app_config.clone(), sys_cfg.db_url.clone());

            for watcher_cfg in &app_config.watchers {
                if watcher_cfg.watcher_type == "filesystem" {
                    let fs_config: FilesystemConfig = serde_json::from_value(watcher_cfg.config.clone())
                        .map_err(|e| anyhow::anyhow!("Invalid filesystem config: {}", e))?;
                    
                    let watcher = Box::new(FilesystemWatcher::new(watcher_cfg.name.clone(), fs_config));
                    engine.register_watcher(watcher_cfg.name.clone(), watcher);
                } else if watcher_cfg.watcher_type == "timer" {
                    let timer_config: TimerConfig = serde_json::from_value(watcher_cfg.config.clone())
                        .map_err(|e| anyhow::anyhow!("Invalid timer config: {}", e))?;
                    
                    let watcher = Box::new(TimerWatcher::new(watcher_cfg.name.clone(), timer_config));
                    engine.register_watcher(watcher_cfg.name.clone(), watcher);
                } else if watcher_cfg.watcher_type == "webhook" {
                    let webhook_config: WebhookConfig = serde_json::from_value(watcher_cfg.config.clone())
                        .map_err(|e| anyhow::anyhow!("Invalid webhook config: {}", e))?;
                    
                    let watcher = Box::new(WebhookWatcher::new(watcher_cfg.name.clone(), webhook_config));
                    engine.register_watcher(watcher_cfg.name.clone(), watcher);
                } else if watcher_cfg.watcher_type == "manual" {
                    let watcher = Box::new(ManualWatcher::new(watcher_cfg.name.clone()));
                    engine.register_watcher(watcher_cfg.name.clone(), watcher);
                } else {
                    tracing::warn!("Unknown watcher type: {}", watcher_cfg.watcher_type);
                }
            }

            engine.register_action("archive".to_string(), Arc::new(ArchiveAction::new()));
            engine.register_action("shell".to_string(), Arc::new(ShellAction::new()));
            engine.register_action("extract".to_string(), Arc::new(ExtractAction::new()));
            engine.register_action("http".to_string(), Arc::new(HttpAction::new()));
            
            // Load WASM plugins
            let plugins_dir = Path::new(&sys_cfg.plugins_dir);
            if plugins_dir.exists() && plugins_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(plugins_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                tracing::info!("Loaded WASM Action plugin: {}", name);
                                engine.register_action(name.to_string(), Arc::new(WasmAction::new(path)));
                            }
                        }
                    }
                }
            }

            let db_pool = sqlx::sqlite::SqlitePoolOptions::new()
                .connect(&sys_cfg.db_url)
                .await
                .expect("Failed to connect to SQLite db");
            
            // Dashboard state setup
            let shared_config = Arc::new(RwLock::new(app_config.clone()));
            
            let dashboard_state = std::sync::Arc::new(DashboardState {
                config: shared_config.clone(),
                pool: db_pool,
                event_sender: engine.get_event_sender(),
            });

            tokio::spawn(async move {
                if let Err(e) = start_dashboard(sys_cfg.dashboard_port, dashboard_state).await {
                    tracing::error!("Dashboard error: {}", e);
                }
            });

            let (reload_tx, reload_rx) = tokio::sync::mpsc::channel(1);
            engine.set_reload_receiver(reload_rx);

            let config_path = config.clone();
            tokio::task::spawn_blocking(move || {
                use notify::{Watcher, RecursiveMode, EventKind};
                let watch_path = config_path.clone();
                let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                    if let Ok(event) = res {
                        if matches!(event.kind, EventKind::Modify(_)) {
                            if let Ok(content) = std::fs::read_to_string(&watch_path) {
                                if let Ok(new_config) = serde_yaml::from_str::<AnyhookConfig>(&content) {
                                    let _ = reload_tx.blocking_send(new_config.clone());
                                    if let Ok(mut w) = shared_config.write() {
                                        *w = new_config;
                                    }
                                }
                            }
                        }
                    }
                }).unwrap();
                watcher.watch(std::path::Path::new(&config_path), RecursiveMode::NonRecursive).unwrap();
                loop { std::thread::sleep(std::time::Duration::from_secs(3600)); }
            });

            engine.start().await?;
        }
        Commands::List { config } => {
            let config_content = fs::read_to_string(config)?;
            let app_config: AnyhookConfig = serde_yaml::from_str(&config_content)?;
            
            println!("Watchers:");
            for w in &app_config.watchers {
                println!("  - {} [{}]", w.name, w.watcher_type);
            }
            println!("\nHooks:");
            for h in &app_config.hooks {
                println!("  - Watcher: {}", h.watcher);
                for a in &h.actions {
                    println!("    -> Action: {} [{}]", a.name.as_deref().unwrap_or("unnamed"), a.action_type);
                }
            }
        }
        Commands::Status => {
            let pool_res = sqlx::sqlite::SqlitePoolOptions::new()
                .connect("sqlite://.anyhook.db").await;
            
            match pool_res {
                Ok(pool) => {
                    println!("Recent Actions:");
                    let rows = sqlx::query("SELECT id, action_name, status, timestamp FROM action_log ORDER BY id DESC LIMIT 10")
                        .fetch_all(&pool)
                        .await?;
                    for row in rows {
                        use sqlx::Row;
                        let id: i32 = row.get("id");
                        let action_name: String = row.get("action_name");
                        let status: String = row.get("status");
                        let timestamp: String = row.get("timestamp");
                        println!("  [{}] {} - {} ({})", id, timestamp, action_name, status);
                    }
                },
                Err(e) => println!("Could not connect to database (is anyhook running?): {}", e),
            }
        }
        Commands::Trigger { watcher, payload } => {
            let p_val: serde_json::Value = payload.as_deref()
                .map(|s| serde_json::from_str(s).unwrap_or(serde_json::json!({})))
                .unwrap_or(serde_json::json!({}));
                
            let client = reqwest::Client::new();
            let res = client.post("http://127.0.0.1:9099/trigger")
                .json(&serde_json::json!({
                    "watcher": watcher,
                    "payload": p_val
                }))
                .send()
                .await?;
                
            println!("Trigger response: {}", res.text().await?);
        }
    }

    Ok(())
}
