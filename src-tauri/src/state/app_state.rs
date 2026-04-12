use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::AppHandle;

use crate::agents::AgentSupervisor;
use crate::adapter::{MockTransport, Transport};
use crate::config::AppConfig;
use crate::db::DatabasePool;
use crate::events::EventBridge;
use crate::llm::{LlmProvider, RigProvider};

/// Application state shared across all Tauri commands.
/// Wrapped in Arc<RwLock<>> for safe concurrent access.
pub struct AppState {
    pub db: DatabasePool,
    pub config: Arc<RwLock<AppConfig>>,
    pub supervisor: Arc<AgentSupervisor>,
}

impl AppState {
    pub fn new(
        db: DatabasePool,
        config: AppConfig,
        app_handle: AppHandle,
    ) -> Self {
        // Create event emitter
        let emitter = EventBridge::new(app_handle);

        // Create transport (using MockTransport for development)
        let transport: Arc<dyn Transport> = Arc::new(MockTransport::new());

        // Create LLM provider
        let llm: Arc<dyn LlmProvider> = Arc::new(RigProvider::new(config.llm.clone()));

        // Create agent supervisor
        let supervisor = Arc::new(AgentSupervisor::new(
            db.clone(),
            transport,
            llm,
            emitter,
        ));

        Self {
            db,
            config: Arc::new(RwLock::new(config)),
            supervisor,
        }
    }
}
