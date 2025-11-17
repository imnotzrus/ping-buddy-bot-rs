//! Storage layer for managing topic subscriptions.
//!
//! This module provides a trait-based abstraction for storage backends,
//! allowing easy swapping between different storage implementations.
//!
//! ## Architecture
//!
//! ```text
//! Storage (thread-safe wrapper)
//!   └── Arc<RwLock<Box<dyn StorageBackend>>>
//!       └── SqliteStorage (concrete implementation)
//! ```
//!
//! ## Usage
//!
//! Storage is initialized internally by the bot and accessed through
//! dependency injection in handlers.

pub mod cache;
pub mod sqlite;
pub mod r#trait;

use std::sync::Arc;
use tokio::sync::RwLock;

pub use r#trait::StorageBackend;
pub use sqlite::SqliteStorage;

/// Thread-safe storage wrapper
#[derive(Clone)]
pub struct Storage(Arc<RwLock<Box<dyn StorageBackend>>>);

impl Storage {
  /// Create a new storage instance with the given backend
  pub fn new(backend: Box<dyn StorageBackend>) -> Self {
    Self(Arc::new(RwLock::new(backend)))
  }

  /// Get read access to the storage
  pub async fn read(
    &self,
  ) -> tokio::sync::RwLockReadGuard<'_, Box<dyn StorageBackend>> {
    self.0.read().await
  }

  /// Get write access to the storage
  pub async fn write(
    &self,
  ) -> tokio::sync::RwLockWriteGuard<'_, Box<dyn StorageBackend>> {
    self.0.write().await
  }
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
