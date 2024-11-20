use teloxide::types::{Message, MessageKind, User};

use crate::handler::helper_messages::Return;
use crate::storage::Storage;

pub mod dynamic;
pub mod r#static;

const MSG_PREFIX: char = '/';

pub trait ChatIdExtractImpl {
  fn cid(&self) -> i64;
}

impl ChatIdExtractImpl for Message {
  fn cid(&self) -> i64 {
    self.chat.id.0
  }
}

pub trait TopicExtractImpl {
  fn topic(&self) -> Option<&str>;
}

impl TopicExtractImpl for Message {
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

pub trait UserRefExtractImpl {
  fn user_ref(&self) -> String;
}

impl UserRefExtractImpl for User {
  fn user_ref(&self) -> String {
    let user_name =
      self.username.as_deref().unwrap_or(self.first_name.as_str());
    format!("[{user_name}](tg://user?id={})", self.id.0)
  }
}

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
  let result = match data.get_subscribers_from_topic(msg.cid(), topic.as_ref())
  {
    Some(users) => {
      let users = users.collect::<Vec<_>>();
      if users.is_empty() {
        Return::no_one(topic.as_ref())
      } else {
        let users = users
          .into_iter()
          .filter(|u| *u != user.as_ref())
          .collect::<Vec<_>>();
        if users.is_empty() {
          Return::no_one_except_sender(topic.as_ref())
        } else {
          Return::users(users)
        }
      }
    }
    None => Return::no_one(topic.as_ref()),
  };
  result
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
