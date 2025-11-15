/// Storage module for managing topic subscriptions across chats
use std::sync::Arc;

use hashbrown::{HashMap, HashSet};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::constants::{MAX_SUBSCRIBERS_PER_TOPIC, MAX_TOPICS_PER_CHAT};
use crate::error::{BotError, Result};

type Map = HashMap<String, HashSet<String>>;

/// Thread-safe storage wrapper for chat data
#[derive(Default, Clone)]
pub struct Storage(Arc<RwLock<ChatStorage>>);

impl Storage {
  /// Get read access to the storage
  pub async fn read(&self) -> RwLockReadGuard<'_, ChatStorage> {
    self.0.read().await
  }

  /// Get write access to the storage
  pub async fn write(&self) -> RwLockWriteGuard<'_, ChatStorage> {
    self.0.write().await
  }

  /// Try to get read access without blocking
  pub fn try_read(&self) -> Option<RwLockReadGuard<'_, ChatStorage>> {
    self.0.try_read().ok()
  }
}

/// Data structure for a single chat group
#[derive(Default, Debug)]
struct Group {
  /// Maps user_id -> set of topics they're subscribed to
  user_topics: Map,
  /// Maps topic -> set of user_ids subscribed to it
  topic_users: Map,
}

/// Main storage structure managing all chat data
#[derive(Default)]
pub struct ChatStorage {
  /// Maps chat_id -> Group data
  chats: HashMap<i64, Group>,
  /// Tracks message IDs that are waiting for topic creation responses
  creating_topics: HashMap<i64, HashSet<i32>>,
}

/// Represents topic and subscriber information for a chat
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChatTopics {
  pub topics: Vec<TopicInfo>,
  pub subscribers: Vec<SubscriberInfo>,
}

/// Information about a topic and its subscribers
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TopicInfo {
  pub name: String,
  pub subscribers: Vec<String>,
}

/// Information about a subscriber and their topics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SubscriberInfo {
  pub user_id: String,
  pub topics: Vec<String>,
}

// Read operations
impl ChatStorage {
  /// Get all chat data
  #[allow(dead_code)]
  pub fn get_all_chats(&self) -> HashMap<i64, ChatTopics> {
    self
      .chats
      .iter()
      .map(|(chat_id, group)| (*chat_id, self.group_to_chat_topics(group)))
      .collect()
  }

  /// Get complete chat information
  #[allow(dead_code)]
  pub fn get_chat(&self, chat_id: i64) -> Option<ChatTopics> {
    self
      .chats
      .get(&chat_id)
      .map(|group| self.group_to_chat_topics(group))
  }

  /// Get all topics in a chat
  pub fn get_topics(&self, chat_id: i64) -> Option<Vec<String>> {
    self
      .chats
      .get(&chat_id)
      .map(|Group { topic_users, .. }| topic_users.keys().cloned().collect())
  }

  /// Get all subscribers in a chat
  #[allow(dead_code)]
  pub fn get_subscribers(&self, chat_id: i64) -> Option<Vec<String>> {
    self
      .chats
      .get(&chat_id)
      .map(|Group { user_topics, .. }| user_topics.keys().cloned().collect())
  }

  /// Get subscribers for a specific topic
  pub fn get_subscribers_from_topic<T>(
    &self,
    chat_id: i64,
    topic: T,
  ) -> Option<Vec<String>>
  where
    T: AsRef<str>,
  {
    self
      .chats
      .get(&chat_id)
      .and_then(|Group { topic_users, .. }| {
        topic_users
          .get(topic.as_ref())
          .map(|users| users.iter().cloned().collect())
      })
  }

  /// Get topics for a specific subscriber
  pub fn get_topics_from_subscriber<U>(
    &self,
    chat_id: i64,
    user: U,
  ) -> Option<Vec<String>>
  where
    U: AsRef<str>,
  {
    self
      .chats
      .get(&chat_id)
      .and_then(|Group { user_topics, .. }| {
        user_topics
          .get(user.as_ref())
          .map(|topics| topics.iter().cloned().collect())
      })
  }

  /// Check if a chat has a specific topic
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

  /// Check if a user is subscribed to a topic
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

  /// Check if a message ID is waiting for topic creation
  pub fn has_create_message_id(&self, chat_id: i64, msg_id: i32) -> bool {
    self
      .creating_topics
      .get(&chat_id)
      .map(|messages| messages.contains(&msg_id))
      .unwrap_or_default()
  }

  /// Helper to convert Group to ChatTopics
  #[allow(dead_code)]
  fn group_to_chat_topics(&self, group: &Group) -> ChatTopics {
    let topics = group
      .topic_users
      .iter()
      .map(|(name, subscribers)| TopicInfo {
        name: name.clone(),
        subscribers: subscribers.iter().cloned().collect(),
      })
      .collect();

    let subscribers = group
      .user_topics
      .iter()
      .map(|(user_id, topics)| SubscriberInfo {
        user_id: user_id.clone(),
        topics: topics.iter().cloned().collect(),
      })
      .collect();

    ChatTopics {
      topics,
      subscribers,
    }
  }
}

// Write operations
impl ChatStorage {
  /// Subscribe users to a topic, creating the topic if it doesn't exist
  pub fn set_topic_and_subscribers<T, L, U>(
    &mut self,
    chat_id: i64,
    topic: T,
    users: L,
  ) -> Result<()>
  where
    T: ToString,
    U: ToString,
    L: AsRef<[U]>,
  {
    let topic_str = topic.to_string();
    let users_slice = users.as_ref();

    // Check limits
    let group = self.chats.entry(chat_id).or_default();

    if group.topic_users.len() >= MAX_TOPICS_PER_CHAT
      && !group.topic_users.contains_key(&topic_str)
    {
      return Err(BotError::Config(format!(
        "Maximum number of topics ({}) reached",
        MAX_TOPICS_PER_CHAT
      )));
    }

    let topic_subscribers =
      group.topic_users.entry(topic_str.clone()).or_default();

    if topic_subscribers.len() + users_slice.len() > MAX_SUBSCRIBERS_PER_TOPIC {
      return Err(BotError::Config(format!(
        "Maximum number of subscribers ({}) would be exceeded",
        MAX_SUBSCRIBERS_PER_TOPIC
      )));
    }

    // Add subscribers to topic
    topic_subscribers.extend(users_slice.iter().map(ToString::to_string));

    // Add topic to each user's subscriptions
    for user in users_slice {
      group
        .user_topics
        .entry(user.to_string())
        .or_default()
        .insert(topic_str.clone());
    }

    log::info!(
      "Added {} subscribers to topic '{}' in chat {}",
      users_slice.len(),
      topic_str,
      chat_id
    );

    Ok(())
  }

  /// Unsubscribe a user from a topic
  pub fn unset_subscriber_from_topic<T, U>(
    &mut self,
    chat_id: i64,
    topic: T,
    user_id: U,
  ) -> Result<()>
  where
    T: AsRef<str>,
    U: AsRef<str>,
  {
    let topic_ref = topic.as_ref();
    let user_ref = user_id.as_ref();

    if let Some(Group {
      topic_users,
      user_topics,
    }) = self.chats.get_mut(&chat_id)
    {
      if let Some(users) = topic_users.get_mut(topic_ref) {
        users.remove(user_ref);

        // Clean up empty topic
        if users.is_empty() {
          topic_users.remove(topic_ref);
        }
      }

      if let Some(topics) = user_topics.get_mut(user_ref) {
        topics.remove(topic_ref);

        // Clean up user with no subscriptions
        if topics.is_empty() {
          user_topics.remove(user_ref);
        }
      }

      log::info!(
        "Removed user '{}' from topic '{}' in chat {}",
        user_ref,
        topic_ref,
        chat_id
      );
    }

    Ok(())
  }

  /// Mark a message as waiting for topic creation response
  pub fn push_creating_message_id(&mut self, chat_id: i64, msg_id: i32) {
    self
      .creating_topics
      .entry(chat_id)
      .or_default()
      .insert(msg_id);

    log::debug!(
      "Marked message {} in chat {} as awaiting topic creation",
      msg_id,
      chat_id
    );
  }

  /// Remove a message from the topic creation waiting list
  pub fn pop_creating_message_id(&mut self, chat_id: i64, msg_id: i32) {
    if let Some(msgs) = self.creating_topics.get_mut(&chat_id) {
      msgs.remove(&msg_id);

      // Clean up empty entry
      if msgs.is_empty() {
        self.creating_topics.remove(&chat_id);
      }
    }

    log::debug!(
      "Removed message {} from chat {} topic creation queue",
      msg_id,
      chat_id
    );
  }

  /// Clean up empty chats (for maintenance)
  #[allow(dead_code)]
  pub fn cleanup_empty_chats(&mut self) -> usize {
    let before = self.chats.len();
    self.chats.retain(|_, group| {
      !group.topic_users.is_empty() || !group.user_topics.is_empty()
    });
    let removed = before - self.chats.len();

    if removed > 0 {
      log::info!("Cleaned up {} empty chats", removed);
    }

    removed
  }
}
