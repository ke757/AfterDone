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
