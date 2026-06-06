use std::path::PathBuf;

use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub log_level: String,
    pub log_dir: PathBuf,
    pub data_dir: PathBuf,
    pub server_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let data_dir = default_data_dir();
        Self {
            log_level: "info".to_string(),
            log_dir: data_dir.join("logs"),
            data_dir,
            server_url: String::new(),
        }
    }
}

impl AppConfig {
    pub fn load() -> AppResult<Self> {
        // 优先加载 .env.dev（开发默认值），再尝试 .env（生产覆盖），最后 shell 环境变量最高优先级
        let _ = dotenvy::from_filename(".env.dev");
        let _ = dotenvy::dotenv();
        let mut config = AppConfig::default();

        if let Ok(val) = std::env::var("LOG_LEVEL") {
            config.log_level = val.to_lowercase();
        }
        if let Ok(val) = std::env::var("DATA_DIR") {
            if !val.is_empty() {
                config.log_dir = PathBuf::from(val).join("logs");
            }
        }
        if let Ok(val) = std::env::var("SERVER_URL") {
            if !val.is_empty() {
                config.server_url = val;
            }
        }

        Ok(config)
    }
}

fn default_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("ai_agent_client");
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return PathBuf::from(xdg).join("ai_agent_client");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("ai_agent_client");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("ai_agent_client");
        }
    }

    // 开发环境回退：当前目录下的 .data
    PathBuf::from(".data")
}
