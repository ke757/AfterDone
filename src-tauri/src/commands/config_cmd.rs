use std::path::Path;
use std::time::Instant;
use tauri::State;
use serde::{Deserialize, Serialize};

use crate::adapter::openclaw::{OpenClawClient, OpenClawConfig as GatewayConfig};
use crate::config::{ConnectionTestResult, LlmConfig, OpenClawConfig};
use crate::error::AppResult;
use crate::llm::LlmProvider;
use crate::llm::RigProvider;
use crate::state::AppState;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPathInfo {
    pub path: String,
    pub name: String,
}

/// 读应用的数据仓库路径
#[tauri::command]
pub async fn config_get_data_path(
    state: State<'_, AppState>,
) -> AppResult<DataPathInfo> {
    let config = state.config.read().await;
    let db_path = config.app.database_path.clone();
    if db_path.is_empty() {
        Ok(DataPathInfo {
            path: String::new(),
            name: String::new(),
        })
    } else {
        let path = Path::new(&db_path);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        Ok(DataPathInfo { path: db_path, name })
    }
}

/// 设置应用的数据仓库路径
#[tauri::command]
pub async fn config_set_data_path(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<DataPathInfo> {
    let mut app_config = state.config.write().await;
    app_config.app.database_path = path.clone();
    crate::config::save_config(&app_config)?;
    drop(app_config);

    let path_obj = Path::new(&path);
    let name = path_obj
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    Ok(DataPathInfo { path, name })
}

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
pub async fn config_test_openclaw(
    state: State<'_, AppState>,
) -> AppResult<ConnectionTestResult> {
    let config = state.config.read().await;
    let url = config.openclaw.gateway_url.clone();
    let timeout_secs = config.openclaw.request_timeout_secs;
    drop(config);

    let start = Instant::now();

    // Create client with the configured URL
    let client_config = GatewayConfig {
        url: url.clone(),
        request_timeout_ms: timeout_secs * 1000,
        ..Default::default()
    };

    let mut client = OpenClawClient::new(client_config);

    // Attempt to connect
    match client.connect().await {
        Ok(()) => {
            let latency = start.elapsed().as_millis() as u64;

            // Get capabilities for additional info
            let caps = client.capabilities().await;
            let version_info = caps
                .map(|c| format!(" (version: {})", c.version))
                .unwrap_or_default();

            // Disconnect
            let _ = client.disconnect().await;

            Ok(ConnectionTestResult {
                success: true,
                message: format!("Successfully connected to OpenClaw Gateway{}", version_info),
                latency_ms: Some(latency),
            })
        }
        Err(e) => {
            let latency = start.elapsed().as_millis() as u64;
            Ok(ConnectionTestResult {
                success: false,
                message: format!("Failed to connect: {}", e),
                latency_ms: Some(latency),
            })
        }
    }
}

#[tauri::command]
pub async fn config_test_llm(
    state: State<'_, AppState>,
) -> AppResult<ConnectionTestResult> {
    let config = state.config.read().await;
    let llm_config = config.llm.clone();
    drop(config);

    if llm_config.api_key.is_empty() {
        return Ok(ConnectionTestResult {
            success: false,
            message: "API key is not configured".to_string(),
            latency_ms: None,
        });
    }

    let start = Instant::now();

    // Create provider and test with a simple completion
    let provider = RigProvider::new(llm_config);

    let response = provider.complete(
        "You are a test assistant. Reply with 'OK'.",
        "Test connection. Reply with just 'OK'.",
        &[],
    ).await;

    let latency = start.elapsed().as_millis() as u64;

    match response {
        Ok(text) => Ok(ConnectionTestResult {
            success: true,
            message: format!("LLM connection successful. Response: {}", text.trim()),
            latency_ms: Some(latency),
        }),
        Err(e) => Ok(ConnectionTestResult {
            success: false,
            message: format!("LLM connection failed: {}", e),
            latency_ms: Some(latency),
        }),
    }
}
