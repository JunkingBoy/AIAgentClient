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
