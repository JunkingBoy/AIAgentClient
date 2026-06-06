use crate::config::AppConfig;
use crate::crypto;
use crate::error::AppResult;
// TODO Phase 3: 恢复 WebSocket 时取消注释
// use crate::protocol::codec;
// use crate::transport::ws::WsConnection;
// use tokio_tungstenite::tungstenite::Message;

/// 连接管理器
///
/// 职责：
/// - 认证（HTTP 获取密钥）
/// - 建立 WebSocket 连接
/// - 事件循环：收消息 → 解码 → 派发 → 回传结果
/// - TODO Phase 3: 心跳 + 自动重连
pub struct Manager {
    config: AppConfig,
}

impl Manager {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// 启动连接管理：认证 → WS 连接 → 事件循环
    ///
    /// 当前事件循环在连接断开后返回，main 保持 pending。
    /// Phase 3 改为内部重连循环，不再返回。
    pub async fn run(&self) -> AppResult<()> {
        let _key = self.authenticate().await?;
        tracing::info!("密钥获取成功");

        // TODO Phase 3: 恢复 WebSocket 连接
        // let mut ws = WsConnection::connect(&self.config.server_url, &key).await?;
        // tracing::info!("WebSocket 连接成功");
        // self.event_loop(&mut ws).await;

        tracing::info!("当前为 HTTP 调试模式，跳过 WebSocket 连接");

        Ok(())
    }

    /// 从服务端获取 AES 密钥（缓存，仅首次请求网络）
    async fn authenticate(&self) -> AppResult<Vec<u8>> {
        let key_bytes = crypto::get_cached_key(&self.config.server_url).await?.clone();
        tracing::info!("密钥获取成功");
        Ok(key_bytes)
    }

    // ═══════════════════════════════════════════════════════════
    //  TODO Phase 3: 恢复 WebSocket 时取消注释以下方法
    // ═══════════════════════════════════════════════════════════

    // /// 事件循环：接收消息 → 解码 → 派发 → 回传结果
    // async fn event_loop(&self, ws: &mut WsConnection) {
    //     loop {
    //         let msg = match ws.recv().await {
    //             Ok(Some(msg)) => msg,
    //             Ok(None) => { tracing::warn!("连接已断开"); break; }
    //             Err(e) => { tracing::error!("接收消息出错: {e}"); break; }
    //         };
    //         match msg {
    //             Message::Text(text) => {
    //                 tracing::debug!("收到消息: {text}");
    //                 self.handle_text(ws, &text).await;
    //             }
    //             Message::Ping(_) => tracing::trace!("收到 Ping"),
    //             Message::Close(frame) => { tracing::info!("服务端关闭连接: {:?}", frame); break; }
    //             Message::Binary(_) => tracing::warn!("收到不支持的二进制消息"),
    //             Message::Pong(_) => {}
    //             Message::Frame(_) => {}
    //         }
    //     }
    // }

    // /// 处理一条文本消息：解码 → 派发 → 回复
    // async fn handle_text(&self, ws: &mut WsConnection, text: &str) {
    //     let cmd = match codec::deserialize_command(text) {
    //         Ok(cmd) => cmd,
    //         Err(e) => { tracing::error!("指令解析失败: {e}"); return; }
    //     };
    //     let event = match crate::handler::dispatcher::dispatch(&cmd).await {
    //         Ok(ev) => ev,
    //         Err(e) => { tracing::error!("指令执行失败: {e}"); return; }
    //     };
    //     let reply_text = match codec::serialize_event(&event) {
    //         Ok(t) => t,
    //         Err(e) => { tracing::error!("事件序列化失败: {e}"); return; }
    //     };
    //     if let Err(e) = ws.send(Message::Text(reply_text)).await {
    //         tracing::error!("回复失败: {e}");
    //     }
    // }
}
