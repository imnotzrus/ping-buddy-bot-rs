//! Extension traits for Telegram types.
//!
//! This module provides convenient extension traits for working with
//! Telegram bot types like Messages and Users.

pub mod message_ext;
pub mod user_ext;

pub use message_ext::{ChatIdExt, TopicExt};
pub use user_ext::UserRefExt;
