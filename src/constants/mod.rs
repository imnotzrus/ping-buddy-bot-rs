//! Application-wide constants and configuration values.
//!
//! This module defines all constant values used throughout the bot,
//! organized into logical submodules.

pub mod defaults;
pub mod limits;
pub mod reserved;
pub mod ui;

// Re-export commonly used constants
pub use defaults::{BOT_MSG_TTL, BOT_USERNAME, GENERAL_TOPIC, MSG_PREFIX};
pub use limits::{
  MAX_SUBSCRIBERS_PER_TOPIC, MAX_TOPICS_PER_CHAT, MAX_TOPIC_LENGTH,
};
pub use reserved::RESERVED_TOPICS;
pub use ui::{NEW_TOPIC_CALLBACK, TOPIC_WRAPPER};

// Re-export less commonly used constants for completeness
#[allow(unused_imports)]
pub use limits::MAX_USER_REF_LENGTH;
