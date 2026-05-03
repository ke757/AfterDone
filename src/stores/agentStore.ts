/**
 * Agent Store
 * ... Agent ...
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
  currentWorkspaceId: string | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchStatus: () => Promise<void>;
  fetchLogs: (goalId?: string, limit?: number) => Promise<void>;
  startAgent: (workspaceId: string) => Promise<void>;
  stopAgent: (workspaceId: string) => Promise<void>;
  clearLogs: () => void;
  clearError: () => void;
}

export const useAgentStore = create<AgentState>((set) => ({
  status: null,
  logs: [],
  isRunning: false,
  currentWorkspaceId: null,
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

  startAgent: async (workspaceId: string) => {
    set({ isLoading: true, error: null, currentWorkspaceId: workspaceId });
    try {
      await commands.startAgent(workspaceId);
      set({ isRunning: true, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  stopAgent: async (workspaceId: string) => {
    set({ isLoading: true, error: null });
    try {
      await commands.stopAgent(workspaceId);
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

// ...
let unlistenAgent: (() => void) | null = null;
let unlistenLog: (() => void) | null = null;

export function initAgentListeners() {
  if (unlistenAgent) return;

  onAgentEvent((event: AgentEvent) => {
    const store = useAgentStore.getState();
    store.fetchStatus();

    // ... agent ... ...
    if (event.status === 'completed' || event.status === 'failed') {
      useAgentStore.setState({ isRunning: false });
    }
  }).then((fn) => {
    unlistenAgent = fn;
  });

  onAgentLog((log) => {
    useAgentStore.setState((state) => ({
      logs: [log, ...state.logs].slice(0, 100), // ... 100 ...
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
