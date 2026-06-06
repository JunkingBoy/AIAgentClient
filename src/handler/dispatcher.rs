use crate::error::AppResult;
use crate::protocol::message::{Command, Event};

/// 派发指令并返回结果事件
///
/// 当前为同步 stub，所有指令返回占位结果。
/// Phase 4 引入 browser 依赖后改为 `Dispatcher` 结构体持有浏览器实例。
pub async fn dispatch(cmd: &Command) -> AppResult<Event> {
    match cmd {
        Command::Navigate { url, id } => {
            tracing::info!("导航指令: {url}");
            // TODO Phase 4: 调用浏览器导航
            Ok(Event::Result {
                id: id.clone(),
                data: serde_json::json!({"status": "ok", "url": url}),
            })
        }
        Command::Click { selector, id } => {
            tracing::info!("点击指令: {selector}");
            Ok(Event::Result {
                id: id.clone(),
                data: serde_json::json!({"status": "ok"}),
            })
        }
        Command::Screenshot { id } => {
            tracing::info!("截图指令");
            Ok(Event::Result {
                id: id.clone(),
                data: serde_json::json!({"status": "pending"}),
            })
        }
        Command::Ping { id: _ } => Ok(Event::Pong),
    }
}
