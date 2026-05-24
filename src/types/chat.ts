import type { SessionLine } from '../services/commands';

export type { SessionLine as Message };
export type MessageRole =
  | 'user'
  | 'assistant'
  | 'system';

export interface ChatStreamChunk {
  goal_id: string;
  message_id: string;
  delta: string;
  index: number;
}

export interface ChatStreamEnd {
  goal_id: string;
  message_id: string;
  full_content: string;
}
