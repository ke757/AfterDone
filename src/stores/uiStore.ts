import { create } from 'zustand';
import * as commands from '@/services/commands';

export type ActivePage = 'welcome' | 'workspace' | 'settings' | 'about' | 'repo';
export type InitStatus = 'loading' | 'initialize' | 'ready';

interface UIState {
  leftSidebarVisible: boolean;
  rightSidebarVisible: boolean;
  leftSidebarWidth: number;
  rightSidebarWidth: number;
  currentRepoName: string | null;
  currentRepoPath: string | null;
  initialRepoPath: string | null;
  activePage: ActivePage;
  initStatus: InitStatus;
  needRestart: boolean;
  initError: string | null;

  toggleLeftSidebar: () => void;
  toggleRightSidebar: () => void;
  setLeftSidebarWidth: (w: number) => void;
  setRightSidebarWidth: (w: number) => void;
  setActivePage: (page: ActivePage) => void;
  launchApp: () => Promise<void>;
  selectRepo: () => Promise<void>;
  completeInit: () => Promise<void>;
}

export const useUIStore = create<UIState>((set, get) => ({
  leftSidebarVisible: true,
  rightSidebarVisible: false,
  leftSidebarWidth: 240,
  rightSidebarWidth: 280,
  currentRepoName: null,
  currentRepoPath: null,
  initialRepoPath: null,
  activePage: 'welcome',
  initStatus: 'loading',
  needRestart: false,
  initError: null,

  toggleLeftSidebar: () => set((s) => ({ leftSidebarVisible: !s.leftSidebarVisible })),
  toggleRightSidebar: () => set((s) => ({ rightSidebarVisible: !s.rightSidebarVisible })),
  setLeftSidebarWidth: (w: number) => set({ leftSidebarWidth: w }),
  setRightSidebarWidth: (w: number) => set({ rightSidebarWidth: w }),
  setActivePage: (page) => set({ activePage: page }),

  launchApp: async () => {
    try {
      const status = await commands.appGetInitStatus();
      if (status.initialized) {
        const info = await commands.configGetResourceRepoPath();
        set({
          initStatus: 'ready',
          currentRepoName: info.name || null,
          currentRepoPath: info.path || null,
          initialRepoPath: status.resource_repo_path,
          activePage: 'welcome',
        });
      } else {
        set({
          initStatus: 'initialize',
          currentRepoPath: status.resource_repo_path,
          initialRepoPath: status.resource_repo_path,
          activePage: 'welcome',
        });
      }
    } catch {
      set({ initStatus: 'initialize', activePage: 'welcome' });
    }
  },

  selectRepo: async () => {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === 'string') {
      try {
        const info = await commands.configSetResourceRepoPath(selected);
        const initial = get().initialRepoPath;
        const needRestart = initial !== null && info.path !== initial;
        set({
          currentRepoName: info.name || null,
          currentRepoPath: info.path || null,
          needRestart,
          initError: null,
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