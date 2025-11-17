//! UI components for the bot.
//!
//! This module contains all UI-related functionality including
//! keyboard layouts and message templates.

pub mod keyboards;
pub mod messages;

pub use keyboards::{build_topic_buttons, remove_message};
pub use messages::Messages;
