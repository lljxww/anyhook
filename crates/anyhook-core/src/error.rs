use thiserror::Error;

/// Anyhook 自定义错误类型
/// 知识点: 使用 `thiserror` 库可以极其优雅地定义库级别的错误枚举。
/// 它能自动帮你实现 `std::error::Error` 以及 `Display` 特征。
#[derive(Error, Debug)]
pub enum AnyhookError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Event routing error: {0}")]
    Routing(String),
    #[error("Execution error: {0}")]
    Execution(String),
    // 知识点: `#[error(transparent)]` 和 `#[from]` 是绝佳组合。
    // 它允许我们无缝地将其他框架 (比如 anyhow::Error) 的错误使用 `?` 操作符直接向上抛出，
    // 而不需要手动做繁琐的 map_err 转换。
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// 全局 Result 别名
/// 知识点: 库作者通常会定义一个局部的 `Result<T>`，把 `E` 绑定为自己的错误类型。
/// 这样在其他模块中只需写 `Result<()>` 即可，大幅简化代码签名。
pub type Result<T> = std::result::Result<T, AnyhookError>;
