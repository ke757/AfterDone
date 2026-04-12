use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use super::types::*;
use super::repo::{NodeSpaceRepo, WorkNodeRepo, GeneratorTaskRepo};

/// Core NodeRepo operations for agent tools
pub struct NodeRepo;

impl NodeRepo {
    // ==================== NodeSpace Operations ====================

    /// Initialize a NodeSpace for a goal
    pub async fn init_nodespace(db: &SqlitePool, goal_id: &str) -> AppResult<NodeSpace> {
        // Check if already exists
        if let Some(existing) = NodeSpaceRepo::get_by_goal_id(db, goal_id).await? {
            return Ok(existing);
        }

        NodeSpaceRepo::create(db, CreateNodeSpaceInput {
            goal_id: goal_id.to_string(),
            goal_md: None,
            plan_md: None,
        }).await
    }

    /// Get NodeSpace by goal_id
    pub async fn get_nodespace_by_goal(db: &SqlitePool, goal_id: &str) -> AppResult<Option<NodeSpace>> {
        NodeSpaceRepo::get_by_goal_id(db, goal_id).await
    }

    /// Get or create NodeSpace for a goal
    pub async fn get_or_create_nodespace(db: &SqlitePool, goal_id: &str) -> AppResult<NodeSpace> {
        match NodeSpaceRepo::get_by_goal_id(db, goal_id).await? {
            Some(ns) => Ok(ns),
            None => Self::init_nodespace(db, goal_id).await,
        }
    }

    // ==================== PLAN.md Operations ====================

    /// Store PLAN.md content
    pub async fn store_plan(db: &SqlitePool, nodespace_id: &str, content: &str) -> AppResult<()> {
        NodeSpaceRepo::update_plan_md(db, nodespace_id, content).await
    }

    /// Get PLAN.md content
    pub async fn get_plan(db: &SqlitePool, nodespace_id: &str) -> AppResult<Option<String>> {
        let ns = NodeSpaceRepo::get_by_id(db, nodespace_id).await?;
        Ok(ns.plan_md)
    }

    // ==================== GOAL.md Operations ====================

    /// Store GOAL.md content
    pub async fn store_goal_md(db: &SqlitePool, nodespace_id: &str, content: &str) -> AppResult<()> {
        NodeSpaceRepo::update_goal_md(db, nodespace_id, content).await
    }

    /// Get GOAL.md content
    pub async fn get_goal_md(db: &SqlitePool, nodespace_id: &str) -> AppResult<Option<String>> {
        let ns = NodeSpaceRepo::get_by_id(db, nodespace_id).await?;
        Ok(ns.goal_md)
    }

    // ==================== WorkNode Operations ====================

    /// Create initial worknode (for BuilderAgent)
    pub async fn create_initial_worknode(db: &SqlitePool, nodespace_id: &str) -> AppResult<WorkNode> {
        // Check if there's already a worknode
        let existing = WorkNodeRepo::list_by_nodespace(db, nodespace_id).await?;
        if !existing.is_empty() {
            return Err(AppError::Validation("Initial worknode already exists".to_string()));
        }

        let node = WorkNodeRepo::create(db, CreateWorkNodeInput {
            nodespace_id: nodespace_id.to_string(),
            parent_node_id: None,
            node_order: 0,
            milestone_id: None,
        }).await?;

        // Set as current node
        NodeSpaceRepo::update_current_node(db, nodespace_id, Some(&node.id)).await?;

        Ok(node)
    }

    /// Get current worknode
    pub async fn get_current_node(db: &SqlitePool, nodespace_id: &str) -> AppResult<Option<WorkNode>> {
        let ns = NodeSpaceRepo::get_by_id(db, nodespace_id).await?;
        match ns.current_node_id {
            Some(node_id) => {
                let node = WorkNodeRepo::get_by_id(db, &node_id).await?;
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }

    /// Get worknode by id
    pub async fn get_worknode(db: &SqlitePool, node_id: &str) -> AppResult<WorkNode> {
        WorkNodeRepo::get_by_id(db, node_id).await
    }

    /// List all worknodes for a nodespace
    pub async fn list_worknodes(db: &SqlitePool, nodespace_id: &str) -> AppResult<Vec<WorkNode>> {
        WorkNodeRepo::list_by_nodespace(db, nodespace_id).await
    }

    // ==================== CurrentNode Operations ====================

    /// Set current node (used by agent to track progress)
    pub async fn set_current_node(db: &SqlitePool, nodespace_id: &str, node_id: &str) -> AppResult<()> {
        // Verify node exists and belongs to nodespace
        let node = WorkNodeRepo::get_by_id(db, node_id).await?;
        if node.nodespace_id != nodespace_id {
            return Err(AppError::Validation("Node does not belong to this nodespace".to_string()));
        }

        NodeSpaceRepo::update_current_node(db, nodespace_id, Some(node_id)).await
    }

    // ==================== BUG.md Operations ====================

    /// Get BUG.md as BugEntry array
    pub async fn get_node_bug(db: &SqlitePool, node_id: &str) -> AppResult<Vec<BugEntry>> {
        WorkNodeRepo::get_bugs(db, node_id).await
    }

    /// Set BUG.md content
    pub async fn set_node_bug(db: &SqlitePool, node_id: &str, bugs: &[BugEntry]) -> AppResult<()> {
        WorkNodeRepo::update_bug_md(db, node_id, bugs).await
    }

    /// Add a bug to the node
    pub async fn add_node_bug(db: &SqlitePool, node_id: &str, bug: BugEntry) -> AppResult<()> {
        let mut bugs = Self::get_node_bug(db, node_id).await?;
        bugs.push(bug);
        Self::set_node_bug(db, node_id, &bugs).await
    }

    /// Remove resolved bugs
    pub async fn clear_resolved_bugs(db: &SqlitePool, node_id: &str) -> AppResult<()> {
        let bugs = Self::get_node_bug(db, node_id).await?;
        let unresolved: Vec<_> = bugs.into_iter().filter(|b| !b.resolved).collect();
        Self::set_node_bug(db, node_id, &unresolved).await
    }

    // ==================== CONCLUSION.md Operations ====================

    /// Get CONCLUSION.md content
    pub async fn get_node_conclusion(db: &SqlitePool, node_id: &str) -> AppResult<Option<String>> {
        let node = WorkNodeRepo::get_by_id(db, node_id).await?;
        Ok(node.conclusion_md)
    }

    /// Set CONCLUSION.md content
    pub async fn set_node_conclusion(db: &SqlitePool, node_id: &str, content: &str) -> AppResult<()> {
        WorkNodeRepo::update_conclusion_md(db, node_id, content).await
    }

    // ==================== USER_MANUAL.md Operations ====================

    /// Get USER_MANUAL.md content
    pub async fn get_node_user_manual(db: &SqlitePool, node_id: &str) -> AppResult<Option<String>> {
        let node = WorkNodeRepo::get_by_id(db, node_id).await?;
        Ok(node.user_manual_md)
    }

    /// Set USER_MANUAL.md content
    pub async fn set_node_user_manual(db: &SqlitePool, node_id: &str, content: &str) -> AppResult<()> {
        WorkNodeRepo::update_user_manual_md(db, node_id, content).await
    }

    // ==================== Node Achievement Operations ====================

    /// Mark goal as achieved, create new node
    pub async fn goal_achieved(
        db: &SqlitePool,
        nodespace_id: &str,
        conclusion: &str,
    ) -> AppResult<String> {
        // Get current node
        let current = Self::get_current_node(db, nodespace_id).await?
            .ok_or_else(|| AppError::Validation("No current node set".to_string()))?;

        // Update current node conclusion and status
        Self::set_node_conclusion(db, &current.id, conclusion).await?;
        WorkNodeRepo::update_status(db, &current.id, &WorkNodeStatus::Completed.to_string()).await?;

        // Create new node
        let new_node = WorkNodeRepo::create(db, CreateWorkNodeInput {
            nodespace_id: nodespace_id.to_string(),
            parent_node_id: Some(current.id),
            node_order: current.node_order + 1,
            milestone_id: None,
        }).await?;

        // Set as current node
        NodeSpaceRepo::update_current_node(db, nodespace_id, Some(&new_node.id)).await?;

        Ok(new_node.id)
    }

    /// Mark node as achieved, create next node
    pub async fn node_achieved(
        db: &SqlitePool,
        parent_node_id: &str,
        conclusion: &str,
    ) -> AppResult<String> {
        let parent = WorkNodeRepo::get_by_id(db, parent_node_id).await?;

        // Update parent conclusion and status
        Self::set_node_conclusion(db, parent_node_id, conclusion).await?;
        WorkNodeRepo::update_status(db, parent_node_id, &WorkNodeStatus::Completed.to_string()).await?;

        // Create new node
        let new_node = WorkNodeRepo::create(db, CreateWorkNodeInput {
            nodespace_id: parent.nodespace_id.clone(),
            parent_node_id: Some(parent_node_id.to_string()),
            node_order: parent.node_order + 1,
            milestone_id: None,
        }).await?;

        // Set as current node
        NodeSpaceRepo::update_current_node(db, &parent.nodespace_id, Some(&new_node.id)).await?;

        Ok(new_node.id)
    }

    // ==================== GeneratorTask Operations ====================

    /// Send specification to generator (create task)
    pub async fn send_to_generator(
        db: &SqlitePool,
        nodespace_id: &str,
        worknode_id: Option<&str>,
        specification: &Specification,
    ) -> AppResult<GeneratorTask> {
        let spec_json = serde_json::to_string(specification)?;

        GeneratorTaskRepo::create(db, CreateGeneratorTaskInput {
            nodespace_id: nodespace_id.to_string(),
            worknode_id: worknode_id.map(|s| s.to_string()),
            specification: spec_json,
        }).await
    }

    /// Get pending generator task
    pub async fn get_pending_generator_task(db: &SqlitePool, nodespace_id: &str) -> AppResult<Option<GeneratorTask>> {
        GeneratorTaskRepo::get_pending_by_nodespace(db, nodespace_id).await
    }

    /// Complete generator task
    pub async fn complete_generator_task(
        db: &SqlitePool,
        task_id: &str,
        result: &str,
        user_manual: Option<&str>,
        skills_json: Option<&str>,
    ) -> AppResult<()> {
        GeneratorTaskRepo::complete(db, task_id, result, user_manual, skills_json).await
    }

    /// Get generator task by id
    pub async fn get_generator_task(db: &SqlitePool, task_id: &str) -> AppResult<GeneratorTask> {
        GeneratorTaskRepo::get_by_id(db, task_id).await
    }
}
