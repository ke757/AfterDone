use serde::Serialize;

pub type AppResult<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Cancelled: {0}")]
    Cancelled(String),
}

impl From<toml::de::Error> for AppError {
    fn from(e: toml::de::Error) -> Self {
        AppError::Toml(e.to_string())
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(e: toml::ser::Error) -> Self {
        AppError::Toml(e.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct ErrorPayload {
            kind: String,
            message: String,
        }

        let kind = match self {
            AppError::Database(_) => "database",
            AppError::Config(_) => "config",
            AppError::Llm(_) => "llm",
            AppError::Transport(_) => "transport",
            AppError::Agent(_) => "agent",
            AppError::Validation(_) => "validation",
            AppError::NotFound(_) => "not_found",
            AppError::Io(_) => "io",
            AppError::Serialization(_) => "serialization",
            AppError::Toml(_) => "toml",
            AppError::Internal(_) => "internal",
            AppError::Cancelled(_) => "cancelled",
        };

        ErrorPayload {
            kind: kind.to_string(),
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}
