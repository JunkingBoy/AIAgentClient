use std::sync::OnceLock;

use aes::Aes128;
use base64::Engine;
use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use cbc::{Decryptor, Encryptor};
use rand::rngs::OsRng;
use rand::RngCore;

use crate::dto::{KeyData, StandardHttpResponse};
use crate::error::{AppError, AppResult};

type AesCbc = Encryptor<Aes128>;

/// 全局 AES 密钥缓存，一个 session 内只请求一次
static AES_KEY: OnceLock<Vec<u8>> = OnceLock::new();

/// AES-128-CBC 加密，返回 base64(iv + ciphertext)
///
/// - `key`: 16 字节密钥
/// - `plaintext`: 待加密的明文字符串
/// - 返回: base64 编码的 `iv(16字节) || ciphertext`
pub fn encrypt(key: &[u8], plaintext: &str) -> AppResult<String> {
    let pt = plaintext.as_bytes();

    // 16 字节随机 IV
    let mut iv = [0u8; 16];
    OsRng.fill_bytes(&mut iv);

    // 缓冲区：明文 + 最大填充长度 (16 字节)
    let mut buf = vec![0u8; pt.len() + 16];
    buf[..pt.len()].copy_from_slice(pt);

    let cipher =
        AesCbc::new_from_slices(key, &iv).map_err(|e| AppError::WebSocket(format!("{:?}", e)))?;

    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buf, pt.len())
        .map_err(|e| AppError::WebSocket(format!("加密失败: {:?}", e)))?;

    // IV || ciphertext → base64
    let mut out = Vec::with_capacity(16 + ciphertext.len());
    out.extend_from_slice(&iv);
    out.extend_from_slice(ciphertext);

    Ok(engine().encode(&out))
}

/// AES-128-CBC 解密，输入为 `encrypt()` 输出的 base64 字符串
///
/// 返回解密后的明文字符串
pub fn decrypt(key: &[u8], data: &str) -> AppResult<String> {
    let encrypted = engine()
        .decode(data)
        .map_err(|e| AppError::WebSocket(format!("base64 解码失败: {e}")))?;

    if encrypted.len() < 17 {
        return Err(AppError::WebSocket("加密数据太短".into()));
    }

    let iv = &encrypted[..16];
    let ciphertext = &encrypted[16..];

    let mut buf = ciphertext.to_vec();
    let cipher = Decryptor::<Aes128>::new_from_slices(key, iv)
        .map_err(|e| AppError::WebSocket(format!("{:?}", e)))?;

    let plaintext = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|e| AppError::WebSocket(format!("解密失败: {:?}", e)))?;

    String::from_utf8(plaintext.to_vec())
        .map_err(|e| AppError::WebSocket(format!("解密结果不是有效 UTF-8: {e}")))
}

/// 获取 AES 密钥（带缓存）
///
/// 首次调用会请求服务端，之后直接返回缓存值。
pub async fn get_cached_key(server_url: &str) -> AppResult<&'static Vec<u8>> {
    if let Some(key) = AES_KEY.get() {
        return Ok(key);
    }
    let key = fetch_public_key(server_url).await?;
    let _ = AES_KEY.set(key); // 并发时仅第一个写入生效
    Ok(AES_KEY.get().unwrap())
}

/// 从服务端获取 AES-128 密钥
///
/// GET `{server_url}/key/public`
///
/// 服务端分发的 key 是 hex 编码的字符串（如 "0123456789abcdef0123456789abcdef"），
/// 客户端需要 hex 解码得到 16 字节密钥。
pub async fn fetch_public_key(server_url: &str) -> AppResult<Vec<u8>> {
    let url = format!("{}/key/public", server_url.trim_end_matches('/'));
    let resp = reqwest::get(&url).await?;
    let body: StandardHttpResponse<KeyData> = resp.json().await?;

    eprintln!("服务端响应: code={}, msg={}", body.code, body.msg);

    if !body.is_success() {
        return Err(AppError::BusinessError(body.code, body.msg));
    }

    let data = body
        .data
        .ok_or_else(|| AppError::WebSocket("响应中 data 字段为空".into()))?;

    let hex_str = extract_key(&data.key, data.index)
        .map_err(|e| AppError::WebSocket(format!("密钥提取失败: {e}")))?;

    eprintln!("提取后的 hex 密钥字符串: {hex_str}");

    // hex 解码 → 16 字节 AES-128 密钥
    let key_bytes = hex_decode(&hex_str)
        .map_err(|e| AppError::WebSocket(format!("hex 解码失败: {e}")))?;

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

    // 去掉 index 位置起的 16 字节填充
    let raw: Vec<u8> = [&decoded[..index], &decoded[index + 16..]].concat();

    String::from_utf8(raw).map_err(|e| format!("密钥不是有效的 UTF-8: {e}"))
}

/// 将 hex 字符串解码为字节数组
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
