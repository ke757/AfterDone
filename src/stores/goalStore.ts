/**
 * Goal Store
 * 管理 Goal 状态
 */
import { create } from 'zustand';
import type { Goal, GoalSummary, CreateGoalRequest } from '../types/goal';
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
  createGoal: (request: CreateGoalRequest) => Promise<Goal>;
  updateGoal: (id: string, request: Partial<CreateGoalRequest>) => Promise<void>;
  deleteGoal: (id: string) => Promise<void>;
  summarizeGoal: (id: string) => Promise<void>;
  pinGoal: (id: string) => Promise<void>;
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

  createGoal: async (request: CreateGoalRequest) => {
    set({ isLoading: true, error: null });
    try {
      const goal = await commands.createGoal(request);
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

  updateGoal: async (id: string, request: Partial<CreateGoalRequest>) => {
    set({ isLoading: true, error: null });
    try {
      const goal = await commands.updateGoal(id, request);
      set((state) => ({
        goals: state.goals.map((g) => (g.id === id ? goal : g)),
        currentGoal: state.currentGoal?.id === id ? goal : state.currentGoal,
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

  summarizeGoal: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      const summary = await commands.summarizeGoal(id);
      set({ currentSummary: summary, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  pinGoal: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      const goal = await commands.pinGoal(id);
      set((state) => ({
        goals: state.goals.map((g) => (g.id === id ? goal : g)),
        currentGoal: state.currentGoal?.id === id ? goal : state.currentGoal,
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
