use reqwest::Client;

use crate::dto::StandardHttpResponse;
use crate::enums::{HttpEndpoint, StandardHttpRequestEnum};
use crate::error::{AppError, AppResult};

/// 统一 HTTP 请求客户端
pub struct StandardMetaRequest {
    inner: Client,
    server_url: String,
}

impl StandardMetaRequest {
    pub fn new(server_url: &str) -> Self {
        Self {
            inner: Client::new(),
            server_url: server_url.trim_end_matches('/').to_string(),
        }
    }

    /// 统一发送 HTTP 请求
    pub async fn send<T: serde::de::DeserializeOwned>(
        &self,
        request: StandardHttpRequestEnum,
    ) -> AppResult<Option<T>> {
        let endpoint = match &request {
            StandardHttpRequestEnum::KeyPublic => HttpEndpoint::KeyPublic,
            StandardHttpRequestEnum::UserBind { .. } => HttpEndpoint::UserBind,
        };
        let url = endpoint.url(&self.server_url);
        let req_builder = match &request {
            StandardHttpRequestEnum::KeyPublic => self.inner.get(&url),
            StandardHttpRequestEnum::UserBind { client_id, email } => {
                let body = serde_json::json!({
                    "client_id": client_id,
                    "email": email,
                });
                self.inner.post(&url).json(&body)
            }
        };
        let resp = req_builder.send().await?;
        let http_status = resp.status();
        if !http_status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Client(format!("HTTP 请求异常{}: {}", http_status, text)));
        }
        let body: StandardHttpResponse<T> = resp.json().await?;
        if body.is_error() {return Err(AppError::BusinessError(body.code, body.msg));}
        Ok(body.data)
    }
}
