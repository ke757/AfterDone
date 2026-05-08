/**
 * Goal Store
 * 管理 Goal 状态
 */
import { create } from 'zustand';
import type { Goal, GoalSummary } from '../types/goal';
import * as commands from '../services/commands';

interface GoalState {
  goals: Goal[];
  currentGoal: Goal | null;
  currentSummary: GoalSummary | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchGoals: () => Promise<void>;
  fetchGoal: (id: string) => Promise<void>;
  createGoal: () => Promise<Goal>;
  updateGoal: (goalId: string, title?: string, summary?: string, rawInput?: string) => Promise<void>;
  deleteGoal: (id: string) => Promise<void>;
  pinGoal: (goalId: string) => Promise<void>;
  setCurrentGoal: (goal: Goal | null) => void;
  clearError: () => void;
}

export const useGoalStore = create<GoalState>((set) => ({
  goals: [],
  currentGoal: null,
  currentSummary: null,
  isLoading: false,
  error: null,

  fetchGoals: async () => {
    set({ isLoading: true, error: null });
    try {
      const goals = await commands.listGoals();
      set({ goals, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  fetchGoal: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      const goal = await commands.getGoal(id);
      set({ currentGoal: goal, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  createGoal: async () => {
    set({ isLoading: true, error: null });
    try {
      const goal = await commands.createGoal();
      set((state) => ({
        goals: [...state.goals, goal],
        currentGoal: goal,
        isLoading: false,
      }));
      return goal;
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  updateGoal: async (goalId, title, summary, rawInput) => {
    set({ isLoading: true, error: null });
    try {
      const goal = await commands.updateGoal(goalId, title, summary, rawInput);
      set((state) => ({
        goals: state.goals.map((g) => (g.id === goalId ? goal : g)),
        currentGoal: state.currentGoal?.id === goalId ? goal : state.currentGoal,
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  deleteGoal: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await commands.deleteGoal(id);
      set((state) => ({
        goals: state.goals.filter((g) => g.id !== id),
        currentGoal: state.currentGoal?.id === id ? null : state.currentGoal,
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  pinGoal: async (goalId: string) => {
    set({ isLoading: true, error: null });
    try {
      const goal = await commands.pinGoal(goalId);
      set((state) => ({
        goals: state.goals.map((g) => (g.id === goalId ? goal : g)),
        currentGoal: state.currentGoal?.id === goalId ? goal : state.currentGoal,
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  setCurrentGoal: (goal: Goal | null) => {
    set({ currentGoal: goal });
  },

  clearError: () => {
    set({ error: null });
  },
}));
