use crate::crypto;
use crate::enums::StandardHttpRequestEnum;
use crate::error::{AppError, AppResult};
use crate::identity::Identity;
use crate::transport::request::StandardMetaRequest;

/// 在终端提示用户输入邮箱
///
/// 读取一行输入，自动 trim 空白字符。
/// 空输入或 EOF 返回错误。
pub fn prompt_email() -> AppResult<String> {
    use std::io::{stdin, stdout, Write};

    print!("请输入您的邮箱地址以绑定此客户端：");
    stdout().flush().map_err(AppError::Io)?;

    let mut input = String::new();
    stdin().read_line(&mut input).map_err(AppError::Io)?;

    let email = input.trim().to_string();
    if email.is_empty() {
        return Err(AppError::UserCancel);
    }
    Ok(email)
}

/// 校验邮箱格式（基础检查）
fn validate_email(email: &str) -> AppResult<()> {
    if email.is_empty() {
        return Err(AppError::InvalidInput("邮箱不能为空".into()));
    }
    if !email.contains('@') || !email.contains('.') {
        return Err(AppError::InvalidInput("邮箱格式不正确".into()));
    }
    Ok(())
}

/// 发送绑定请求到服务端
///
/// `client_id` 和 `email` 分别用 AES-128-CBC 加密后发送。
async fn bind_to_server(client: &StandardMetaRequest, aes_key: &[u8], client_id: &str, email: &str) -> AppResult<()> {
    let enc_client_id = crypto::encrypt(aes_key, client_id)?;
    let enc_email = crypto::encrypt(aes_key, email)?;

    eprintln!("加密后 client_id: {enc_client_id}");
    eprintln!("加密后 email: {enc_email}");

    let _: Option<serde_json::Value> = client
        .send(StandardHttpRequestEnum::UserBind {
            client_id: enc_client_id,
            email: enc_email,
        })
        .await?;

    Ok(())
}

/// 运行完整的绑定流程：终端提示 → 加密 → 请求服务端 → 返回 Identity
///
/// `aes_key` 由调用方传入（来自 `service::key::get_cached_key`）。
/// 调用方负责将返回的 Identity 加密保存到磁盘。
pub async fn run_binding_flow(client: &StandardMetaRequest, aes_key: &[u8]) -> AppResult<Identity> {
    let email = prompt_email()?;
    validate_email(&email)?;
    let client_id = Identity::generate_client_id();
    bind_to_server(client, aes_key, &client_id, &email).await?;

    let bound_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    Ok(Identity {
        client_id,
        email,
        bound_at,
    })
}
