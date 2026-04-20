use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use super::types::*;
use super::repos::{WorkSpaceRepo, WorkNodeRepo, GeneratorTaskRepo};

/// Core WorkHub operations for agent tools
pub struct WorkHub;

impl WorkHub {
    // ==================== WorkSpace Operations ====================

    /// Initialize a WorkSpace for a goal
    pub async fn init_workspace(db: &SqlitePool, goal_id: &str) -> AppResult<WorkSpace> {
        // Check if already exists
        if let Some(existing) = WorkSpaceRepo::get_by_goal_id(db, goal_id).await? {
            return Ok(existing);
        }

        WorkSpaceRepo::create(db, CreateWorkSpaceInput {
            goal_id: goal_id.to_string(),
            goal_md: None,
            plan_md: None,
        }).await
    }
    
    /// List all WorkSpaces
    pub async fn list_workspaces(db: &SqlitePool) -> AppResult<Vec<WorkSpace>> {
        WorkSpaceRepo::list_all(db).await
    }

    /// Get WorkSpace by goal_id
    pub async fn get_workspace_by_goal(db: &SqlitePool, goal_id: &str) -> AppResult<Option<WorkSpace>> {
        WorkSpaceRepo::get_by_goal_id(db, goal_id).await
    }

    /// Get or create WorkSpace for a goal
    pub async fn get_or_create_workspace(db: &SqlitePool, goal_id: &str) -> AppResult<WorkSpace> {
        match WorkSpaceRepo::get_by_goal_id(db, goal_id).await? {
            Some(ws) => Ok(ws),
            None => Self::init_workspace(db, goal_id).await,
        }
    }

    // ==================== PLAN.md Operations ====================

    /// Store PLAN.md content
    pub async fn store_plan(db: &SqlitePool, workspace_id: &str, content: &str) -> AppResult<()> {
        WorkSpaceRepo::update_plan_md(db, workspace_id, content).await
    }

    /// Get PLAN.md content
    pub async fn get_plan(db: &SqlitePool, workspace_id: &str) -> AppResult<Option<String>> {
        let ws = WorkSpaceRepo::get_by_id(db, workspace_id).await?;
        Ok(ws.plan_md)
    }

    // ==================== GOAL.md Operations ====================

    /// Store GOAL.md content
    pub async fn store_goal_md(db: &SqlitePool, workspace_id: &str, content: &str) -> AppResult<()> {
        WorkSpaceRepo::update_goal_md(db, workspace_id, content).await
    }

    /// Get GOAL.md content
    pub async fn get_goal_md(db: &SqlitePool, workspace_id: &str) -> AppResult<Option<String>> {
        let ws = WorkSpaceRepo::get_by_id(db, workspace_id).await?;
        Ok(ws.goal_md)
    }

    // ==================== WorkNode Operations ====================

    /// Create initial worknode (for BuilderAgent)
    pub async fn create_initial_worknode(db: &SqlitePool, workspace_id: &str) -> AppResult<WorkNode> {
        // Check if there's already a worknode
        let existing = WorkNodeRepo::list_by_workspace(db, workspace_id).await?;
        if !existing.is_empty() {
            return Err(AppError::Validation("Initial worknode already exists".to_string()));
        }

        let node = WorkNodeRepo::create(db, CreateWorkNodeInput {
            workspace_id: workspace_id.to_string(),
            parent_node_id: None,
            node_order: 0,
            milestone_id: None,
        }).await?;

        // Set as current node
        WorkSpaceRepo::update_current_node(db, workspace_id, Some(&node.id)).await?;

        Ok(node)
    }

    /// Get current worknode
    pub async fn get_current_node(db: &SqlitePool, workspace_id: &str) -> AppResult<Option<WorkNode>> {
        let ws = WorkSpaceRepo::get_by_id(db, workspace_id).await?;
        match ws.current_node_id {
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

    /// List all worknodes for a workspace
    pub async fn list_worknodes(db: &SqlitePool, workspace_id: &str) -> AppResult<Vec<WorkNode>> {
        WorkNodeRepo::list_by_workspace(db, workspace_id).await
    }

    // ==================== CurrentNode Operations ====================

    /// Set current node (used by agent to track progress)
    pub async fn set_current_node(db: &SqlitePool, workspace_id: &str, node_id: &str) -> AppResult<()> {
        // Verify node exists and belongs to workspace
        let node = WorkNodeRepo::get_by_id(db, node_id).await?;
        if node.workspace_id != workspace_id {
            return Err(AppError::Validation("Node does not belong to this workspace".to_string()));
        }

        WorkSpaceRepo::update_current_node(db, workspace_id, Some(node_id)).await
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
        workspace_id: &str,
        conclusion: &str,
    ) -> AppResult<String> {
        // Get current node
        let current = Self::get_current_node(db, workspace_id).await?
            .ok_or_else(|| AppError::Validation("No current node set".to_string()))?;

        // Update current node conclusion and status
        Self::set_node_conclusion(db, &current.id, conclusion).await?;
        WorkNodeRepo::update_status(db, &current.id, &WorkNodeStatus::Completed.to_string()).await?;

        // Create new node
        let new_node = WorkNodeRepo::create(db, CreateWorkNodeInput {
            workspace_id: workspace_id.to_string(),
            parent_node_id: Some(current.id),
            node_order: current.node_order + 1,
            milestone_id: None,
        }).await?;

        // Set as current node
        WorkSpaceRepo::update_current_node(db, workspace_id, Some(&new_node.id)).await?;

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
            workspace_id: parent.workspace_id.clone(),
            parent_node_id: Some(parent_node_id.to_string()),
            node_order: parent.node_order + 1,
            milestone_id: None,
        }).await?;

        // Set as current node
        WorkSpaceRepo::update_current_node(db, &parent.workspace_id, Some(&new_node.id)).await?;

        Ok(new_node.id)
    }

    // ==================== GeneratorTask Operations ====================

    /// Send specification to generator (create task)
    pub async fn send_to_generator(
        db: &SqlitePool,
        workspace_id: &str,
        worknode_id: Option<&str>,
        specification: &Specification,
    ) -> AppResult<GeneratorTask> {
        let spec_json = serde_json::to_string(specification)?;

        GeneratorTaskRepo::create(db, CreateGeneratorTaskInput {
            workspace_id: workspace_id.to_string(),
            worknode_id: worknode_id.map(|s| s.to_string()),
            specification: spec_json,
        }).await
    }

    /// Get pending generator task
    pub async fn get_pending_generator_task(db: &SqlitePool, workspace_id: &str) -> AppResult<Option<GeneratorTask>> {
        GeneratorTaskRepo::get_pending_by_workspace(db, workspace_id).await
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
