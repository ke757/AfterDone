//! OpenClaw-specific types and error definitions

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur when communicating with OpenClaw Gateway
#[derive(Debug, Error)]
pub enum OpenClawError {
    #[error("WebSocket connection error: {0}")]
    Connection(String),

    #[error("Failed to serialize message: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Failed to send message: {0}")]
    SendError(String),

    #[error("Connection closed unexpectedly")]
    ConnectionClosed,

    #[error("Request timed out")]
    Timeout,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Gateway error: code={code:?}, message={message}")]
    GatewayError {
        code: Option<i32>,
        message: String,
    },

    #[error("Not connected to gateway")]
    NotConnected,
}

/// Gateway message types for internal handling
#[derive(Debug, Clone)]
pub enum GatewayMessage {
    /// Incoming event from gateway
    Event(crate::adapter::frame::GatewayEvent),
    /// Response to a request
    Response(crate::adapter::frame::GatewayResponse),
}

/// OpenClaw Gateway capabilities discovered during handshake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayCapabilities {
    pub version: String,
    pub features: Vec<String>,
    pub agent_types: Vec<String>,
}

impl Default for GatewayCapabilities {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            features: vec![],
            agent_types: vec![],
        }
    }
}

/// Configuration for OpenClaw Gateway connection
#[derive(Debug, Clone)]
pub struct OpenClawConfig {
    pub url: String,
    pub reconnect_interval_ms: u64,
    pub request_timeout_ms: u64,
    pub max_reconnect_attempts: u32,
}

impl Default for OpenClawConfig {
    fn default() -> Self {
        Self {
            url: "ws://127.0.0.1:8080/gateway".to_string(),
            reconnect_interval_ms: 1000,
            request_timeout_ms: 30000,
            max_reconnect_attempts: 5,
        }
    }
}
