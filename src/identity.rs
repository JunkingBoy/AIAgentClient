use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto;
use crate::error::AppResult;

/// 客户端身份信息
///
/// 首次绑定时生成，AES 加密后持久化到 `{data_dir}/identity.json`。
/// 后续启动先拿密钥，再解密读取。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub client_id: String,
    pub email: String,
    pub bound_at: String,
}

/// 加密存储的包装结构
#[derive(Serialize, Deserialize)]
struct EncryptedPayload {
    encrypted: String,
}

impl Identity {
    /// 生成 UUID v4 作为 client_id
    pub fn generate_client_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// 快速判断是否已绑定（identity 文件是否存在）
    pub fn is_bound(data_dir: &Path) -> bool {
        Self::path(data_dir).exists()
    }

    fn path(data_dir: &Path) -> std::path::PathBuf {
        data_dir.join("identity.json")
    }

    /// 加密保存身份信息到 `{data_dir}/identity.json`
    ///
    /// 将自身序列化为 JSON → AES-128-CBC 加密 → base64 → 写入文件
    pub fn save_encrypted(&self, data_dir: &Path, aes_key: &[u8]) -> AppResult<()> {
        let plaintext = serde_json::to_string(self)?;
        let encrypted = crypto::encrypt(aes_key, &plaintext)?;
        let payload = EncryptedPayload { encrypted };
        std::fs::write(Self::path(data_dir), serde_json::to_string(&payload)?)?;
        Ok(())
    }

    /// 从 `{data_dir}/identity.json` 解密加载身份信息
    pub fn load_encrypted(data_dir: &Path, aes_key: &[u8]) -> AppResult<Option<Self>> {
        let path = Self::path(data_dir);
        if !path.exists() {
            return Ok(None);
        }
        // 读取加密载荷 → 提取 base64 字符串 → 解密 → 反序列化
        let content = std::fs::read_to_string(path)?;
        // 兼容旧版明文格式（直接 JSON 解析）
        if let Ok(identity) = serde_json::from_str::<Self>(&content) {
            return Ok(Some(identity));
        }
        // 新版加密格式
        let payload: EncryptedPayload = serde_json::from_str(&content)?;
        let json_str = crypto::decrypt(aes_key, &payload.encrypted)?;
        let identity = serde_json::from_str(&json_str)?;
        Ok(Some(identity))
    }
}
