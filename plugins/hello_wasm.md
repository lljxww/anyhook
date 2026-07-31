# Hello WASM Plugin

这是一个供开发者参考的基础 WebAssembly 插件示例。

## 功能介绍

该插件主要用于演示 Anyhook 是如何与 `.wasm` 模块进行数据交互的。当它被触发时，会简单地将传入的 `ANYHOOK_CONTEXT` 上下文、以及自身在 `anyhook.yaml` 中的全局 `ANYHOOK_PLUGIN_CONFIG` 配置，全部合并并原封不动地返回输出到结果文件中。

## 配置说明

在 `anyhook.yaml` 中的 `plugins` 块进行配置。由于该插件是演示用途，它可以接收任意形式的 JSON 配置。

### 示例配置

```yaml
plugins:
  - name: "hello_wasm"
    config:
      greeting: "Hello from Anyhook config!"
      retry_count: 3
```

### 参数字典

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `greeting` | `String` | 否 | 无 | 欢迎语，会在执行输出中原样返回以供验证。 |
| `[其他]` | `Any` | 否 | 无 | 任意键值对均可传入，插件会将其包含在最终的 `out.json` 结果中。 |

## 开发建议

通过查看本插件的源码（位于 `examples/hello_wasm/src/main.rs`），你可以学习到：
1. 如何读取和解析来自引擎的环境变量输入。
2. 如何将标准化的 JSON 结果写入由 `ANYHOOK_OUTPUT_FILE` 指定的路径。
