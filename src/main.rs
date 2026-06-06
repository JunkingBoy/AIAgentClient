use std::future::pending;

use ai_agent_client::binding;
use ai_agent_client::config::AppConfig;
use ai_agent_client::connection::Manager;
use ai_agent_client::key;
use ai_agent_client::error::AppResult;
use ai_agent_client::identity::Identity;
use ai_agent_client::instance_guard::{FileLockGuard, InstanceGuard};
use ai_agent_client::telemetry;
use ai_agent_client::transport::request::StandardMetaRequest;

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = AppConfig::load()?;
    std::fs::create_dir_all(&config.data_dir)?;

    // ★ 统一 HTTP 客户端（所有请求走这里）
    let client = StandardMetaRequest::new(&config.server_url);

    // ★ 提前获取密钥（后续加密 identity 和 WS 通信都需要）
    let aes_key = key::get_cached_key(&client).await?;

    // ★ 首次运行：引导用户输入邮箱并绑定身份（加密存储）
    if !Identity::is_bound(&config.data_dir) {
        eprintln!("首次运行，请绑定客户端身份");
        let identity = binding::run_binding_flow(&client, aes_key).await?;
        identity.save_encrypted(&config.data_dir, aes_key)?;
        eprintln!("绑定成功！邮箱: {}", identity.email);
    }

    let _lock = match FileLockGuard::try_lock(&config.data_dir.join("daemon.lock")) {
        Ok(lock) => lock,
        Err(e) => {
            eprintln!("错误: {e}");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            std::process::exit(1);
        }
    };

    let _guard = telemetry::init(&config)?;

    tracing::info!("ai-agent-daemon 启动成功");

    let manager = Manager::new(&config);
    manager.run().await?;

    pending::<()>().await;

    Ok(())
}
