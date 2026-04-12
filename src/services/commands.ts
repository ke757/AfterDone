/**
 * Tauri 命令调用工具
 * 封装 @tauri-apps/api/command 的 invoke 函数
 */
import { invoke } from '@tauri-apps/api/core';
import type { Goal, GoalSummary, CreateGoalRequest } from '../types/goal';
import type { Milestone, CreateMilestoneRequest } from '../types/milestone';
import type { Skill, CreateSkillRequest } from '../types/skill';
import type { Message } from '../types/chat';
import type { OpenClawConfig, LlmConfig } from '../types/settings';
import type { AgentStatus, AgentLog } from '../types/agent';
import type {
  NodeSpace,
  WorkNode,
  BugEntry,
} from '../types/noderepo';

// ============================================================================
// Goal Commands
// ============================================================================

export async function listGoals(): Promise<Goal[]> {
  return invoke<Goal[]>('list_goals');
}

export async function getGoal(id: string): Promise<Goal> {
  return invoke<Goal>('get_goal', { id });
}

export async function createGoal(request: CreateGoalRequest): Promise<Goal> {
  return invoke<Goal>('create_goal', { request });
}

export async function updateGoal(id: string, request: Partial<CreateGoalRequest>): Promise<Goal> {
  return invoke<Goal>('update_goal', { id, request });
}

export async function deleteGoal(id: string): Promise<void> {
  return invoke('delete_goal', { id });
}

export async function summarizeGoal(id: string): Promise<GoalSummary> {
  return invoke<GoalSummary>('summarize_goal', { id });
}

export async function pinGoal(id: string): Promise<Goal> {
  return invoke<Goal>('pin_goal', { id });
}

// ============================================================================
// Milestone Commands
// ============================================================================

export async function listMilestones(goalId: string): Promise<Milestone[]> {
  return invoke<Milestone[]>('list_milestones', { goalId });
}

export async function getMilestone(id: string): Promise<Milestone> {
  return invoke<Milestone>('get_milestone', { id });
}

export async function createMilestone(request: CreateMilestoneRequest): Promise<Milestone> {
  return invoke<Milestone>('create_milestone', { request });
}

export async function updateMilestone(id: string, request: Partial<CreateMilestoneRequest>): Promise<Milestone> {
  return invoke<Milestone>('update_milestone', { id, request });
}

export async function deleteMilestone(id: string): Promise<void> {
  return invoke('delete_milestone', { id });
}

// ============================================================================
// Skill Commands
// ============================================================================

export async function listSkills(goalId?: string): Promise<Skill[]> {
  return invoke<Skill[]>('list_skills', { goalId });
}

export async function getSkill(id: string): Promise<Skill> {
  return invoke<Skill>('get_skill', { id });
}

export async function createSkill(request: CreateSkillRequest): Promise<Skill> {
  return invoke<Skill>('create_skill', { request });
}

export async function updateSkill(id: string, request: Partial<CreateSkillRequest>): Promise<Skill> {
  return invoke<Skill>('update_skill', { id, request });
}

export async function deleteSkill(id: string): Promise<void> {
  return invoke('delete_skill', { id });
}

// ============================================================================
// Chat Commands
// ============================================================================

export async function getChatHistory(goalId: string): Promise<Message[]> {
  return invoke<Message[]>('get_chat_history', { goalId });
}

export async function sendMessage(goalId: string, content: string): Promise<Message> {
  return invoke<Message>('send_message', { goalId, content });
}

export async function streamMessage(
  goalId: string,
  content: string,
  _onChunk: (chunk: string) => void
): Promise<Message> {
  // 流式消息发送，通过事件接收
  return invoke<Message>('stream_message', { goalId, content });
}

// ============================================================================
// Agent Commands
// ============================================================================

export async function getAgentStatus(): Promise<AgentStatus> {
  return invoke<AgentStatus>('get_agent_status');
}

export async function getAgentLogs(goalId?: string, limit?: number): Promise<AgentLog[]> {
  return invoke<AgentLog[]>('get_agent_logs', { goalId, limit });
}

export async function startSummarizer(goalId: string): Promise<void> {
  return invoke('start_summarizer', { goalId });
}

export async function stopAgent(agentId: string): Promise<void> {
  return invoke('stop_agent', { agentId });
}

// ============================================================================
// Config Commands
// ============================================================================

export async function getOpenClawConfig(): Promise<OpenClawConfig> {
  return invoke<OpenClawConfig>('get_openclaw_config');
}

export async function updateOpenClawConfig(config: Partial<OpenClawConfig>): Promise<OpenClawConfig> {
  return invoke<OpenClawConfig>('update_openclaw_config', { config });
}

export async function testOpenClawConnection(): Promise<boolean> {
  return invoke<boolean>('test_openclaw_connection');
}

export async function getLlmConfig(): Promise<LlmConfig> {
  return invoke<LlmConfig>('get_llm_config');
}

export async function updateLlmConfig(config: Partial<LlmConfig>): Promise<LlmConfig> {
  return invoke<LlmConfig>('update_llm_config', { config });
}

export async function testLlmConnection(): Promise<boolean> {
  return invoke<boolean>('test_llm_connection');
}

// ============================================================================
// NodeSpace Commands
// ============================================================================

export async function nodespaceInit(goalId: string): Promise<NodeSpace> {
  return invoke<NodeSpace>('nodespace_init', { goal_id: goalId });
}

export async function nodespaceGet(goalId: string): Promise<NodeSpace | null> {
  return invoke<NodeSpace | null>('nodespace_get', { goal_id: goalId });
}

export async function nodespaceGetOrCreate(goalId: string): Promise<NodeSpace> {
  return invoke<NodeSpace>('nodespace_get_or_create', { goal_id: goalId });
}

export async function nodespaceUpdatePlan(nodespaceId: string, content: string): Promise<void> {
  return invoke('nodespace_update_plan', { nodespace_id: nodespaceId, content });
}

export async function nodespaceGetPlan(nodespaceId: string): Promise<string | null> {
  return invoke<string | null>('nodespace_get_plan', { nodespace_id: nodespaceId });
}

// ============================================================================
// WorkNode Commands
// ============================================================================

export async function worknodeCreateInitial(
  nodespaceId: string,
  milestoneId?: string
): Promise<WorkNode> {
  return invoke<WorkNode>('worknode_create_initial', { 
    nodespace_id: nodespaceId, 
    milestone_id: milestoneId 
  });
}

export async function worknodeGetCurrent(nodespaceId: string): Promise<WorkNode | null> {
  return invoke<WorkNode | null>('worknode_get_current', { nodespace_id: nodespaceId });
}

export async function worknodeGet(nodeId: string): Promise<WorkNode | null> {
  return invoke<WorkNode | null>('worknode_get', { node_id: nodeId });
}

export async function worknodeList(nodespaceId: string): Promise<WorkNode[]> {
  return invoke<WorkNode[]>('worknode_list', { nodespace_id: nodespaceId });
}

export async function worknodeGetBugs(nodeId: string): Promise<BugEntry[]> {
  return invoke<BugEntry[]>('worknode_get_bugs', { node_id: nodeId });
}

export async function worknodeAddBug(
  nodeId: string,
  description: string,
  errorOutput?: string
): Promise<BugEntry> {
  return invoke<BugEntry>('worknode_add_bug', { 
    node_id: nodeId, 
    description, 
    error_output: errorOutput 
  });
}

export async function worknodeGetConclusion(nodeId: string): Promise<string | null> {
  return invoke<string | null>('worknode_get_conclusion', { node_id: nodeId });
}

export async function worknodeGetUserManual(nodeId: string): Promise<string | null> {
  return invoke<string | null>('worknode_get_user_manual', { node_id: nodeId });
}

export async function worknodeGoalAchieved(
  nodespaceId: string, 
  conclusion: string
): Promise<string> {
  return invoke<string>('worknode_goal_achieved', { 
    nodespace_id: nodespaceId, 
    conclusion 
  });
}

export async function worknodeAchieved(
  parentNodeId: string, 
  conclusion: string
): Promise<string> {
  return invoke<string>('worknode_achieved', { 
    parent_node_id: parentNodeId, 
    conclusion 
  });
}
