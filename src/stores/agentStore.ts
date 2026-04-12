/**
 * Agent Store
 * 管理 Agent 状态
 */
import { create } from 'zustand';
import type { AgentStatus, AgentLog } from '../types/agent';
import * as commands from '../services/commands';
import { onAgentEvent, onAgentLog } from '../services/events';
import type { AgentEvent } from '../services/events';

interface AgentState {
  status: AgentStatus | null;
  logs: AgentLog[];
  isRunning: boolean;
  currentGoalId: string | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchStatus: () => Promise<void>;
  fetchLogs: (goalId?: string, limit?: number) => Promise<void>;
  startSummarizer: (goalId: string) => Promise<void>;
  stopAgent: (agentId: string) => Promise<void>;
  clearLogs: () => void;
  clearError: () => void;
}

export const useAgentStore = create<AgentState>((set) => ({
  status: null,
  logs: [],
  isRunning: false,
  currentGoalId: null,
  isLoading: false,
  error: null,

  fetchStatus: async () => {
    set({ isLoading: true, error: null });
    try {
      const status = await commands.getAgentStatus();
      set({
        status,
        isRunning: status.running_agents.length > 0,
        isLoading: false,
      });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  fetchLogs: async (goalId?: string, limit?: number) => {
    set({ isLoading: true, error: null });
    try {
      const logs = await commands.getAgentLogs(goalId, limit);
      set({ logs, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  startSummarizer: async (goalId: string) => {
    set({ isLoading: true, error: null, currentGoalId: goalId });
    try {
      await commands.startSummarizer(goalId);
      set({ isRunning: true, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  stopAgent: async (agentId: string) => {
    set({ isLoading: true, error: null });
    try {
      await commands.stopAgent(agentId);
      // 更新状态
      const status = await commands.getAgentStatus();
      set({
        status,
        isRunning: status.running_agents.length > 0,
        isLoading: false,
      });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  clearLogs: () => {
    set({ logs: [] });
  },

  clearError: () => {
    set({ error: null });
  },
}));

// 初始化事件监听
let unlistenAgent: (() => void) | null = null;
let unlistenLog: (() => void) | null = null;

export function initAgentListeners() {
  if (unlistenAgent) return;

  onAgentEvent((event: AgentEvent) => {
    const store = useAgentStore.getState();
    store.fetchStatus();

    // 如果 agent 完成或失败，更新状态
    if (event.status === 'completed' || event.status === 'failed') {
      useAgentStore.setState({ isRunning: false });
    }
  }).then((fn) => {
    unlistenAgent = fn;
  });

  onAgentLog((log) => {
    useAgentStore.setState((state) => ({
      logs: [log, ...state.logs].slice(0, 100), // 保留最近 100 条日志
    }));
  }).then((fn) => {
    unlistenLog = fn;
  });
}

export function cleanupAgentListeners() {
  if (unlistenAgent) {
    unlistenAgent();
    unlistenAgent = null;
  }
  if (unlistenLog) {
    unlistenLog();
    unlistenLog = null;
  }
}
