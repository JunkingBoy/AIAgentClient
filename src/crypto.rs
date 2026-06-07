use aes::Aes128;
use base64::Engine;
use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use cbc::{Decryptor, Encryptor};
use rand::rngs::OsRng;
use rand::RngCore;

use crate::service::key;
use crate::error::{AppError, AppResult};

type AesCbc = Encryptor<Aes128>;

/// AES-128-CBC 加密，返回 base64(iv + ciphertext)
pub fn encrypt(key: &[u8], plaintext: &str) -> AppResult<String> {
    let pt = plaintext.as_bytes();

    let mut iv = [0u8; 16];
    OsRng.fill_bytes(&mut iv);

    let mut buf = vec![0u8; pt.len() + 16];
    buf[..pt.len()].copy_from_slice(pt);

    let cipher =
        AesCbc::new_from_slices(key, &iv).map_err(|e| AppError::Client(format!("{:?}", e)))?;

    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buf, pt.len())
        .map_err(|e| AppError::Client(format!("加密失败: {:?}", e)))?;

    let mut out = Vec::with_capacity(16 + ciphertext.len());
    out.extend_from_slice(&iv);
    out.extend_from_slice(ciphertext);

    Ok(engine().encode(&out))
}

/// AES-128-CBC 解密，输入为 `encrypt()` 输出的 base64 字符串
pub fn decrypt(data: &str) -> AppResult<String> {
    let key = key::get_cached_aes_key()?;
    let encrypted = engine()
        .decode(data)
        .map_err(|e| AppError::Client(format!("base64 解码失败: {e}")))?;

    if encrypted.len() < 17 {
        return Err(AppError::Client("加密数据太短".into()));
    }

    let iv = &encrypted[..16];
    let ciphertext = &encrypted[16..];

    let mut buf = ciphertext.to_vec();
    let cipher = Decryptor::<Aes128>::new_from_slices(key, iv)
        .map_err(|e| AppError::Client(format!("{:?}", e)))?;

    let plaintext = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|e| AppError::Client(format!("解密失败: {:?}", e)))?;

    String::from_utf8(plaintext.to_vec())
        .map_err(|e| AppError::Client(format!("解密结果不是有效 UTF-8: {e}")))
}

fn engine() -> base64::engine::general_purpose::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}
