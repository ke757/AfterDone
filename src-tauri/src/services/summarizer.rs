use crate::db::DatabasePool;
use crate::error::AppResult;
use crate::events::EventBridge;
use crate::session::Message;
use crate::workhub::{Goal, GoalStatus, GoalSummary, GoalsRepo, WorkSpaceRepo};

/// Summarizer 持久化服务
///
/// 从 session 中提取 GoalSummary 并写入数据库。
/// 与 Agent 推理逻辑解耦，仅负责数据操作。
pub struct SummarizerService;

impl SummarizerService {
    /// 从消息列表中提取最后一条 assistant 消息中的 GoalSummary
    pub fn extract_summary_from_messages(messages: &[Message]) -> AppResult<GoalSummary> {
        let last_assistant = messages
            .iter()
            .rev()
            .find(|m| m.role == "assistant")
            .map(|m| m.content.clone())
            .unwrap_or_default();

        if last_assistant.is_empty() {
            return Ok(GoalSummary {
                title: "Goal".to_string(),
                description: "No summary available".to_string(),
                acceptance_criteria: vec![],
                constraints: vec![],
                refinement_questions: vec!["What would you like to achieve?".to_string()],
            });
        }

        parse_summary_response(&last_assistant)
    }

    /// 将 GoalSummary 持久化到数据库
    pub async fn persist_summary(
        pool: &DatabasePool,
        goal_id: &str,
        summary: &GoalSummary,
    ) -> AppResult<Goal> {
        let summary_json = serde_json::json!({
            "title": summary.title,
            "description": summary.description,
            "acceptance_criteria": summary.acceptance_criteria,
            "constraints": summary.constraints,
            "refinement_questions": summary.refinement_questions,
        });
        let summary_str = serde_json::to_string_pretty(&summary_json)?;

        let goal = GoalsRepo::update_summary(pool, goal_id, &summary_str).await?;
        GoalsRepo::update(pool, goal_id, crate::workhub::UpdateGoalInput {
            title: Some(summary.title.clone()),
            summary: None,
            raw_input: None,
        }).await?;

        Ok(goal)
    }

    /// 确认并持久化 Summary
    ///
    /// 从 cell 的会话历史中提取 GoalSummary → 写入 DB → 更新状态
    pub async fn confirm(
        pool: &DatabasePool,
        workspace_id: &str,
        messages: &[Message],
        emitter: &EventBridge,
    ) -> AppResult<Goal> {
        let workspace = WorkSpaceRepo::get_by_id(pool, workspace_id).await?;
        let goal = GoalsRepo::get_by_id(pool, &workspace.goal_id).await?;

        let summary = Self::extract_summary_from_messages(messages)?;
        let goal = Self::persist_summary(pool, &goal.id, &summary).await?;

        GoalsRepo::update_status(pool, &goal.id, GoalStatus::Draft).await?;
        emitter.emit_goal_status(&goal.id, "draft", "draft");

        Ok(goal)
    }
}

/// 从 LLM 响应中解析 GoalSummary JSON
fn parse_summary_response(response: &str) -> AppResult<GoalSummary> {
    let json_str = extract_json(response);

    match serde_json::from_str::<GoalSummary>(&json_str) {
        Ok(summary) => Ok(summary),
        Err(_) => {
            // 解析失败时，将整段文字作为 description
            Ok(GoalSummary {
                title: "Goal".to_string(),
                description: response.trim().to_string(),
                acceptance_criteria: vec!["Goal must be verifiably completed".to_string()],
                constraints: vec![],
                refinement_questions: vec!["Could you provide more details?".to_string()],
            })
        }
    }
}

/// 从可能被 Markdown 包裹的响应中提取 JSON
fn extract_json(text: &str) -> String {
    let trimmed = text.trim();

    if let Some(start) = trimmed.find("```json") {
        if let Some(end) = trimmed.rfind("```") {
            let json_start = start + 7;
            return trimmed[json_start..end].trim().to_string();
        }
    }

    if let Some(start) = trimmed.find("```") {
        if let Some(end) = trimmed.rfind("```") {
            if start != end {
                let json_start = start + 3;
                return trimmed[json_start..end].trim().to_string();
            }
        }
    }

    if trimmed.starts_with('{') {
        if let Some(end) = trimmed.rfind('}') {
            return trimmed[..=end].to_string();
        }
    }

    trimmed.to_string()
}
