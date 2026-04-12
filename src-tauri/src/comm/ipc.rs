use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde_json::{json, Value};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::comm::frame::{GatewayEvent, GatewayResponse};
use crate::comm::transport::{EventStream, Transport};
use crate::error::{AppError, AppResult};

/// In-process IPC transport using tokio mpsc channels.
/// Implements Transport for testing and local agent communication.
pub struct IpcTransport {
    request_tx: mpsc::UnboundedSender<(String, Value, mpsc::UnboundedSender<GatewayResponse>)>,
    event_rx: Arc<RwLock<mpsc::UnboundedReceiver<GatewayEvent>>>,
    connected: Arc<RwLock<bool>>,
}

impl IpcTransport {
    /// Create a pair of IpcTransport instances for bidirectional communication.
    pub fn channel() -> (Self, Self) {
        let (req_tx_a, req_rx_a) = mpsc::unbounded_channel();
        let (evt_tx_a, evt_rx_a) = mpsc::unbounded_channel();
        let (req_tx_b, req_rx_b) = mpsc::unbounded_channel();
        let (evt_tx_b, evt_rx_b) = mpsc::unbounded_channel();

        let a = IpcTransport {
            request_tx: req_tx_a,
            event_rx: Arc::new(RwLock::new(evt_rx_a)),
            connected: Arc::new(RwLock::new(true)),
        };

        let b = IpcTransport {
            request_tx: req_tx_b,
            event_rx: Arc::new(RwLock::new(evt_rx_b)),
            connected: Arc::new(RwLock::new(true)),
        };

        // Spawn handlers that forward requests as events
        Self::spawn_request_handler(req_rx_b, evt_tx_a);
        Self::spawn_request_handler(req_rx_a, evt_tx_b);

        (a, b)
    }

    /// Create a stub IpcTransport that echoes back responses.
    pub fn new_stub() -> Self {
        type RequestChannel = (String, Value, mpsc::UnboundedSender<GatewayResponse>);
        let (req_tx, mut req_rx): (mpsc::UnboundedSender<RequestChannel>, _) = mpsc::unbounded_channel();
        let (evt_tx, evt_rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some((method, params, reply_tx)) = req_rx.recv().await {
                let response = GatewayResponse {
                    frame_type: "res".to_string(),
                    id: uuid::Uuid::new_v4().to_string(),
                    ok: true,
                    payload: Some(json!({
                        "method": method,
                        "params": params,
                        "stub": true,
                    })),
                    error: None,
                };
                let _ = reply_tx.send(response);
            }
        });

        // Drop evt_tx to avoid leak — events only come from external sources
        drop(evt_tx);

        Self {
            request_tx: req_tx,
            event_rx: Arc::new(RwLock::new(evt_rx)),
            connected: Arc::new(RwLock::new(true)),
        }
    }

    fn spawn_request_handler(
        mut rx: mpsc::UnboundedReceiver<(String, Value, mpsc::UnboundedSender<GatewayResponse>)>,
        evt_tx: mpsc::UnboundedSender<GatewayEvent>,
    ) {
        tokio::spawn(async move {
            while let Some((method, params, reply_tx)) = rx.recv().await {
                // Forward as event
                let event = GatewayEvent {
                    frame_type: "event".to_string(),
                    event: format!("request.{}", method),
                    payload: params,
                    seq: None,
                    state_version: None,
                };
                let _ = evt_tx.send(event);

                // Echo a simple response
                let response = GatewayResponse {
                    frame_type: "res".to_string(),
                    id: uuid::Uuid::new_v4().to_string(),
                    ok: true,
                    payload: Some(json!({"echo": method})),
                    error: None,
                };
                let _ = reply_tx.send(response);
            }
        });
    }
}

#[async_trait]
impl Transport for IpcTransport {
    async fn send_request(&self, method: &str, params: Value) -> AppResult<Value> {
        let (reply_tx, mut reply_rx) = mpsc::unbounded_channel();

        self.request_tx
            .send((method.to_string(), params, reply_tx))
            .map_err(|e| AppError::Transport(format!("Failed to send IPC request: {}", e)))?;

        match reply_rx.recv().await {
            Some(resp) if resp.ok => resp.payload.ok_or_else(|| {
                AppError::Transport("Response has no payload".to_string())
            }),
            Some(resp) => Err(AppError::Transport(
                resp.error
                    .map(|e| e.message)
                    .unwrap_or_else(|| "Unknown IPC error".to_string()),
            )),
            None => Err(AppError::Transport("IPC channel closed".to_string())),
        }
    }

    async fn subscribe_events(&self, _filter: &str) -> AppResult<EventStream> {
        let rx = self.event_rx.clone();
        let stream = async_stream::stream! {
            let mut guard = rx.write().await;
            while let Some(event) = guard.recv().await {
                yield event;
            }
        };
        Ok(Box::pin(stream))
    }

    fn is_connected(&self) -> bool {
        *self.connected.blocking_read()
    }

    async fn disconnect(&self) -> AppResult<()> {
        let mut connected = self.connected.write().await;
        *connected = false;
        Ok(())
    }
}
