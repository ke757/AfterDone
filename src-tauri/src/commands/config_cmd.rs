use tauri::State;

use crate::config::{AppConfig, ConnectionTestResult, LlmConfig, OpenClawConfig};
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub async fn config_get_openclaw(
    state: State<'_, AppState>,
) -> AppResult<OpenClawConfig> {
    let config = state.config.read().await;
    Ok(config.openclaw.clone())
}

#[tauri::command]
pub async fn config_set_openclaw(
    state: State<'_, AppState>,
    config: OpenClawConfig,
) -> AppResult<OpenClawConfig> {
    let mut app_config = state.config.write().await;
    app_config.openclaw = config;
    crate::config::save_config(&app_config)?;
    Ok(app_config.openclaw.clone())
}

#[tauri::command]
pub async fn config_get_llm(
    state: State<'_, AppState>,
) -> AppResult<LlmConfig> {
    let config = state.config.read().await;
    Ok(config.llm.clone())
}

#[tauri::command]
pub async fn config_set_llm(
    state: State<'_, AppState>,
    config: LlmConfig,
) -> AppResult<LlmConfig> {
    let mut app_config = state.config.write().await;
    app_config.llm = config;
    crate::config::save_config(&app_config)?;
    Ok(app_config.llm.clone())
}

#[tauri::command]
pub async fn config_test_openclaw() -> AppResult<ConnectionTestResult> {
    // TODO: Implement actual connection test
    Ok(ConnectionTestResult {
        success: false,
        message: "Connection test not yet implemented".to_string(),
        latency_ms: None,
    })
}

#[tauri::command]
pub async fn config_test_llm() -> AppResult<ConnectionTestResult> {
    // TODO: Implement actual LLM connection test
    Ok(ConnectionTestResult {
        success: false,
        message: "Connection test not yet implemented".to_string(),
        latency_ms: None,
    })
}
