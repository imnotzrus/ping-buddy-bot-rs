//! In-memory cache for fast access to subscription data.
//!
//! This module provides a thread-safe in-memory cache that mirrors
//! the database state for faster read operations.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory cache for subscription data
///
/// Structure:
/// - chat_id -> topic -> Set<user_id>
#[derive(Debug, Clone, Default)]
pub struct SubscriptionCache {
  /// Maps (chat_id, topic) -> Set of user_ids
  data: Arc<RwLock<HashMap<i64, HashMap<String, HashSet<String>>>>>,
}

impl SubscriptionCache {
  /// Create a new empty cache
  pub fn new() -> Self {
    Self {
      data: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  /// Get all topics for a chat
  pub async fn get_topics(&self, chat_id: i64) -> Option<Vec<String>> {
    let cache = self.data.read().await;
    cache.get(&chat_id).map(|topics| {
      let mut topic_list: Vec<String> = topics.keys().cloned().collect();
      topic_list.sort();
      topic_list
    })
  }

  /// Get all subscribers for a specific topic in a chat
  pub async fn get_subscribers_from_topic(
    &self,
    chat_id: i64,
    topic: &str,
  ) -> Option<Vec<String>> {
    let cache = self.data.read().await;
    cache.get(&chat_id).and_then(|topics| {
      topics.get(topic).map(|users| {
        let mut user_list: Vec<String> = users.iter().cloned().collect();
        user_list.sort();
        user_list
      })
    })
  }

  /// Get all topics a user is subscribed to in a chat
  pub async fn get_topics_from_subscriber(
    &self,
    chat_id: i64,
    user_id: &str,
  ) -> Option<Vec<String>> {
    let cache = self.data.read().await;
    cache.get(&chat_id).map(|topics| {
      let mut user_topics: Vec<String> = topics
        .iter()
        .filter(|(_, users)| users.contains(user_id))
        .map(|(topic, _)| topic.clone())
        .collect();
      user_topics.sort();
      if user_topics.is_empty() {
        return None;
      }
      Some(user_topics)
    })?
  }

  /// Check if a topic exists in a chat
  #[allow(dead_code)]
  pub async fn does_group_have_topic(&self, chat_id: i64, topic: &str) -> bool {
    let cache = self.data.read().await;
    cache
      .get(&chat_id)
      .map(|topics| topics.contains_key(topic))
      .unwrap_or(false)
  }

  /// Check if a user is subscribed to a topic
  pub async fn is_subscriber_in_topic(
    &self,
    chat_id: i64,
    user_id: &str,
    topic: &str,
  ) -> bool {
    let cache = self.data.read().await;
    cache
      .get(&chat_id)
      .and_then(|topics| topics.get(topic))
      .map(|users| users.contains(user_id))
      .unwrap_or(false)
  }

  /// Add a subscription to the cache
  pub async fn add_subscription(&self, chat_id: i64, topic: &str, user_id: &str) {
    let mut cache = self.data.write().await;
    cache
      .entry(chat_id)
      .or_insert_with(HashMap::new)
      .entry(topic.to_string())
      .or_insert_with(HashSet::new)
      .insert(user_id.to_string());
  }

  /// Add multiple subscriptions to the cache
  pub async fn add_subscriptions(
    &self,
    chat_id: i64,
    topic: &str,
    user_ids: &[&str],
  ) {
    let mut cache = self.data.write().await;
    let topic_map = cache.entry(chat_id).or_insert_with(HashMap::new);
    let user_set = topic_map
      .entry(topic.to_string())
      .or_insert_with(HashSet::new);

    for user_id in user_ids {
      user_set.insert(user_id.to_string());
    }
  }

  /// Remove a subscription from the cache
  pub async fn remove_subscription(
    &self,
    chat_id: i64,
    topic: &str,
    user_id: &str,
  ) -> bool {
    let mut cache = self.data.write().await;
    if let Some(topics) = cache.get_mut(&chat_id) {
      if let Some(users) = topics.get_mut(topic) {
        let removed = users.remove(user_id);

        // Clean up empty structures
        if users.is_empty() {
          topics.remove(topic);
        }
        if topics.is_empty() {
          cache.remove(&chat_id);
        }

        return removed;
      }
    }
    false
  }

  /// Load all subscriptions from database into cache
  pub async fn warm_from_db(
    &self,
    subscriptions: Vec<(i64, String, String)>,
  ) {
    let mut cache = self.data.write().await;
    cache.clear();

    for (chat_id, topic, user_id) in subscriptions {
      cache
        .entry(chat_id)
        .or_insert_with(HashMap::new)
        .entry(topic)
        .or_insert_with(HashSet::new)
        .insert(user_id);
    }

    log::info!("Cache warmed with subscriptions from database");
  }

  /// Get cache statistics
  #[allow(dead_code)]
  pub async fn get_stats(&self) -> CacheStats {
    let cache = self.data.read().await;
    let total_chats = cache.len();
    let mut total_topics = 0;
    let mut total_subscriptions = 0;

    for topics in cache.values() {
      total_topics += topics.len();
      for users in topics.values() {
        total_subscriptions += users.len();
      }
    }

    CacheStats {
      total_chats,
      total_topics,
      total_subscriptions,
    }
  }

  /// Clear the entire cache
  #[allow(dead_code)]
  pub async fn clear(&self) {
    let mut cache = self.data.write().await;
    cache.clear();
  }
}

/// Cache statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CacheStats {
  pub total_chats: usize,
  pub total_topics: usize,
  pub total_subscriptions: usize,
}

