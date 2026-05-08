//! WorkHub Module
//!
//! Manages the full lifecycle of goals and their associated workspaces.
//! Contains all data models, repositories, and business operations for:
//! - Goals (目标)
//! - AgentLogs (智能体日志)
//! - WorkSpaces / WorkNodes / GeneratorTasks (工作区)

pub mod types;
pub mod repos;
pub mod ops;

pub use types::*;
pub use repos::{GoalsRepo, AgentLogsRepo, WorkSpaceRepo, WorkNodeRepo, GeneratorTaskRepo};
pub use ops::WorkHub;
