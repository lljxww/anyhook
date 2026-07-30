# anyhook 功能开发与验收清单

本文档用于追踪 `anyhook` 项目的开发进度。后续开发将严格按照此清单进行，并在每个功能实现后更新状态 (将 `[ ]` 更改为 `[x]`)。

---

## Phase 1: 最小可行性产品 (MVP) 核心骨架
**目标:** 跑通“配置解析 -> 监听文件事件 -> 路由分发 -> 触发打包动作”的核心闭环。

### 功能列表
- [x] **初始化 Workspace**
  - [x] 创建 `Cargo.toml` workspace。
  - [x] 创建对应的 Crates (`anyhook-cli`, `anyhook-core`, `anyhook-engine`, `anyhook-watchers`, `anyhook-actions`, `anyhook-plugin`)。
- [x] **核心抽象设计 (anyhook-core)**
  - [x] 定义统一 JSON 兼容的 `Event` 结构体。
  - [x] 定义 `ActionContext` 结构体。
  - [x] 定义异步的 `Watcher` 和 `Action` Trait。
  - [x] 定义统一的错误类型处理。
- [x] **配置系统 (anyhook-core / anyhook-cli)**
  - [x] 基于 `serde_yaml` 定义配置结构 (`WatcherConfig`, `ActionConfig`, `HookConfig`)。
- [x] **内置 Watcher (anyhook-watchers)**
  - [x] 基于 `notify` crate 实现跨平台的 `filesystem` 监听器 (支持新增、修改、删除、重命名)。
- [x] **内置 Action (anyhook-actions)**
  - [x] 基于 `tar` 和 `flate2` crate 实现 `archive` 动作 (支持打包为 `tar.gz`)。
- [x] **事件引擎 (anyhook-engine)**
  - [x] 实现 `Event Bus` (基于 `tokio::sync::mpsc`)。
  - [x] 实现 `Router` (基于配置将事件分发给对应的动作)。
  - [x] 实现 `Executor` (初步的异步任务调度)。
- [x] **命令行 CLI (anyhook-cli)**
  - [x] `anyhook init`: 在当前目录生成默认的 `anyhook.yaml` 配置文件。
  - [x] `anyhook start`: 加载配置并启动前台运行（包含基础日志输出）。

### Phase 1 验收标准 (Acceptance Criteria)
1. 能够编译出单一可执行文件 `anyhook`。
2. 在任意空目录运行 `anyhook init` 可生成合法的配置文件模板。
3. 运行 `anyhook start` 能够成功加载配置，不报错并持续运行。
4. **端到端测试**: 配置监听目录 A，触发动作打包到目录 B。在目录 A 中新建或修改文件，能够观察到日志输出，并且目录 B 中成功生成 `.tar.gz` 压缩包。

---

## Phase 2: 核心能力扩展与可靠性提升
**目标:** 补全常用的 Watcher 和 Action，支持多种触发方式，加入任务持久化和可靠性保证。

### 功能列表
- [x] **扩展 Watcher (anyhook-watchers)**
  - [x] `timer`: 基于 cron 表达式的定时任务监听器。
  - [x] `manual`: 允许通过 CLI 手动触发事件。
  - [x] `webhook`: 提供 HTTP 接口接收外部触发。
- [x] **扩展 Action (anyhook-actions)**
  - [x] `shell`: 跨平台命令执行 (Windows 使用 `powershell`，Unix 使用 `bash`)。
  - [x] `http`: 发送 HTTP Webhook 请求。
  - [x] `extract`: 解压缩动作。
- [x] **引擎可靠性升级 (anyhook-engine)**
  - [x] 引入 `SQLite` 记录任务执行状态 (Event 持久化、防止重复执行)。
  - [x] 增加 Action 失败重试机制。
  - [x] 限制多任务并发数量。
  - [x] 优雅退出机制 (捕捉 SIGTERM/SIGINT，等待当前任务完成)。
- [x] **CLI 扩展 (anyhook-cli)**
  - [x] `anyhook list`: 查看当前配置的 Watchers 和 Actions。
  - [x] `anyhook status`: 查看当前运行状态及历史任务记录。
  - [x] `anyhook trigger <watcher_name>`: 配合 `manual` Watcher 触发事件。

### Phase 2 验收标准 (Acceptance Criteria)
1. 配置 `timer` 能够按照 cron 表达式准时触发任务。
2. 执行 `anyhook trigger xxx` 能够立刻拉起对应的 Action。
3. `shell` 动作能够成功在 Windows 和 macOS/Linux 上执行系统命令并捕获输出。
4. 强行终止程序 (Ctrl+C) 时，能够看到“等待任务完成”的日志。
5. 任务执行历史（成功/失败）可以通过 `anyhook status` 或查询 SQLite 数据库验证。

---

## Phase 3: 扩展性与企业级特性
**目标:** 实现高级插件化架构，支持跨语言扩展和远程管理。

### 功能列表
- [x] **WASM 插件系统 (anyhook-plugin)**
  - [x] 集成 WASM 运行时 (如 `wasmtime` 或 `wasmer`)。
  - [x] 定义 WASM 插件的数据交换标准 (Input JSON -> WASM -> Output JSON)。
  - [x] 实现动态加载目录中的 `.wasm` 文件作为 Watcher 或 Action。
- [x] **配置与管理 (anyhook-engine)**
  - [x] 支持通过监听 `anyhook.yaml` 文件的变化，自动热重载 Watcher/Action 配置。
- [x] **可视化与远程控制**
  - [x] 提供一个可视化的 Web Dashboard (比如集成一个极简 HTTP Server 提供静态页面)。 (仪表盘：查看路由、日志、状态)。
  - [ ] 提供 REST API 或 gRPC 接口实现远程控制引擎。

### Phase 3 验收标准 (Acceptance Criteria)
1. 能够用 Rust 或 Go 编写一个简单的 WASM 模块，放到 `plugins/` 目录下即可被 `anyhook` 识别并执行。
2. 运行时修改配置文件，系统能够输出 "Config reloaded" 并按新规则执行，服务不中断。
