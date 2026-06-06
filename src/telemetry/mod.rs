mod appender;
mod formatter;

use std::panic;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::Layer;

use crate::config::AppConfig;
use crate::error::AppResult;
use crate::telemetry::appender::DailyRollingFile;
use crate::telemetry::formatter::AgentFormatter;

/// 初始化日志系统
///
/// 控制台 + 文件双输出，文件按日滚动。
/// 返回 `WorkerGuard`，析构前保证日志刷完，main 结束前不能 drop。
pub fn init(config: &AppConfig) -> AppResult<WorkerGuard> {
    std::fs::create_dir_all(&config.log_dir)?;

    let file_appender = DailyRollingFile::new(config.log_dir.clone(), "agent");
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let stdout_layer = Layer::default()
        .event_format(AgentFormatter)
        .with_writer(std::io::stdout);

    let file_layer = Layer::default()
        .event_format(AgentFormatter)
        .with_writer(non_blocking_file)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let (file, line) = panic_info
            .location()
            .map(|loc| (loc.file(), loc.line()))
            .unwrap_or(("<unknown>", 0));
        let payload = panic_info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| panic_info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "Unknown panic payload".to_string());

        tracing::error!("程序崩溃 ({}:{}): {}", file, line, payload);
        prev_hook(panic_info);
    }));

    Ok(guard)
}
