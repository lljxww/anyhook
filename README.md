# Anyhook ⚓

> 一个使用 Rust 编写的高性能、跨平台事件驱动自动化引擎。

**Anyhook** 是一个模块化的 CLI 应用程序，它通过可配置的路由规则（**Hooks**），将事件源（**Watchers**）与事件处理程序（**Actions**）连接起来。它就像一个轻量级且极具可靠性的本地自动化服务器。

## ✨ 核心特性

- **跨平台:** 在 Windows、macOS 和 Linux 上均可无缝运行，底层实现完全隔离。
- **事件驱动架构:** 将事件监听 (Watchers) 与任务执行 (Actions) 完全解耦。
- **高并发与高可靠性:** 
  - 基于 Tokio 的异步任务执行引擎。
  - 支持并发限流 (Semaphore) 以及任务失败后的自动重试机制。
  - 使用 SQLite (`.anyhook.db`) 持久化保存任务执行日志与历史记录。
- **配置热重载 (Zero-Downtime):** 运行时直接修改 `anyhook.yaml`，引擎会在不漏掉任何事件、不重启服务的情况下，瞬间更新所有路由规则。
- **WASM 插件系统:** 无需重新编译引擎即可无限扩展！只需将任意编译为 `wasm32-wasip1` 的 `.wasm` 文件放入 `plugins/` 目录，引擎便会动态将其作为全新的 Action 加载。
- **精美的 Web Dashboard:** 引擎内置了一个拥有玻璃拟物化 (Glassmorphism) 和深色模式的 Web UI 控制台，直接从 CLI 二进制文件中提供服务，方便你实时监控任务状态和执行日志。

## 🚀 快速开始

### 1. 从源码编译

```bash
cargo build --release
# 编译后的可执行文件将生成在 target/release/anyhook-cli
```

### 2. 编写配置 (`anyhook.yaml`)

在你的工作区创建一个 `anyhook.yaml` 配置文件：

```yaml
plugins:
  - name: "hello_wasm"
    # path: "./plugins/hello_wasm.wasm" # 可选，如果不填则默认使用 plugins_dir 中的 {name}.wasm
    config:
      greeting: "Hello from Anyhook config!"

watchers:
  - name: "my_fs_watcher"
    type: "filesystem"
    config:
      path: "./data"

  - name: "my_cron"
    type: "timer"
    config:
      cron: "0/10 * * * * * *" # 每 10 秒触发一次

hooks:
  - watcher: "my_fs_watcher"
    actions:
      - type: "shell"
        name: "log_file_change"
        config:
          command: "echo 发现文件变动！"

  - watcher: "my_cron"
    actions:
      - type: "hello_wasm" # 将调用 plugins/hello_wasm.wasm 插件
        name: "run_wasm_plugin"
        config: {}
```

### 3. 系统级配置 (可选)

除了 `anyhook.yaml`，你还可以在全局环境配置 `anyhook.conf`，设置如监听端口、日志级别等参数。配置文件会依次从 `./anyhook.conf`, `~/.config/anyhook/anyhook.conf` (Linux/macOS), 或 `/etc/anyhook/anyhook.conf` 读取。

```yaml
dashboard_port: 3000
plugins_dir: "/etc/anyhook/plugins"
db_url: "sqlite://.anyhook.db?mode=rwc"
log_level: "info" # 可选: debug, info, warn, error
```

### 4. 启动引擎

```bash
anyhook start -c anyhook.yaml
```

引擎启动后，将在后台守护运行，并自动在 `http://127.0.0.1:3000` 开启 **Web Dashboard**。

## 🛠️ CLI 命令详解

Anyhook 提供了一套完善的命令来管理你的自动化任务：

- `anyhook start -c <config.yaml>`: 在**前台**启动自动化引擎与 Web 控制台（适合 Systemd/Docker 等环境）。
- `anyhook start -d -c <config.yaml>`: 在**后台**以守护进程模式启动引擎（原生支持，仅限 Unix 系统）。
- `anyhook stop`: 发送信号优雅停止后台守护进程。
- `anyhook watchers list`: 拉取当前正在运行的监听器列表。
- `anyhook hooks list`: 打印引擎中正在生效的事件路由映射表。
- `anyhook actions list`: 打印系统目前支持的所有动作（包含原生动作和成功加载的 WASM 插件）。
- `anyhook status`: 获取当前引擎最近执行的任务日志和状态（支持终端颜色高亮）。
- `anyhook trigger <watcher_name>`: 手动强制触发某个特定的 Watcher，立即执行绑定的 Actions。

## 🔌 Web Dashboard

Dashboard 是通过 `rust-embed` 直接打包进单一二进制程序中的，运行它不需要依赖任何外部静态资源。
打开浏览器访问 `http://127.0.0.1:3000` 即可看到：
- 当前激活的 Watcher 列表与 Hook 路由映射。
- 直接从底层 SQLite 数据库拉取的实时任务执行日志与报错信息。

## 🧩 WASM 插件系统与开发规范

Anyhook 原生支持动态加载 WebAssembly 插件，作为自定义事件处理器。由于采用了标准 WASI 接口，这使得它支持使用任何现代编程语言（Rust, Go, C++, Python 等）来扩展 Anyhook 的能力，同时无需关心复杂的内存指针传递（ABI）。

### 1. 插件生命周期与加载
1. **编译目标**: 请确保你的代码编译目标为 `wasm32-wasi` (或 `wasm32-wasip1`)。
2. **配置插件**: 在 `anyhook.yaml` 的 `plugins` 块中声明插件。Anyhook 会将其作为全新的 Action 加载。
3. **存放规范**: 如果未在配置中指定 `path`，需将编译好的 `.wasm` 文件放入 `anyhook.conf` 配置的 `plugins_dir` 中（默认是当前目录下的 `plugins/`）。也可以不声明配置，让 Anyhook 自动发现 `plugins_dir` 下的 `.wasm` 文件。
4. **文档规范**: 所有开发的插件 **必须** 包含一份完整的 Markdown 自述文件（如 `hello_wasm.md`），该文件需要与对应的 `.wasm` 文件同名并放置在同一目录下。文档内容必须包含：插件功能说明、`anyhook.yaml` 配置文件中的 `config` 参数说明与示例。引擎将在启动时解析此文件，并在 Web Dashboard 的 "Plugins" 面板中提供给用户可视化查询。
5. **命名规范**: Anyhook 会截取文件名或从配置中的 `name` 字段作为 Action 的 `type`。例如，`name: "discord_notifier"`，在 Actions 中调用时应当设置 `type: "discord_notifier"`。

### 2. 数据交互协议 (WASI)
为了跨语言通讯的极致简便，Anyhook 摒弃了复杂的线性内存操作，转而使用**环境变量**和**文件 I/O** 来完成通信：

#### 输入层 (Input)
- 环境变量 `ANYHOOK_CONTEXT` 会传入一个合法的 JSON 字符串，包含本次触发的事件及动作配置。
- 环境变量 `ANYHOOK_PLUGIN_CONFIG` 会传入一个 JSON 字符串，包含插件在 `plugins` 块中的全局配置。
- `ANYHOOK_CONTEXT` 的 Schema 如下：
  ```json
  {
    "event": {
      "id": "uuid字符串",
      "source": "触发事件的 Watcher 名字",
      "event_type": "如 timer.trigger 或 filesystem.create",
      "payload": { ...任意事件负载对象... },
      "timestamp": "ISO8601格式时间戳"
    },
    "config": { ...该 Action 在 anyhook.yaml 中配置的自定义参数... },
    "runtime_env": { ...当前运行时的系统环境变量映射... }
  }
  ```

#### 输出层 (Output)
- 环境变量 `ANYHOOK_OUTPUT_FILE` 会下发一个位于临时沙盒中的目标文件路径（例如 `/tmp/anyhook_out_xx.json`）。
- 插件在处理完逻辑后，**必须**将执行结果以 JSON 格式写入这个文件，供引擎统一拉取收集。
- 推荐输出格式 Schema：
  ```json
  {
    "status": "success 或是 error",
    "message": "执行结果的描述信息",
    "data": { ...插件自定义返回的数据内容... }
  }
  ```

#### 异常与日志
- 任何直接打印到标准输出（`stdout`）或标准错误（`stderr`）的内容，都会被引擎无缝捕获，并作为该 Task 的日志持久化存储到 SQLite 中供 Dashboard 展示。
- 若程序的退出码 (Exit Code) 不为 `0`，引擎会直接判定任务执行失败，并可能触发在引擎中配置的重试机制。

### 进阶指南
- **插件开发规范**：如果你希望使用 Rust, Go, C# 等语言开发自定义 WASM 插件，请参阅 [插件开发规范指南](./README.md#六-插件开发规范-wasi)。
- **发布与分发指南**：如需打包 Anyhook 为多平台可执行文件或 `.deb` 包，请参阅 [发布与分发指南](./docs/RELEASE_GUIDE.md)。

### 3. 多语言插件开发示例

<details>
<summary>🦀 Rust 插件开发示例</summary>

```rust
use std::env;
use std::fs;
use serde_json::Value;

fn main() {
    // 1. 获取输入上下文与目标输出路径
    let context_str = env::var("ANYHOOK_CONTEXT").unwrap_or_else(|_| "{}".to_string());
    let out_file = env::var("ANYHOOK_OUTPUT_FILE").expect("Missing ANYHOOK_OUTPUT_FILE");

    let context: Value = serde_json::from_str(&context_str).unwrap();
    
    // 可以直接使用标准输出打印日志，引擎会自动捕获
    println!("接收到的触发事件来源: {}", context["event"]["source"]);

    // 2. 执行你的业务逻辑...

    // 3. 将结果输出到目标文件
    let output = format!(
        "{{\"status\": \"success\", \"message\": \"已成功处理！\"}}"
    );
    fs::write(out_file, output).unwrap();
}
```
</details>

<details>
<summary>🐹 Go 插件开发示例 (使用 TinyGo)</summary>

```go
package main

import (
	"fmt"
	"os"
)

func main() {
	contextStr := os.Getenv("ANYHOOK_CONTEXT")
	outFile := os.Getenv("ANYHOOK_OUTPUT_FILE")

	fmt.Printf("处理输入事件: %s\n", contextStr)

	// 将结果写入引擎指定的文件
	result := `{"status":"success","message":"Hello from TinyGo!"}`
	err := os.WriteFile(outFile, []byte(result), 0644)
	if err != nil {
		fmt.Fprintf(os.Stderr, "写入失败: %v\n", err)
		os.Exit(1)
	}
}
```
*编译命令: `tinygo build -o my_plugin.wasm -target=wasi main.go`*
</details>

<details>
<summary>🟣 .NET (C#) 插件开发示例</summary>

```csharp
using System;
using System.IO;

class Program
{
    static void Main()
    {
        // 1. 获取输入上下文与目标输出路径
        string contextStr = Environment.GetEnvironmentVariable("ANYHOOK_CONTEXT") ?? "{}";
        string? outFile = Environment.GetEnvironmentVariable("ANYHOOK_OUTPUT_FILE");

        if (string.IsNullOrEmpty(outFile))
        {
            Console.Error.WriteLine("写入失败: 缺少 ANYHOOK_OUTPUT_FILE 环境变量");
            Environment.Exit(1);
        }

        // 可以直接使用标准输出打印日志，引擎会自动捕获
        Console.WriteLine($"接收到的上下文: {contextStr}");

        // 2. 执行你的业务逻辑...

        // 3. 将结果输出到目标文件
        string result = "{\"status\":\"success\",\"message\":\"Hello from .NET WASI!\"}";
        File.WriteAllText(outFile, result);
    }
}
```
*编译命令 (需安装 `wasi-experimental` workload): `dotnet build -c Release -r wasi-wasm`*
</details>

## 🛡️ 后台守护进程部署 (Daemonization)

Anyhook 支持多种方式进行后台守护部署。需要注意的是，如果你赋予可执行文件 `+x` 权限并直接运行 `anyhook start`，它会默认在**前台（Foreground）**运行，一旦你关闭终端或者按下 `Ctrl+C` 进程就会结束。

如果你希望在无界面服务器（Headless Server）上保持 Anyhook 后台常驻，你可以选择以下方式：

### 1. 原生守护进程模式 (推荐轻量级部署)
在 Linux / macOS 系统上，你可以直接追加 `-d` (或 `--daemon`) 标志启动，引擎会自动脱离当前终端并在后台驻留：
```bash
anyhook start -d -c anyhook.yaml
```
- **进程追踪**: 引擎会在 `/tmp/anyhook.pid` 记录进程号。
- **日志输出**: 所有标准输出和错误会被重定向到 `/tmp/anyhook.out` 和 `/tmp/anyhook.err`。
- **停止服务**: 想要关闭后台引擎时，只需执行 `anyhook stop` 即可优雅退出。

### 2. Linux (Systemd) 生产环境推荐
作为一款生产级的现代应用程序，长期运行我们更推荐将任务交由系统的守护进程管理器处理。创建 `/etc/systemd/system/anyhook.service`：

```ini
[Unit]
Description=Anyhook Automation Engine
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/path/to/workspace
ExecStart=/path/to/anyhook-cli start -c anyhook.yaml
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```
启动并配置开机自启：
```bash
systemctl daemon-reload
systemctl enable --now anyhook
```

### macOS (Launchd)
在 `~/Library/LaunchAgents/com.anyhook.daemon.plist` 创建文件：
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.anyhook.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/path/to/anyhook-cli</string>
        <string>start</string>
        <string>-c</string>
        <string>anyhook.yaml</string>
    </array>
    <key>WorkingDirectory</key>
    <string>/path/to/workspace</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```
加载并启动：
```bash
launchctl load ~/Library/LaunchAgents/com.anyhook.daemon.plist
```

### Windows (NSSM)
在 Windows 环境下，推荐使用 [NSSM (Non-Sucking Service Manager)](http://nssm.cc/) 将 Anyhook 注册为后台服务：
```cmd
nssm install Anyhook C:\path\to\anyhook-cli.exe start -c anyhook.yaml
nssm start Anyhook
```

### 简易后台部署 (nohup / pm2)
如果你不需要配置系统服务，也可以使用开发环境中常见的通用方式挂载至后台：
```bash
# Unix 平台通过 nohup 挂载
nohup anyhook-cli start -c anyhook.yaml > anyhook.log 2>&1 &

# 或者使用 Node.js 生态的 PM2 (全平台通用)
pm2 start anyhook-cli --name "anyhook" -- start -c anyhook.yaml
```

## 🏗️ 架构设计

Anyhook 项目划分为以下几个高度解耦的模块 (Crates)：
- `anyhook-core`: 核心抽象，包含 Traits、Errors 和通用数据模型。
- `anyhook-engine`: 引擎运行时、任务执行器、路由分配器以及 SQLite 状态持久化模块。
- `anyhook-watchers`: 内置的事件触发源（文件监控、Cron 定时器、手动触发、Webhook 等）。
- `anyhook-actions`: 内置的任务处理器（Shell 脚本、HTTP 请求、解压缩、WASM 加载器等）。
- `anyhook-dashboard`: 基于 Axum 的后端，以及通过 `rust-embed` 打包的原生前端 HTML/CSS/JS 静态资源。
- `anyhook-cli`: 基于 `clap` 构建的命令行用户交互接口。

---
*Built with ❤️ using Rust.*
