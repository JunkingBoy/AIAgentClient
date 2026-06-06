use serde::{Deserialize, Serialize};

/// 服务端下发的指令
///
/// JSON 格式: `{"type": "navigate", "url": "https://..."}`
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    #[serde(rename = "navigate")]
    Navigate {
        url: String,
        #[serde(default)]
        id: Option<String>,
    },
    #[serde(rename = "click")]
    Click {
        selector: String,
        #[serde(default)]
        id: Option<String>,
    },
    #[serde(rename = "screenshot")]
    Screenshot {
        #[serde(default)]
        id: Option<String>,
    },
    #[serde(rename = "ping")]
    Ping {
        #[serde(default)]
        id: Option<String>,
    },
}

/// 客户端回传的事件 / 结果
///
/// JSON 格式: `{"type": "pong"}` 或 `{"type": "result", "data": {...}}`
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "result")]
    Result {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: serde_json::Value,
    },
    #[serde(rename = "error")]
    Error {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        message: String,
    },
}
