use crate::config::AppConfig;
use crate::crypto;
use crate::dto::StandardHttpResponse;
use crate::error::{AppError, AppResult};
use crate::identity::Identity;

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
///
/// POST `{server_url}/user/bind`
/// ```json
/// {
///   "client_id": "<base64(iv + ciphertext)>",
///   "email": "<base64(iv + ciphertext)>"
/// }
/// ```
async fn bind_to_server(server_url: &str, aes_key: &[u8], client_id: &str, email: &str) -> AppResult<()> {
    let url = format!("{}/user/bind", server_url.trim_end_matches('/'));

    let enc_client_id = crypto::encrypt(aes_key, client_id)?;
    let enc_email = crypto::encrypt(aes_key, email)?;

    let body = serde_json::json!({
        "client_id": enc_client_id,
        "email": enc_email,
    });

    eprintln!("正在发送绑定请求到 {url}");
    eprintln!("加密后 client_id: {enc_client_id}");
    eprintln!("加密后 email: {enc_email}");

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send().await?;

    // 检查 HTTP 状态码
    let http_status = resp.status();
    if !http_status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::WebSocket(format!(
            "HTTP {}: {}",
            http_status, text
        )));
    }

    // 解析业务信封
    let body: StandardHttpResponse<serde_json::Value> = resp.json().await?;
    eprintln!("服务端绑定响应: code={}, msg={}", body.code, body.msg);

    if !body.is_success() {
        return Err(AppError::BusinessError(body.code, body.msg));
    }

    Ok(())
}

/// 运行完整的绑定流程：终端提示 → 加密 → 请求服务端 → 返回 Identity
///
/// `aes_key` 由调用方传入（来自 `crypto::get_cached_key`）。
/// 调用方负责将返回的 Identity 加密保存到磁盘。
pub async fn run_binding_flow(config: &AppConfig, aes_key: &[u8]) -> AppResult<Identity> {

    // 2. 提示输入邮箱
    let email = prompt_email()?;

    // 3. 校验邮箱
    validate_email(&email)?;

    // 4. 生成 client_id
    let client_id = Identity::generate_client_id();

    // 5. 加密后发送绑定请求
    bind_to_server(&config.server_url, aes_key, &client_id, &email).await?;

    // 6. 构造身份信息（记录 Unix 时间戳）
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
