//! OpenClaw Gateway WebSocket Client
//!
//! Manages WebSocket connection to OpenClaw Gateway, handling:
//! - Connection lifecycle (connect, reconnect, disconnect)
//! - JSON-RPC message framing
//! - Request/response correlation
//! - Event subscription and distribution

use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info, warn};

use crate::adapter::frame::{GatewayEvent, GatewayRequest, GatewayResponse};
use crate::adapter::openclaw::types::{GatewayCapabilities, OpenClawConfig, OpenClawError};
use crate::adapter::transport::Transport;
use crate::error::{AppError, AppResult};

/// Pending request tracking
type PendingRequest = oneshot::Sender<Result<Value, OpenClawError>>;

/// Event stream type alias
type EventStream = Pin<Box<dyn futures::Stream<Item = GatewayEvent> + Send>>;

/// OpenClaw Gateway WebSocket client
pub struct OpenClawClient {
    config: OpenClawConfig,
    capabilities: Arc<RwLock<Option<GatewayCapabilities>>>,
    pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
    /// Broadcast channel for events (allows multiple subscribers)
    event_tx: broadcast::Sender<GatewayEvent>,
    connected: Arc<AtomicBool>,
    request_id: Arc<AtomicU64>,
    /// Channel for sending messages to the gateway
    send_tx: Option<mpsc::UnboundedSender<Value>>,
    /// Stop signal for background tasks
    stop_tx: Option<mpsc::Sender<()>>,
}

impl OpenClawClient {
    /// Create a new OpenClaw client with the given configuration
    pub fn new(config: OpenClawConfig) -> Self {
        let (event_tx, _) = broadcast::channel(256);

        Self {
            config,
            capabilities: Arc::new(RwLock::new(None)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            connected: Arc::new(AtomicBool::new(false)),
            request_id: Arc::new(AtomicU64::new(0)),
            send_tx: None,
            stop_tx: None,
        }
    }

    /// Create a new OpenClaw client with default configuration
    pub fn with_url(url: impl Into<String>) -> Self {
        let mut config = OpenClawConfig::default();
        config.url = url.into();
        Self::new(config)
    }

    /// Get the next request ID
    fn next_request_id(&self) -> String {
        format!("req-{}", self.request_id.fetch_add(1, Ordering::SeqCst))
    }

    /// Connect to the OpenClaw Gateway
    pub async fn connect(&mut self) -> AppResult<()> {
        let url = self.config.url.clone();
        info!("Connecting to OpenClaw Gateway: {}", url);

        // Establish WebSocket connection (use string directly for IntoClientRequest)
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| AppError::Transport(format!("WebSocket connection failed: {}", e)))?;

        info!("WebSocket connection established to {}", url);
        self.connected.store(true, Ordering::SeqCst);

        // Split into sink and stream
        let (ws_sink, ws_stream) = ws_stream.split();

        // Create channels for message handling
        let (send_tx, send_rx) = mpsc::unbounded_channel::<Value>();
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

        // Update self with new channels
        self.send_tx = Some(send_tx.clone());
        self.stop_tx = Some(stop_tx);

        // Shared state for tasks
        let pending = self.pending_requests.clone();
        let connected = self.connected.clone();
        let capabilities = self.capabilities.clone();
        let event_tx = self.event_tx.clone();

        // Spawn message sender task
        let connected_clone = connected.clone();
        tokio::spawn(async move {
            let mut send_rx = send_rx;
            let mut ws_sink = ws_sink;
            while let Some(msg) = send_rx.recv().await {
                if !connected_clone.load(Ordering::SeqCst) {
                    break;
                }
                let json = serde_json::to_string(&msg).unwrap_or_default();
                if let Err(e) = ws_sink.send(WsMessage::Text(json.into())).await {
                    error!("Failed to send WebSocket message: {}", e);
                    break;
                }
            }
            debug!("Message sender task stopped");
        });

        // Spawn message receiver task
        tokio::spawn(async move {
            let mut ws_stream = ws_stream;

            loop {
                tokio::select! {
                    _ = stop_rx.recv() => {
                        debug!("Stop signal received, shutting down receiver");
                        break;
                    }

                    msg = ws_stream.next() => {
                        match msg {
                            Some(Ok(WsMessage::Text(text))) => {
                                let text = text.to_string();
                                if let Err(e) = Self::handle_incoming_message(
                                    &text,
                                    &pending,
                                    &event_tx,
                                    &capabilities,
                                ).await {
                                    error!("Failed to handle incoming message: {}", e);
                                }
                            }
                            Some(Ok(WsMessage::Ping(data))) => {
                                debug!("Received ping, sending pong");
                                let _ = data;
                            }
                            Some(Ok(WsMessage::Close(frame))) => {
                                warn!("Gateway closed connection: {:?}", frame);
                                connected.store(false, Ordering::SeqCst);
                                break;
                            }
                            Some(Ok(WsMessage::Pong(_))) => {
                                debug!("Received pong");
                            }
                            Some(Err(e)) => {
                                error!("WebSocket error: {}", e);
                                connected.store(false, Ordering::SeqCst);
                                break;
                            }
                            None => {
                                warn!("WebSocket stream ended");
                                connected.store(false, Ordering::SeqCst);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Clean up pending requests
            let mut pending_guard = pending.write().await;
            for (_, tx) in pending_guard.drain() {
                let _ = tx.send(Err(OpenClawError::ConnectionClosed));
            }

            debug!("Message receiver task stopped");
        });

        // Perform handshake
        self.handshake().await?;

        Ok(())
    }

    /// Perform initial handshake with gateway
    async fn handshake(&self) -> AppResult<()> {
        debug!("Performing gateway handshake");

        // Request capabilities
        let result = self.send_request("gateway.info", Value::Null).await?;

        let caps: GatewayCapabilities = serde_json::from_value(result)
            .unwrap_or_else(|_| GatewayCapabilities::default());

        info!(
            "Gateway capabilities: version={}, features={:?}",
            caps.version, caps.features
        );

        let mut capabilities = self.capabilities.write().await;
        *capabilities = Some(caps);

        Ok(())
    }

    /// Handle incoming WebSocket message
    async fn handle_incoming_message(
        text: &str,
        pending: &Arc<RwLock<HashMap<String, PendingRequest>>>,
        event_tx: &broadcast::Sender<GatewayEvent>,
        capabilities: &Arc<RwLock<Option<GatewayCapabilities>>>,
    ) -> AppResult<()> {
        let value: Value = serde_json::from_str(text)
            .map_err(|e| AppError::Transport(format!("Invalid JSON: {}", e)))?;

        let frame_type = value.get("type").and_then(|v| v.as_str()).unwrap_or("");

        match frame_type {
            "res" => {
                // Response to a request
                let response: GatewayResponse = serde_json::from_value(value)
                    .map_err(|e| AppError::Transport(format!("Invalid response: {}", e)))?;

                let mut pending_guard = pending.write().await;
                if let Some(tx) = pending_guard.remove(&response.id) {
                    if response.ok {
                        if let Some(payload) = response.payload {
                            let _ = tx.send(Ok(payload));
                        } else {
                            let _ = tx.send(Err(OpenClawError::InvalidResponse("No payload".into())));
                        }
                    } else {
                        let err = response.error.unwrap_or_else(|| {
                            crate::adapter::frame::GatewayError {
                                code: None,
                                message: "Unknown error".into(),
                                data: None,
                            }
                        });
                        let _ = tx.send(Err(OpenClawError::GatewayError {
                            code: err.code,
                            message: err.message,
                        }));
                    }
                } else {
                    debug!("Received response for unknown request: {}", response.id);
                }
            }
            "event" => {
                // Event from gateway
                let event: GatewayEvent = serde_json::from_value(value)
                    .map_err(|e| AppError::Transport(format!("Invalid event: {}", e)))?;

                debug!("Received event: {} (seq={:?})", event.event, event.seq);

                // Handle special events
                if event.event == "gateway.ready" {
                    if let Ok(caps) =
                        serde_json::from_value::<GatewayCapabilities>(event.payload.clone())
                    {
                        let mut cap_guard = capabilities.write().await;
                        *cap_guard = Some(caps);
                    }
                }

                // Broadcast to all subscribers
                if let Err(e) = event_tx.send(event) {
                    debug!("No event subscribers to receive: {}", e);
                }
            }
            "req" => {
                // Request from gateway (shouldn't happen for our use case, but log it)
                warn!("Unexpected request from gateway");
            }
            _ => {
                warn!("Unknown frame type: {}", frame_type);
            }
        }

        Ok(())
    }

    /// Get gateway capabilities
    pub async fn capabilities(&self) -> Option<GatewayCapabilities> {
        self.capabilities.read().await.clone()
    }

    /// Subscribe to gateway events
    pub fn subscribe_events(&self) -> broadcast::Receiver<GatewayEvent> {
        self.event_tx.subscribe()
    }

    /// Check if connected to gateway
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Disconnect from gateway
    pub async fn disconnect(&mut self) -> AppResult<()> {
        info!("Disconnecting from OpenClaw Gateway");

        self.connected.store(false, Ordering::SeqCst);

        // Send stop signal
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(()).await;
        }

        self.send_tx = None;

        Ok(())
    }

    /// Get the gateway URL
    pub fn url(&self) -> &str {
        &self.config.url
    }
}

#[async_trait]
impl Transport for OpenClawClient {
    async fn send_request(&self, method: &str, params: Value) -> AppResult<Value> {
        if !self.is_connected() {
            return Err(AppError::Transport("Not connected to gateway".into()));
        }

        let send_tx = self
            .send_tx
            .as_ref()
            .ok_or_else(|| AppError::Transport("No send channel available".into()))?;

        let id = self.next_request_id();
        let mut request = GatewayRequest::new(method, params);
        request.id = id.clone();

        // Create response channel
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(request.id.clone(), tx);
        }

        // Send request
        let request_value = serde_json::to_value(&request)
            .map_err(|e| AppError::Transport(format!("Failed to serialize request: {}", e)))?;

        send_tx
            .send(request_value)
            .map_err(|e| AppError::Transport(format!("Failed to send request: {}", e)))?;

        // Wait for response with timeout
        let timeout = Duration::from_millis(self.config.request_timeout_ms);
        let result = tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| {
                // Remove pending request on timeout
                let pending = self.pending_requests.clone();
                let id = request.id.clone();
                tokio::spawn(async move {
                    let mut pending = pending.write().await;
                    pending.remove(&id);
                });
                AppError::Transport("Request timed out".into())
            })?
            .map_err(|_| AppError::Transport("Response channel closed".into()))?
            .map_err(|e| AppError::Transport(e.to_string()))?;

        Ok(result)
    }

    async fn subscribe_events(&self, _filter: &str) -> AppResult<EventStream> {
        // Create a receiver from the broadcast channel
        let mut receiver = self.event_tx.subscribe();

        // Convert to stream
        Ok(Box::pin(async_stream::stream! {
            loop {
                match receiver.recv().await {
                    Ok(event) => yield event,
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Event receiver lagged by {} messages", n);
                        // Continue receiving
                    }
                }
            }
        }))
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    async fn disconnect(&self) -> AppResult<()> {
        self.connected.store(false, Ordering::SeqCst);
        Ok(())
    }
}
