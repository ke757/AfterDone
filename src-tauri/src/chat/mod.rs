//! Chat Module
//!
//! Manages chat messages (对话消息) for goals.
//! Contains the Message model and MessagesRepo for CRUD operations.

pub mod types;
pub mod repo;

pub use types::*;
pub use repo::MessagesRepo;
