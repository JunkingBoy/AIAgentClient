use std::io;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O 错误: {0}")]
    Io(#[from] io::Error),

    #[error("已有实例在运行: {0}")]
    InstanceExists(String),

    #[error("HTTP 请求失败: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON 序列化/反序列化失败: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("用户取消了操作")]
    UserCancel,

    #[error("输入验证失败: {0}")]
    InvalidInput(String),

    #[error("客户端错误: {0}")]
    Client(String),

    #[error("业务错误 (code={0}): {1}")]
    BusinessError(i32, String),
}
