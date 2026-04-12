//! OpenClaw Gateway Integration
//!
//! This module provides a WebSocket client for connecting to OpenClaw Gateway,
//! implementing the Transport trait for seamless integration with the agent system.

mod client;
mod types;

pub use client::OpenClawClient;
pub use types::{GatewayCapabilities, OpenClawConfig, OpenClawError};
