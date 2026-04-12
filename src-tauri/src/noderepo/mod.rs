//! NodeRepo Module
//!
//! Manages NodeSpaces (workspaces for goals) and WorkNodes (milestone nodes).
//! Provides resource management for GOAL.md, PLAN.md, BUG.md, USER_MANUAL.md, CONCLUSION.md.
//!
//! Key concepts:
//! - NodeSpace: A workspace that corresponds 1:1 with a Goal
//! - WorkNode: A milestone node in the execution tree
//! - CurrentNode: The agent's current position in the tree (no need for agent to remember)

pub mod types;
pub mod repo;
pub mod ops;

pub use types::*;
pub use repo::{NodeSpaceRepo, WorkNodeRepo, GeneratorTaskRepo};
pub use ops::NodeRepo;
