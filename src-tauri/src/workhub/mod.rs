//! WorkHub Module
//!
//! Manages the full lifecycle of goals and their associated workspaces.
//! Contains all data models, repositories, and business operations for:
//! - Goals (目标)
//! - Milestones (里程碑)
//! - Skills (产物)
//! - AgentLogs (智能体日志)
//! - WorkSpaces / WorkNodes / GeneratorTasks (工作区)

pub mod types;
pub mod repos;
pub mod ops;

pub use types::*;
pub use repos::{GoalsRepo, MilestonesRepo, SkillsRepo, AgentLogsRepo, WorkSpaceRepo, WorkNodeRepo, GeneratorTaskRepo};
pub use ops::WorkHub;
