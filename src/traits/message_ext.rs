//! Extension traits for Message types.

use teloxide::types::Message;

use crate::constants::MSG_PREFIX;

/// Extract chat ID from a message
pub trait ChatIdExt {
  /// Get the chat ID as i64
  fn cid(&self) -> i64;
}

impl ChatIdExt for Message {
  fn cid(&self) -> i64 {
    self.chat.id.0
  }
}

/// Extract topic name from a message
pub trait TopicExt {
  /// Extract topic name from message text (if it starts with MSG_PREFIX)
  fn topic(&self) -> Option<&str>;
}

impl TopicExt for Message {
  fn topic(&self) -> Option<&str> {
    let text = self.text()?;
    if text.starts_with(MSG_PREFIX) {
      text
        .split_whitespace()
        .next()
        .map(|t| t.trim_start_matches(MSG_PREFIX))
    } else {
      None
    }
  }
}
