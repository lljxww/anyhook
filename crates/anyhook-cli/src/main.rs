#![allow(clippy::collapsible_if)]
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
use anyhook_actions::delay::DelayAction;
use anyhook_actions::write_file::WriteFileAction;
use anyhook_actions::discord::DiscordAction;
use anyhook_actions::slack::SlackAction;
use anyhook_actions::github_dispatch::GithubDispatchAction;
use anyhook_actions::git_sync::GitSyncAction;
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
        /// Run as a background daemon
        #[arg(short, long)]
        daemon: bool,
    },
    /// Stop the anyhook background daemon
    Stop,
    /// Manage watchers
    Watchers {
        #[command(subcommand)]
        cmd: ListCmd,
    },
    /// Manage hooks
    Hooks {
        #[command(subcommand)]
        cmd: ListCmd,
    },
    /// Manage actions
    Actions {
        #[command(subcommand)]
        cmd: ListCmd,
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

#[derive(Subcommand)]
enum ListCmd {
    List,
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
        Commands::Start { config, daemon } => {
            if *daemon {
                #[cfg(unix)]
                {
                    use daemonize::Daemonize;
                    use std::fs::File;
                    
                    let stdout = File::create("/tmp/anyhook.out").unwrap();
                    let stderr = File::create("/tmp/anyhook.err").unwrap();
                    
                    let daemonize = Daemonize::new()
                        .pid_file("/tmp/anyhook.pid")
                        .chown_pid_file(true)      
                        .working_directory(std::env::current_dir().unwrap())
                        .stdout(stdout)
                        .stderr(stderr)
                        .privileged_action(|| "Executed before drop privileges");

                    match daemonize.start() {
                        Ok(_) => tracing::info!("Success, daemonized"),
                        Err(e) => {
                            tracing::error!("Error, {}", e);
                            return Err(anyhow::anyhow!("Daemonize failed: {}", e));
                        }
                    }
                }
                #[cfg(not(unix))]
                {
                    tracing::warn!("Daemon mode is only supported on Unix systems.");
                }
            }
            
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
            engine.register_action("delay".to_string(), Arc::new(DelayAction::new()));
            engine.register_action("write_file".to_string(), Arc::new(WriteFileAction::new()));
            engine.register_action("discord".to_string(), Arc::new(DiscordAction::new()));
            engine.register_action("slack".to_string(), Arc::new(SlackAction::new()));
            engine.register_action("github_dispatch".to_string(), Arc::new(GithubDispatchAction::new()));
            engine.register_action("git_sync".to_string(), Arc::new(GitSyncAction::new()));
            
            let mut valid_actions = std::collections::HashSet::new();
            for a in ["archive", "shell", "extract", "http", "delay", "write_file", "discord", "slack", "github_dispatch", "git_sync"] {
                valid_actions.insert(a.to_string());
            }
            
            // Load explicitly configured WASM plugins
            let mut loaded_plugins = std::collections::HashSet::new();
            for plugin in &app_config.plugins {
                let wasm_path = if let Some(p) = &plugin.path {
                    std::path::PathBuf::from(p)
                } else {
                    Path::new(&sys_cfg.plugins_dir).join(format!("{}.wasm", plugin.name))
                };
                
                if wasm_path.exists() {
                    tracing::info!("Loaded configured WASM Action plugin: {} from {:?}", plugin.name, wasm_path);
                    engine.register_action(plugin.name.clone(), Arc::new(WasmAction::new(wasm_path, plugin.config.clone())));
                    loaded_plugins.insert(plugin.name.clone());
                    valid_actions.insert(plugin.name.clone());
                } else {
                    tracing::error!("Configured WASM plugin {} not found at {:?}", plugin.name, wasm_path);
                }
            }

            // Auto-discover other WASM plugins in plugins_dir
            let plugins_dir = Path::new(&sys_cfg.plugins_dir);
            if plugins_dir.exists() && plugins_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(plugins_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                if !loaded_plugins.contains(name) {
                                    tracing::info!("Auto-discovered WASM Action plugin: {}", name);
                                    engine.register_action(name.to_string(), Arc::new(WasmAction::new(path.clone(), serde_json::json!({}))));
                                    valid_actions.insert(name.to_string());
                                }
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
            let shared_config_error = Arc::new(RwLock::new(None::<String>));
            
            let dashboard_state = std::sync::Arc::new(DashboardState {
                config: shared_config.clone(),
                pool: db_pool,
                event_sender: engine.get_event_sender(),
                plugins_dir: sys_cfg.plugins_dir.clone(),
                config_error: shared_config_error.clone(),
                auth: sys_cfg.dashboard_auth.clone(),
                registered_actions: valid_actions.iter().cloned().collect(),
            });

            tokio::spawn(async move {
                if let Err(e) = start_dashboard(sys_cfg.dashboard_port, dashboard_state).await {
                    tracing::error!("Dashboard error: {}", e);
                }
            });

            let (reload_tx, reload_rx) = tokio::sync::mpsc::channel(1);
            engine.set_reload_receiver(reload_rx);

            let config_path = config.clone();
            let valid_actions_for_reload = valid_actions.clone();
            tokio::task::spawn_blocking(move || {
                use notify::{Watcher, RecursiveMode, EventKind};
                let watch_path = config_path.clone();
                let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                    if let Ok(event) = res {
                        if matches!(event.kind, EventKind::Modify(_)) {
                            if let Ok(content) = std::fs::read_to_string(&watch_path) {
                                match serde_yaml::from_str::<AnyhookConfig>(&content) {
                                    Ok(new_config) => {
                                        // Logical Validation
                                        let mut config_err = None;
                                        let mut watcher_names = std::collections::HashSet::new();
                                        for w in &new_config.watchers {
                                            if !watcher_names.insert(&w.name) {
                                                config_err = Some(format!("Duplicate watcher name: {}", w.name));
                                            }
                                        }
                                        if config_err.is_none() {
                                            for h in &new_config.hooks {
                                                if !watcher_names.contains(&h.watcher) {
                                                    config_err = Some(format!("Hook refers to nonexistent watcher: {}", h.watcher));
                                                    break;
                                                }
                                                for a in &h.actions {
                                                    if !valid_actions_for_reload.contains(&a.action_type) {
                                                        config_err = Some(format!("Hook refers to nonexistent action type: {}", a.action_type));
                                                        break;
                                                    }
                                                }
                                                if config_err.is_some() { break; }
                                            }
                                        }

                                        if let Some(err) = config_err {
                                            tracing::error!("Logical validation failed on reload: {}", err);
                                            if let Ok(mut w_err) = shared_config_error.write() {
                                                *w_err = Some(err);
                                            }
                                        } else {
                                            let _ = reload_tx.blocking_send(new_config.clone());
                                            if let Ok(mut w) = shared_config.write() {
                                                *w = new_config;
                                            }
                                            if let Ok(mut w_err) = shared_config_error.write() {
                                                *w_err = None;
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        tracing::error!("Config reload failed: {}", e);
                                        if let Ok(mut w_err) = shared_config_error.write() {
                                            *w_err = Some(e.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }).unwrap();
                watcher.watch(std::path::Path::new(&config_path), RecursiveMode::NonRecursive).unwrap();
                loop { std::thread::sleep(std::time::Duration::from_secs(3600)); }
            });

            // Initial Startup Logical Validation
            {
                let mut watcher_names = std::collections::HashSet::new();
                for w in &app_config.watchers {
                    if !watcher_names.insert(&w.name) {
                        anyhow::bail!("Duplicate watcher name found: {}", w.name);
                    }
                }
                for h in &app_config.hooks {
                    if !watcher_names.contains(&h.watcher) {
                        anyhow::bail!("Hook refers to nonexistent watcher: {}", h.watcher);
                    }
                    for a in &h.actions {
                        if !valid_actions.contains(&a.action_type) {
                            anyhow::bail!("Hook refers to nonexistent action type: {}", a.action_type);
                        }
                    }
                }
            }

            engine.start().await?;
        }
        Commands::Stop => {
            #[cfg(unix)]
            {
                if let Ok(pid_str) = fs::read_to_string("/tmp/anyhook.pid") {
                    if let Ok(pid) = pid_str.trim().parse::<i32>() {
                        unsafe {
                            if libc::kill(pid, libc::SIGTERM) == 0 {
                                println!("Successfully sent SIGTERM to anyhook daemon (PID: {})", pid);
                                let _ = fs::remove_file("/tmp/anyhook.pid");
                            } else {
                                println!("Failed to stop anyhook daemon. Is it running?");
                            }
                        }
                    } else {
                        println!("Invalid PID in /tmp/anyhook.pid");
                    }
                } else {
                    println!("No PID file found. Is the daemon running?");
                }
            }
            #[cfg(not(unix))]
            {
                println!("Stop command is only supported on Unix systems.");
            }
        }
        Commands::Watchers { cmd: ListCmd::List } => {
            if let Ok(status) = fetch_status(&sys_cfg).await {
                use comfy_table::{Table, Cell, Color, Attribute};
                let mut table = Table::new();
                table.set_header(vec![
                    Cell::new("Watcher Name").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    Cell::new("Type").add_attribute(Attribute::Bold).fg(Color::Cyan),
                ]);

                if let Some(watchers) = status.get("watchers").and_then(|v| v.as_array()) {
                    for w in watchers {
                        table.add_row(vec![
                            w["name"].as_str().unwrap_or(""),
                            w["type"].as_str().unwrap_or(""),
                        ]);
                    }
                }
                println!("{}", table);
            }
        }
        Commands::Hooks { cmd: ListCmd::List } => {
            if let Ok(status) = fetch_status(&sys_cfg).await {
                use comfy_table::{Table, Cell, Color, Attribute};
                let mut table = Table::new();
                table.set_header(vec![
                    Cell::new("Trigger (Watcher)").add_attribute(Attribute::Bold).fg(Color::Magenta),
                    Cell::new("Bound Actions").add_attribute(Attribute::Bold).fg(Color::Magenta),
                ]);

                if let Some(hooks) = status.get("hooks").and_then(|v| v.as_array()) {
                    for h in hooks {
                        let watcher = h["watcher"].as_str().unwrap_or("");
                        let mut action_strs = Vec::new();
                        if let Some(actions) = h["actions"].as_array() {
                            for a in actions {
                                let name = a["name"].as_str().unwrap_or("unnamed");
                                let type_ = a["type"].as_str().unwrap_or("unknown");
                                action_strs.push(format!("{} [{}]", name, type_));
                            }
                        }
                        table.add_row(vec![
                            watcher.to_string(),
                            if action_strs.is_empty() { "<None>".to_string() } else { action_strs.join(", ") }
                        ]);
                    }
                }
                println!("{}", table);
            }
        }
        Commands::Actions { cmd: ListCmd::List } => {
            if let Ok(status) = fetch_status(&sys_cfg).await {
                use comfy_table::{Table, Cell, Color, Attribute};
                let mut table = Table::new();
                table.set_header(vec![
                    Cell::new("Available Action Type").add_attribute(Attribute::Bold).fg(Color::Green),
                ]);

                if let Some(actions) = status.get("registered_actions").and_then(|v| v.as_array()) {
                    let mut sorted = actions.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>();
                    sorted.sort();
                    for a in sorted {
                        table.add_row(vec![a]);
                    }
                }
                println!("{}", table);
            }
        }
        Commands::Status => {
            let pool_res = sqlx::sqlite::SqlitePoolOptions::new()
                .connect("sqlite://.anyhook.db").await;
            
            match pool_res {
                Ok(pool) => {
                    let rows = sqlx::query("SELECT id, action_name, status, timestamp FROM action_log ORDER BY id DESC LIMIT 15")
                        .fetch_all(&pool)
                        .await?;
                    
                    use comfy_table::{Table, Cell, Color, Attribute};
                    let mut table = Table::new();
                    table.set_header(vec![
                        Cell::new("ID").add_attribute(Attribute::Bold),
                        Cell::new("Timestamp").add_attribute(Attribute::Bold),
                        Cell::new("Action").add_attribute(Attribute::Bold),
                        Cell::new("Status").add_attribute(Attribute::Bold),
                    ]);

                    for row in rows {
                        use sqlx::Row;
                        let id: i32 = row.get("id");
                        let action_name: String = row.get("action_name");
                        let status: String = row.get("status");
                        let timestamp: String = row.get("timestamp");
                        
                        let status_cell = if status.eq_ignore_ascii_case("success") {
                            Cell::new(status).fg(Color::Green)
                        } else {
                            Cell::new(status).fg(Color::Red)
                        };
                        
                        table.add_row(vec![
                            Cell::new(id),
                            Cell::new(timestamp),
                            Cell::new(action_name),
                            status_cell,
                        ]);
                    }
                    println!("\nRecent Actions Execution Log:");
                    println!("{}", table);
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

async fn fetch_status(sys_cfg: &sysconfig::SystemConfig) -> anyhow::Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let mut req = client.get(format!("http://127.0.0.1:{}/api/status", sys_cfg.dashboard_port));
    
    if let Some(auth) = &sys_cfg.dashboard_auth {
        let parts: Vec<&str> = auth.splitn(2, ':').collect();
        if parts.len() == 2 {
            req = req.basic_auth(parts[0], Some(parts[1]));
        }
    }
    
    let res = req.send().await?;
    if !res.status().is_success() {
        anyhow::bail!("Failed to fetch status (is the daemon running?): HTTP {}", res.status());
    }
    
    let json: serde_json::Value = res.json().await?;
    Ok(json)
}
