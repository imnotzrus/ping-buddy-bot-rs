//! Default values and bot configuration.

/// Default topic name for new groups
pub const GENERAL_TOPIC: &str = "general";

/// Bot username for self-identification
pub const BOT_USERNAME: &str = "PingBuddyBot";

/// Time to live for bot messages before auto-deletion (in seconds)
pub const BOT_MSG_TTL: u64 = 30;

/// Message prefix for topic commands
pub const MSG_PREFIX: char = '/';
