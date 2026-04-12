//! Mock Transport for testing and development
//!
//! Provides a mock implementation of the Transport trait that simulates
//! Generator responses without requiring a real HarnessAgent connection.

use async_trait::async_trait;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::adapter::frame::GatewayEvent;
use crate::adapter::transport::Transport;
use crate::error::{AppError, AppResult};

/// Mock transport that simulates generator responses
pub struct MockTransport {
    /// Simulated task states
    tasks: Arc<RwLock<HashMap<String, MockTask>>>,
    /// Whether connected
    connected: bool,
}

/// Mock task state
#[derive(Debug, Clone)]
struct MockTask {
    status: String,
    progress: u8,
    current_step: Option<String>,
    result: Option<serde_json::Value>,
    error: Option<String>,
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl MockTransport {
    /// Create a new mock transport
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            connected: true,
        }
    }

    /// Handle generator.execute
    async fn handle_execute(&self, params: serde_json::Value) -> AppResult<serde_json::Value> {
        let task_id = params["task_id"].as_str().unwrap_or("unknown").to_string();
        
        // Create mock task
        let task = MockTask {
            status: "running".to_string(),
            progress: 0,
            current_step: Some("Initializing...".to_string()),
            result: None,
            error: None,
        };
        
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), task);
        }

        // Clone task_id for the spawned task
        let task_id_clone = task_id.clone();
        
        // Simulate async work
        let tasks = self.tasks.clone();
        tokio::spawn(async move {
            // Simulate progress
            for i in 1..=5 {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let mut tasks = tasks.write().await;
                if let Some(task) = tasks.get_mut(&task_id_clone) {
                    task.progress = i * 20;
                    task.current_step = Some(format!("Processing step {}/5...", i));
                }
            }
            
            // Complete
            let mut tasks = tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id_clone) {
                task.status = "completed".to_string();
                task.progress = 100;
                task.current_step = Some("Done".to_string());
                task.result = Some(serde_json::json!({
                    "summary": "Mock generation completed successfully",
                    "user_manual": "# User Manual\n\nThis is a mock user manual generated for testing purposes.\n\n## Usage\n\n1. Step one\n2. Step two\n3. Step three\n",
                    "skills": [
                        {"name": "mock_skill", "description": "A mock skill for testing"}
                    ],
                    "files_modified": ["src/main.rs", "README.md"]
                }));
            }
        });

        Ok(serde_json::json!({
            "task_id": task_id,
            "accepted": true,
            "message": "Task accepted by mock generator"
        }))
    }

    /// Handle generator.status
    async fn handle_status(&self, params: serde_json::Value) -> AppResult<serde_json::Value> {
        let task_id = params["task_id"].as_str().unwrap_or("unknown");
        
        let tasks = self.tasks.read().await;
        let task = tasks.get(task_id);
        
        let response = if let Some(task) = task {
            serde_json::json!({
                "task_id": task_id,
                "status": task.status,
                "progress": task.progress,
                "current_step": task.current_step,
                "result": task.result,
                "error": task.error
            })
        } else {
            serde_json::json!({
                "task_id": task_id,
                "status": "pending",
                "progress": 0,
                "current_step": null,
                "result": null,
                "error": "Task not found"
            })
        };
        
        Ok(response)
    }

    /// Handle generator.cancel
    async fn handle_cancel(&self, params: serde_json::Value) -> AppResult<serde_json::Value> {
        let task_id = params["task_id"].as_str().unwrap_or("unknown");
        
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = "cancelled".to_string();
        }
        
        Ok(serde_json::json!({
            "cancelled": true,
            "message": "Task cancelled"
        }))
    }

    /// Handle generator.listSkills
    async fn handle_list_skills(&self) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({
            "skills": [
                {
                    "name": "create_file",
                    "description": "Create a new file",
                    "parameters_schema": null
                },
                {
                    "name": "edit_file",
                    "description": "Edit an existing file",
                    "parameters_schema": null
                }
            ]
        }))
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn send_request(&self, method: &str, params: serde_json::Value) -> AppResult<serde_json::Value> {
        if !self.connected {
            return Err(AppError::Transport("Not connected".to_string()));
        }

        match method {
            "generator.execute" => self.handle_execute(params).await,
            "generator.status" => self.handle_status(params).await,
            "generator.cancel" => self.handle_cancel(params).await,
            "generator.listSkills" => self.handle_list_skills().await,
            "gateway.info" => Ok(serde_json::json!({
                "version": "1.0.0-mock",
                "features": ["generator", "skills"]
            })),
            _ => Err(AppError::Transport(format!("Unknown method: {}", method))),
        }
    }

    async fn subscribe_events(
        &self,
        _filter: &str,
    ) -> AppResult<Pin<Box<dyn Stream<Item = GatewayEvent> + Send>>> {
        // Return an empty stream for mock
        Ok(Box::pin(futures::stream::empty()))
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&self) -> AppResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_transport_execute() {
        let transport = MockTransport::new();
        
        let result = transport.send_request("generator.execute", serde_json::json!({
            "task_id": "test-1",
            "nodespace_id": "ns-1",
            "specification": {
                "project_background": "Test",
                "module_breakdown": [],
                "acceptance_criteria": []
            }
        })).await.unwrap();
        
        assert_eq!(result["accepted"], true);
    }

    #[tokio::test]
    async fn test_mock_transport_status() {
        let transport = MockTransport::new();
        
        // First execute
        let _ = transport.send_request("generator.execute", serde_json::json!({
            "task_id": "test-2",
            "nodespace_id": "ns-1",
            "specification": {
                "project_background": "Test",
                "module_breakdown": [],
                "acceptance_criteria": []
            }
        })).await.unwrap();
        
        // Check status
        let status = transport.send_request("generator.status", serde_json::json!({
            "task_id": "test-2"
        })).await.unwrap();
        
        assert_eq!(status["status"], "running");
    }
}
