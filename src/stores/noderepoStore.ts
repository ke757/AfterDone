/**
 * NodeRepo Store
 * Manages NodeSpace and WorkNode state using Zustand
 */
import { create } from 'zustand';
import type {
  NodeSpace,
  WorkNode,
  BugEntry,
  WorkNodeTree,
} from '../types/noderepo';
import * as commands from '../services/commands';

// ============================================================================
// NodeSpace Store
// ============================================================================

interface NodeSpaceState {
  current: NodeSpace | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  init: (goalId: string) => Promise<NodeSpace | null>;
  load: (goalId: string) => Promise<NodeSpace | null>;
  updatePlan: (nodespaceId: string, content: string) => Promise<void>;
  getPlan: (nodespaceId: string) => Promise<string | null>;
  clear: () => void;
}

export const useNodeSpaceStore = create<NodeSpaceState>((set) => ({
  current: null,
  isLoading: false,
  error: null,

  init: async (goalId: string) => {
    set({ isLoading: true, error: null });
    try {
      const nodespace = await commands.nodespaceGetOrCreate(goalId);
      set({ current: nodespace, isLoading: false });
      return nodespace;
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e);
      set({ isLoading: false, error });
      return null;
    }
  },

  load: async (goalId: string) => {
    set({ isLoading: true, error: null });
    try {
      const nodespace = await commands.nodespaceGet(goalId);
      set({ current: nodespace, isLoading: false });
      return nodespace;
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e);
      set({ isLoading: false, error });
      return null;
    }
  },

  updatePlan: async (nodespaceId: string, content: string) => {
    try {
      await commands.nodespaceUpdatePlan(nodespaceId, content);
      set((state) => ({
        current: state.current?.id === nodespaceId 
          ? { ...state.current, plan_md: content } 
          : state.current,
      }));
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e);
      set({ error });
      throw e;
    }
  },

  getPlan: async (nodespaceId: string) => {
    return commands.nodespaceGetPlan(nodespaceId);
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
  getCurrent: (nodespaceId: string) => Promise<WorkNode | null>;
  createInitial: (nodespaceId: string, milestoneId?: string) => Promise<WorkNode | null>;
  loadBugs: (worknodeId: string) => Promise<BugEntry[]>;
  addBug: (worknodeId: string, description: string, errorOutput?: string) => Promise<BugEntry | null>;
  getConclusion: (worknodeId: string) => Promise<string | null>;
  getUserManual: (worknodeId: string) => Promise<string | null>;
  markGoalAchieved: (nodespaceId: string, conclusion: string) => Promise<string | null>;
  markNodeAchieved: (parentNodeId: string, conclusion: string) => Promise<string | null>;
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
      // First get the nodespace
      const nodespace = await commands.nodespaceGet(goalId);
      if (!nodespace) {
        set({ nodes: [], currentNode: null, isLoading: false });
        return [];
      }

      // Load worknodes
      const nodes = await commands.worknodeList(nodespace.id);

      // Get current node
      const currentNodeId = nodespace.current_node_id;
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

  getCurrent: async (nodespaceId: string) => {
    try {
      const node = await commands.worknodeGetCurrent(nodespaceId);
      set({ currentNode: node });
      return node;
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e);
      set({ error });
      return null;
    }
  },

  createInitial: async (nodespaceId: string, milestoneId?: string) => {
    set({ isLoading: true, error: null });
    try {
      const node = await commands.worknodeCreateInitial(nodespaceId, milestoneId);
      set((state) => ({
        nodes: [...state.nodes, node],
        currentNode: node,
        isLoading: false,
      }));
      return node;
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e);
      set({ isLoading: false, error });
      return null;
    }
  },

  loadBugs: async (worknodeId: string) => {
    try {
      const bugs = await commands.worknodeGetBugs(worknodeId);
      set({ bugs });
      return bugs;
    } catch (e) {
      return [];
    }
  },

  addBug: async (worknodeId: string, description: string, errorOutput?: string) => {
    try {
      const bug = await commands.worknodeAddBug(worknodeId, description, errorOutput);
      set((state) => ({ bugs: [...state.bugs, bug] }));
      return bug;
    } catch (e) {
      return null;
    }
  },

  getConclusion: async (worknodeId: string) => {
    return commands.worknodeGetConclusion(worknodeId);
  },

  getUserManual: async (worknodeId: string) => {
    return commands.worknodeGetUserManual(worknodeId);
  },

  markGoalAchieved: async (nodespaceId: string, conclusion: string) => {
    try {
      const newNodeId = await commands.worknodeGoalAchieved(nodespaceId, conclusion);
      return newNodeId;
    } catch (e) {
      return null;
    }
  },

  markNodeAchieved: async (parentNodeId: string, conclusion: string) => {
    try {
      const newNodeId = await commands.worknodeAchieved(parentNodeId, conclusion);
      return newNodeId;
    } catch (e) {
      return null;
    }
  },

  clear: () => {
    set({ nodes: [], currentNode: null, isLoading: false, error: null, bugs: [] });
  },
}));

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Build a tree structure from flat worknode list
 */
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

/**
 * Get bugs for a specific worknode
 */
export async function getWorkNodeBugs(worknodeId: string): Promise<BugEntry[]> {
  return commands.worknodeGetBugs(worknodeId);
}
