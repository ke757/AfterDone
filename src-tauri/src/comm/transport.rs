use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;
use std::pin::Pin;

use crate::comm::frame::GatewayEvent;
use crate::error::AppResult;

/// Transport trait: the pluggable communication seam.
/// Agents use this to talk to Harness Agents — they never know
/// whether it's a real OpenClaw instance or an in-process mock.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a JSON-RPC request and wait for the response.
    async fn send_request(&self, method: &str, params: Value) -> AppResult<Value>;

    /// Subscribe to events matching the given filter pattern.
    async fn subscribe_events(
        &self,
        filter: &str,
    ) -> AppResult<Pin<Box<dyn Stream<Item = GatewayEvent> + Send>>>;

    /// Check if the transport is currently connected.
    fn is_connected(&self) -> bool;

    /// Gracefully disconnect.
    async fn disconnect(&self) -> AppResult<()>;
}

/// Type alias for the boxed stream returned by subscribe_events
pub type EventStream = Pin<Box<dyn Stream<Item = GatewayEvent> + Send>>;
