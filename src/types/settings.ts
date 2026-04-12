export interface OpenClawConfig {
  gateway_url: string;
  auth_token: string;
  auto_connect: boolean;
  request_timeout_secs: number;
  device_identity: DeviceIdentity;
}

export interface DeviceIdentity {
  fingerprint: string;
  public_key: string;
  signature: string;
}

export interface LlmConfig {
  provider: LlmProviderType;
  model: string;
  api_key: string;
  base_url: string;
  max_tokens: number;
  temperature: number;
  streaming: boolean;
}

export type LlmProviderType = 'openai' | 'anthropic' | 'local';

export interface AppSettings {
  database_path: string;
  max_concurrent_agents: number;
  log_level: string;
}

export interface ConnectionTestResult {
  success: boolean;
  message: string;
  latency_ms: number | null;
}

export type ConnectionStatus =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'error';
