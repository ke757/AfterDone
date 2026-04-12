/**
 * Settings Store
 * 管理应用设置状态
 */
import { create } from 'zustand';
import type { OpenClawConfig, LlmConfig } from '../types/settings';
import * as commands from '../services/commands';

interface SettingsState {
  openClawConfig: OpenClawConfig | null;
  llmConfig: LlmConfig | null;
  isOpenClawConnected: boolean;
  isLlmConnected: boolean;
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchOpenClawConfig: () => Promise<void>;
  updateOpenClawConfig: (config: Partial<OpenClawConfig>) => Promise<void>;
  testOpenClawConnection: () => Promise<boolean>;
  fetchLlmConfig: () => Promise<void>;
  updateLlmConfig: (config: Partial<LlmConfig>) => Promise<void>;
  testLlmConnection: () => Promise<boolean>;
  clearError: () => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  openClawConfig: null,
  llmConfig: null,
  isOpenClawConnected: false,
  isLlmConnected: false,
  isLoading: false,
  error: null,

  fetchOpenClawConfig: async () => {
    set({ isLoading: true, error: null });
    try {
      const config = await commands.getOpenClawConfig();
      set({ openClawConfig: config, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  updateOpenClawConfig: async (config: Partial<OpenClawConfig>) => {
    set({ isLoading: true, error: null });
    try {
      const updated = await commands.updateOpenClawConfig(config);
      set({ openClawConfig: updated, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  testOpenClawConnection: async () => {
    set({ isLoading: true, error: null });
    try {
      const connected = await commands.testOpenClawConnection();
      set({ isOpenClawConnected: connected, isLoading: false });
      return connected;
    } catch (error) {
      set({ isOpenClawConnected: false, error: String(error), isLoading: false });
      return false;
    }
  },

  fetchLlmConfig: async () => {
    set({ isLoading: true, error: null });
    try {
      const config = await commands.getLlmConfig();
      set({ llmConfig: config, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  updateLlmConfig: async (config: Partial<LlmConfig>) => {
    set({ isLoading: true, error: null });
    try {
      const updated = await commands.updateLlmConfig(config);
      set({ llmConfig: updated, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  testLlmConnection: async () => {
    set({ isLoading: true, error: null });
    try {
      const connected = await commands.testLlmConnection();
      set({ isLlmConnected: connected, isLoading: false });
      return connected;
    } catch (error) {
      set({ isLlmConnected: false, error: String(error), isLoading: false });
      return false;
    }
  },

  clearError: () => {
    set({ error: null });
  },
}));
