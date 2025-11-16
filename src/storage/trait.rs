//! Storage trait definitions and related types.
//!
//! This module defines the `StorageBackend` trait that all storage
//! implementations must satisfy, along with supporting types.

use crate::error::Result;
use async_trait::async_trait;

/// Trait defining storage operations for the bot
/// All storage backends must implement this trait
#[async_trait]
pub trait StorageBackend: Send + Sync {
  /// Get all topics in a chat
  async fn get_topics(&self, chat_id: i64) -> Result<Option<Vec<String>>>;

  /// Get all subscribers in a chat
  #[allow(dead_code)]
  async fn get_subscribers(&self, chat_id: i64) -> Result<Option<Vec<String>>>;

  /// Get subscribers for a specific topic
  async fn get_subscribers_from_topic(
    &self,
    chat_id: i64,
    topic: &str,
  ) -> Result<Option<Vec<String>>>;

  /// Get topics for a specific subscriber
  async fn get_topics_from_subscriber(
    &self,
    chat_id: i64,
    user_id: &str,
  ) -> Result<Option<Vec<String>>>;

  /// Check if a chat has a specific topic
  #[allow(dead_code)]
  async fn does_group_have_topic(
    &self,
    chat_id: i64,
    topic: &str,
  ) -> Result<bool>;

  /// Check if a user is subscribed to a topic
  async fn is_subscriber_in_topic(
    &self,
    chat_id: i64,
    user_id: &str,
    topic: &str,
  ) -> Result<bool>;

  /// Subscribe users to a topic, creating the topic if it doesn't exist
  /// Returns error if limits are exceeded
  async fn set_topic_and_subscribers(
    &mut self,
    chat_id: i64,
    topic: &str,
    users: &[&str],
  ) -> Result<()>;

  /// Unsubscribe a user from a topic
  /// Automatically removes topic if no subscribers remain
  async fn unset_subscriber_from_topic(
    &mut self,
    chat_id: i64,
    topic: &str,
    user_id: &str,
  ) -> Result<()>;

  /// Mark a message as waiting for topic creation response
  async fn push_creating_message_id(
    &mut self,
    chat_id: i64,
    msg_id: i32,
  ) -> Result<()>;

  /// Remove a message from the topic creation waiting list
  async fn pop_creating_message_id(
    &mut self,
    chat_id: i64,
    msg_id: i32,
  ) -> Result<()>;

  /// Check if a message ID is waiting for topic creation
  async fn has_create_message_id(
    &self,
    chat_id: i64,
    msg_id: i32,
  ) -> Result<bool>;

  /// Clean up old pending topic creation requests (older than 1 hour)
  #[allow(dead_code)]
  async fn cleanup_old_pending_requests(&mut self) -> Result<usize>;

  /// Get statistics about storage usage
  #[allow(dead_code)]
  async fn get_stats(&self) -> Result<StorageStats>;

  /// Perform a health check on the storage backend
  ///
  /// Returns `Ok(())` if the storage is healthy and accessible,
  /// otherwise returns an error with details about the problem.
  async fn health_check(&self) -> Result<()>;
}

/// Statistics about storage usage
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StorageStats {
  pub total_chats: usize,
  pub total_topics: usize,
  pub total_subscriptions: usize,
  pub pending_requests: usize,
}
