use crate::error::{AppError, AppResult};
use crate::protocol::message::{Command, Event};

/// 将 JSON 文本解析为服务端指令
pub fn deserialize_command(text: &str) -> AppResult<Command> {
    serde_json::from_str(text)
        .map_err(|e| AppError::WebSocket(format!("指令解析失败: {e}")))
}

/// 将事件序列化为 JSON 文本（用于通过 WebSocket 发送）
pub fn serialize_event(event: &Event) -> AppResult<String> {
    serde_json::to_string(event)
        .map_err(|e| AppError::WebSocket(format!("事件序列化失败: {e}")))
}
