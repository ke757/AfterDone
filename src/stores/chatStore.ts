/**
 * Chat Store
 * 管理聊天消息状态
 */
import { create } from 'zustand';
import type { Message } from '../types/chat';
import * as commands from '../services/commands';
import { onChatStream, onNewMessage } from '../services/events';

interface ChatState {
  messages: Message[];
  isStreaming: boolean;
  streamingContent: string;
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchHistory: (goalId: string) => Promise<void>;
  sendMessage: (goalId: string, content: string) => Promise<void>;
  startStreaming: () => void;
  appendStreamChunk: (chunk: string) => void;
  endStreaming: () => void;
  clearMessages: () => void;
  clearError: () => void;
}

export const useChatStore = create<ChatState>((set) => ({
  messages: [],
  isStreaming: false,
  streamingContent: '',
  isLoading: false,
  error: null,

  fetchHistory: async (goalId: string) => {
    set({ isLoading: true, error: null });
    try {
      const messages = await commands.getChatHistory(goalId);
      set({ messages, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  sendMessage: async (goalId: string, content: string) => {
    set({ isLoading: true, error: null });
    try {
      const message = await commands.sendMessage(goalId, content);
      set((state) => ({
        messages: [...state.messages, message],
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  startStreaming: () => {
    set({ isStreaming: true, streamingContent: '' });
  },

  appendStreamChunk: (chunk: string) => {
    set((state) => ({
      streamingContent: state.streamingContent + chunk,
    }));
  },

  endStreaming: () => {
    set({ isStreaming: false, streamingContent: '' });
  },

  clearMessages: () => {
    set({ messages: [] });
  },

  clearError: () => {
    set({ error: null });
  },
}));

// 初始化事件监听
let unlistenStream: (() => void) | null = null;
let unlistenMessage: (() => void) | null = null;

export function initChatListeners() {
  if (unlistenStream) return;

  onChatStream((event) => {
    const store = useChatStore.getState();
    if (!store.isStreaming) {
      store.startStreaming();
    }
    store.appendStreamChunk(event.chunk);
    if (event.is_final) {
      store.endStreaming();
    }
  }).then((fn) => {
    unlistenStream = fn;
  });

  onNewMessage((message) => {
    useChatStore.setState((state) => ({
      messages: [...state.messages, message],
    }));
  }).then((fn) => {
    unlistenMessage = fn;
  });
}

export function cleanupChatListeners() {
  if (unlistenStream) {
    unlistenStream();
    unlistenStream = null;
  }
  if (unlistenMessage) {
    unlistenMessage();
    unlistenMessage = null;
  }
}
