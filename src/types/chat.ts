export interface Message {
  id: string;
  goal_id: string;
  role: MessageRole;
  content: string;
  metadata: string | null;
  created_at: string;
}

export type MessageRole =
  | 'user'
  | 'agent_summarizer'
  | 'agent_executor'
  | 'agent_optimizer'
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
