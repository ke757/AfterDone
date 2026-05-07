/**
 * Stores 模块入口
 * 导出所有 Zustand stores
 */
export { useGoalStore } from './goalStore';
export { useChatStore, initChatListeners, cleanupChatListeners } from './chatStore';
export { useSettingsStore } from './settingsStore';
export { useAgentStore, initAgentListeners, cleanupAgentListeners } from './agentStore';
export {
  useWorkSpaceStore,
  useWorkNodeStore,
  buildWorkNodeTree,
} from './noderepoStore';

// 从各模块导入函数
import { initChatListeners, cleanupChatListeners } from './chatStore';
import { initAgentListeners, cleanupAgentListeners } from './agentStore';

// 统一初始化所有监听器
export function initAllListeners() {
  initChatListeners();
  initAgentListeners();
}

// 统一清理所有监听器
export function cleanupAllListeners() {
  cleanupChatListeners();
  cleanupAgentListeners();
}
