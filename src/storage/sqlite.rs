//! SQLite storage backend implementation.
//!
//! This module provides a production-ready SQLite storage backend with:
//! - Connection pooling for concurrent access
//! - Automatic migrations via sqlx
//! - WAL mode for better performance
//! - Transaction support for data integrity
//! - Comprehensive error handling

use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

use crate::constants::{
  GENERAL_TOPIC, MAX_SUBSCRIBERS_PER_TOPIC, MAX_TOPICS_PER_CHAT,
};
use crate::error::{BotError, Result};
use crate::storage::cache::SubscriptionCache;
use crate::storage::r#trait::{StorageBackend, StorageStats};

/// SQLite-based storage backend with in-memory cache
pub struct SqliteStorage {
  pool: SqlitePool,
  cache: SubscriptionCache,
}

impl SqliteStorage {
  /// Create a new SQLite storage backend
  ///
  /// # Arguments
  /// * `database_path` - Path to the SQLite database file (e.g., "bot_data.db")
  ///
  /// # Errors
  /// Returns error if database connection fails or migrations fail
  pub async fn new(database_path: &str) -> Result<Self> {
    log::info!("Initializing SQLite storage at: {}", database_path);

    // Configure SQLite connection
    let options = SqliteConnectOptions::from_str(database_path)
      .map_err(|e| BotError::Config(format!("Invalid database path: {}", e)))?
      .create_if_missing(true)
      .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
      .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

    // Create connection pool
    let pool = SqlitePoolOptions::new()
      .max_connections(5)
      .connect_with(options)
      .await
      .map_err(|e| {
        BotError::Config(format!("Failed to connect to database: {}", e))
      })?;

    // Run migrations
    log::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
      .run(&pool)
      .await
      .map_err(|e| {
        BotError::Config(format!("Failed to run migrations: {}", e))
      })?;

    // Initialize cache
    let cache = SubscriptionCache::new();

    // Warm cache from database
    log::info!("Warming cache from database...");
    let subscriptions: Vec<(i64, String, String)> = sqlx::query_as(
      "SELECT chat_id, topic, user_id FROM subscriptions ORDER BY chat_id, topic",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
      BotError::Config(format!("Failed to load subscriptions for cache: {}", e))
    })?;

    cache.warm_from_db(subscriptions).await;

    log::info!("SQLite storage initialized successfully");

    Ok(Self { pool, cache })
  }

  /// Check topic count limit for a chat
  async fn check_topic_limit(&self, chat_id: i64, topic: &str) -> Result<()> {
    let count: i64 = sqlx::query_scalar(
      "SELECT COUNT(DISTINCT topic) FROM subscriptions WHERE chat_id = ?",
    )
    .bind(chat_id)
    .fetch_one(&self.pool)
    .await
    .map_err(|e| BotError::Storage {
      chat_id,
      message: format!("Failed to count topics: {}", e),
    })?;

    // Check if topic already exists
    let topic_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM subscriptions WHERE chat_id = ? AND topic = ?)"
        )
        .bind(chat_id)
        .bind(topic)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| BotError::Storage {
            chat_id,
            message: format!("Failed to check topic existence: {}", e),
        })?;

    if !topic_exists && count >= MAX_TOPICS_PER_CHAT as i64 {
      return Err(BotError::TopicLimitReached {
        limit: MAX_TOPICS_PER_CHAT,
        chat_id,
      });
    }

    Ok(())
  }

  /// Check subscriber count limit for a topic
  async fn check_subscriber_limit(
    &self,
    chat_id: i64,
    topic: &str,
    new_users_count: usize,
  ) -> Result<()> {
    // Exclude __SYSTEM__ sentinel from subscriber count
    let count: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM subscriptions WHERE chat_id = ? AND topic = ? AND user_id != '__SYSTEM__'",
    )
    .bind(chat_id)
    .bind(topic)
    .fetch_one(&self.pool)
    .await
    .map_err(|e| BotError::Storage {
      chat_id,
      message: format!("Failed to count subscribers: {}", e),
    })?;

    if count as usize + new_users_count > MAX_SUBSCRIBERS_PER_TOPIC {
      return Err(BotError::SubscriberLimitReached {
        limit: MAX_SUBSCRIBERS_PER_TOPIC,
        topic: topic.to_string(),
      });
    }

    Ok(())
  }
}

#[async_trait]
impl StorageBackend for SqliteStorage {
  async fn get_topics(&self, chat_id: i64) -> Result<Option<Vec<String>>> {
    // Read from cache
    let topics = self.cache.get_topics(chat_id).await;

    // Ensure the "general" topic has a __SYSTEM__ sentinel for persistence
    // This handles existing chats that were created before the sentinel system
    if let Some(ref topic_list) = topics {
      if topic_list.contains(&GENERAL_TOPIC.to_string()) {
        let has_sentinel = self
          .cache
          .is_subscriber_in_topic(chat_id, "__SYSTEM__", GENERAL_TOPIC)
          .await;

        // If general topic exists but has no sentinel, add it to both DB and cache
        if !has_sentinel {
          let _ = sqlx::query(
            "INSERT OR IGNORE INTO subscriptions (chat_id, topic, user_id) VALUES (?, ?, ?)"
          )
          .bind(chat_id)
          .bind(GENERAL_TOPIC)
          .bind("__SYSTEM__")
          .execute(&self.pool)
          .await;

          self
            .cache
            .add_subscription(chat_id, GENERAL_TOPIC, "__SYSTEM__")
            .await;
        }
      }
    }

    Ok(topics)
  }

  async fn get_subscribers(&self, chat_id: i64) -> Result<Option<Vec<String>>> {
    let subscribers: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT user_id FROM subscriptions WHERE chat_id = ? AND user_id != '__SYSTEM__' ORDER BY user_id"
        )
        .bind(chat_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| BotError::Storage {
            chat_id,
            message: format!("Failed to fetch subscribers: {}", e),
        })?;

    Ok(if subscribers.is_empty() {
      None
    } else {
      Some(subscribers)
    })
  }

  async fn get_subscribers_from_topic(
    &self,
    chat_id: i64,
    topic: &str,
  ) -> Result<Option<Vec<String>>> {
    // Read from cache and filter out __SYSTEM__
    let subscribers = self
      .cache
      .get_subscribers_from_topic(chat_id, topic)
      .await
      .map(|users| {
        users
          .into_iter()
          .filter(|u| u != "__SYSTEM__")
          .collect::<Vec<_>>()
      })
      .and_then(|users| if users.is_empty() { None } else { Some(users) });

    Ok(subscribers)
  }

  async fn get_topics_from_subscriber(
    &self,
    chat_id: i64,
    user_id: &str,
  ) -> Result<Option<Vec<String>>> {
    // Read from cache
    Ok(self.cache.get_topics_from_subscriber(chat_id, user_id).await)
  }

  async fn does_group_have_topic(
    &self,
    chat_id: i64,
    topic: &str,
  ) -> Result<bool> {
    // Read from cache
    Ok(self.cache.does_group_have_topic(chat_id, topic).await)
  }

  async fn is_subscriber_in_topic(
    &self,
    chat_id: i64,
    user_id: &str,
    topic: &str,
  ) -> Result<bool> {
    // Read from cache
    Ok(
      self
        .cache
        .is_subscriber_in_topic(chat_id, user_id, topic)
        .await,
    )
  }

  async fn set_topic_and_subscribers(
    &mut self,
    chat_id: i64,
    topic: &str,
    users: &[&str],
  ) -> Result<()> {
    // Check limits
    self.check_topic_limit(chat_id, topic).await?;
    self
      .check_subscriber_limit(chat_id, topic, users.len())
      .await?;

    // Start transaction
    let mut tx = self.pool.begin().await.map_err(|e| BotError::Storage {
      chat_id,
      message: format!("Failed to start transaction: {}", e),
    })?;

    // Special handling for "general" topic with no subscribers
    // We insert a placeholder entry to ensure the topic exists in the database
    if topic == GENERAL_TOPIC && users.is_empty() {
      sqlx::query(
        "INSERT OR IGNORE INTO subscriptions (chat_id, topic, user_id) VALUES (?, ?, ?)"
      )
      .bind(chat_id)
      .bind(topic)
      .bind("__SYSTEM__") // Sentinel value to mark general topic exists
      .execute(&mut *tx)
      .await
      .map_err(|e| BotError::Storage {
        chat_id,
        message: format!("Failed to initialize general topic: {}", e),
      })?;
    } else {
      // Insert subscriptions (ignore duplicates)
      for user in users {
        sqlx::query(
          "INSERT OR IGNORE INTO subscriptions (chat_id, topic, user_id) VALUES (?, ?, ?)"
        )
        .bind(chat_id)
        .bind(topic)
        .bind(user)
        .execute(&mut *tx)
        .await
        .map_err(|e| BotError::Storage {
          chat_id,
          message: format!("Failed to insert subscription: {}", e),
        })?;
      }
    }

    // Commit transaction
    tx.commit().await.map_err(|e| BotError::Storage {
      chat_id,
      message: format!("Failed to commit transaction: {}", e),
    })?;

    // Update cache after successful DB write
    if topic == GENERAL_TOPIC && users.is_empty() {
      self
        .cache
        .add_subscription(chat_id, topic, "__SYSTEM__")
        .await;
    } else {
      self.cache.add_subscriptions(chat_id, topic, users).await;
    }

    Ok(())
  }

  async fn unset_subscriber_from_topic(
    &mut self,
    chat_id: i64,
    topic: &str,
    user_id: &str,
  ) -> Result<()> {
    // Start transaction to ensure atomicity
    let mut tx = self.pool.begin().await.map_err(|e| BotError::Storage {
      chat_id,
      message: format!("Failed to start transaction: {}", e),
    })?;

    // Remove the subscription (but never remove __SYSTEM__ sentinel)
    let result = sqlx::query(
            "DELETE FROM subscriptions WHERE chat_id = ? AND topic = ? AND user_id = ? AND user_id != '__SYSTEM__'"
        )
        .bind(chat_id)
        .bind(topic)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| BotError::Storage {
            chat_id,
            message: format!("Failed to remove subscription: {}", e),
        })?;

    if result.rows_affected() > 0 {
      // Check if there are any remaining real subscribers for this topic
      // Skip this check for the "general" topic - it should never be deleted
      // Also exclude __SYSTEM__ from the count as it's just a placeholder
      if topic != GENERAL_TOPIC {
        let remaining_count: i64 = sqlx::query_scalar(
          "SELECT COUNT(*) FROM subscriptions WHERE chat_id = ? AND topic = ? AND user_id != '__SYSTEM__'",
        )
        .bind(chat_id)
        .bind(topic)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| BotError::Storage {
          chat_id,
          message: format!("Failed to count remaining subscribers: {}", e),
        })?;

        // If no real subscribers remain, the topic is automatically removed
        // (no explicit DELETE needed since the table only contains subscriptions)
        if remaining_count == 0 {
          log::debug!(
            "Topic '{}' in chat {} has no remaining subscribers",
            topic,
            chat_id
          );
        }
      }
    }

    // Commit transaction
    tx.commit().await.map_err(|e| BotError::Storage {
      chat_id,
      message: format!("Failed to commit transaction: {}", e),
    })?;

    // Update cache after successful DB write
    if result.rows_affected() > 0 {
      self
        .cache
        .remove_subscription(chat_id, topic, user_id)
        .await;
    }

    Ok(())
  }

  async fn push_creating_message_id(
    &mut self,
    chat_id: i64,
    msg_id: i32,
  ) -> Result<()> {
    sqlx::query(
            "INSERT OR REPLACE INTO pending_topic_creation (chat_id, message_id) VALUES (?, ?)"
        )
        .bind(chat_id)
        .bind(msg_id)
        .execute(&self.pool)
        .await
        .map_err(|e| BotError::Storage {
            chat_id,
            message: format!("Failed to mark message as pending: {}", e),
        })?;

    log::debug!(
      "Marked message {} in chat {} as awaiting topic creation",
      msg_id,
      chat_id
    );

    Ok(())
  }

  async fn pop_creating_message_id(
    &mut self,
    chat_id: i64,
    msg_id: i32,
  ) -> Result<()> {
    sqlx::query(
      "DELETE FROM pending_topic_creation WHERE chat_id = ? AND message_id = ?",
    )
    .bind(chat_id)
    .bind(msg_id)
    .execute(&self.pool)
    .await
    .map_err(|e| BotError::Storage {
      chat_id,
      message: format!("Failed to remove pending message: {}", e),
    })?;

    log::debug!(
      "Removed message {} from chat {} topic creation queue",
      msg_id,
      chat_id
    );

    Ok(())
  }

  async fn has_create_message_id(
    &self,
    chat_id: i64,
    msg_id: i32,
  ) -> Result<bool> {
    let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pending_topic_creation WHERE chat_id = ? AND message_id = ?)"
        )
        .bind(chat_id)
        .bind(msg_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| BotError::Storage {
            chat_id,
            message: format!("Failed to check pending message: {}", e),
        })?;

    Ok(exists)
  }

  async fn cleanup_old_pending_requests(&mut self) -> Result<usize> {
    let result = sqlx::query(
            "DELETE FROM pending_topic_creation WHERE created_at < datetime('now', '-1 hour')"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| BotError::Storage {
            chat_id: 0,
            message: format!("Failed to cleanup old requests: {}", e),
        })?;

    let removed = result.rows_affected() as usize;

    if removed > 0 {
      log::info!("Cleaned up {} old pending topic creation requests", removed);
    }

    Ok(removed)
  }

  async fn get_stats(&self) -> Result<StorageStats> {
    let total_chats: i64 =
      sqlx::query_scalar("SELECT COUNT(DISTINCT chat_id) FROM subscriptions")
        .fetch_one(&self.pool)
        .await
        .map_err(|e| BotError::Storage {
          chat_id: 0,
          message: format!("Failed to get chat count: {}", e),
        })?;

    let total_topics: i64 = sqlx::query_scalar(
      "SELECT COUNT(DISTINCT chat_id || ':' || topic) FROM subscriptions",
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| BotError::Storage {
      chat_id: 0,
      message: format!("Failed to get topic count: {}", e),
    })?;

    // Exclude __SYSTEM__ sentinel from subscription count
    let total_subscriptions: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM subscriptions WHERE user_id != '__SYSTEM__'",
    )
    .fetch_one(&self.pool)
    .await
    .map_err(|e| BotError::Storage {
      chat_id: 0,
      message: format!("Failed to get subscription count: {}", e),
    })?;

    let pending_requests: i64 =
      sqlx::query_scalar("SELECT COUNT(*) FROM pending_topic_creation")
        .fetch_one(&self.pool)
        .await
        .map_err(|e| BotError::Storage {
          chat_id: 0,
          message: format!("Failed to get pending request count: {}", e),
        })?;

    Ok(StorageStats {
      total_chats: total_chats as usize,
      total_topics: total_topics as usize,
      total_subscriptions: total_subscriptions as usize,
      pending_requests: pending_requests as usize,
    })
  }

  async fn health_check(&self) -> Result<()> {
    // Perform a simple query to verify database connectivity
    sqlx::query_scalar::<_, i64>("SELECT 1")
      .fetch_one(&self.pool)
      .await
      .map_err(|e| BotError::Database(format!("Health check failed: {}", e)))?;

    log::debug!("Storage health check passed");
    Ok(())
  }
}
