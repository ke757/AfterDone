use std::sync::Arc;
use tauri::AppHandle;

use crate::agents::AgentSupervisor;
use crate::agents::runtime::AgentRuntime;
use crate::adapter::{MockTransport, Transport};
use crate::config::AppConfig;
use crate::db::DatabasePool;
use crate::events::EventBridge;
use crate::llm::{LlmProvider, RigProvider};
use crate::session::CellManager;

pub struct AppState {
    pub db: DatabasePool,
    pub config: Arc<tokio::sync::RwLock<AppConfig>>,
    pub supervisor: Arc<AgentSupervisor>,
    pub cell_manager: Arc<CellManager>,
    pub emitter: EventBridge,
}

impl AppState {
    pub fn new(
        db: DatabasePool,
        config: AppConfig,
        app_handle: AppHandle,
    ) -> Self {
        // Create event emitter
        let emitter = EventBridge::new(app_handle);

        // Create gen transport
        let transport: Arc<dyn Transport> = Arc::new(MockTransport::new());

        // Create LLM provider
        let llm: Arc<dyn LlmProvider> = Arc::new(RigProvider::new(config.llm.clone()));
        let cell_manager = Arc::new(CellManager::new());

        // Create AgentRuntime（持有 transport, llm, sessions, emitter）
        let runtime = Arc::new(AgentRuntime::new(
            db.clone(),
            transport,
            llm,
            emitter.clone(),
            cell_manager.clone(),
        ));

        // Create AgentSupervisor（仅持有 runtime + tasks）
        let supervisor = Arc::new(AgentSupervisor::new(runtime));

        Self {
            db,
            config: Arc::new(tokio::sync::RwLock::new(config)),
            supervisor,
            cell_manager,
            emitter,
        }
    }
}