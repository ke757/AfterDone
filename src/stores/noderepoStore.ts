/**
 * NodeRepo Store
 * Manages WorkSpace and WorkNode state
 */
import { create } from 'zustand';
import type {
  WorkNode,
  BugEntry,
  WorkNodeTree,
} from '../types/noderepo';
import * as commands from '../services/commands';

interface WorkSpace {
  id: string;
  goal_id: string;
  current_node_id: string | null;
  goal_md: string | null;
  plan_md: string | null;
  created_at: string;
  updated_at: string;
}

// ============================================================================
// WorkSpace Store
// ============================================================================

interface WorkSpaceState {
  current: WorkSpace | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  init: (title: string, rawInput: string) => Promise<{ root_node_id: string; goal_id: string; workspace_id: string } | null>;
  load: (goalId: string) => Promise<WorkSpace | null>;
  clear: () => void;
}

export const useWorkSpaceStore = create<WorkSpaceState>((set) => ({
  current: null,
  isLoading: false,
  error: null,

  init: async (title, rawInput) => {
    set({ isLoading: true, error: null });
    try {
      const result = await commands.workspaceInit(title, rawInput);
      set({ current: result.workspace, isLoading: false });
      return {
        root_node_id: result.root_node_id,
        goal_id: result.goal.id,
        workspace_id: result.workspace.id,
      };
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e);
      set({ isLoading: false, error });
      return null;
    }
  },

  load: async (goalId: string) => {
    set({ isLoading: true, error: null });
    try {
      const ws = await commands.workspaceGetByGoal(goalId);
      set({ current: ws, isLoading: false });
      return ws;
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e);
      set({ isLoading: false, error });
      return null;
    }
  },

  clear: () => {
    set({ current: null, isLoading: false, error: null });
  },
}));

// ============================================================================
// WorkNode Store
// ============================================================================

interface WorkNodeState {
  nodes: WorkNode[];
  currentNode: WorkNode | null;
  isLoading: boolean;
  error: string | null;
  bugs: BugEntry[];

  // Actions
  loadForGoal: (goalId: string) => Promise<WorkNode[]>;
  loadBugs: (worknodeId: string) => Promise<BugEntry[]>;
  clear: () => void;
}

export const useWorkNodeStore = create<WorkNodeState>((set) => ({
  nodes: [],
  currentNode: null,
  isLoading: false,
  error: null,
  bugs: [],

  loadForGoal: async (goalId: string) => {
    set({ isLoading: true, error: null });
    try {
      const ws = await commands.workspaceGetByGoal(goalId);
      if (!ws) {
        set({ nodes: [], currentNode: null, isLoading: false });
        return [];
      }

      const nodes = await commands.worknodeList(ws.id);
      const currentNodeId = ws.current_node_id;
      const currentNode = currentNodeId
        ? nodes.find((n) => n.id === currentNodeId) || null
        : null;

      set({ nodes, currentNode, isLoading: false });
      return nodes;
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e);
      set({ isLoading: false, error });
      return [];
    }
  },

  loadBugs: async (worknodeId: string) => {
    try {
      const bugs = await commands.worknodeGetBugs(worknodeId);
      set({ bugs });
      return bugs;
    } catch {
      return [];
    }
  },

  clear: () => {
    set({ nodes: [], currentNode: null, isLoading: false, error: null, bugs: [] });
  },
}));

// ============================================================================
// Helper Functions
// ============================================================================

export function buildWorkNodeTree(nodes: WorkNode[]): WorkNodeTree[] {
  const buildTree = (parentId: string | null = null): WorkNodeTree[] => {
    return nodes
      .filter((n) => n.parent_node_id === parentId)
      .sort((a, b) => a.node_order - b.node_order)
      .map((node) => ({
        node,
        children: buildTree(node.id),
      }));
  };

  return buildTree();
}
