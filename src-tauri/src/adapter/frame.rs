use serde::{Deserialize, Serialize};

/// JSON-RPC request frame (matches OpenClaw Gateway protocol)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayRequest {
    #[serde(rename = "type")]
    pub frame_type: String, // always "req"
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

impl GatewayRequest {
    pub fn new(method: &str, params: serde_json::Value) -> Self {
        Self {
            frame_type: "req".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            method: method.to_string(),
            params,
        }
    }
}

/// JSON-RPC response frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayResponse {
    #[serde(rename = "type")]
    pub frame_type: String, // always "res"
    pub id: String,
    pub ok: bool,
    pub payload: Option<serde_json::Value>,
    pub error: Option<GatewayError>,
}

/// JSON-RPC error in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayError {
    pub code: Option<i32>,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC event frame (server-pushed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayEvent {
    #[serde(rename = "type")]
    pub frame_type: String, // always "event"
    pub event: String,
    pub payload: serde_json::Value,
    pub seq: Option<u64>,
    pub state_version: Option<u64>,
}

/// Unified frame type for decoding
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GatewayFrame {
    #[serde(rename = "req")]
    Request(GatewayRequest),
    #[serde(rename = "res")]
    Response(GatewayResponse),
    #[serde(rename = "event")]
    Event(GatewayEvent),
}
