//! Error types and result aliases.
//!
//! This module defines all error types used throughout the application,
//! providing detailed context for debugging and user-friendly messages.

use thiserror::Error;

/// Main error type for the bot application.
///
/// This enum encompasses all possible errors that can occur during bot operation,
/// providing detailed context for debugging and user feedback.
#[derive(Debug, Error)]
pub enum BotError {
  /// Storage operation failed
  #[error("Storage error in chat {chat_id}: {message}")]
  Storage {
    /// Chat ID where the error occurred
    chat_id: i64,
    /// Detailed error message
    message: String,
  },

  /// Telegram API request failed
  #[error("Telegram API error: {0}")]
  Telegram(#[from] teloxide::RequestError),

  /// Topic name validation failed
  #[error("Invalid topic: {reason}")]
  InvalidTopic {
    /// Reason for validation failure
    reason: String,
  },

  /// Requested topic does not exist
  #[error("Topic '{topic}' not found")]
  TopicNotFound {
    /// Name of the missing topic
    topic: String,
  },

  /// Chat not found in storage
  #[error("Chat not found: {chat_id}")]
  ChatNotFound {
    /// ID of the missing chat
    chat_id: i64,
  },

  /// Environment variable error
  #[error("Environment variable error: {0}")]
  EnvVar(#[from] std::env::VarError),

  /// URL parsing error
  #[error("URL parse error: {0}")]
  UrlParse(#[from] url::ParseError),

  /// Configuration error
  #[error("Invalid configuration: {0}")]
  Config(String),

  /// User not found in message
  #[error("User not found")]
  UserNotFound,

  /// Message not found
  #[error("Message not found")]
  MessageNotFound,

  /// Operation was cancelled
  #[error("Operation cancelled")]
  Cancelled,

  /// Address parsing error
  #[error("Address parse error: {0}")]
  AddrParse(String),

  /// Maximum topics per chat exceeded
  #[error("Topic limit reached: {limit} topics in chat {chat_id}")]
  TopicLimitReached {
    /// Maximum allowed topics
    limit: usize,
    /// Chat ID where limit was reached
    chat_id: i64,
  },

  /// Maximum subscribers per topic exceeded
  #[error("Subscriber limit reached: {limit} subscribers for topic '{topic}'")]
  SubscriberLimitReached {
    /// Maximum allowed subscribers
    limit: usize,
    /// Topic name where limit was reached
    topic: String,
  },

  /// Database operation error
  #[error("Database error: {0}")]
  Database(String),
}

/// Result type alias using `BotError` as the error type.
pub type Result<T = (), E = BotError> = std::result::Result<T, E>;

/// Validation errors for topic names.
///
/// These errors are returned when a topic name fails validation checks.
#[derive(Debug, Error)]
pub enum TopicValidationError {
  /// Topic name is empty
  #[error("Topic name is empty")]
  Empty,

  /// Topic name exceeds maximum length
  #[error("Topic name is too long (max {max} characters, got {actual})")]
  TooLong {
    /// Maximum allowed length
    max: usize,
    /// Actual length provided
    actual: usize,
  },

  /// Topic name contains invalid characters
  #[error("Topic name contains invalid characters (only alphanumeric and underscore allowed)")]
  InvalidCharacters,

  /// Topic name doesn't start with a letter
  #[error("Topic name must start with a letter")]
  InvalidStart,

  /// Topic name is reserved and cannot be used
  #[error("Topic name is reserved: {0}")]
  Reserved(String),
}
