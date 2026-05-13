use std::path::PathBuf;

use crate::config::AppConfig;
use crate::error::{AppError, AppResult};

/// 默认路径在 data_dir 的 AfterDone
pub fn config_dir() -> AppResult<PathBuf> {
    let base = dirs::data_dir().ok_or_else(|| {
        AppError::Config("Cannot determine application data directory".to_string())
    })?;
    Ok(base.join("AfterDone"))
}

pub fn config_file_path() -> AppResult<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn database_path(config: &AppConfig) -> AppBuf {
    if config.app.database_path.is_empty() {
        Ok(config_dir()?.join("after-done.db"))
    } else {
        Ok(PathBuf::from(&config.app.database_path))
    }
}

type AppBuf = AppResult<PathBuf>;

pub fn load_config() -> AppResult<AppConfig> {
    let path = config_file_path()?;
    if !path.exists() {
        let config = AppConfig::default();
        save_config(&config)?;
        return Ok(config);
    }

    let content = std::fs::read_to_string(&path)?;
    let config: AppConfig = toml::from_str(&content)?;

    let config = apply_env_overrides(config);
    Ok(config)
}

pub fn save_config(config: &AppConfig) -> AppResult<()> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir)?;

    let content = toml::to_string_pretty(config)?;
    let path = config_file_path()?;
    std::fs::write(path, content)?;

    Ok(())
}

/// 环境变量覆盖至 config
fn apply_env_overrides(mut config: AppConfig) -> AppConfig {
    if let Ok(val) = std::env::var("RYG_OPENCLAW_URL") {
        config.openclaw.gateway_url = val;
    }
    if let Ok(val) = std::env::var("RYG_OPENCLAW_AUTH_TOKEN") {
        config.openclaw.auth_token = val;
    }
    if let Ok(val) = std::env::var("RYG_LLM_API_KEY") {
        config.llm.api_key = val;
    }
    if let Ok(val) = std::env::var("RYG_LLM_BASE_URL") {
        config.llm.base_url = val;
    }
    if let Ok(val) = std::env::var("RYG_LLM_MODEL") {
        config.llm.model = val;
    }
    if let Ok(val) = std::env::var("RYG_LOG_LEVEL") {
        config.app.log_level = val;
    }
    config
}
