/// Default topic name for new groups
pub const GENERAL_TOPIC: &str = "general";

/// Bot username for self-identification
pub const BOT_USERNAME: &str = "PingBuddyBot";

/// Time to live for bot messages before auto-deletion (in seconds)
pub const BOT_MSG_TTL: u64 = 30;

/// Maximum length for topic names
pub const MAX_TOPIC_LENGTH: usize = 50;

/// Maximum number of topics per chat
pub const MAX_TOPICS_PER_CHAT: usize = 100;

/// Maximum number of subscribers per topic
pub const MAX_SUBSCRIBERS_PER_TOPIC: usize = 1000;

/// Character used to wrap topic names in callbacks
pub const TOPIC_WRAPPER: char = '#';

/// Callback data for creating a new topic
pub(crate) const NEW_TOPIC_CALLBACK: &str = "new_topic";

/// Message prefix for topic commands
pub const MSG_PREFIX: char = '/';

/// Reserved topic names that cannot be used
pub const RESERVED_TOPICS: &[&str] = &["all", "everyone", "here", "channel"];

/// Maximum length for user references
#[allow(dead_code)]
pub const MAX_USER_REF_LENGTH: usize = 200;
