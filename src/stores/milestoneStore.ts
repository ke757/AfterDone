/**
 * Milestone Store
 * 管理 Milestone 状态
 */
import { create } from 'zustand';
import type { Milestone, CreateMilestoneRequest } from '../types/milestone';
import * as commands from '../services/commands';

interface MilestoneState {
  milestones: Milestone[];
  currentMilestone: Milestone | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchMilestones: (goalId: string) => Promise<void>;
  fetchMilestone: (id: string) => Promise<void>;
  createMilestone: (request: CreateMilestoneRequest) => Promise<Milestone>;
  updateMilestone: (id: string, request: Partial<CreateMilestoneRequest>) => Promise<void>;
  deleteMilestone: (id: string) => Promise<void>;
  setCurrentMilestone: (milestone: Milestone | null) => void;
  clearError: () => void;
}

export const useMilestoneStore = create<MilestoneState>((set) => ({
  milestones: [],
  currentMilestone: null,
  isLoading: false,
  error: null,

  fetchMilestones: async (goalId: string) => {
    set({ isLoading: true, error: null });
    try {
      const milestones = await commands.listMilestones(goalId);
      set({ milestones, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  fetchMilestone: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      const milestone = await commands.getMilestone(id);
      set({ currentMilestone: milestone, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  createMilestone: async (request: CreateMilestoneRequest) => {
    set({ isLoading: true, error: null });
    try {
      const milestone = await commands.createMilestone(request);
      set((state) => ({
        milestones: [...state.milestones, milestone].sort(
          (a, b) => a.milestone_order - b.milestone_order
        ),
        isLoading: false,
      }));
      return milestone;
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  updateMilestone: async (id: string, request: Partial<CreateMilestoneRequest>) => {
    set({ isLoading: true, error: null });
    try {
      const milestone = await commands.updateMilestone(id, request);
      set((state) => ({
        milestones: state.milestones.map((m) => (m.id === id ? milestone : m)),
        currentMilestone: state.currentMilestone?.id === id ? milestone : state.currentMilestone,
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  deleteMilestone: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await commands.deleteMilestone(id);
      set((state) => ({
        milestones: state.milestones.filter((m) => m.id !== id),
        currentMilestone: state.currentMilestone?.id === id ? null : state.currentMilestone,
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  setCurrentMilestone: (milestone: Milestone | null) => {
    set({ currentMilestone: milestone });
  },

  clearError: () => {
    set({ error: null });
  },
}));
