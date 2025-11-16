//! # Ping Buddy Bot Library
//!
//! A high-performance Telegram bot for managing topic-based subscriptions and user tagging.
//!
//! ## Features
//!
//! - **Topic Management**: Create and manage custom topics for organizing group members
//! - **Smart Subscriptions**: Users can subscribe/unsubscribe to topics via inline buttons
//! - **Efficient Tagging**: Ping all subscribers of a topic with a simple command
//! - **Persistent Storage**: SQLite-based storage with automatic migrations
//! - **Production Ready**: Comprehensive error handling, logging, and validation
//!
//! ## Architecture
//!
//! The bot is built with a modular architecture:
//!
//! - **Handlers**: Process incoming messages and callback queries
//! - **Storage**: Trait-based storage abstraction with SQLite implementation
//! - **Validation**: Input validation for security and data integrity
//! - **Error Handling**: Custom error types with detailed context
//!
//! ## Example Usage
//!
//! ```no_run
//! use ping_buddy_lib::{setup_logger, spin_up};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     setup_logger();
//!     spin_up().await?;
//!     Ok(())
//! }
//! ```

#![deny(warnings)]
#![warn(missing_docs)]

use std::sync::Arc;

use teloxide::adaptors::DefaultParseMode;
use teloxide::dispatching::{Dispatcher, HandlerExt, UpdateFilterExt};
use teloxide::dptree;
use teloxide::requests::RequesterExt;
use teloxide::types::{ParseMode, Update};
use teloxide::update_listeners::webhooks;
use teloxide::Bot as TBot;

use crate::command::Command;
use crate::env::Env;
use crate::storage::{SqliteStorage, Storage};

// Re-export error types for convenience
pub use crate::error::{BotError, Result};

mod command;
mod constants;
mod env;
mod error;
mod handler;
mod storage;
mod traits;
mod utils;
mod validation;

type Bot = DefaultParseMode<TBot>;

/// Initialize the logger with environment-based configuration.
///
/// This sets up `pretty_env_logger` which reads from the `RUST_LOG` environment variable.
/// Default log level is `info` if not specified.
///
/// # Example
///
/// ```bash
/// RUST_LOG=debug cargo run
/// ```
pub fn setup_logger() {
  pretty_env_logger::init();
}

/// Main entry point for the bot application.
///
/// This function:
/// 1. Loads and validates environment configuration
/// 2. Initializes the Telegram bot with webhook support
/// 3. Sets up SQLite storage with automatic migrations
/// 4. Configures message and callback handlers
/// 5. Starts the dispatcher to process updates
///
/// # Errors
///
/// Returns `BotError` if:
/// - Environment variables are missing or invalid
/// - Database connection or migration fails
/// - Webhook setup fails
/// - Dispatcher encounters a fatal error
///
/// # Example
///
/// ```no_run
/// # use ping_buddy_lib::spin_up;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// spin_up().await?;
/// # Ok(())
/// # }
/// ```
pub async fn spin_up() -> Result {
  log::info!("Starting Ping Buddy Bot...");

  let env = Env::init()?;

  let bot = TBot::from_env().parse_mode(ParseMode::MarkdownV2);

  log::info!("Setting up webhook listener...");

  // Extract socket address from inbound URL
  let inbound_host = env.inbound.host_str().ok_or_else(|| {
    BotError::Config("INBOUND URL must have a host".to_string())
  })?;
  let inbound_port = env.inbound.port().ok_or_else(|| {
    BotError::Config("INBOUND URL must have a port".to_string())
  })?;
  let inbound_addr = format!("{}:{}", inbound_host, inbound_port)
    .parse()
    .map_err(|e| {
      BotError::AddrParse(format!("inbound socket address: {}", e))
    })?;

  let options = webhooks::Options::new(inbound_addr, env.outbound.clone());

  let listener =
    webhooks::axum(Clone::clone(&bot), options)
      .await
      .map_err(|e| {
        BotError::Config(format!("Failed to build webhook listener: {}", e))
      })?;

  log::info!("Initializing storage...");
  let sqlite_backend = SqliteStorage::new(&env.database_path).await?;
  let storage = Storage::new(Box::new(sqlite_backend));

  // Perform initial health check
  log::info!("Performing storage health check...");
  storage.read().await.health_check().await?;
  log::info!("Storage health check passed");

  // Spawn periodic cleanup task
  let cleanup_storage = storage.clone();
  tokio::spawn(async move {
    let mut interval =
      tokio::time::interval(std::time::Duration::from_secs(3600)); // Every hour
    loop {
      interval.tick().await;
      log::debug!("Running periodic cleanup task");
      match cleanup_storage
        .write()
        .await
        .cleanup_old_pending_requests()
        .await
      {
        Ok(count) if count > 0 => {
          log::info!("Cleaned up {} old pending requests", count);
        }
        Ok(_) => log::debug!("No old pending requests to clean up"),
        Err(e) => log::error!("Failed to cleanup old requests: {}", e),
      }
    }
  });

  log::info!("Building message handler...");
  let handler = dptree::entry()
    .branch(
      Update::filter_message()
        .branch(
          dptree::entry()
            .filter_command::<Command>()
            .endpoint(handler::message::r#static::handle),
        )
        .branch(dptree::entry().endpoint(handler::message::dynamic::handle)),
    )
    .branch(
      Update::filter_callback_query().endpoint(handler::callback::handle),
    );

  log::info!("Bot is ready and listening for updates");

  Dispatcher::builder(bot, handler)
    .dependencies(dptree::deps![storage])
    .enable_ctrlc_handler()
    .build()
    .dispatch_with_listener(
      listener,
      Arc::new(|err| async move {
        log::error!("Dispatcher error: {:?}", err);
      }),
    )
    .await;

  log::info!("Bot shutting down");
  Ok(())
}
