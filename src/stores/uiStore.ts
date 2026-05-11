import { create } from 'zustand';
import * as commands from '@/services/commands';

export type ActivePage = 'welcome' | 'workspace' | 'settings' | 'about' | 'repo';

interface UIState {
  leftSidebarVisible: boolean;
  rightSidebarVisible: boolean;
  leftSidebarWidth: number;
  rightSidebarWidth: number;
  currentRepoName: string | null;
  currentRepoPath: string | null;
  activePage: ActivePage;
  repoInitialized: boolean;
  needRestart: boolean;

  toggleLeftSidebar: () => void;
  toggleRightSidebar: () => void;
  setLeftSidebarWidth: (w: number) => void;
  setRightSidebarWidth: (w: number) => void;
  setRepo: (name: string | null, path: string | null) => void;
  setActivePage: (page: ActivePage) => void;
  initRepo: () => Promise<void>;
  selectRepo: () => Promise<void>;
  dismissRestart: () => void;
}

export const useUIStore = create<UIState>((set) => ({
  leftSidebarVisible: true,
  rightSidebarVisible: false,
  leftSidebarWidth: 240,
  rightSidebarWidth: 280,
  currentRepoName: null,
  currentRepoPath: null,
  activePage: 'welcome',
  repoInitialized: false,
  needRestart: false,

  toggleLeftSidebar: () =>
    set((s) => ({ leftSidebarVisible: !s.leftSidebarVisible })),

  toggleRightSidebar: () =>
    set((s) => ({ rightSidebarVisible: !s.rightSidebarVisible })),

  setLeftSidebarWidth: (w: number) => set({ leftSidebarWidth: w }),
  setRightSidebarWidth: (w: number) => set({ rightSidebarWidth: w }),

  setRepo: (name, path) =>
    set({ currentRepoName: name, currentRepoPath: path, repoInitialized: true }),

  setActivePage: (page) => set({ activePage: page }),

  initRepo: async () => {
    try {
      const info = await commands.configGetDataPath();
      if (info.path) {
        set({
          currentRepoName: info.name,
          currentRepoPath: info.path,
          repoInitialized: true,
          activePage: 'workspace',
        });
      } else {
        set({ repoInitialized: true, activePage: 'welcome' });
      }
    } catch {
      set({ repoInitialized: true, activePage: 'welcome' });
    }
  },

  selectRepo: async () => {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected && typeof selected === 'string') {
      try {
        const info = await commands.configSetDataPath(selected);
        set({
          currentRepoName: info.name,
          currentRepoPath: info.path,
          needRestart: true,
        });
      } catch (e) {
        console.error('Failed to set data path:', e);
      }
    }
  },
dismissRestart: () => set({ needRestart: false }),
}));