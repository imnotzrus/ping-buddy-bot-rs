/// Message handler utilities and traits
use teloxide::types::{Message, MessageKind, User};

use crate::constants::MSG_PREFIX;
use crate::handler::helper_messages::Return;
use crate::storage::Storage;

pub mod dynamic;
pub mod r#static;

/// Extract chat ID from a message
pub trait ChatIdExtractImpl {
    fn cid(&self) -> i64;
}

impl ChatIdExtractImpl for Message {
    fn cid(&self) -> i64 {
        self.chat.id.0
    }
}

/// Extract topic name from a message
pub trait TopicExtractImpl {
    fn topic(&self) -> Option<&str>;
}

impl TopicExtractImpl for Message {
    fn topic(&self) -> Option<&str> {
        let text = self.text()?;
        if text.starts_with(MSG_PREFIX) {
            text.split_whitespace()
                .next()
                .map(|t| t.trim_start_matches(MSG_PREFIX))
        } else {
            None
        }
    }
}

/// Create a Telegram user reference (clickable mention)
pub trait UserRefExtractImpl {
    fn user_ref(&self) -> String;
}

impl UserRefExtractImpl for User {
    fn user_ref(&self) -> String {
        let user_name = self.username.as_deref().unwrap_or(self.first_name.as_str());
        format!("[{user_name}](tg://user?id={})", self.id.0)
    }
}

/// List all users subscribed to a topic, excluding the requesting user
/// 
/// Returns appropriate message based on subscription status
async fn list_users<T, U>(
    storage: Storage,
    msg: &Message,
    topic: T,
    user: U,
) -> Return
where
    T: AsRef<str>,
    U: AsRef<str>,
{
    let data = storage.read().await;
    let topic_ref = topic.as_ref();
    let user_ref = user.as_ref();

    match data.get_subscribers_from_topic(msg.cid(), topic_ref) {
        Some(users) => {
            if users.is_empty() {
                Return::no_one(topic_ref)
            } else {
                // Filter out the requesting user
                let other_users: Vec<_> = users
                    .into_iter()
                    .filter(|u| u != user_ref)
                    .collect();

                if other_users.is_empty() {
                    Return::no_one_except_sender(topic_ref)
                } else {
                    Return::users(other_users)
                }
            }
        }
        None => Return::no_one(topic_ref),
    }
}

#[allow(dead_code)]
pub fn is_topic_or_new_member(msg: Message, storage: Storage) -> bool {
  match msg.kind {
    MessageKind::NewChatMembers(_) => msg
      .new_chat_members()
      .map(|l| !l.is_empty())
      .unwrap_or_default(),
    MessageKind::Common(_) => {
      let (Some(topic), Some(data)) = (msg.topic(), storage.try_read()) else {
        return false;
      };
      data.does_group_have_topic(msg.cid(), topic)
    }
    _ => false,
  }
}
