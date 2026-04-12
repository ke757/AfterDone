export interface AgentStatusPayload {
  agent_type: string;
  status: string;
  phase: string;
}

export interface AgentStreamPayload {
  delta: string;
  finished: boolean;
}

export interface AgentDecisionPayload {
  decision: string;
  reasoning: string;
  data: Record<string, unknown> | null;
}

export interface AgentStatus {
  status: 'starting' | 'running' | 'canceling' | 'completed' | 'failed';
  running_agents: RunningAgent[];
}

export interface RunningAgent {
  id: string;
  agent_type: 'summarizer' | 'executor' | 'supervisor';
  goal_id: string;
  started_at: string;
}

export interface AgentLog {
  id: string;
  agent_id: string;
  goal_id: string | null;
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  metadata: Record<string, unknown> | null;
  created_at: string;
}
