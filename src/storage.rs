use std::sync::Arc;

use hashbrown::{HashMap, HashSet};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

type Map = HashMap<String, HashSet<String>>;

#[derive(Default, Clone)]
pub struct Storage(Arc<RwLock<ListGroup>>);

impl Storage {
  pub async fn read(&self) -> RwLockReadGuard<ListGroup> {
    self.0.read().await
  }

  pub async fn write(&self) -> RwLockWriteGuard<ListGroup> {
    self.0.write().await
  }

  #[allow(dead_code)]
  pub fn try_read(&self) -> Option<RwLockReadGuard<ListGroup>> {
    self.0.try_read().ok()
  }
}

#[derive(Default)]
struct Group {
  user_topics: Map,
  topic_users: Map,
}

#[derive(Default)]
pub struct ListGroup {
  chats: HashMap<i64, Group>,
  creating_topics: HashMap<i64, HashSet<i32>>,
}

#[allow(dead_code)]
impl ListGroup {
  #[deprecated]
  #[inline]
  pub fn get_all_chat(
    &self,
  ) -> impl Iterator<
    Item = (
      i64,
      (
        impl Iterator<Item = (&str, impl Iterator<Item = &str>)>,
        impl Iterator<Item = (&str, impl Iterator<Item = &str>)>,
      ),
    ),
  > {
    self.chats.iter().map(
      |(
        id,
        Group {
          topic_users,
          user_topics,
        },
      )| {
        (
          *id,
          (
            topic_users.iter().map(|(topic, users)| {
              (topic.as_str(), users.iter().map(|user| user.as_str()))
            }),
            user_topics.iter().map(|(user, topics)| {
              (user.as_str(), topics.iter().map(|topic| topic.as_str()))
            }),
          ),
        )
      },
    )
  }

  #[inline]
  pub fn get_chat(
    &self,
    chat_id: i64,
  ) -> Option<(
    impl Iterator<Item = (&str, impl Iterator<Item = &str>)>,
    impl Iterator<Item = (&str, impl Iterator<Item = &str>)>,
  )> {
    self.chats.get(&chat_id).map(
      |Group {
         topic_users,
         user_topics,
       }| {
        (
          topic_users.iter().map(|(topic, users)| {
            (topic.as_str(), users.iter().map(|user| user.as_str()))
          }),
          user_topics.iter().map(|(user, topics)| {
            (user.as_str(), topics.iter().map(|topic| topic.as_str()))
          }),
        )
      },
    )
  }

  #[inline]
  pub fn get_topics_and_subscribers(
    &self,
    chat_id: i64,
  ) -> Option<impl Iterator<Item = (&str, impl Iterator<Item = &str>)>> {
    self.chats.get(&chat_id).map(|Group { topic_users, .. }| {
      topic_users.iter().map(|(topic, users)| {
        (topic.as_str(), users.iter().map(|user| user.as_str()))
      })
    })
  }

  #[inline]
  pub fn get_subscribers_and_topics(
    &self,
    chat_id: i64,
  ) -> Option<impl Iterator<Item = (&str, impl Iterator<Item = &str>)>> {
    self.chats.get(&chat_id).map(|Group { user_topics, .. }| {
      user_topics.iter().map(|(user, topics)| {
        (user.as_str(), topics.iter().map(|user| user.as_str()))
      })
    })
  }

  #[inline]
  pub fn get_topics(&self, chat_id: i64) -> Option<impl Iterator<Item = &str>> {
    self.chats.get(&chat_id).map(|Group { topic_users, .. }| {
      topic_users.keys().map(|topic| topic.as_str())
    })
  }

  #[inline]
  pub fn get_subscribers(
    &self,
    chat_id: i64,
  ) -> Option<impl Iterator<Item = &str>> {
    self.chats.get(&chat_id).map(|Group { user_topics, .. }| {
      user_topics.keys().map(|user| user.as_str())
    })
  }

  #[inline]
  pub fn get_subscribers_from_topic<T>(
    &self,
    chat_id: i64,
    topic: T,
  ) -> Option<impl Iterator<Item = &str>>
  where
    T: AsRef<str>,
  {
    self
      .chats
      .get(&chat_id)
      .and_then(|Group { topic_users, .. }| {
        topic_users
          .get(topic.as_ref())
          .map(|users| users.iter().map(|user| user.as_str()))
      })
  }

  #[inline]
  pub fn get_topics_from_subscriber<U>(
    &self,
    chat_id: i64,
    user: U,
  ) -> Option<impl Iterator<Item = &str>>
  where
    U: AsRef<str>,
  {
    self
      .chats
      .get(&chat_id)
      .and_then(|Group { user_topics, .. }| {
        user_topics
          .get(user.as_ref())
          .map(|topics| topics.iter().map(|topic| topic.as_str()))
      })
  }

  #[inline]
  pub fn does_group_have_topic<T>(&self, chat_id: i64, topic: T) -> bool
  where
    T: AsRef<str>,
  {
    self
      .chats
      .get(&chat_id)
      .and_then(|Group { topic_users, .. }| topic_users.get(topic.as_ref()))
      .is_some()
  }

  #[inline]
  pub fn is_subscriber_in_topic<T, U>(
    &self,
    chat_id: i64,
    user_id: U,
    topic: T,
  ) -> bool
  where
    T: AsRef<str>,
    U: AsRef<str>,
  {
    self
      .chats
      .get(&chat_id)
      .and_then(|Group { topic_users, .. }| {
        topic_users
          .get(topic.as_ref())
          .and_then(|users| users.get(user_id.as_ref()))
      })
      .is_some()
  }

  pub fn has_create_message_id(&self, chat_id: i64, msg_id: i32) -> bool {
    self
      .creating_topics
      .get(&chat_id)
      .map(|messages| messages.get(&msg_id).is_some())
      .unwrap_or_default()
  }
}

impl ListGroup {
  #[inline]
  pub fn set_topic_and_subscribers<T, L, U>(
    &mut self,
    chat_id: i64,
    topic: T,
    users: L,
  ) where
    T: ToString,
    U: ToString,
    L: AsRef<[U]>,
  {
    self
      .chats
      .entry(chat_id)
      .and_modify(
        |Group {
           topic_users,
           user_topics,
         }| {
          topic_users
            .entry(topic.to_string())
            .and_modify(|subscribers| {
              subscribers
                .extend(users.as_ref().iter().map(ToString::to_string));
            })
            .or_insert(
              users.as_ref().iter().map(ToString::to_string).collect(),
            );
          for user in users.as_ref() {
            user_topics
              .entry(user.to_string())
              .and_modify(|topics| {
                topics.insert(topic.to_string());
              })
              .or_insert(HashSet::from([topic.to_string()]));
          }
        },
      )
      .or_insert(Group {
        topic_users: Map::from([(
          topic.to_string(),
          users.as_ref().iter().map(ToString::to_string).collect(),
        )]),
        user_topics: users
          .as_ref()
          .iter()
          .map(|user| (user.to_string(), HashSet::from([topic.to_string()])))
          .collect(),
      });
  }

  #[inline]
  pub fn unset_subscriber_from_topic<T, U>(
    &mut self,
    chat_id: i64,
    topic: T,
    user_id: U,
  ) where
    T: AsRef<str>,
    U: AsRef<str>,
  {
    if let Some(Group {
      topic_users,
      user_topics,
    }) = self.chats.get_mut(&chat_id)
    {
      if let Some(users) = topic_users.get_mut(topic.as_ref()) {
        users.remove(user_id.as_ref());
      }
      if let Some(topics) = user_topics.get_mut(user_id.as_ref()) {
        topics.remove(topic.as_ref());
      }
    }
  }

  #[inline]
  pub fn push_creating_message_id(&mut self, chat_id: i64, msg_id: i32) {
    self
      .creating_topics
      .entry(chat_id)
      .and_modify(|msgs| {
        msgs.insert(msg_id);
      })
      .or_insert(HashSet::from([msg_id]));
  }

  #[inline]
  pub fn pop_creating_message_id(&mut self, chat_id: i64, msg_id: i32) {
    let have_to_remove = self
      .creating_topics
      .get_mut(&chat_id)
      .map(|msgs| {
        msgs.remove(&msg_id);
        msgs.is_empty()
      })
      .unwrap_or_default();
    if have_to_remove {
      self.creating_topics.remove_entry(&chat_id);
    }
  }
}
