//! Resource limits and constraints.

/// Maximum length for topic names
pub const MAX_TOPIC_LENGTH: usize = 50;

/// Maximum number of topics per chat
pub const MAX_TOPICS_PER_CHAT: usize = 100;

/// Maximum number of subscribers per topic
pub const MAX_SUBSCRIBERS_PER_TOPIC: usize = 1000;

/// Maximum length for user references
#[allow(dead_code)]
pub const MAX_USER_REF_LENGTH: usize = 200;
