/**
 * Tauri 命令调用工具
 * 封装 @tauri-apps/api/command 的 invoke 函数
 */
import { invoke } from '@tauri-apps/api/core';
import type { Goal } from '../types/goal';
import type { OpenClawConfig, LlmConfig } from '../types/settings';
import type { AgentStatus, AgentLog } from '../types/agent';
import type { WorkNode, BugEntry } from '../types/noderepo';

// ============================================================================
// Goal Commands
// ============================================================================

export async function listGoals(): Promise<Goal[]> {
  return invoke<Goal[]>('goals_list');
}

export async function getGoal(id: string): Promise<Goal> {
  return invoke<Goal>('goals_get', { goalId: id });
}

export async function createGoal(): Promise<Goal> {
  return invoke<Goal>('goals_create');
}

export async function updateGoal(
  goalId: string,
  title?: string,
  summary?: string,
  rawInput?: string,
): Promise<Goal> {
  return invoke<Goal>('goals_update', {
    goalId,
    title: title ?? null,
    summary: summary ?? null,
    rawInput: rawInput ?? null,
  });
}

export async function deleteGoal(id: string): Promise<void> {
  return invoke('goals_delete', { goalId: id });
}

export async function pinGoal(goalId: string): Promise<Goal> {
  return invoke<Goal>('goals_pin', { goalId });
}

// ============================================================================
// WorkSpace Commands
// ============================================================================

interface WorkSpace {
  id: string;
  goal_id: string;
  current_node_id: string | null;
  goal_md: string | null;
  plan_md: string | null;
  created_at: string;
  updated_at: string;
}

/** 初始化 Workspace — 创建 Goal + WorkSpace + RootNode */
export async function workspaceInit(): Promise<{ workspace: WorkSpace; root_node_id: string; goal: Goal }> {
  return invoke('workspace_init');
}

export async function workspaceList(): Promise<WorkSpace[]> {
  return invoke<WorkSpace[]>('workspace_list');
}

export async function workspaceGetByGoal(goalId: string): Promise<WorkSpace | null> {
  return invoke<WorkSpace | null>('workspace_get', { goalId });
}

// ============================================================================
// Summarizer Commands (对话式探需)
// ============================================================================

/** 投递一条用户消息给 Summarizer Cell，获取一轮对话响应 */
export async function summarizerSendMessage(
  cellId: string,
  message: string
): Promise<{ cell_id: string; message: string; summary_ready: boolean }> {
  return invoke('summarizer_send_message', { cellId, message });
}

/** 确认 Summarizer 生成的 GoalSummary，持久化到数据库 */
export async function summarizerConfirm(cellId: string): Promise<Goal> {
  return invoke<Goal>('summarizer_confirm', { cellId });
}

/** 查询 Cell 的当前结果状态 */
export async function summarizerStatus(
  cellId: string
): Promise<{ cell_id: string; result_status: string | null }> {
  return invoke('summarizer_status', { cellId });
}

// ============================================================================
// Cell Commands
// ============================================================================

export interface CellInfo {
  cell_id: string;
  cell_type: string;
  agent_type: string | null;
  status: string;
  message_count: number;
  result_status: string | null;
}

/** 创建新的 Cell */
export async function cellCreate(
  workspaceId: string,
  nodeId: string,
  cellType: string
): Promise<CellInfo> {
  return invoke<CellInfo>('cell_create', { workspaceId, nodeId, cellType });
}

/** 列出某 WorkNode 下所有 cells */
export async function cellListByNode(nodeId: string): Promise<CellInfo[]> {
  return invoke<CellInfo[]>('cell_list_by_node', { nodeId });
}

/** 获取 Cell 的完整会话历史 */
export async function cellGetHistory(
  cellId: string
): Promise<{ cell_id: string; messages: Array<{ role: string; content: string; agent_type: string; created_at: string }> }> {
  return invoke('cell_get_history', { cellId });
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

export async function startAgent(workspaceId: string): Promise<void> {
  return invoke('agents_start', { workspaceId });
}

export async function stopAgent(workspaceId: string): Promise<void> {
  return invoke('agents_stop', { workspaceId });
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
// ResourceRepo Commands
// ============================================================================

export interface ResourceRepoInfo {
  path: string;
  name: string;
}

export interface AppInitStatus {
  initialized: boolean;
  resource_repo_path: string;
}

export async function configGetResourceRepoPath(): Promise<ResourceRepoInfo> {
  return invoke<ResourceRepoInfo>('config_get_resource_repo_path');
}

export async function configSetResourceRepoPath(path: string): Promise<ResourceRepoInfo> {
  return invoke<ResourceRepoInfo>('config_set_resource_repo_path', { path });
}

export async function appGetInitStatus(): Promise<AppInitStatus> {
  return invoke<AppInitStatus>('app_get_init_status');
}

export async function appCompleteInit(): Promise<AppInitStatus> {
  return invoke<AppInitStatus>('app_complete_init');
}

// ============================================================================
// WorkNode Commands
// ============================================================================

export async function worknodeCreateInitial(
  workspaceId: string
): Promise<WorkNode> {
  return invoke<WorkNode>('worknode_create_initial', { 
    workspace_id: workspaceId,
  });
}

export async function worknodeGetCurrent(workspaceId: string): Promise<WorkNode | null> {
  return invoke<WorkNode | null>('worknode_get_current', { workspace_id: workspaceId });
}

export async function worknodeGet(nodeId: string): Promise<WorkNode | null> {
  return invoke<WorkNode | null>('worknode_get', { node_id: nodeId });
}

export async function worknodeList(workspaceId: string): Promise<WorkNode[]> {
  return invoke<WorkNode[]>('worknode_list', { workspace_id: workspaceId });
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
  workspaceId: string, 
  conclusion: string
): Promise<string> {
  return invoke<string>('worknode_goal_achieved', { 
    workspace_id: workspaceId, 
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
