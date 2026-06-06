use serde::Deserialize;

/// `/key/public` 接口的 data 载荷
#[derive(Debug, Deserialize)]
pub struct KeyData {
    pub key: String,
    pub index: usize,
}
