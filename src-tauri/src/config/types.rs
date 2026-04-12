use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub openclaw: OpenClawConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub app: AppSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            openclaw: OpenClawConfig::default(),
            llm: LlmConfig::default(),
            app: AppSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawConfig {
    #[serde(default = "default_gateway_url")]
    pub gateway_url: String,
    #[serde(default)]
    pub auth_token: String,
    #[serde(default)]
    pub auto_connect: bool,
    #[serde(default = "default_timeout")]
    pub request_timeout_secs: u64,
    #[serde(default)]
    pub device_identity: DeviceIdentity,
}

fn default_gateway_url() -> String {
    "ws://localhost:9090/gateway".to_string()
}

fn default_timeout() -> u64 {
    30
}

impl Default for OpenClawConfig {
    fn default() -> Self {
        Self {
            gateway_url: default_gateway_url(),
            auth_token: String::new(),
            auto_connect: false,
            request_timeout_secs: default_timeout(),
            device_identity: DeviceIdentity::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceIdentity {
    #[serde(default)]
    pub fingerprint: String,
    #[serde(default)]
    pub public_key: String,
    #[serde(default)]
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_llm_provider")]
    pub provider: LlmProviderType,
    #[serde(default = "default_llm_model")]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_streaming")]
    pub streaming: bool,
}

fn default_llm_provider() -> LlmProviderType {
    LlmProviderType::OpenAi
}

fn default_llm_model() -> String {
    "gpt-4o".to_string()
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_temperature() -> f32 {
    0.7
}

fn default_streaming() -> bool {
    true
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_llm_provider(),
            model: default_llm_model(),
            api_key: String::new(),
            base_url: String::new(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            streaming: default_streaming(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProviderType {
    OpenAi,
    Anthropic,
    Local,
}

impl std::fmt::Display for LlmProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmProviderType::OpenAi => write!(f, "openai"),
            LlmProviderType::Anthropic => write!(f, "anthropic"),
            LlmProviderType::Local => write!(f, "local"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub database_path: String,
    #[serde(default = "default_max_concurrent_agents")]
    pub max_concurrent_agents: u32,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_max_concurrent_agents() -> u32 {
    3
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            database_path: String::new(),
            max_concurrent_agents: default_max_concurrent_agents(),
            log_level: default_log_level(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub latency_ms: Option<u64>,
}
