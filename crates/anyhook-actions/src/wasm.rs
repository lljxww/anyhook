use anyhook_core::traits::{Action, ActionContext};
use anyhook_core::error::{Result, AnyhookError};
use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;

/// WasmAction (WebAssembly 插件动作)
/// 允许用户在不重新编译引擎的情况下，使用任意语言编写并加载 WASM 插件。
/// 知识点: WASM (WebAssembly) 提供了一个极度安全的沙盒环境。
/// 配合 WASI (WebAssembly System Interface)，我们可以严格控制插件能访问哪些文件和环境变量。
#[derive(Debug)]
pub struct WasmAction {
    pub wasm_path: PathBuf,
}

impl WasmAction {
    pub fn new(wasm_path: PathBuf) -> Self {
        Self { wasm_path }
    }
}

#[async_trait]
impl Action for WasmAction {
    fn action_type(&self) -> &'static str {
        "wasm"
    }

    async fn execute(&self, ctx: ActionContext) -> Result<Value> {
        let wasm_path = self.wasm_path.clone();
        
        // 知识点: wasmtime 的模块编译和实例化属于 CPU 密集型甚至可能是阻塞型操作。
        // 在 tokio (异步运行时) 中，直接运行长耗时的同步代码会"阻塞(Block)"工作线程。
        // 因此必须使用 `spawn_blocking` 将其转移到专门的同步线程池中执行。
        let result = tokio::task::spawn_blocking(move || -> std::result::Result<Value, String> {
            let engine = Engine::default();
            let mut linker = Linker::new(&engine);
            wasmtime_wasi::add_to_linker(&mut linker, |s| s).map_err(|e| e.to_string())?;

            let module = Module::from_file(&engine, &wasm_path).map_err(|e| e.to_string())?;
            
            let temp_dir = std::env::temp_dir();
            let out_file_name = format!("anyhook_wasm_{}.out", uuid::Uuid::new_v4());
            let host_out_file = temp_dir.join(&out_file_name);
            let guest_out_file = format!("/tmp/{}", out_file_name);
            
            let mut builder = WasiCtxBuilder::new();
            builder.env("ANYHOOK_CONTEXT", &serde_json::to_string(&ctx).unwrap()).unwrap();
            builder.env("ANYHOOK_OUTPUT_FILE", &guest_out_file).unwrap();
            builder.inherit_stdout();
            builder.inherit_stderr();
            
            let preopen_dir = cap_std::fs::Dir::open_ambient_dir(&temp_dir, cap_std::ambient_authority()).map_err(|e| e.to_string())?;
            builder.preopened_dir(preopen_dir, "/tmp").map_err(|e| e.to_string())?;
            
            let wasi_ctx = builder.build();
            let mut store = Store::new(&engine, wasi_ctx);
            
            let instance = linker.instantiate(&mut store, &module).map_err(|e| e.to_string())?;
            let start = instance.get_typed_func::<(), ()>(&mut store, "_start").map_err(|e| e.to_string())?;
            start.call(&mut store, ()).map_err(|e| e.to_string())?;
            
            let output = std::fs::read_to_string(&host_out_file).unwrap_or_else(|_| "{}".to_string());
            let _ = std::fs::remove_file(&host_out_file);
            
            let val: Value = serde_json::from_str(&output).unwrap_or_else(|_| serde_json::json!({
                "output": output.trim()
            }));
            
            Ok(val)
        }).await.map_err(|e| AnyhookError::Execution(e.to_string()))?;
        
        result.map_err(|e| AnyhookError::Execution(e))
    }
}
