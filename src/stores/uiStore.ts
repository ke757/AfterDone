import { create } from 'zustand';
import * as commands from '@/services/commands';

export type ActivePage = 'welcome' | 'workspace' | 'settings' | 'about' | 'repo';
export type InitStatus = 'loading' | 'first_launch' | 'ready';

interface UIState {
  leftSidebarVisible: boolean;
  rightSidebarVisible: boolean;
  leftSidebarWidth: number;
  rightSidebarWidth: number;
  currentRepoName: string | null;
  currentRepoPath: string | null;
  activePage: ActivePage;
  initStatus: InitStatus;
  needRestart: boolean;
  initError: string | null;

  toggleLeftSidebar: () => void;
  toggleRightSidebar: () => void;
  setLeftSidebarWidth: (w: number) => void;
  setRightSidebarWidth: (w: number) => void;
  setActivePage: (page: ActivePage) => void;
  initApp: () => Promise<void>;
  selectRepo: () => Promise<void>;
  completeInit: () => Promise<void>;
}

export const useUIStore = create<UIState>((set) => ({
  leftSidebarVisible: true,
  rightSidebarVisible: false,
  leftSidebarWidth: 240,
  rightSidebarWidth: 280,
  currentRepoName: null,
  currentRepoPath: null,
  activePage: 'welcome',
  initStatus: 'loading',
  needRestart: false,
  initError: null,

  toggleLeftSidebar: () => set((s) => ({ leftSidebarVisible: !s.leftSidebarVisible })),
  toggleRightSidebar: () => set((s) => ({ rightSidebarVisible: !s.rightSidebarVisible })),
  setLeftSidebarWidth: (w: number) => set({ leftSidebarWidth: w }),
  setRightSidebarWidth: (w: number) => set({ rightSidebarWidth: w }),
  setActivePage: (page) => set({ activePage: page }),

  initApp: async () => {
    try {
      const status = await commands.appGetInitStatus();
      if (status.initialized) {
        const info = await commands.configGetResourceRepoPath();
        set({
          initStatus: 'ready',
          currentRepoName: info.name || null,
          currentRepoPath: info.path || null,
          activePage: 'workspace',
        });
      } else {
        set({
          initStatus: 'first_launch',
          currentRepoPath: status.resource_repo_path,
          activePage: 'welcome',
        });
      }
    } catch {
      set({ initStatus: 'first_launch', activePage: 'welcome' });
    }
  },

  selectRepo: async () => {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === 'string') {
      try {
        const info = await commands.configSetResourceRepoPath(selected);
        set({
          currentRepoName: info.name || null,
          currentRepoPath: info.path || null,
          needRestart: true,
        });
      } catch (e) {
        set({ initError: e instanceof Error ? e.message : String(e) });
      }
    }
  },

  completeInit: async () => {
    try {
      const status = await commands.appCompleteInit();
      if (status.initialized) {
        set({ initStatus: 'ready', activePage: 'workspace' });
      }
    } catch (e) {
      set({ initError: e instanceof Error ? e.message : String(e) });
    }
  },
}));