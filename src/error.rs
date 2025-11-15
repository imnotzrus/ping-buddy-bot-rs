/// Custom error types for the Ping Buddy Bot
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BotError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Telegram API error: {0}")]
    Telegram(#[from] teloxide::RequestError),

    #[error("Invalid topic: {reason}")]
    InvalidTopic { reason: String },

    #[error("Topic not found: {topic}")]
    TopicNotFound { topic: String },

    #[error("Chat not found: {chat_id}")]
    ChatNotFound { chat_id: i64 },

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("User not found")]
    UserNotFound,

    #[error("Message not found")]
    MessageNotFound,

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Address parse error: {0}")]
    AddrParse(String),
}

pub type Result<T = (), E = BotError> = std::result::Result<T, E>;

/// Validation errors for topic names
#[derive(Debug, Error)]
pub enum TopicValidationError {
    #[error("Topic name is empty")]
    Empty,

    #[error("Topic name is too long (max 50 characters)")]
    TooLong,

    #[error("Topic name contains invalid characters (only alphanumeric and underscore allowed)")]
    InvalidCharacters,

    #[error("Topic name must start with a letter")]
    InvalidStart,

    #[error("Topic name is reserved: {0}")]
    Reserved(String),
}

