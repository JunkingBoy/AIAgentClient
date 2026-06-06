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

    #[error("WebSocket 错误: {0}")]
    WebSocket(String),
}
