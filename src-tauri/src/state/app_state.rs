use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::db::DatabasePool;

/// Application state shared across all Tauri commands.
/// Wrapped in Arc<RwLock<>> for safe concurrent access.
pub struct AppState {
    pub db: DatabasePool,
    pub config: Arc<RwLock<AppConfig>>,
}

impl AppState {
    pub fn new(db: DatabasePool, config: AppConfig) -> Self {
        Self {
            db,
            config: Arc::new(RwLock::new(config)),
        }
    }
}
