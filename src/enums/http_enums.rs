/// 统一请求枚举 — 每种变体对应一个 API 接口及其参数
pub enum StandardHttpRequestEnum {
    /// GET /key/public — 获取 AES 公钥（无参数）
    KeyPublic,
    UserBind { client_id: String, email: String },
}

/// HTTP API 接口路径枚举
pub enum HttpEndpoint {
    KeyPublic,
    UserBind,
}

impl HttpEndpoint {
    fn path(&self) -> &'static str {
        match self {
            HttpEndpoint::KeyPublic => "/key/public",
            HttpEndpoint::UserBind => "/user/bind",
        }
    }

    /// 完整 URL：`{server_url}{path}`
    pub fn url(&self, server_url: &str) -> String {
        format!("{}{}", server_url.trim_end_matches('/'), self.path())
    }
}
