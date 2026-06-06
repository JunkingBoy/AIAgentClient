use serde::Deserialize;

/// 服务端标准响应信封
#[derive(Debug, Deserialize)]
pub struct StandardHttpResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> StandardHttpResponse<T> {
    pub fn is_success(&self) -> bool {self.code == 1001}
    pub fn is_error(&self) -> bool {self.code != 1001}
    pub fn msg(&self) -> &str {&self.msg}
}