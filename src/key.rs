use std::sync::OnceLock;

use base64::Engine;

use crate::dto::{KeyData, StandardHttpResponse};
use crate::enums::HttpEndpoint;
use crate::error::{AppError, AppResult};

/// 全局 AES 密钥缓存，一个 session 内只请求一次
static AES_KEY: OnceLock<Vec<u8>> = OnceLock::new();

/// 获取 AES 密钥（带缓存）
pub async fn get_cached_key(server_url: &str) -> AppResult<&'static Vec<u8>> {
    if let Some(key) = AES_KEY.get() {
        return Ok(key);
    }
    let key = fetch_public_key(server_url).await?;
    let _ = AES_KEY.set(key);
    Ok(AES_KEY.get().unwrap())
}

/// 从服务端获取 AES-128 密钥
async fn fetch_public_key(server_url: &str) -> AppResult<Vec<u8>> {
    let url = HttpEndpoint::KeyPublic.url(server_url);
    let resp = reqwest::get(&url).await?;
    let body: StandardHttpResponse<KeyData> = resp.json().await?;

    eprintln!("服务端响应: code={}, msg={}", body.code, body.msg);

    if body.is_error() {
        return Err(AppError::BusinessError(body.code, body.msg));
    }

    let data = body
        .data
        .ok_or_else(|| AppError::Client("响应中 data 字段为空".into()))?;

    let hex_str = extract_key(&data.key, data.index)
        .map_err(|e| AppError::Client(format!("密钥提取失败: {e}")))?;

    eprintln!("提取后的 hex 密钥字符串: {hex_str}");

    let key_bytes = hex_decode(&hex_str)
        .map_err(|e| AppError::Client(format!("hex 解码失败: {e}")))?;

    eprintln!("AES 密钥字节长度: {} (期望 16)", key_bytes.len());

    Ok(key_bytes)
}

/// 从服务端返回的填充密钥中提取真实密钥
///
/// 服务端在原始密钥的 `index` 位置插入 16 字节随机填充后 base64 编码，
/// 客户端解码后去掉这 16 字节即得到真实密钥。
pub fn extract_key(filled_key: &str, index: usize) -> Result<String, String> {
    let decoded = engine()
        .decode(filled_key)
        .map_err(|e| format!("base64 解码失败: {e}"))?;

    if index + 16 > decoded.len() {
        return Err(format!(
            "index({index}) + 16 超出解码数据长度({})",
            decoded.len()
        ));
    }

    let raw: Vec<u8> = [&decoded[..index], &decoded[index + 16..]].concat();

    String::from_utf8(raw).map_err(|e| format!("密钥不是有效的 UTF-8: {e}"))
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("hex 字符串长度必须是偶数".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| format!("hex 解码失败: {e}"))
        })
        .collect()
}

fn engine() -> base64::engine::general_purpose::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}
