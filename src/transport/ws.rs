use crate::error::{AppError, AppResult};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// WebSocket 传输层
///
/// 只关心原始帧的收发，不知道消息格式或业务逻辑。
pub struct WsConnection {
    stream: WsStream,
}

impl WsConnection {
    /// 向服务端建立 WebSocket 连接
    pub async fn connect(server_url: &str, key: &str) -> AppResult<Self> {
        let ws_scheme = if server_url.starts_with("https") {
            "wss"
        } else {
            "ws"
        };
        let rest = server_url
            .trim_start_matches("http://")
            .trim_start_matches("https://");
        let url = format!(
            "{}://{}/ws?key={}",
            ws_scheme,
            rest.trim_end_matches('/'),
            key
        );

        tracing::debug!("正在连接 WebSocket: {}", url);
        let (stream, _) = connect_async(&url)
            .await
            .map_err(|e| AppError::WebSocket(format!("连接失败: {e}")))?;

        Ok(Self { stream })
    }

    /// 发送一条 WebSocket 帧
    pub async fn send(&mut self, msg: Message) -> AppResult<()> {
        self.stream
            .send(msg)
            .await
            .map_err(|e| AppError::WebSocket(format!("发送失败: {e}")))
    }

    /// 接收一条 WebSocket 帧，`None` 表示连接已关闭
    pub async fn recv(&mut self) -> AppResult<Option<Message>> {
        match self.stream.next().await {
            Some(Ok(msg)) => Ok(Some(msg)),
            Some(Err(e)) => Err(AppError::WebSocket(format!("接收失败: {e}"))),
            None => Ok(None),
        }
    }

    /// 主动关闭连接
    pub async fn close(mut self) -> AppResult<()> {
        self.stream
            .close(None)
            .await
            .map_err(|e| AppError::WebSocket(format!("关闭失败: {e}")))
    }
}
