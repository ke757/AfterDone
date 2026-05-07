/**
 * Tauri 事件监听工具
 */
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { AgentLog } from '../types/agent';

// ============================================================================
// Event Types
// ============================================================================

export interface AgentEvent {
  agent_id: string;
  agent_type: 'summarizer' | 'executor' | 'supervisor';
  status: 'started' | 'running' | 'completed' | 'failed';
  goal_id?: string;
  message: string;
  timestamp: string;
}

export interface GoalEvent {
  goal_id: string;
  event_type: 'created' | 'updated' | 'deleted' | 'status_changed';
  status?: string;
  timestamp: string;
}

export interface ChatStreamEvent {
  message_id: string;
  goal_id: string;
  chunk: string;
  is_final: boolean;
}

// ============================================================================
// Event Listeners
// ============================================================================

/**
 * 监听 Agent 状态变化事件
 */
export function onAgentEvent(
  callback: (event: AgentEvent) => void
): Promise<UnlistenFn> {
  return listen<AgentEvent>('agent:event', (event) => {
    callback(event.payload);
  });
}

/**
 * 监听 Goal 变化事件
 */
export function onGoalEvent(
  callback: (event: GoalEvent) => void
): Promise<UnlistenFn> {
  return listen<GoalEvent>('goal:event', (event) => {
    callback(event.payload);
  });
}

export function onChatStream(
  callback: (event: ChatStreamEvent) => void
): Promise<UnlistenFn> {
  return listen<ChatStreamEvent>('chat:stream', (event) => {
    callback(event.payload);
  });
}

/**
 * 监听 Agent 日志
 */
export function onAgentLog(
  callback: (log: AgentLog) => void
): Promise<UnlistenFn> {
  return listen<AgentLog>('agent:log', (event) => {
    callback(event.payload);
  });
}

/**
 * 监听新消息
 */
export function onNewMessage(
  callback: (message: { role: string; content: string; agent_type: string; created_at: string }) => void
): Promise<UnlistenFn> {
  return listen('chat:message', (event: any) => {
    callback(event.payload);
  });
}

// ============================================================================
// Event Emitter (for testing in web mode)
// ============================================================================

/**
 * 模拟事件发射器（用于 web 模式开发测试）
 */
export const mockEventEmitter = {
  emitAgentEvent: (event: AgentEvent) => {
    window.dispatchEvent(new CustomEvent('agent:event', { detail: event }));
  },
  emitGoalEvent: (event: GoalEvent) => {
    window.dispatchEvent(new CustomEvent('goal:event', { detail: event }));
  },
  emitChatStream: (event: ChatStreamEvent) => {
    window.dispatchEvent(new CustomEvent('chat:stream', { detail: event }));
  },
};
